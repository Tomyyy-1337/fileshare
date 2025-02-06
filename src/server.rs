use std::{net::IpAddr, path::{Path, PathBuf}};
use warp::{http::header, reply::Response, Filter};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use warp::hyper::Body;

pub async fn server(ip: IpAddr, port: u16, path: PathBuf) {
    let download_route = warp::path("download").and_then(move || {
        let path_clone = path.clone();
        async move {
            let file_name = Path::new(&path_clone)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| warp::reject::not_found())?;
            
            let file = File::open(&path_clone).await.map_err(|_| warp::reject::not_found())?;
            let stream = ReaderStream::new(file);
            let body = Body::wrap_stream(stream);
            let response = Response::new(body);

            let response = warp::reply::with_header(
                response,
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file_name),
            );

            Ok::<_, warp::Rejection>(warp::reply::with_header(response, "Connection", "close"))
        }
    });

    warp::serve(download_route)
        .run((ip, port))
        .await;
}