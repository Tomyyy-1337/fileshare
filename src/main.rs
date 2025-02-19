#![windows_subsystem = "windows"]

use std::sync::Arc;

use iced::{theme::{Custom, Palette}, Size, Theme};

mod state;

mod update;
use update::update;

mod view;
use view::view;

mod server;

use subscriptions::subscription;
mod subscriptions;

mod styles;

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
        .theme(|state| match state.dark_mode {
            true => Theme::Dracula,
            false => Theme::Custom(Arc::new(Custom::new("Light".to_string(), Palette {
                background: iced::Color::WHITE,
                text: iced::Color::BLACK,
                primary: iced::Color::from_rgb8(159, 99, 246),
                success: iced::Color::from_rgb8(0, 120, 212),
                danger: iced::Color::from_rgb8(255, 0, 0),
            })))
        })
        .run()
}
