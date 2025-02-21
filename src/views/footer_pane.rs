use iced::widget::{self, button, container, pick_list, row, text, tooltip};
use crate::{state::state::State, state::update::Message, views::{styles::CustomStyles, styles::color_multiply}};

use super::root_view::{H2_SIZE, P_SIZE};

pub fn footer_pane(state: &State) -> iced::Element<Message> {
    let settings_text = text!("Theme:")
        .size(H2_SIZE);

    let theme_button = pick_list(state.theme.available_themes(), Some(state.theme.get()),Message::ThemeChanged)
        .style(CustomStyles::pick_list);

    let theme_button = tooltip(
        theme_button,
        container(text("You can change the theme of the application using the up and down arrow keys.").size(P_SIZE))
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
            format!("Invaid port number. Please enter a number between 0 and 65535. (Active Port: {})", state.port)
        },
        Ok(n) if state.port != n => {
            port_text = port_text.style(CustomStyles::textfield_background(color_multiply(state.theme.get().palette().primary, 0.8)));
            format!("Press Enter to change the port. (Active Port: {})", state.port)
        }  
        _ => {
            port_text = port_text.style(CustomStyles::textfield_background(state.theme.get().palette().background)); 
            "Change the port the server is running on. If you want to serve the files on the internet, make sure to open the port in your router settings.".to_owned()
        }
    };

    if state.client_manager.active_downloads() > 0 {
        port_tooltip = format!("Cannot change the port while downloads are active. (Active Port: {})", state.port);
    }

    let port_tooltip = text(port_tooltip)
        .size(P_SIZE);

    let port_text = tooltip( 
        port_text,
        container(port_tooltip)
            .padding(10)
            .width(iced::Length::Fixed(300.0))
            .style(container::rounded_box),
        tooltip::Position::Top
    );

    let text_view = text!("Settings:")
        .size(H2_SIZE);

    let connection_info = match state.show_connections {
        true => "Hide Connections",
        false => "Show Connections"
    };
    let button_connection_info = button(connection_info)
        .on_press(Message::ToggleConnectionsView)
        .width(iced::Length::Fixed(160.0));

    let footer = row![
        text_view,
        button_connection_info,
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