use std::{path::PathBuf, process::Command};
use copypasta::{ClipboardContext, ClipboardProvider};
use rfd::FileDialog;
use iced::Task;

use crate::{server::server, state::State};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDarkMode,
    CopyUrl,
    None,
    OpenInBrowser,
    FileDropped(std::path::PathBuf),
    ServerStopped,
    DeleteFile,
    OpenFile,
    ShowInExplorer,
    SelectPath,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleDarkMode             => state.dark_mode = !state.dark_mode,
        Message::CopyUrl                    => copy_url_to_clipboard(state),
        Message::FileDropped(path) => return start_server_with_path(state, path),
        Message::ServerStopped              => state.server_handle = None,
        Message::OpenInBrowser              => webbrowser::open(&state.create_url_string()).unwrap(),
        Message::DeleteFile                 => stop_server(state),        
        Message::OpenFile                   => open_in_explorer(state),
        Message::ShowInExplorer             => show_in_explorer(state),
        Message::SelectPath                 => return select_path(state),
        Message::None                       => {}
    }

    Task::none()
}

fn select_path(state: &mut State) -> Task<Message> {
    let path = FileDialog::new()
        .add_filter("Any", &["*"])
        .pick_file();

    if let Some(path) = path {
        return start_server_with_path(state, path);
    }
    Task::none()
}

fn show_in_explorer(state: &mut State) {
    if let Some(path) = &state.file_path {
        Command::new( "explorer" )
            .arg("/select,")
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn open_in_explorer(state: &mut State) {
    if let Some(path) = &state.file_path {
        Command::new( "explorer" )
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn stop_server(state: &mut State) {
    if let Some(handle) = &state.server_handle {
        handle.abort();
        state.server_handle = None;
    }
    state.file_path = None;
}

fn start_server_with_path(state: &mut State, path: PathBuf) -> Task<Message> {
    stop_server(state);
    state.file_path = Some(path.clone());
    let task =  Task::perform(server(state.ip_adress.unwrap(), state.port, path), |_result| Message::ServerStopped);
    let (task, handle) = Task::abortable(task);
    state.server_handle = Some(handle);
    task
}

fn copy_url_to_clipboard(state: &State) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(state.create_url_string()).unwrap();
}