use std::{fs::{read_to_string, File}, io::Write, net::{IpAddr, Ipv4Addr}, sync::{atomic::AtomicBool, Arc}, vec};
use local_ip_address::local_ip;
use iced::widget;
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};

use crate::{state::client_manager::ClientManager, state::file_manager::FileManager, state::theme_selector::ThemeSelector, views::root_view::CONNECTION_PANE_WIDTH};

pub struct State {
    pub theme: ThemeSelector,
    pub client_manager: ClientManager,
    pub file_manager: FileManager,
    pub ip_adress: Option<IpAddr>,
    pub ip_adress_public: Option<IpAddr>,
    pub port: u16,
    pub qr_code: widget::image::Handle,
    pub server_handle: Option<iced::task::Handle>,
    pub port_buffer: String,
    pub local_host: bool,
    pub size: (f32, f32),
    pub show_connections: bool,
    pub block_external_connections: Arc<AtomicBool>,
    pub show_qr_code: bool,
}

impl Default for State {
    fn default() -> Self {
        let ip = local_ip().ok();
        let ip_public = public_ip_address::perform_lookup(None).map(|lookup|lookup.ip).ok();
        let config_path = format!("{}/config.json", config_path());
        
        let mut theme = ThemeSelector::new();
        let mut port = 8080;
        let mut show_connections = true;
        let mut show_qr_code = true;
        let mut port_buffer = "8080".to_string();
        
        if let Ok(file) = read_to_string(config_path) {
            let json = serde_json::from_str::<PersistantState>(&file);
            if let Ok(data) = json {
                theme.set_indx(data.theme);
                port = data.port;
                port_buffer = port.to_string();
                show_connections = data.show_connections;
                show_qr_code = data.show_qr_code;
            }   
        }
        
        let qr_code = Self::create_qr_code(&Self::url_string(&ip.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))), port));
        
        Self {
            theme,
            client_manager: ClientManager::default(),
            ip_adress: ip,
            ip_adress_public: ip_public,
            port,
            file_manager: FileManager::new(),
            qr_code,
            server_handle: None,
            port_buffer,
            local_host: true,
            size: (0.0, 0.0),
            show_connections,
            block_external_connections: Arc::new(AtomicBool::new(true)),
            show_qr_code,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PersistantState {
    theme: usize,
    port: u16,
    show_connections: bool,
    show_qr_code: bool
}

impl State {
    pub fn create_url_string(&self) -> String {
        if self.local_host {
            return Self::url_string(&self.ip_adress.unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))), self.port);
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

    pub fn backup_state(&self) {
        let persistant_state = PersistantState {
            theme: self.theme.get_indx(),
            port: self.port,
            show_connections: self.show_connections,
            show_qr_code: self.show_qr_code
        };
        let config_path = config_path();
        let json = serde_json::to_string(&persistant_state).unwrap();
        
        let _ = std::fs::create_dir_all(&config_path);

        if let Ok(mut file) = File::create(format!("{}/config.json",config_path)) {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

fn config_path() -> String {
    #[cfg(feature = "appdata")]
    {
        let appdata_path = std::env::var("APPDATA").unwrap_or_else(|_| String::from("./appdata/config"));
        return format!("{}/Fileshare", appdata_path);
    }
    #[cfg(not(feature = "appdata"))]
    {
        return String::from("./config");
    }
}

