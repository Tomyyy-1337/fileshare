use iced::keyboard::key::Named;
use iced::{keyboard, window, Size, Subscription};
use crate::state::State;

use crate::update::Message;

pub fn subscription(state: &State) -> Subscription<Message> {
    let keyboard = keyboard_input(state);
    let window = window_events();
    // let update_loop = iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::None);

    Subscription::batch([
        keyboard, 
        window,
        // update_loop,
    ])
}

fn window_events() -> Subscription<Message> {
    window::events().map(|(_, event)| match event {
        window::Event::FileDropped(path) => Message::FileDropped(path),
        window::Event::Resized(Size { width, height }) => Message::Resize(width, height),
        _ => Message::None,
    })
}

fn keyboard_input(_state: &State) -> Subscription<Message> {
    keyboard::on_key_press(|key, _modifyer| {
        match key {
            keyboard::Key::Named(Named::Space) => Some(Message::SelectFilesExplorer),
            keyboard::Key::Named(Named::Backspace) => Some(Message::DeleteAllFiles),
            _ => None,
        }
    })
}
