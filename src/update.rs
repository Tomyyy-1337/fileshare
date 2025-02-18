use std::{path::PathBuf, process::Command, thread::sleep};
use copypasta::{ClipboardContext, ClipboardProvider};
use rfd::FileDialog;
use iced::{stream::channel, window::Event, Size, Task};

use crate::{server::{self, server, ServerMessage}, state::{ClientInfo, FileInfo, State}};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDarkMode,
    CopyUrl,
    None,
    OpenInBrowser,
    DeleteFile(usize),
    OpenFile(usize),
    ShowInExplorer(usize),
    SelectFilesExplorer,
    SelectFolderExplorer,
    DeleteAllFiles,
    Localhost,
    PublicIp,
    ChangePort,
    PortTextUpdate(String),
    ToggleConnectionsView,
    BlockExternalConnections(bool),
    ServerMessage(server::ServerMessage),
    UpdateSpeed,
    ShowQrCode(bool),
    WindowEvent(iced::window::Event),
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleDarkMode => state.dark_mode = !state.dark_mode,

        Message::ToggleConnectionsView => state.show_connections = !state.show_connections,

        Message::ShowQrCode(show) => state.show_qr_code = show,

        Message::BlockExternalConnections(block) => {
            state.block_external_connections.store(block, std::sync::atomic::Ordering::Relaxed);
        },

        Message::OpenInBrowser => webbrowser::open(&state.create_url_string()).unwrap(),

        Message::CopyUrl => {
            let mut ctx = ClipboardContext::new().unwrap();
            ctx.set_contents(state.create_url_string()).unwrap();
        }
        
        Message::PortTextUpdate(port) => {
            match port.parse::<u16>() {
                Err(_) if !port.is_empty() => {},
                _ => state.port_buffer = port,        
            }
        },
        
        Message::PublicIp => {
            if state.ip_adress_public.is_some() {
                state.local_host = false;
                state.qr_code = State::create_qr_code(&state.create_url_string());
                state.block_external_connections.store(false, std::sync::atomic::Ordering::Relaxed);
            } else {
                state.ip_adress_public = public_ip_address::perform_lookup(None).map(|lookup|lookup.ip).ok();
            }
        },

        Message::Localhost => {
            state.local_host = true;
            state.qr_code = State::create_qr_code(&state.create_url_string());
        },
        
        Message::DeleteFile(indx) => {
            state.file_path.write().unwrap().remove(indx);
            if state.file_path.read().unwrap().is_empty() {
                stop_server(state);
            } 
        },        

        Message::DeleteAllFiles => {
            state.file_path.write().unwrap().clear();
            stop_server(state);
        },

        Message::OpenFile(indx) => {
            if let Some(FileInfo { path, .. }) = &state.file_path.read().unwrap().get(indx) {
                Command::new( "explorer" )
                    .arg(path)
                    .spawn( )
                    .unwrap( );
            }
        },

        Message::ShowInExplorer(indx) => {
            if let Some(FileInfo { path, .. }) = &state.file_path.read().unwrap().get(indx) {
                Command::new( "explorer" )
                    .arg("/select,")
                    .arg(path)
                    .spawn( )
                    .unwrap( );
            }
        },

        Message::SelectFolderExplorer => {
            let paths: Option<Vec<PathBuf>> = FileDialog::new()
                .add_filter("Any", &["*"])
                .pick_folders();
            if let Some(paths) = paths {
                return add_files_from_path_list(state, paths);
            }
        },
        
        Message::ChangePort => {
            let port = state.port_buffer.parse::<u16>();
            if port.is_err() {
                return Task::none();
            }
            let port = port.unwrap();
            if port == state.port {
                return Task::none();
            }
            state.port = port;
            state.qr_code = State::create_qr_code(&state.create_url_string());
            if let Some(handle) = &state.server_handle {
                handle.abort();
                state.server_handle = None;
                sleep(std::time::Duration::from_millis(50));
                return start_server(state);
            }
        },

        Message::SelectFilesExplorer => {
            let paths: Option<Vec<PathBuf>> = FileDialog::new()
                .add_filter("Any", &["*"])
                .pick_files();
            if let Some(paths) = paths {
                return add_files_from_path_list(state, paths);
            } 
        },
        
        Message::WindowEvent(Event::Resized(Size { width, height })) => state.size = (width as f32, height as f32),

        Message::WindowEvent(Event::FileDropped(path)) => {
            if let Some(task) = add_files_from_path(state, path) {
                return task;
            }
        },

        Message::ServerMessage(ServerMessage::Downloaded { index, ip }) => {
            state.file_path.write().unwrap()[index].download_count += 1;
            state.clients.entry(ip).and_modify(|client| {
                client.download_count += 1;
                client.last_connection = std::time::Instant::now();
            });

            state.total_downloads += 1;
        },

        Message::ServerMessage(ServerMessage::ClientConnected { ip }) => {
            let len = state.clients.len();
            state.clients
                .entry(ip)
                .and_modify(|client| client.last_connection = std::time::Instant::now())
                .or_insert(ClientInfo { 
                    index: len,
                    download_count: 0, 
                    last_connection: std::time::Instant::now(), 
                    download_size: 0, 
                    last_download: std::time::Instant::now() - std::time::Duration::from_secs(10), 
                    received_data: 0, 
                    speed: 0, 
                    max_speed: 0 
                });

            state.active_connections = state.clients
                .iter()
                .filter(|(_, client)| client.last_connection.elapsed().as_millis() < 3500)
                .count();

            state.active_downloads = state.clients
                .iter()
                .filter(|(_, client)| client.last_download.elapsed().as_millis() < 3500)
                .count();
        },

        Message::ServerMessage(ServerMessage::DownloadActive { ip, num_packets }) => {
            state.clients.entry(ip).and_modify(|client| {
                client.last_connection = std::time::Instant::now();
                client.last_download = std::time::Instant::now();
                client.received_data += num_packets;
                client.download_size += num_packets * 4096;
            });

            state.transmitted_data += num_packets * 4096;
        },

        Message::UpdateSpeed => {
            for (_, client) in state.clients.iter_mut() {
                client.speed = client.received_data * 4096;
                client.received_data = 0;
                client.max_speed = client.speed.max(client.max_speed);
            }
            state.throughput = state.clients.iter().map(|(_, client)| client.speed).sum();
        },

        Message::WindowEvent(_) => {},
        Message::None => {}
    }

    Task::none()
}

