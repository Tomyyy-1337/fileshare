use std::{net::IpAddr, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use serde::Serialize;
use warp::{http::header, reply::Response, Filter};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use tera::{Tera, Context};

pub async fn server(ip: IpAddr, port: u16, path: Arc<Mutex<Vec<PathBuf>>>) {
    let html_route = use_template(path.clone(), "index", "index.html");
    let update_route = use_template(path.clone(), "update-content", "file_list.html");
    let static_route = create_static_route();
    let download_route = create_download_route(path.clone());

    let routes = html_route
        .or(update_route)
        .or(static_route)
        .or(download_route);

    warp::serve(routes)
        .run((ip, port))
        .await;
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    index: usize,
}

fn create_static_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::fs::dir("./static"))
}

fn use_template(path: Arc<Mutex<Vec<PathBuf>>>, route: &'static str, template: &'static str) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let html_route = warp::path::path(route)
        .map(move || {
            let tera = Tera::new("template/*.html").unwrap();
            let mut context = Context::new();
        
            let files: Vec<FileInfo> = path.lock().unwrap().iter().enumerate().map(|(i, path)| {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                FileInfo { name, index: i }
            }).collect();
        
            context.insert("files", &files);
        
            let html = tera.render(template, &context).unwrap();

            let response = warp::reply::html(html);
            response
            // warp::reply::with_header(response, "Connection", "close")    
        });
    html_route
}

fn create_download_route(path: Arc<Mutex<Vec<PathBuf>>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let download_route = warp::path!("download" / usize).and_then(move |index| {
        let path: PathBuf = path.lock().unwrap().get(index).cloned().unwrap();
        async move {
            let file_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| warp::reject::not_found())?;
        
            let file = File::open(&path).await.map_err(|_| warp::reject::not_found())?;
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
    download_route
}
