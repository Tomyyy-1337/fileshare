use iced::widget::{column, container, row};
use local_ip_address::local_ip;
use crate::{state::state::State, state::update::Message};

use super::{connection_info_pane::connection_info_pane, download_pane::download_pane, footer_pane::footer_pane, no_connection_pane::no_connection_pane, upload_pane::upload_pane};

pub const H1_SIZE: u16 = 30;
pub const H2_SIZE: u16 = 20;
pub const P_SIZE: u16 = 13;
pub const CONNECTION_PANE_WIDTH: f32 = 250.0;
pub const DOWNLOAD_PANE_WIDTH: f32 = 250.0;

pub fn view(state: &State) -> iced::Element<Message> {
    let max_width = 1100.0 + if state.show_connections { CONNECTION_PANE_WIDTH } else { 0.0 };

    let mut main = row![]
        .width(iced::Length::Fixed(max_width))
        .height(iced::Length::Fill)
        .padding(5)
        .spacing(5);

    if state.show_connections {
        main = main.push(connection_info_pane(state));
    }  

    main = main.push(upload_pane(state));

    if !local_ip().is_ok_and(|ip| state.ip_adress.is_some_and(|ip_2| ip == ip_2))  {
        main = main.push(no_connection_pane(state, iced::Length::Fill));
    } else if !state.file_manager.get_view().is_empty() {
        main = main.push(download_pane(state));
    } 
    
    let main = container(main)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    let main = column!(
        main,
        footer_pane(state)
    );

    main.into()
}