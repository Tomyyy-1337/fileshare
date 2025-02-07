use std::{net::IpAddr, path::{Path, PathBuf}};
use warp::{filters::multipart::FormData, http::header, reply::Response, Filter};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::{bytes::Buf, io::ReaderStream};
use warp::hyper::Body;
use futures::TryStreamExt;

pub async fn server(ip: IpAddr, port: u16, path: Vec<PathBuf>) {
    let html_route = create_index_page(path.clone());
    let css_route = create_css_route();
    let download_route = create_route(path);

    let routes = html_route.or(css_route).or(download_route);

    warp::serve(routes)
        .run((ip, port))
        .await;
}

fn create_index_page(path: Vec<PathBuf>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let links = path.iter().enumerate().map(|(i, path)| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown");
        let file_format = path.extension().and_then(|n| n.to_str()).unwrap_or("Unknown");
        match file_format {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => format!(
                r#"
                <div class="row">
                    <p class="name">{}</p>
                    <a class="link" href="download/{}">Download</a>
                    <button class="toggle-button" onclick="toggleImage('image-{}')">Show/Hide Image</button>
                    <div id="image-{}" class="image-container" style="display: none;">
                        <img src="download/{}" alt="{}" style="max-width: 100%; height: auto;">
                    </div>
                </div>
                "#,
                name, i, i, i, i, name
            ),
            _ => format!(
                r#"
                <div class="row">
                    <p class="name">{}</p>
                    <a class="link" href="download/{}">Download</a>
                </div>
                "#,
                name, i
            ),
        }
    }).collect::<Vec<String>>().join("<br>");

//     let form = 
// r#"<form action="/upload" method="post" enctype="multipart/form-data">
//     <label for="file">Select file:</label>
//     <input type="file" id="file" name="file">
//     <button type="submit" class="button">Upload</button>
// </form>"#;

    let script = 
        "<script>
            document.getElementById('downloadAll').addEventListener('click', function() {
                const links = document.querySelectorAll('a.link');
                links.forEach(link => {
                    const url = link.href;
                    const fileName = link.previousElementSibling.textContent;
                    fetch(url)
                        .then(response => response.blob())
                        .then(blob => {
                            const a = document.createElement('a');
                            a.href = URL.createObjectURL(blob);
                            a.download = fileName;
                            document.body.appendChild(a);
                            a.click();
                            document.body.removeChild(a);
                        })
                        .catch(console.error);
                });
            });
        </script>";

    let html_route = warp::path::path("index")
        .map(move || {
            let html = format!(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Download Page</title>
                    <link rel="stylesheet" type="text/css" href="/static/style.css">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                </head>
                <body>
                    <h1>Download Files</h1>
                    <div id = "allrow" class="row"><p class="name">Download all Files</p><button id="downloadAll">Download All Files</button></div><br>
                    <h1>File List</h1>                    
                    {}
                    {script}
                </body>
                </html>
                "#, links);

            let response = warp::reply::html(html);
            warp::reply::with_header(response, "Connection", "close")    
        });
    html_route
}

fn create_css_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::path("style.css"))
        .and(warp::fs::file("./style.css"))
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