fn find_files_recursive(path: &PathBuf, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                find_files_recursive(&path, files);
            } else {
                files.push(path);
            }
        }
    } else {
        files.push(path.clone());
    }
}

fn find_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    find_files_recursive(path, &mut files);
    files
}

fn add_files_from_path_list(state: &mut State, paths: Vec<PathBuf>) -> Task<Message> {
    let mut return_tasks = Task::none();
    for path in paths {
        if let Some(task) = add_files_from_path(state, path) {
            return_tasks = task
        }
    }
    return_tasks
}

fn add_files_from_path(state: &mut State, path: PathBuf) -> Option<Task<Message>> {
    let paths = find_files(&path);

    for file in paths {
        if state.file_path.read().unwrap().iter().find(| FileInfo { path, .. }| path == &file).is_some() {
            continue;
        }
        state.file_path.write().unwrap().insert(0, FileInfo { 
            path: file.clone(),
            size: file.metadata().unwrap().len() as usize,
            download_count: 0,
        });
    } 
    if state.server_handle.is_none() {
        return Some(start_server(state));
    }
    None
}

fn start_server(state: &mut State) -> Task<Message> {
    if state.file_path.read().unwrap().is_empty() {
        return Task::none();
    }
    let filepaths = state.file_path.clone();
    let block_external_connections = state.block_external_connections.clone();
    let ip_adress = state.ip_adress;
    let port = state.port;
    let stream = channel(10, move |tx: futures::channel::mpsc::Sender<_>| {
        let tx = tx.clone();
        async move {
            server(ip_adress, port, filepaths, tx, block_external_connections).await;
        }
    });

    let task = Task::run(stream, |server_message| {
        Message::ServerMessage(server_message)
    });

    let (task, handle) = Task::abortable(task);
    state.server_handle = Some(handle);
    task
}

fn stop_server(state: &mut State) {
    if let Some(handle) = &state.server_handle {
        handle.abort();
        state.server_handle = None;
    }
}