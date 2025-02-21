use std::{cmp::Reverse, collections::HashMap, net::IpAddr};

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
    pub canceled_download_size: usize,
    pub canceled_download_progress: usize,
    pub received_data: usize,
    pub speed: usize,
    pub max_speed: usize,
    pub current_downloads_size: usize,
    pub state: ClientState,
    pub current_download_progress: usize,
}

#[derive(Default)]
pub struct ClientManager {
    clients: HashMap<IpAddr, ClientInfo>,
    throughput: usize,
    active_connections: usize,
    active_downloads: usize,
    total_downloads: usize,
    transmitted_data: usize,
}

impl ClientManager {
    pub fn active_downloads(&self) -> usize {
        self.active_downloads
    }

    pub fn active_connections(&self) -> usize {
        self.active_connections
    }

    pub fn total_downloads(&self) -> usize {
        self.total_downloads
    }

    pub fn num_clients(&self) -> usize {
        self.clients.len()
    }

    pub fn throughput(&self) -> usize {
        self.throughput
    }

    pub fn transmitted_data(&self) -> usize {
        self.transmitted_data
    }

    pub fn sorted_clients(&self) -> Vec<(&IpAddr, &ClientInfo)> {
        let mut clients: Vec<_> = self.clients.iter().collect();
        clients.sort_by_key(|(_, client)| Reverse(client.index));
        clients
    }

    pub fn add_download(&mut self, ip: IpAddr, file_size: usize) {
        self.clients.entry(ip).and_modify(|client| {
            client.current_downloads_size += file_size;
            client.last_connection = std::time::Instant::now();
            client.last_download = std::time::Instant::now();
            client.state = ClientState::Downloading;
        });
        if self.active_downloads == 0 {
            self.active_downloads = 1;
        }
    }

    pub fn download_done(&mut self, ip: IpAddr) {
        self.clients.entry(ip).and_modify(|client| {
            client.download_count += 1;
            client.last_connection = std::time::Instant::now();
        });

        self.total_downloads += 1;
    }

    pub fn add_connection(&mut self, ip: IpAddr) {
        let len = self.clients.len();
        self.clients
            .entry(ip)
            .and_modify(|client| client.last_connection = std::time::Instant::now())
            .or_insert(ClientInfo { 
                index: len,
                download_count: 0, 
                last_connection: std::time::Instant::now(), 
                download_size: 0, 
                canceled_download_size: 0,
                canceled_download_progress: 0,
                last_download: std::time::Instant::now() - std::time::Duration::from_secs(10), 
                received_data: 0, 
                speed: 0, 
                max_speed: 0,
                current_downloads_size: 0,
                state: ClientState::Connected,
                current_download_progress: 0,
            });
    }

    pub fn download_progress(&mut self, ip: IpAddr, progress: usize) {
        self.clients.entry(ip).and_modify(|client| {
            if client.state != ClientState::Downloading {
                client.current_downloads_size = client.canceled_download_size;
                client.canceled_download_size = 0;
                client.current_download_progress = client.canceled_download_progress;
                client.canceled_download_progress = 0;
            }
            client.last_connection = std::time::Instant::now();
            client.last_download = std::time::Instant::now();
            client.received_data += progress * 4096;
            client.download_size += progress * 4096;
            client.current_download_progress += progress * 4096;
            client.state = if client.current_downloads_size == client.download_size {
                ClientState::Connected
            } else {
                ClientState::Downloading
            };
        });

        self.transmitted_data += progress * 4096;
    }

    pub fn update(&mut self) {
        let mut active = 0;
        let mut downloading = 0;

        for (_, client) in self.clients.iter_mut() {
            client.speed = client.received_data;
            client.received_data = 0;
            client.max_speed = client.speed.max(client.max_speed);

            if client.last_download.elapsed().as_millis() < 2000 {
                client.state = ClientState::Downloading;
                downloading += 1;
            } else if client.last_connection.elapsed().as_millis() < 4000 {
                client.state = ClientState::Connected;
                active += 1;
            } else {
                client.state = ClientState::Disconnected;
            }

            if client.state != ClientState::Downloading {
                client.canceled_download_size = client.current_downloads_size;
                client.canceled_download_progress = client.current_download_progress;
                client.current_downloads_size = 0;
                client.current_download_progress = 0;
            }
        }
        self.throughput = self.clients.iter().map(|(_, client)| client.speed).sum();
        self.active_connections = active + downloading;
        self.active_downloads = downloading;
    }
}