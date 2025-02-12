use std::{net::IpAddr, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use serde::Serialize;
use warp::{http::header, reply::Response, Filter};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use tera::{Tera, Context};

pub async fn server(ip: IpAddr, port: u16, path: Arc<Mutex<Vec<(PathBuf, usize)>>>, num_send_files: Arc<Mutex<usize>>) {
    let html_route = create_index_route(path.clone());
    let update_route = create_refresh_route(path.clone());
    let static_route = create_static_route();
    let download_route = create_download_route(path.clone(), num_send_files.clone());

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
    size: String,
}

fn create_static_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::fs::dir("./static"))
}

fn create_index_route(path: Arc<Mutex<Vec<(PathBuf, usize)>>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let html_route = warp::path::path("index")
        .map(move || {
            let path = path.clone();
            let html = fill_template(path, "index.html");
            warp::reply::html(html)
        });
    html_route
}

#[derive(Serialize)]
struct UpdateData {
    size: String,
    html: String
}

fn create_refresh_route(path: Arc<Mutex<Vec<(PathBuf, usize)>>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let refresh_route = warp::path("update-content")
        .map(move || {
            let html = fill_template(path.clone(), "file_list.html");
            warp::reply::json(&UpdateData {
                size: size_string(&path.lock().unwrap().iter().map(|(_, size)| size).sum()),
                html
            })
        });
    refresh_route
}

fn fill_template(path: Arc<Mutex<Vec<(PathBuf, usize)>>>, template: &'static str) -> String {
    let tera: Tera = Tera::new("template/*.html").unwrap();
    let mut context = Context::new();

    let files: Vec<FileInfo> = path.lock().unwrap().iter().enumerate().map(|(i, (path, size))| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        let size_string = size_string(size);
        FileInfo { name, index: i, size: size_string }
    }).collect();
    context.insert("files", &files);

    let all_size: usize = path.lock().unwrap().iter().map(|(_, size)| size).sum();
    context.insert("all_size", &size_string(&all_size));

    tera.render(template, &context).unwrap()
}

fn size_string(size: &usize) -> String {
    match size {
        s if *s < 1024 => format!("{} B", s),
        s if *s < 1024 * 1024 => format!("{:.1} KB", *s as f64 / 1024.0),
        s if *s < 1024 * 1024 * 1024 => format!("{:.1} MB", *s as f64 / 1024.0 / 1024.0),
        s if *s < 1024 * 1024 * 1024 * 1024 => format!("{:.1} GB", *s as f64 / 1024.0 / 1024.0 / 1024.0),
        s => format!("{:.1} TB", *s as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0),
    }
}

fn create_download_route(path: Arc<Mutex<Vec<(PathBuf, usize)>>>, num_send_files: Arc<Mutex<usize>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let download_route = warp::path!("download" / usize).and_then(move |index| {
        let (path, _) = path.lock().unwrap().get(index).cloned().unwrap();
        let num_send_files = num_send_files.clone();
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

            *num_send_files.lock().unwrap() += 1;
            Ok::<_, warp::Rejection>(warp::reply::with_header(response, "Connection", "close"))
        }
    });
    download_route
}
