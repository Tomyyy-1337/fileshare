use std::time::Duration;

use iced::widget::{column, container, horizontal_rule, row, text, tooltip, Space};
use crate::{server::webpage_service::size_string, state::{client_manager::ClientState, state::State}, state::update::Message, views::styles::CustomStyles};

use super::root_view::{CONNECTION_PANE_WIDTH, H1_SIZE, P_SIZE};

pub fn connection_info_pane(state: &State) -> iced::Element<Message> {
    let text_connections = text(state.language.connections())
        .size(H1_SIZE)
        .align_x(iced::alignment::Horizontal::Center)
        .width(iced::Length::Fill);

    let mut connections = column![];

    let clients = state.client_manager.sorted_clients();

    for (indx, (ip, client_info)) in clients.iter().enumerate() {

        let color = match client_info.state {
            ClientState::Downloading => state.theme.get().palette().primary,
            ClientState::Connected => state.theme.get().palette().success,
            ClientState::Disconnected => state.theme.get().palette().danger,
        };

        let text_ip = text!("{}", ip.to_string())
            .size(P_SIZE)
            .width(iced::Length::Fill)
            .color(color);

        let text_count = column![
            text!("{} Downloads", client_info.download_count).size(12),
            text!("of size {}", size_string(client_info.download_size)).size(12),
            text!("{}/s", size_string(client_info.speed)).size(12)	
        ]
        .width(iced::Length::Shrink)
        .align_x(iced::alignment::Horizontal::Right);

        let conection = row![text_ip, text_count]
            .align_y(iced::alignment::Vertical::Center);

        let progress_bar = iced::widget::progress_bar(
            0.0..=client_info.current_downloads_size as f32,
            client_info.current_download_progress as f32
        ).height(8.0)
        .style(CustomStyles::progress_bar);

        let progress_bar = container(progress_bar)
            .padding(2);

        let mut conection = column![
            Space::new(iced::Length::Shrink, iced::Length::Fixed(12.0)),
            conection
        ];
        
        if client_info.state == ClientState::Downloading {
            conection = conection.push(progress_bar);
        } else {
            conection = conection.push(Space::new(iced::Length::Shrink, iced::Length::Fixed(12.0)));
        }

        let conection = row![
            Space::new(iced::Length::Fixed(12.0), iced::Length::Shrink),
            conection,
            Space::new(iced::Length::Fixed(12.0), iced::Length::Shrink)
        ];

        let factor = if indx & 1 == 0 { 0.9 } else { 0.7 };
        let conection = container(conection)
            .padding(2)
            .style(CustomStyles::darker_background(factor));

        let last_connection_text = match client_info.state {
            ClientState::Downloading => state.language.downloading_tooltip(size_string(client_info.max_speed), size_string(client_info.current_download_progress), size_string(client_info.current_downloads_size)),
            ClientState::Connected => {
                let last = if client_info.download_count > 0 {
                    state.language.last_download(format_time(client_info.last_download.elapsed()), size_string(client_info.max_speed))
                } else {
                    "".to_owned()
                };
                format!("{}\n{}",state.language.connected(), last)
            },
            ClientState::Disconnected => {
                let last = if client_info.download_count > 0 {
                    state.language.last_download(format_time(client_info.last_download.elapsed()), size_string(client_info.max_speed))
                } else {
                    "".to_owned()
                };
                format!("{}\n{}", state.language.last_seen(format_time(client_info.last_connection.elapsed())), last)
            },
        };

        let tooltip_conection = text(last_connection_text)
            .size(P_SIZE);

        let conection = tooltip(
            conection,
            container(tooltip_conection)
                .padding(10)
                .width(iced::Length::Shrink)
                .style(container::rounded_box),
            tooltip::Position::Bottom
        );

        connections = connections.push(conection);
    }

    let connections: iced::Element<Message> = iced::widget::scrollable(connections).style(CustomStyles::scrollable)
        .height(iced::Length::Fill).into();

    let connections = container(connections)
        .width(iced::Length::Fill)
        .padding(1)
        .style(CustomStyles::container_border(state.client_manager.active_downloads() > 0));

    let stats_text = text(state.language.stats())
        .size(H1_SIZE)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

    let name_column = column![
        text(state.language.active_downlaods()).size(P_SIZE).width(iced::Length::Shrink),
        text(state.language.active_clients()).size(P_SIZE).width(iced::Length::Shrink),
        text(state.language.total_clients()).size(P_SIZE).width(iced::Length::Shrink),
        text(state.language.total_downloads()).size(P_SIZE).width(iced::Length::Shrink),
        text(state.language.current_upload()).size(P_SIZE).width(iced::Length::Shrink),
        text(state.language.transmitted_data()).size(P_SIZE).width(iced::Length::Shrink),
    ]
    .spacing(5);

    let value_column = column![
        text!("{}", state.client_manager.active_downloads()).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.client_manager.active_connections()).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.client_manager.num_clients()).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.client_manager.total_downloads()).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}/s", size_string(state.client_manager.throughput())).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", size_string(state.client_manager.transmitted_data())).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
    ]
    .spacing(5);

    let stats_row = row![name_column, value_column]
        .spacing(5)
        .width(iced::Length::Fill);

    let connections = column![
        text_connections, 
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
        connections, 
        stats_text,
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
        stats_row
    ]
    .padding(5)
    .spacing(10);

    let connections = container(connections)
        .style(CustomStyles::darker_background(0.8))
        .width(iced::Length::Fixed(CONNECTION_PANE_WIDTH))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    connections.into()
}

pub fn format_time(time: Duration) -> String {
    let secs = time.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    let secs = secs % 60;
    let mins = mins % 60;
    if hours > 0 {
        return format!("{:02}h {:02}m {:02}s", hours, mins, secs);
    }
    if mins > 0 {
        return format!("{:02}m {:02}s", mins, secs);
    }
    format!("{:02}s", secs)
}