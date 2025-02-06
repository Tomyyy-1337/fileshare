use iced::{Size, Theme};

mod state;

mod update;
use update::update;

mod view;
use view::view;

mod server;

use subscriptions::subscription;
mod subscriptions;

fn main() -> iced::Result { 
    iced::application("Fileshare", update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            resizable: true,
            size: (Size::new(800.0, 580.0)),
            min_size: Some(Size::new(620.0, 400.0)),
            ..iced::window::Settings::default()
        })
        .theme(|state| match state.dark_mode {
            true => Theme::Dracula,
            false => Theme::Light,
        })
        .antialiasing(true)
        .run()
}
