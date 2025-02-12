use std::{net::IpAddr, path::PathBuf, sync::{Arc, Mutex}};
use local_ip_address::local_ip;
use iced::widget;
use qrcode_generator::QrCodeEcc;

pub struct State {
    pub dark_mode: bool,
    pub ip_adress: IpAddr,
    pub ip_adress_public: Option<IpAddr>,
    pub port: u16,
    pub file_path: Arc<Mutex<Vec<(PathBuf, usize)>>>,
    pub qr_code: widget::image::Handle,
    pub server_handle: Option<iced::task::Handle>,
    pub port_buffer: String,
    pub local_host: bool,
    pub num_send_files: Arc<Mutex<usize>>,
}

impl Default for State {
    fn default() -> Self {
        let ip = local_ip().unwrap();
        let ip_public = public_ip_address::perform_lookup(None).map(|lookup|lookup.ip).ok();
        let qr_code = Self::create_qr_code(&Self::url_string(&ip, 8080), 1200);

        Self {
            dark_mode: true,
            ip_adress: ip,
            ip_adress_public: ip_public,
            port: 8080,
            file_path: Arc::new(Mutex::new(Vec::new())),
            qr_code,
            server_handle: None,
            port_buffer: "8080".to_string(),
            local_host: true,
            num_send_files: Arc::new(Mutex::new(0)),
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

    pub fn create_qr_code(url: &String, size: usize) -> widget::image::Handle {
        let data = qrcode_generator::to_image(url, QrCodeEcc::Quartile, size).expect("Couldn't generate QR code.")
            .into_iter()
            .flat_map(|pixel| {
                vec![pixel, pixel, pixel, 255]
            }).collect::<Vec<u8>>();
        
        widget::image::Handle::from_rgba(size as u32, size as u32, data)
    }
}

