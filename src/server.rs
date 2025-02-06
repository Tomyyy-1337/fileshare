use std::{fs::File, io::Read, net::IpAddr, path::{Path, PathBuf}};

use warp::Filter;

pub async fn server(ip: IpAddr, port: u16, path: PathBuf) {
    let download_route = warp::path("download")
        .and_then( move || { 
            let path_clone = path.clone();
            async move {
                let file_name = Path::new(&path_clone).file_name().unwrap().to_str().unwrap();
                let mut file = match File::open(&path_clone) {
                    Ok(file) => file,
                    Err(_) => return Err(warp::reject::not_found()),
                };
                
                let mut buffer = Vec::new();
                if let Err(_) = file.read_to_end(&mut buffer) {
                    return Err(warp::reject::not_found());
                }
              
                Ok(warp::reply::with_header(
                    warp::reply::with_header(
                        buffer,
                        "Content-Disposition",
                        format!("attachment; filename=\"{}\"", file_name) 
                    ),
                    "Connection",
                    "close"
                ))
            }
        });

    warp::serve(download_route)
        .run((ip, port))
        .await;
}
