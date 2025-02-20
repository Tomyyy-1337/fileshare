// #![windows_subsystem = "windows"]

use iced::Size;

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
        .theme(|state| state.theme.get())
        .run()
}
