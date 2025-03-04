use iced::widget::{self, container, pick_list, row, text, tooltip};
use crate::{state::state::State, state::update::Message, views::{styles::CustomStyles, styles::color_multiply}};

use super::{language::Language, root_view::{H2_SIZE, P_SIZE}};

pub fn footer_pane(state: &State) -> iced::Element<Message> {
    let settings_text = text!("Theme:")
        .size(H2_SIZE);

    let theme_button = pick_list(state.theme.available_themes(), Some(state.theme.get()),Message::ThemeChanged)
        .style(CustomStyles::pick_list);

    let theme_button = tooltip(
        theme_button,
        container(text(state.language.theme_tooltip()).size(P_SIZE))
            .padding(10)
            .width(iced::Length::Fixed(180.0))
            .style(container::rounded_box),
        tooltip::Position::Right
    );

    
    let port_title = text!("Port:")
        .size(H2_SIZE);

    let mut port_text = widget::text_input("Port", &state.port_buffer)
        .width(iced::Length::Fixed(100.0));

    if state.client_manager.active_downloads() == 0 {
        port_text = port_text.on_submit(Message::ChangePort);
        port_text = port_text.on_input(Message::PortTextUpdate);
    }

    let mut port_tooltip = match state.port_buffer.parse::<u16>() {
        Err(_) => {
            port_text = port_text.style(CustomStyles::textfield_background(state.theme.get().palette().danger));
            state.language.invalid_port(state.port)
        },
        Ok(n) if state.port != n => {
            port_text = port_text.style(CustomStyles::textfield_background(color_multiply(state.theme.get().palette().primary, 0.8)));
            state.language.change_port(state.port)
        }  
        _ => {
            port_text = port_text.style(CustomStyles::textfield_background(state.theme.get().palette().background)); 
            state.language.standard_port().to_string()
        }
    };

    if state.client_manager.active_downloads() > 0 {
        port_tooltip = state.language.locked_port(state.port);
    }

    let port_tooltip = text(port_tooltip)
        .size(P_SIZE);

    let port_text = tooltip( 
        port_text,
        container(port_tooltip)
            .padding(10)
            .width(iced::Length::Fixed(250.0))
            .style(container::rounded_box),
        tooltip::Position::Top
    );

    let text_view = text(state.language.language())
        .size(H2_SIZE)
        .width(iced::Length::Fixed(100.0))
        .align_x(iced::alignment::Horizontal::Right);

    let language_button = pick_list(Language::all_variants(), Some(state.language), Message::LanguageChanged)
        .style(CustomStyles::pick_list);

    let footer = row![
        text_view,
        language_button,
        settings_text,
        theme_button,
        port_title,
        port_text,
    ]
    .spacing(20)
    .padding(10)
    .width(iced::Length::Shrink)
    .align_y(iced::alignment::Vertical::Center);

    let footer = container(footer)
        .style(CustomStyles::darker_background(0.8))
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

    footer.into()
}