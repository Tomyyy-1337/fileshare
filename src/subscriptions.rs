use iced::keyboard::key::Named;
use iced::{keyboard, window, Subscription};
use crate::state::state::State;

use crate::update::Message;

pub fn subscription(state: &State) -> Subscription<Message> {
    let keyboard = keyboard_input(state);
    let window = window_events();
    let refresh_loop = iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Refresh);
    let update_loop = iced::time::every(std::time::Duration::from_millis(200)).map(|_| Message::None);

    Subscription::batch([
        keyboard, 
        window,
        refresh_loop,
        update_loop
    ])
}

fn window_events() -> Subscription<Message> {
    window::events().map(|(_, event)| 
        Message::WindowEvent(event)
    )
}

fn keyboard_input(_state: &State) -> Subscription<Message> {
    keyboard::on_key_press(|key, _modifyer| {
        match key {
            keyboard::Key::Named(Named::Space) => Some(Message::SelectFilesExplorer),
            keyboard::Key::Named(Named::Backspace) => Some(Message::DeleteAllFiles),
            keyboard::Key::Named(Named::ArrowUp) => Some(Message::PreviousTheme),
            keyboard::Key::Named(Named::ArrowDown) => Some(Message::NextTheme),
            _ => None,
        }
    })
    
}
