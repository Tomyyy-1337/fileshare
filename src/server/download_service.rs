use std::{collections::HashMap, net::IpAddr, sync::{Arc, RwLock}};
use warp::{http::header, reply::Response, Filter};
use tokio::{fs::File, sync::Mutex};
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use futures::channel::mpsc::Sender;
use crate::{state::file_manager, state::update::ServerMessage};

use super::counting_stream::CountingStream;

const PERMITS_PER_CLIENT: usize = 5;

pub fn download_route(
    files: Arc<RwLock<HashMap<usize, file_manager::FileInfo>>>, 
    tx: Sender<ServerMessage>, 
    semaphor: Arc<Mutex<HashMap<IpAddr, Arc<tokio::sync::Semaphore>>>>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("download" / usize / usize)
        .and(warp::addr::remote())
        .and_then(move |index, is_single, addr: Option<std::net::SocketAddr>| {
            let mut tx = tx.clone();
            let files = files.clone();
            let semaphor = semaphor.clone();
            async move {
                if is_single == 1 {
                    tx.try_send(ServerMessage::DownloadRequest { index, ip: addr.unwrap().ip() })
                        .map_err(|_| warp::reject::reject())?;
                }
                let file_info: file_manager::FileInfo = files.read()
                    .unwrap()
                    .get(&index)
                    .cloned()
                    .ok_or_else(|| warp::reject::not_found())?;
                let file = File::open(&file_info.path)
                    .await
                    .map_err(|_| warp::reject::not_found())?;
                let semaphor = semaphor.lock().await
                    .entry(addr.unwrap().ip())
                    .or_insert_with(|| Arc::new(tokio::sync::Semaphore::new(PERMITS_PER_CLIENT)))
                    .clone();
                let permit = semaphor.acquire_owned().await.unwrap();
                let stream = CountingStream::new(ReaderStream::new(file), tx, index, addr.unwrap().ip(), permit);
                let body = Body::wrap_stream(stream);
                let response = warp::reply::with_header(
                    Response::new(body), 
                    header::CONTENT_DISPOSITION, 
                    format!("attachment; filename=\"{}\"", 
                    file_info.path.file_name().unwrap().to_str().unwrap()
                ));
                Ok::<_, warp::Rejection>(response)
            }
    })
}

pub fn download_all_route(
    tx: Sender<ServerMessage>, 
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("download-all")
        .and(warp::addr::remote())
        .and_then(move |ip: Option<std::net::SocketAddr>| {
            let mut tx = tx.clone();
            async move {
                let _ = tx.try_send(ServerMessage::DownloadAllRequest { ip: ip.unwrap().ip() });
                Ok::<_, warp::Rejection>(warp::reply::with_status("Download started", warp::http::StatusCode::OK))
            }
        })
}
