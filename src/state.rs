use std::{collections::HashMap, fs::{read_to_string, File}, io::Write, net::{IpAddr, Ipv4Addr}, path::PathBuf, sync::{atomic::AtomicBool, Arc, RwLock}, vec};
use local_ip_address::local_ip;
use iced::{theme::{Custom, Palette}, widget, Theme};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};

use crate::view::CONNECTION_PANE_WIDTH;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
    pub download_count: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClientState {
    Connected,
    Downloading,
    Disconnected,
}

pub struct ClientInfo {
    pub index: usize,
    pub download_count: usize,
    pub download_size: usize,
    pub last_connection: std::time::Instant,
    pub last_download: std::time::Instant,
    pub received_data: usize,
    pub speed: usize,
    pub max_speed: usize,
    pub current_downloads_size: usize,
    pub state: ClientState,
    pub current_download_progress: usize,
}

pub struct State {
    pub theme: ThemeSelector,
    pub current_theme: Arc<RwLock<Theme>>,
    pub ip_adress:Option<IpAddr>,
    pub ip_adress_public: Option<IpAddr>,
    pub port: u16,
    pub file_path: Arc<RwLock<HashMap<usize, FileInfo>>>,
    pub file_index: usize,
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
            current_theme: Arc::new(RwLock::new(theme.get())),
            theme,
            ip_adress: ip,
            ip_adress_public: ip_public,
            port,
            file_path: Arc::new(RwLock::new(HashMap::new())),
            file_index: 0,
            qr_code,
            server_handle: None,
            port_buffer,
            local_host: true,
            size: (0.0, 0.0),
            clients: HashMap::new(),
            show_connections,
            transmitted_data: 0,
            block_external_connections: Arc::new(AtomicBool::new(true)),
            active_downloads: 0,
            total_downloads: 0,
            active_connections: 0,
            throughput: 0,
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
            theme: self.theme.indx,
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

pub struct ThemeSelector {
    indx: usize,
    themes: [iced::Theme; 20],
}

impl ThemeSelector {
    fn set_indx(&mut self, indx: usize) {
        self.indx = indx;
    }

    pub fn get(&self) -> iced::Theme {
        self.themes[self.indx].clone()
    }

    pub fn next(&mut self) {
        self.indx = (self.indx + 1).min(self.themes.len() - 1);
    }

    pub fn previous(&mut self) {
        if let Some(val) = self.indx.checked_sub(1) {
            self.indx = val;
        }
    }

    pub fn available_themes(&self) -> &[iced::Theme] {
        &self.themes
    }

    pub fn set(&mut self, theme: &iced::Theme) {
        self.indx = self.themes.iter().position(|t| t == theme).unwrap_or(0);
    }

    fn new() -> Self {
        Self {
            indx: 17,
            themes: [
                Theme::Custom(Arc::new(Custom::new("Dracula Light".to_string(), Palette {
                    background: iced::Color::WHITE,
                    text: iced::Color::BLACK,
                    primary: iced::Color::from_rgb8(159, 99, 246),
                    success: iced::Color::from_rgb8(20, 180, 20),
                    danger: iced::Color::from_rgb8(255, 0, 0),
                }))),
                Theme::Light,
                Theme::SolarizedLight,
                Theme::CatppuccinLatte,
                Theme::GruvboxLight,
                Theme::TokyoNightLight,
                Theme::Nord,
                Theme::CatppuccinFrappe,
                Theme::CatppuccinMocha,
                Theme::Dracula,
                Theme::Dark,
                Theme::Ferra,
                Theme::GruvboxDark,
                Theme::Oxocarbon,
                Theme::TokyoNight,
                Theme::TokyoNightStorm,
                Theme::SolarizedDark,
                Theme::Nightfly,
                Theme::Moonfly,
                Theme::KanagawaDragon,
            ],
        }
    }
}



