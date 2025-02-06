use iced::{keyboard, Subscription, window};
use crate::state::State;

use crate::update::Message;


pub fn subscription(state: &State) -> Subscription<Message> {
    let keyboard = keyboard_input(state);
    let window = window_events();

    Subscription::batch([
        keyboard, 
        window
    ])
}

fn window_events() -> Subscription<Message> {
    window::events().map(|(_, event)| match event {
        window::Event::FileDropped(path) => Message::FileDropped(path),
        _ => Message::None,
    })
}

fn keyboard_input(_state: &State) -> Subscription<Message> {
    keyboard::on_key_press(|key, _modifyer| {
        match key {
            _ => None,
        }
    })
}
