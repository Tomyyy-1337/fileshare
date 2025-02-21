use std::{net::IpAddr, path::PathBuf, process::Command, thread::sleep};
use copypasta::{ClipboardContext, ClipboardProvider};
use local_ip_address::local_ip;
use rfd::FileDialog;
use iced::{stream::channel, window::Event, Size, Task};

use crate::{server::router::server, state::{file_manager::FileInfo, state::State}};

#[derive(Debug, Clone)]
pub enum ServerMessage {
    Downloaded { index: usize , ip: IpAddr },
    ClientConnected { ip: IpAddr },
    DownloadActive { ip: IpAddr, num_packets: usize },
    DownloadRequest { index: usize, ip: IpAddr },
    DownloadAllRequest { ip: IpAddr },
}

#[derive(Debug, Clone)]
pub enum Message {
    ServerMessage(ServerMessage),
    ThemeChanged(iced::Theme),
    NextTheme,
    PreviousTheme,
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
    Refresh,
    ShowQrCode(bool),
    WindowEvent(iced::window::Event),
    RetryIp
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::RetryIp => {
            state.ip_adress = local_ip().ok();
            if state.ip_adress.is_some() {
                state.qr_code = State::create_qr_code(&state.create_url_string());
            }
        },

        Message::ThemeChanged(theme) => {
            state.theme.set(&theme);
            *state.theme.get_arc().write().unwrap() = state.theme.get();
            state.backup_state();
        },
        Message::NextTheme => {
            state.theme.next();
            state.backup_state();
            *state.theme.get_arc().write().unwrap() = state.theme.get();
        }
        Message::PreviousTheme => {
            state.theme.previous();
            state.backup_state();
            *state.theme.get_arc().write().unwrap() = state.theme.get();
        }

        Message::ToggleConnectionsView => {
            state.show_connections = !state.show_connections;
            state.backup_state();
        },

        Message::ShowQrCode(show) => {
            state.show_qr_code = show;
            state.backup_state();
        },

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
            state.file_manager.remove(indx);
            if state.file_manager.get_view().is_empty() {
                stop_server(state);
            } 
        },        

        Message::DeleteAllFiles => {
            state.file_manager.clear();
            stop_server(state);
        },

        Message::OpenFile(indx) => {
            if let Some(FileInfo { path, .. }) = &state.file_manager.get(indx) {
                open::that(path).unwrap();
            }
        },

        Message::ShowInExplorer(indx) => {
            if let Some(FileInfo { path, .. }) = &state.file_manager.get(indx) {
                #[cfg(target_os = "windows")]
                let _result = Command::new("explorer")
                    .arg("/select,")
                    .arg(path)
                    .spawn();
            
                #[cfg(target_os = "macos")]
                let _result = Command::new("open")
                    .arg("-R")
                    .arg(path)
                    .spawn();
            
                #[cfg(target_os = "linux")]
                let _result = Command::new("xdg-open")
                    .arg(path.parent().unwrap_or_else(|| Path::new("/")))
                    .spawn();
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
            state.backup_state();
            state.qr_code = State::create_qr_code(&state.create_url_string());
            if let Some(handle) = &state.server_handle {
                handle.abort();
                state.server_handle = None;
                sleep(std::time::Duration::from_millis(100));
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

        Message::ServerMessage(ServerMessage::DownloadAllRequest { ip }) => {
            let file_path = state.file_manager.get_view();
            let file_size = file_path.iter().map(|(_, file)| file.size).sum();

            state.client_manager.add_download(ip, file_size);
        },

        Message::ServerMessage(ServerMessage::Downloaded { index, ip }) => {
            state.file_manager.increment_download_count(index);
            state.client_manager.download_done(ip);
        },

        Message::ServerMessage(ServerMessage::ClientConnected { ip }) => {
            state.client_manager.add_connection(ip);
        },

        Message::ServerMessage(ServerMessage::DownloadRequest { index, ip } ) => {
            let size = state.file_manager.get(index).map(|file| file.size).unwrap_or(0);
            state.client_manager.add_download(ip, size);
        },

        Message::ServerMessage(ServerMessage::DownloadActive { ip, num_packets }) => {
            state.client_manager.download_progress(ip, num_packets);
        },

        Message::Refresh => {
            match local_ip() {
                Ok(ip) if Some(ip) == state.ip_adress => {},
                Ok(ip) => {
                    stop_server(state);
                    state.ip_adress = Some(ip);
                    state.qr_code = State::create_qr_code(&state.create_url_string());
                    sleep(std::time::Duration::from_millis(200));
                    return start_server(state);
                },
                Err(_) => {
                    state.ip_adress = None;
                    stop_server(state);
                }
            }

            state.client_manager.update();
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
        if state.file_manager.get_view().iter().find(| (_, FileInfo { path, .. })| path == &file).is_some() {
            continue;
        }
        state.file_manager.push(file.clone(), file.metadata().unwrap().len() as usize);
    } 
    if state.server_handle.is_none() {
        return Some(start_server(state));
    }
    None
}

fn start_server(state: &mut State) -> Task<Message> {
    if state.file_manager.get_view().is_empty() {
        return Task::none();
    }
    if state.ip_adress.is_none() || state.ip_adress != local_ip().ok() {
        return Task::none();
    }

    let filepaths = state.file_manager.get_arc();
    let block_external_connections = state.block_external_connections.clone();
    let ip_adress = state.ip_adress;
    let port = state.port;
    let current_theme = state.theme.get_arc().clone();
    let stream = channel(10, move |tx: futures::channel::mpsc::Sender<_>| {
        let tx = tx.clone();
        async move {
            server(ip_adress.unwrap(), port, filepaths, tx, block_external_connections, current_theme.clone()).await;
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