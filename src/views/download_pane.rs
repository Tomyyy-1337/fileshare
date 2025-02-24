use iced::widget::{self, button, checkbox, column, container, horizontal_rule, row, text, tooltip};
use crate::{state::state::State, views::styles::CustomStyles, state::update::Message};

use super::root_view::{DOWNLOAD_PANE_WIDTH, H1_SIZE, H2_SIZE, P_SIZE};

pub fn download_pane(state: &State) -> iced::Element<Message> {
    let image = widget::image(&state.qr_code)
        .width(iced::Length::Fill);

    let url_text = text(state.language.download())
        .size(H1_SIZE);

    let url_string =  state.create_url_string();
    let url_text_field = widget::text_input("", &url_string)
        .size(P_SIZE)
        .on_input(|_| Message::None);

    let copy_button = button(state.language.copy_url())
        .on_press(Message::CopyUrl)
        .width(iced::Length::FillPortion(1));

    let browser_button = button(state.language.open_in_browser())
        .on_press(Message::OpenInBrowser)
        .width(iced::Length::FillPortion(1));

    let text_mode = match state.local_host {
        true => "Mode: Localhost",
        false => "Mode: Public IP"
    };
    let text_mode = text(text_mode)
        .size(H2_SIZE);

    let text_connection_info = text(state.language.connection_info())
        .size(H2_SIZE);

    let block_external_connections = checkbox(state.language.block_external_connections(), state.block_external_connections.load(std::sync::atomic::Ordering::Relaxed))
        .on_toggle(Message::BlockExternalConnections)
        .size(16)
        .text_size(16)
        .width(iced::Length::Fill);

    let block_external_connections_tooltip = text(state.language.block_external_connections_tooltip())
        .size(P_SIZE);

    let block_external_connections = tooltip( 
        block_external_connections,
        container(block_external_connections_tooltip)
            .padding(10)
            .width(iced::Length::Fixed(300.0))
            .style(container::rounded_box),
        tooltip::Position::Bottom
    );

    let url_buttons_row = row![
        copy_button,
        browser_button
    ]
    .spacing(5);

    let select_row = row![
        button("Localhost").on_press(Message::Localhost).width(iced::Length::FillPortion(1)),
        button("Public IP").on_press(Message::PublicIp).width(iced::Length::FillPortion(1))
    ]
    .spacing(5);

    let show_qr_code = checkbox(state.language.show_qr_code(), state.show_qr_code)
        .on_toggle(|show| Message::ShowQrCode(show))
        .size(16)
        .text_size(16)
        .width(iced::Length::Fill);

    let mut download_pane = column![
        url_text,
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
        text_mode,
        select_row,
        block_external_connections,
        text_connection_info,
        url_text_field.width(iced::Length::Fill),
        url_buttons_row,
        show_qr_code,
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    if state.show_qr_code {
        download_pane = download_pane.push(horizontal_rule(5).style(CustomStyles::horizontal_rule));
        download_pane = download_pane.push(image);
        download_pane = download_pane.push(horizontal_rule(5).style(CustomStyles::horizontal_rule));
    }

    let download_pane = container(download_pane)
        .style(CustomStyles::darker_background(0.8))
        .width(iced::Length::Fixed(DOWNLOAD_PANE_WIDTH))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    download_pane.into()
}