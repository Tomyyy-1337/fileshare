use std::{collections::HashMap, net::IpAddr, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}};
use iced::{advanced::graphics::text, theme, Color, Theme};
use serde::Serialize;
use warp::{http::header, reject::Rejection, reply::Response, Filter};
use tokio::{fs::File, sync::Mutex};
use tokio_util::io::ReaderStream;
use warp::hyper::Body;
use tera::Tera;
use futures::{channel::mpsc::Sender, stream::Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::{state, styles::color_multiply};

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Downloaded { index: usize , ip: IpAddr },
    ClientConnected { ip: IpAddr },
    DownloadActive { ip: IpAddr, num_packets: usize },
    DownloadRequest { index: usize, ip: IpAddr },
    DownloadAllRequest { ip: IpAddr },
}

pub async fn server(
    ip: IpAddr, 
    port: u16, 
    path: Arc<RwLock<HashMap<usize, state::FileInfo>>>, 
    tx: Sender<ServerMessage>,
    block_external_connections: Arc<AtomicBool>,
    theme: Arc<RwLock<Theme>>
) {
    let semaphors: Arc<Mutex<HashMap<IpAddr, Arc<tokio::sync::Semaphore>>>> = Arc::new(Mutex::new(HashMap::<IpAddr, Arc<tokio::sync::Semaphore>>::new()));

    let html_route = create_index_route(path.clone(), theme.clone());
    let update_route = create_refresh_route(path.clone(), theme.clone());
    let static_route = create_static_route();
    let download_route = create_download_route(path, tx.clone(), semaphors.clone());
    let download_all_route = create_download_all_route(tx.clone());
    
    let tx = Arc::new(Mutex::new(tx.clone()));
    let block_external = block_external_connections.clone();

    let routes = html_route
        .or(update_route)
        .or(static_route)
        .or(download_route)
        .or(download_all_route)
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
                    let mut tx = tx.lock().await;
                    tx.try_send(ServerMessage::ClientConnected { ip })
                        .map_err(|_| warp::reject::reject())?;
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

fn create_index_route(
    path: Arc<RwLock<HashMap<usize, state::FileInfo>>>, 
    theme: Arc<RwLock<Theme>>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    let html_route = warp::path("index")
        .map(move || {
            let path = path.clone();
            let html_str = fill_template(path, "index.html", theme.clone());
            let response = warp::reply::html(html_str);
            response
        });
    html_route
}

#[derive(Serialize)]
struct UpdateData {
    html: String,
    size: String,
    primary: SendColor,
    secondary: SendColor,
    background: SendColor,
    dark_background: SendColor,
    text: SendColor,
    text_secondary: SendColor,
}

#[derive(Serialize)]
struct SendColor {
    r: u8,
    g: u8,
    b: u8,
}

fn color_to_arr (color: iced::Color) -> SendColor {
    let [r,g,b,_] = color.into_rgba8();
    SendColor { r, g, b }
}

fn colors(theme: &Theme) -> (SendColor, SendColor, SendColor, SendColor, SendColor, SendColor) {
    let primary = color_to_arr(theme.palette().primary);
    let secondary = color_to_arr(color_multiply(theme.palette().primary, 0.8));
    let background = color_to_arr(theme.palette().background);
    let dark_background = color_to_arr(color_multiply(theme.palette().background, 0.8));
    let text = color_to_arr(theme.palette().text);
    let text_secondary = color_to_arr({
        let iced::Color { r, g, b, .. } = theme.palette().primary;
        if r + g + b > 1.5 {
            iced::Color::BLACK
        } else {
            iced::Color::WHITE
        }
    });
    (primary, secondary, background, dark_background, text, text_secondary)
}

fn create_refresh_route(
    path: Arc<RwLock<HashMap<usize, state::FileInfo>>>,
    theme: Arc<RwLock<Theme>>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    let refresh_route = warp::path("update-content")
        .map(move || {
            let html = fill_template(path.clone(), "file_list.html", theme.clone());
            let theme = theme.read().unwrap();
            let (primary, secondary, background, dark_background, text, text_secondary) = colors(&theme);
            let response = warp::reply::json(&UpdateData {
                html,
                size: size_string(path.read().unwrap().iter().map(|(_, state::FileInfo{size, ..})| size).sum()),
                primary,
                secondary,
                background,
                dark_background,
                text,
                text_secondary
            });
            response
        });
    refresh_route
}

fn fill_template(
    path: Arc<RwLock<HashMap<usize, state::FileInfo>>>, 
    template: &'static str,
    theme: Arc<RwLock<Theme>>
) -> String {
    let tera: Tera = Tera::new("template/*.html").unwrap();
    let mut context = tera::Context::new();

    let mut path = path.read().unwrap()
        .iter()
        .map(|(i, f)| (*i, f.clone()))
        .collect::<Vec<_>>();
    path.sort_by_key(|(indx, _)| *indx);

    let files: Vec<FileInfo> = path.iter().rev().map(|(i, state::FileInfo{path, size, ..})| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        let size_string = size_string(*size);
        FileInfo { name, index: *i, size: size_string }
    }).collect();
    context.insert("files", &files);

    let all_size: usize = path.iter().map(|(_, state::FileInfo{size, ..})| size).sum();
    context.insert("all_size", &size_string(all_size));

    let theme = theme.read().unwrap();
    let (primary, secondary, background, dark_background, text, text_secondary) = colors(&theme);
    context.insert("primary", &to_rgb_string(primary));
    context.insert("secondary", &secondary);
    context.insert("background", &background);
    context.insert("dark_background", &dark_background);
    context.insert("text", &text);
    context.insert("text_secondary", &text_secondary);

    tera.render(template, &context).unwrap()
}

