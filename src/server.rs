use std::{net::IpAddr, path::{Path, PathBuf}};
use serde::Serialize;
use warp::{http::header, reply::Response, Filter};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use tera::{Tera, Context};

pub async fn server(ip: IpAddr, port: u16, path: Vec<PathBuf>) {
    let html_route = create_index_page(path.clone());
    let css_route = create_css_route();
    let js_route = create_script_route();
    let download_route = create_route(path.clone());
    let update_content_route = create_update_content_route(path);

    let routes = html_route
        .or(update_content_route)
        .or(css_route)
        .or(js_route)
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

fn html_template(path: &Vec<PathBuf>) -> String {
    let tera = Tera::new("static/*.html").unwrap();
    let mut context = Context::new();

    let files: Vec<FileInfo> = path.iter().enumerate().map(|(i, path)| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        FileInfo { name, index: i }
    }).collect();

    context.insert("files", &files);

    tera.render("index.html", &context).unwrap()
}

fn create_update_content_route(path: Vec<PathBuf>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("update-content")
    .map(move || {
        let tera = Tera::new("static/*.html").unwrap();
        let mut context = Context::new();

        let files: Vec<FileInfo> = path.iter().enumerate().map(|(i, path)| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            FileInfo { name, index: i }
        }).collect();

        context.insert("files", &files);

        let html = tera.render("file_list.html", &context).unwrap();
        let response = warp::reply::html(html);
        warp::reply::with_header(response, "Connection", "close")
    })
}

fn create_index_page(path: Vec<PathBuf>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let html_route = warp::path::path("index")
        .map(move || {
            let html = html_template(&path);

            let response = warp::reply::html(html);
            warp::reply::with_header(response, "Connection", "close")    
        });
    html_route
}

fn create_css_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::path("style.css"))
        .and(warp::fs::file("./static/style.css"))
}

fn create_script_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::path("script.js"))
        .and(warp::fs::file("./static/script.js"))
}

fn create_route(path: Vec<PathBuf>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let download_route = warp::path!("download" / usize).and_then(move |index| {
        let path: PathBuf = path.get(index).cloned().unwrap();
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

