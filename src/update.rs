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
    DeleteFile(usize),
    OpenFile(usize),
    ShowInExplorer(usize),
    SelectFilesExplorer,
    SelectFolderExplorer,
    DeleteAllFiles,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleDarkMode             => state.dark_mode = !state.dark_mode,
        Message::CopyUrl                    => copy_url_to_clipboard(state),
        Message::FileDropped(path) => return add_path(state, path),
        Message::ServerStopped              => state.server_handle = None,
        Message::OpenInBrowser              => webbrowser::open(&state.create_url_string()).unwrap(),
        Message::DeleteFile(indx)    => delete_file(state, indx),        
        Message::OpenFile(indx)      => open_in_explorer(state, indx),
        Message::ShowInExplorer(indx)=> show_in_explorer(state, indx),
        Message::SelectFilesExplorer        => return select_files(state),
        Message::SelectFolderExplorer       => return select_folders(state),
        Message::DeleteAllFiles             => delete_all_files(state),
        Message::None                       => {}
    }

    Task::none()
}

fn select_files(state: &mut State) -> Task<Message> {
    let paths: Option<Vec<PathBuf>> = FileDialog::new()
        .add_filter("Any", &["*"])
        .pick_files();
    if let Some(paths) = paths {
        return add_selected_files(state, paths);
    } 
    Task::none()
}


fn select_folders(state: &mut State) -> Task<Message> {
    let paths: Option<Vec<PathBuf>> = FileDialog::new()
        .add_filter("Any", &["*"])
        .pick_folders();
    if let Some(paths) = paths {
        return add_selected_files(state, paths);
    }
    Task::none()
}

fn add_selected_files(state: &mut State, paths: Vec<PathBuf>) -> Task<Message> {
    let mut tasks = Vec::new();
    for path in paths {
        tasks.push(add_path(state, path));
    }
    Task::batch(tasks)
}

fn show_in_explorer(state: &mut State, indx: usize) {
    if let Some(path) = &state.file_path.lock().unwrap().get(indx) {
        Command::new( "explorer" )
            .arg("/select,")
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn open_in_explorer(state: &mut State, indx: usize) {
    if let Some(path) = &state.file_path.lock().unwrap().get(indx) {
        Command::new( "explorer" )
            .arg(path)
            .spawn( )
            .unwrap( );
    }
}

fn delete_all_files(state: &mut State) {
    state.file_path.lock().unwrap().clear();
    stop_server(state);
}

fn stop_server(state: &mut State) {
    if let Some(handle) = &state.server_handle {
        handle.abort();
        state.server_handle = None;
    }
}

fn delete_file(state: &mut State, indx: usize) {
    state.file_path.lock().unwrap().remove(indx);
    if state.file_path.lock().unwrap().is_empty() {
        stop_server(state);
    } 
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

fn add_path(state: &mut State, path: PathBuf) -> Task<Message> {
    let mut paths = Vec::new();
    find_files_recursive(&path, &mut paths);

    let mut task = Task::none();
    for file in paths {
        if state.file_path.lock().unwrap().contains(&file) {
            continue;
        }
        state.file_path.lock().unwrap().push(file);
        state.file_path.lock().unwrap().sort();
        if state.server_handle.is_none() {
            task = start_server(state);
        }
    } 

    task
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