fn to_rgb_string(color: SendColor) -> String {
    let SendColor { r, g, b } = color;
    format!("rgb({}, {}, {})", r, g, b)
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

fn create_download_all_route(
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

fn create_download_route(
    files: Arc<RwLock<HashMap<usize, state::FileInfo>>>, 
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
                let file_info: state::FileInfo = files.read()
                    .unwrap()
                    .get(&index)
                    .cloned()
                    .ok_or_else(|| warp::reject::not_found())?;
                let file = File::open(&file_info.path)
                    .await
                    .map_err(|_| warp::reject::not_found())?;
                let semaphor = semaphor.lock().await
                    .entry(addr.unwrap().ip())
                    .or_insert_with(|| Arc::new(tokio::sync::Semaphore::new(3)))
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
                Ok::<_, warp::Rejection>(warp::reply::with_header(response, "Connection", "close"))
            }
    })
}

struct CountingStream<S> {
    inner: S,
    tx: Sender<ServerMessage>,
    index: usize,
    ip: IpAddr,
    counter: usize,
    last_send_time: tokio::time::Instant,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<S> CountingStream<S> {
    fn new(inner: S, tx: Sender<ServerMessage>, index: usize, ip: IpAddr, permit: tokio::sync::OwnedSemaphorePermit) -> CountingStream<S> {
        CountingStream { inner, tx, index, ip, counter: 0, last_send_time: tokio::time::Instant::now(), _permit: permit }
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
            Poll::Ready(Some(Err(_))) => Poll::Ready(None),
            Poll::Ready(Some(data)) => {
                self.counter += 1;
                if self.last_send_time.elapsed().as_millis() > 250 {
                    let ip = self.ip;
                    let counter = self.counter;
                    let _ = self.tx.try_send(ServerMessage::DownloadActive { ip, num_packets: counter });
                    self.counter = 0;
                    self.last_send_time = tokio::time::Instant::now();
                }
                Poll::Ready(Some(data))
            }
            p @ Poll::Pending => p,
        }
    }
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unique_local(),
    }
}