use iced::{widget::{button, column, container, horizontal_rule, text}, Length};
use crate::{state::state::State, views::styles::CustomStyles, update::Message};

use super::root_view::{DOWNLOAD_PANE_WIDTH, H1_SIZE, H2_SIZE};

pub fn no_connection_pane(_state: &State, height: Length)  -> iced::Element<Message> {
    let collumn = column![
        text!("Network Error")
            .size(H1_SIZE)
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
        text!("Check your network connection. The fileserver is unable to start.")
            .size(H2_SIZE)
            .width(iced::Length::Fill),
        button("Retry")
            .on_press(Message::RetryIp)
            .width(iced::Length::FillPortion(1))

    ]
    .spacing(10)
    .padding(5);

    let pane = container(collumn)
        .width(iced::Length::Fill)
        .height(height)
        .padding(5)
        .style(CustomStyles::darker_background(0.8)).padding(5);

    let pane = container(pane)
        .width(iced::Length::Fixed(DOWNLOAD_PANE_WIDTH))
        .height(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

    pane.into()
}
