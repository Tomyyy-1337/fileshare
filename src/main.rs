#![windows_subsystem = "windows"]

mod state {
    pub mod state;
    pub mod update;
    pub mod client_manager;
    pub mod file_manager;
    pub mod subscriptions;
    mod theme_selector;
}
mod views {
    pub mod root_view;
    pub mod styles;
    mod no_connection_pane;
    mod upload_pane;
    mod download_pane;
    mod footer_pane;
    mod connection_info_pane;
    pub mod language;
}
mod server {
    pub mod router;
    pub mod webpage_service;
    mod download_service;
    mod counting_stream;
}

use state::{file_manager::FileManager, subscriptions::subscription, update::update};
use views::root_view::view;
use iced::Size;

fn main() -> iced::Result { 
    let _ = iced::application("Fileshare", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            resizable: true,
            size: (Size::new(1040.0, 660.0)),
            min_size: Some(Size::new(940.0, 500.0)),
            icon: Some(iced::window::icon::from_file("./assets/icon.ico").unwrap()),
            ..iced::window::Settings::default()
        })
        .theme(|state| state.theme.get())
        .run();

    let _ = std::fs::remove_dir_all(FileManager::temp_dir());

    Ok(())
}
