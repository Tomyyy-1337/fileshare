use std::{net::IpAddr, path::PathBuf};
use local_ip_address::local_ip;
use iced::widget;
use qrcode_generator::QrCodeEcc;

pub struct State {
    pub dark_mode: bool,
    pub ip_adress: Option<IpAddr>,
    pub port: u16,
    pub file_path: Vec<PathBuf>,
    pub qr_code: widget::image::Handle,
    pub server_handle: Option<iced::task::Handle>,
}

impl Default for State {
    fn default() -> Self {
        let ip = local_ip().ok();
        let qr_code = Self::create_qr_code(&Self::url_string(&ip.unwrap(), 8080), 1200);
        Self {
            dark_mode: true,
            ip_adress: ip,
            port: 8080,
            file_path: Vec::new(),
            qr_code,
            server_handle: None,
        }
    }

}
impl State {
    pub fn create_url_string(&self) -> String {
        Self::url_string(&self.ip_adress.unwrap(), self.port)
    }

    fn url_string(ip: &IpAddr, port: u16) -> String {
        format!("http://{}:{}/index", ip, port)
    }

    fn create_qr_code(url: &String, size: usize) -> widget::image::Handle {
        let data = qrcode_generator::to_image(url, QrCodeEcc::Medium, size).expect("Couldn't generate QR code.")
            .into_iter()
            .flat_map(|pixel| {
                vec![pixel, pixel, pixel, 255]
            }).collect::<Vec<u8>>();
        
        widget::image::Handle::from_rgba(size as u32, size as u32, data)
    }
}

