use std::{path::PathBuf, process::Command, thread::{sleep}};
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
    DeleteFile(usize),
    OpenFile(usize),
    ShowInExplorer(usize),
    SelectPath,
    DeleteAllFiles,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleDarkMode             => state.dark_mode = !state.dark_mode,
        Message::CopyUrl                    => copy_url_to_clipboard(state),
        Message::FileDropped(path) => return add_path(state, path),
        Message::ServerStopped              => state.server_handle = None,
        Message::OpenInBrowser              => webbrowser::open(&state.create_url_string()).unwrap(),
        Message::DeleteFile(indx)    => return delete_file(state, indx),        
        Message::OpenFile(indx)      => open_in_explorer(state, indx),
        Message::ShowInExplorer(indx)=> show_in_explorer(state, indx),
        Message::SelectPath                 => return select_path(state),
        Message::DeleteAllFiles             => delete_all_files(state),
        Message::None                       => {}
    }

    Task::none()
}

fn select_path(state: &mut State) -> Task<Message> {
    let path = FileDialog::new()
        .add_filter("Any", &["*"])
        .pick_file();

    if let Some(path) = path {
        return add_path(state, path);
    }
    Task::none()
}

fn show_in_explorer(state: &mut State, indx: usize) {
    if let Some(path) = &state.file_path.get(indx) {
        Command::new( "explorer" )
            .arg("/select,")
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn open_in_explorer(state: &mut State, indx: usize) {
    if let Some(path) = &state.file_path.get(indx) {
        Command::new( "explorer" )
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn delete_all_files(state: &mut State) {
    state.file_path.clear();
    stop_server(state);
}

fn stop_server(state: &mut State) {
    if let Some(handle) = &state.server_handle {
        handle.abort();
        state.server_handle = None;
    }
}

fn delete_file(state: &mut State, indx: usize) -> Task<Message> {
    state.file_path.remove(indx);
    stop_server(state);
    if !state.file_path.is_empty() {
        sleep(std::time::Duration::from_millis(1));
        return start_server(state);
    } 
    Task::none()
}

fn add_path(state: &mut State, path: PathBuf) -> Task<Message> {
    if !path.is_file() || state.file_path.contains(&path) {
        return Task::none();
    }

    stop_server(state);
    state.file_path.push(path);
    state.file_path.sort();
    sleep(std::time::Duration::from_millis(1));
    start_server(state)
}

fn start_server(state: &mut State) -> Task<Message> {
    let task =  Task::perform(server(state.ip_adress.unwrap(), state.port, state.file_path.clone()), |_result| Message::ServerStopped);
    let (task, handle) = Task::abortable(task);
    state.server_handle = Some(handle);
    task
}

fn copy_url_to_clipboard(state: &State) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(state.create_url_string()).unwrap();
}