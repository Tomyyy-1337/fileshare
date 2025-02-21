#![windows_subsystem = "windows"]

mod update;
mod state {
    pub mod state;
    pub mod client_manager;
    pub mod file_manager;
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
}
mod server;
mod subscriptions;

use update::update;
use views::root_view::view;
use iced::Size;

use subscriptions::subscription;

fn main() -> iced::Result { 
    iced::application("Fileshare", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            resizable: true,
            size: (Size::new(1040.0, 660.0)),
            min_size: Some(Size::new(860.0, 500.0)),
            icon: Some(iced::window::icon::from_file("./assets/icon.ico").unwrap()),
            ..iced::window::Settings::default()
        })
        .theme(|state| state.theme.get())
        .run()
}
