use std::{collections::HashMap, net::IpAddr, path::PathBuf, sync::{atomic::AtomicBool, Arc, RwLock}};
use local_ip_address::local_ip;
use iced::widget;
use qrcode_generator::QrCodeEcc;

use crate::view::CONNECTION_PANE_WIDTH;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
    pub download_count: usize,
}

pub struct ClientInfo {
    pub download_count: usize,
    pub download_size: usize,
    pub last_connection: std::time::Instant,
    pub last_download: std::time::Instant,
    pub received_data: usize,
    pub speed: usize,
    pub max_speed: usize,
}

pub struct State {
    pub dark_mode: bool,
    pub ip_adress: IpAddr,
    pub ip_adress_public: Option<IpAddr>,
    pub port: u16,
    pub file_path: Arc<RwLock<Vec<FileInfo>>>,
    pub qr_code: widget::image::Handle,
    pub server_handle: Option<iced::task::Handle>,
    pub port_buffer: String,
    pub local_host: bool,
    pub size: (f32, f32),
    pub clients: HashMap<IpAddr, ClientInfo>,
    pub show_connections: bool,
    pub transmitted_data: usize,
    pub block_external_connections: Arc<AtomicBool>,
    pub active_downloads: usize,
    pub total_downloads: usize,
    pub active_connections: usize,
    pub throughput: usize,
    pub show_qr_code: bool,
}

impl Default for State {
    fn default() -> Self {
        let ip = local_ip().unwrap();
        let ip_public = public_ip_address::perform_lookup(None).map(|lookup|lookup.ip).ok();
        let qr_code = Self::create_qr_code(&Self::url_string(&ip, 8080));

        Self {
            dark_mode: true,
            ip_adress: ip,
            ip_adress_public: ip_public,
            port: 8080,
            file_path: Arc::new(RwLock::new(Vec::new())),
            qr_code,
            server_handle: None,
            port_buffer: "8080".to_string(),
            local_host: true,
            size: (0.0, 0.0),
            clients: HashMap::new(),
            show_connections: true,
            transmitted_data: 0,
            block_external_connections: Arc::new(AtomicBool::new(true)),
            active_downloads: 0,
            total_downloads: 0,
            active_connections: 0,
            throughput: 0,
            show_qr_code: true,
        }
    }
}

impl State {
    pub fn create_url_string(&self) -> String {
        if self.local_host {
            return Self::url_string(&self.ip_adress, self.port)
        }
        Self::url_string(&self.ip_adress_public.unwrap(), self.port)
    }

    fn url_string(ip: &IpAddr, port: u16) -> String {
        format!("http://{}:{}/index", ip, port)
    }

    pub fn create_qr_code(url: &String) -> widget::image::Handle {
        let size = CONNECTION_PANE_WIDTH as usize;
        let data = qrcode_generator::to_image(url, QrCodeEcc::Quartile, size).expect("Couldn't generate QR code.")
            .into_iter()
            .flat_map(|pixel| {
                vec![pixel, pixel, pixel, 255]
            }).collect::<Vec<u8>>();
        
        widget::image::Handle::from_rgba(size as u32, size as u32, data)
    }
}

