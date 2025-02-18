use std::{net::IpAddr, path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex, RwLock}};
use serde::Serialize;
use warp::{http::header, reject::Rejection, reply::Response, Filter};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use tera::Tera;
use futures::{channel::mpsc::Sender, stream::Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::state;

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Downloaded { index: usize , ip: IpAddr },
    ClientConnected { ip: IpAddr },
    DownloadActive { ip: IpAddr, num_packets: usize },
}

pub async fn server(
    ip: IpAddr, 
    port: u16, 
    path: Arc<RwLock<Vec<state::FileInfo>>>, 
    tx: Sender<ServerMessage>,
    block_external_connections: Arc<AtomicBool>)
{
    let html_route = create_index_route(path.clone());
    let update_route = create_refresh_route(path.clone());
    let static_route = create_static_route();
    let download_route = create_download_route(path, tx.clone());
    
    let tx = Arc::new(Mutex::new(tx.clone()));
    let block_external = block_external_connections.clone();

    let routes = html_route
        .or(update_route)
        .or(static_route)
        .or(download_route)
        .and(warp::addr::remote())
        .and_then(move |reply, addr: Option<std::net::SocketAddr>| {
            let tx = tx.clone();
            let block_external = block_external.load(Ordering::Relaxed);
            async move {
                if let Some(socket_address) = addr {
                    let ip = socket_address.ip();
                    if block_external && !is_private_ip(ip) {
                        return Err(warp::reject::reject());
                    }
                    tx.lock().unwrap().try_send(ServerMessage::ClientConnected { ip }).unwrap();
                }
                Ok::<_, Rejection>(warp::reply::with_header(reply, "Connection", "close"))
            }
        });

    warp::serve(routes)
        .run((ip, port))
        .await;
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    index: usize,
    size: String,
}

fn create_static_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::fs::dir("./static"))
}

fn create_index_route(path: Arc<RwLock<Vec<state::FileInfo>>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let html_route = warp::path("index")
        .map(move || {
            let path = path.clone();
            let html_str = fill_template(path, "index.html");
            let response = warp::reply::html(html_str);
            response
        });
    html_route
}

#[derive(Serialize)]
struct UpdateData {
    size: String,
    html: String
}

fn create_refresh_route(path: Arc<RwLock<Vec<state::FileInfo>>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let refresh_route = warp::path("update-content")
        .map(move || {
            let html = fill_template(path.clone(), "file_list.html");
            let response = warp::reply::json(&UpdateData {
                size: size_string(path.read().unwrap().iter().map(|state::FileInfo{size, ..}| size).sum()),
                html
            });
            response
        });
    refresh_route
}

fn fill_template(path: Arc<RwLock<Vec<state::FileInfo>>>, template: &'static str) -> String {
    let tera: Tera = Tera::new("template/*.html").unwrap();
    let mut context = tera::Context::new();

    let files: Vec<FileInfo> = path.read().unwrap().iter().enumerate().map(|(i, state::FileInfo{path, size, ..})| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        let size_string = size_string(*size);
        FileInfo { name, index: i, size: size_string }
    }).collect();
    context.insert("files", &files);

    let all_size: usize = path.read().unwrap().iter().map(|state::FileInfo{size, ..}| size).sum();
    context.insert("all_size", &size_string(all_size));

    tera.render(template, &context).unwrap()
}

pub fn size_string(size: usize) -> String {
    match size {
        s if s < 1000 => format!("{} B", s),
        s if s < 1024 * 1000 => format!("{:.1} KB", s as f64 / 1024.0),
        s if s < 1024 * 1024 * 1000 => format!("{:.1} MB", s as f64 / 1024.0 / 1024.0),
        s if s < 1024 * 1024 * 1024 * 1000 => format!("{:.1} GB", s as f64 / 1024.0 / 1024.0 / 1024.0),
        s => format!("{:.1} TB", s as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0),
    }
}

fn create_download_route(files: Arc<RwLock<Vec<state::FileInfo>>>, tx: Sender<ServerMessage>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("download" / usize)
        .and(warp::addr::remote())
        .and_then(move |index, addr: Option<std::net::SocketAddr>| {
            let tx = tx.clone();
            let files = files.clone();
            async move {
                let file_info = {
                    let files = files.read().unwrap();
                    if index >= files.len() {
                        return Err(warp::reject::not_found());
                    }
                    files.get(index).cloned().unwrap()
                };

                let state::FileInfo { path, .. } = file_info;
                let file_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| warp::reject::not_found())?;

                let file = File::open(&path).await.map_err(|_| warp::reject::not_found())?;
                let stream = ReaderStream::new(file);
                let counting_stream = CountingStream::new(stream, tx, index, addr.unwrap().ip());
                let body = Body::wrap_stream(counting_stream);
                let response = Response::new(body);

                let response = warp::reply::with_header(
                    response,
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file_name),
                );

                Ok::<_, warp::Rejection>(response)
            }
        })
}

struct CountingStream<S> {
    inner: S,
    tx: Sender<ServerMessage>,
    index: usize,
    ip: IpAddr,
    counter: usize,
    last_send_time: std::time::Instant,
}

impl<S> CountingStream<S> {
    fn new(inner: S, tx: Sender<ServerMessage>, index: usize, ip: IpAddr) -> Self {
        CountingStream { inner, tx, index, ip, counter: 0, last_send_time: std::time::Instant::now() }
    }
}

impl<S> Stream for CountingStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin,
{
    type Item = Result<bytes::Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(None) => {
                let index = self.index;
                let ip = self.ip;
                let counter = self.counter;
                let _ = self.tx.try_send(ServerMessage::DownloadActive { ip, num_packets: counter });
                let _ = self.tx.try_send(ServerMessage::Downloaded { index, ip });
                Poll::Ready(None)
            }
            poll @ Poll::Ready(_) => {
                self.counter = self.counter + 1;
                if self.last_send_time.elapsed().as_millis() > 1000 {
                    let ip = self.ip;
                    let counter = self.counter;
                    let _ = self.tx.try_send(ServerMessage::DownloadActive { ip, num_packets: counter });
                    self.counter = 0;
                    self.last_send_time = std::time::Instant::now();
                }
                poll
            }
            other => other,
        }
    }
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unique_local(),
    }
}