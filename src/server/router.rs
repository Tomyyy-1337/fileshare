use std::{collections::HashMap, net::IpAddr, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}};
use iced::Theme;
use tokio::sync::Mutex;
use warp::{reject::Rejection, reply::Reply, Filter};
use futures::channel::mpsc::Sender;

use crate::{state::file_manager, state::update::ServerMessage};

use super::{download_service::{download_all_route, download_route}, webpage_service::{index_route, refresh_route, static_route}};

pub async fn server(
    ip: IpAddr, 
    port: u16, 
    path: Arc<RwLock<HashMap<usize, file_manager::FileInfo>>>, 
    tx: Sender<ServerMessage>,
    block_external_connections: Arc<AtomicBool>,
    theme: Arc<RwLock<Theme>>
) {
    let routes = warp::any()
        .and(block_external(block_external_connections))
        .and(index_route(path.clone(), theme.clone())
            .or(refresh_route(path.clone(), theme.clone()))
            .or(static_route())
            .or(download_route(path, tx.clone(), Arc::new(Mutex::new(HashMap::<IpAddr, Arc<tokio::sync::Semaphore>>::new()))))
            .or(download_all_route(tx.clone())))
        .and_then(move |addr: std::net::IpAddr, reply| {
            notify_application_and_reply(addr, tx.clone(), reply)
        });
        
    warp::serve(routes)
        .run((ip, port))
        .await;
}

fn block_external(
    block_external: Arc<AtomicBool>,
) -> impl Filter<Extract = (std::net::IpAddr,), Error = Rejection> + Clone {
    warp::addr::remote()
        .and_then(move |addr: Option<std::net::SocketAddr>| {
            let block_external = block_external.load(Ordering::Relaxed);
            async move {
                let ip = addr
                    .ok_or_else(|| warp::reject::reject())
                    .map(|addr| addr.ip())?;
                    
                if block_external && !is_private_ip(ip) {
                    return Err(warp::reject::reject());
                } 
                Ok::<_, Rejection>(ip)
            }
        })
}

async fn notify_application_and_reply(
    ip: std::net::IpAddr,
    mut tx: Sender<ServerMessage>,
    reply: impl Reply,
) -> Result<impl Reply, Rejection> {
    tx.try_send(ServerMessage::ClientConnected { ip })
        .map_err(|_| warp::reject::reject())?;
    
    Ok(warp::reply::with_header(reply, "Connection", "close"))
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unique_local(),
    }
}
