use std::{cmp::Reverse, time::Duration};

use iced::widget::{self, button, checkbox, column, container, horizontal_rule, hover, pick_list, row, text, tooltip, Space};
use crate::{server::size_string, state::{self, State}, styles::{color_multiply, CustomStyles}, update::Message};

const H1_SIZE: u16 = 30;
const H2_SIZE: u16 = 20;
const P_SIZE: u16 = 13;
pub const CONNECTION_PANE_WIDTH: f32 = 250.0;
const DOWNLOAD_PANE_WIDTH: f32 = 250.0;

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
    
    if state.server_handle.is_some() {
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


fn upload_pane(state: &State) -> iced::Element<Message> {
    let upload_files = text!("Upload File")
        .size(H1_SIZE);

    let url_select_button = button("Select File")
        .on_press(Message::SelectFilesExplorer)
        .width(iced::Length::FillPortion(1));

    let url_select_button2 = button("Select Folder")
        .on_press(Message::SelectFolderExplorer)
        .width(iced::Length::FillPortion(1));

    let url_select_row = row![url_select_button, url_select_button2]
        .spacing(5);

    let mut pane = column![
        upload_files,
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill);
    
    let mut file_path = state.file_path.read().unwrap()
        .iter()
        .map(|(i, f)| (*i, f.clone()))
        .collect::<Vec<_>>();
    file_path.sort_by_key(|(indx, _)| *indx);

    if !file_path.is_empty() {
        let shared_files_text = match file_path.len() {
            1 => "Shared File".to_owned(),
            _ => format!("Shared Files [{}]", file_path.len())   
        };

        let uploaded_files = text(shared_files_text)
            .size(H1_SIZE);

        let mut files_list = column![];

        for (color, (i, state::FileInfo{path, download_count, size})) in file_path.iter().cloned().rev().enumerate() {
            let text_file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            let text_file_name = text(text_file_name)
                .size(H2_SIZE)
                .height(iced::Length::Fixed(32.0))
                .width(iced::Length::Fill);

            let text_current_file = widget::text_input("", path.to_str().unwrap())
                .size(P_SIZE)
                .on_input(|_| Message::None);

            let text_current_file = container(text_current_file)
                .height(iced::Length::Fixed(32.0))
                .padding(2);
                    
            let open_button = button("Open")
                .on_press(Message::OpenFile(i))
                .width(iced::Length::FillPortion(1));

            let show_in_explorer_button = button("Show")
                .on_press(Message::ShowInExplorer(i))
                .width(iced::Length::FillPortion(1));

            let delete_button = button("Remove")
                .width(iced::Length::FillPortion(1));

            let delete_button: iced::Element<Message> = if state.active_downloads == 0 {
                delete_button.on_press(Message::DeleteFile(i)).into()
            } else {
                tooltip(delete_button, container(text("Cannot delete files while downloads are active.").size(P_SIZE))
                    .padding(10)
                    .width(iced::Length::Fixed(200.0))
                    .style(container::rounded_box),
                    tooltip::Position::Right
                ).into()
            };

            let text_download_count = text!("Downloads: {}", download_count)
                .size(P_SIZE)
                .width(iced::Length::Shrink);

            let text_size = text!("{}", size_string(size))
                .size(P_SIZE)
                .width(iced::Length::Shrink);

            let meta_col = column![text_size, text_download_count]
                .align_x(iced::alignment::Horizontal::Right)
                .width(iced::Length::Shrink);

            let row = row![
                open_button,
                show_in_explorer_button,
                delete_button
            ]
            .spacing(5);

            let row = container(row)
                .style(CustomStyles::darker_background(if i & 1 == 0 { 0.9 } else { 0.7 }));

            let title_row = row![text_file_name, meta_col]
                .spacing(5)
                .width(iced::Length::Fill)
                .align_y(iced::alignment::Vertical::Center);
            
            let col = column![
                title_row,
                text_current_file
            ];

            let col = container(col)
                .padding(12)
                .style(CustomStyles::darker_background(if color & 1 == 0 { 0.9 } else { 0.7 }));

            let col = hover(col, column![
                Space::new(iced::Length::Fill, iced::Length::Fill),
                row
            ].padding(12));

            files_list = files_list.push(col);
        }

        let files_list = iced::widget::scrollable(files_list)
            .height(iced::Length::Fill)
            .style(CustomStyles::scrollable);

        let delete_all_button = button("Remove All")
            .width(iced::Length::FillPortion(1));

        let delete_all_button: iced::Element<Message> = if state.active_downloads == 0 {
            delete_all_button.on_press(Message::DeleteAllFiles).into()
        } else {
            tooltip(delete_all_button, container(text("Cannot delete files while downloads are active.").size(P_SIZE))
                .padding(10)
                .width(iced::Length::Fixed(200.0))
                .style(container::rounded_box),
                tooltip::Position::Right
            ).into()
        };
        
        pane = pane.push(url_select_row.width(iced::Length::Fill));
        pane = pane.push(uploaded_files);
        pane = pane.push(horizontal_rule(5).style(CustomStyles::horizontal_rule)
        );
        pane = pane.push(files_list);
        pane = pane.push(delete_all_button);
        pane = pane.align_x(iced::alignment::Horizontal::Center);
    } else {
        pane = pane.push(text!("No file selected!")
            .size(P_SIZE));
        pane = pane.push(text!("Drag and drop a file inside the window or click the button below to select a file.")
            .size(P_SIZE));
        pane = pane.push(url_select_row.width(iced::Length::Fill));
    }

    let upload_pane = container(pane)
        .style(CustomStyles::darker_background(0.8))
        .width(iced::Length::FillPortion(3))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    upload_pane.into()
}

fn download_pane(state: &State) -> iced::Element<Message> {
    let image = widget::image(&state.qr_code)
        .width(iced::Length::Fill);

    let url_text = text!("Download")
        .size(H1_SIZE);

    let url_string =  state.create_url_string();
    let url_text_field = widget::text_input("", &url_string)
        .size(P_SIZE)
        .on_input(|_| Message::None);

    let copy_button = button("Copy URL")
        .on_press(Message::CopyUrl)
        .width(iced::Length::FillPortion(1));

    let browser_button = button("Open")
        .on_press(Message::OpenInBrowser)
        .width(iced::Length::FillPortion(1));

    let text_mode = match state.local_host {
        true => "Mode: Localhost",
        false => "Mode: Public IP"
    };
    let text_mode = text(text_mode)
        .size(H2_SIZE);

    let text_connection_info = text("Connection Info")
        .size(H2_SIZE);

    let block_external_connections = checkbox("Block External Connections", state.block_external_connections.load(std::sync::atomic::Ordering::Relaxed))
        .on_toggle(Message::BlockExternalConnections)
        .size(16)
        .text_size(16)
        .width(iced::Length::Fill);

    let block_external_connections_tooltip = text("Block external connections to the server. Check this box if you want only devices on the local network to access the files.")
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

    let show_qr_code = checkbox("Show QR Code", state.show_qr_code)
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

fn footer_pane(state: &State) -> iced::Element<Message> {
    let settings_text = text!("Theme:")
        .size(H2_SIZE);

    let theme_button = pick_list(state.theme.available_themes(), Some(state.theme.get()),Message::ThemeChanged)
        .style(CustomStyles::pick_list);

    let port_title = text!("Port:")
        .size(H2_SIZE);

    let mut port_text = widget::text_input("Port", &state.port_buffer)
        .width(iced::Length::Fixed(100.0));

    if state.active_downloads == 0 {
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

    if state.active_downloads > 0 {
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
        tooltip::Position::Right
    );

    let text_view = text!("Connections:")
        .size(H2_SIZE);

    let connection_info = match state.show_connections {
        true => "Hide Connections",
        false => "Show Connections"
    };
    let button_connection_info = button(connection_info)
        .on_press(Message::ToggleConnectionsView)
        .width(iced::Length::Fixed(160.0));

    let footer = row![
        settings_text,
        theme_button,
        port_title,
        port_text,
        text_view,
        button_connection_info
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

fn connection_info_pane(state: &State) -> iced::Element<Message> {
    let text_connections = text!("Connections")
        .size(H1_SIZE)
        .align_x(iced::alignment::Horizontal::Center)
        .width(iced::Length::Fill);

    let mut connections = column![];

    let mut clients = state.clients.iter().collect::<Vec<_>>();
    clients.sort_by_key(|(_, client_info)| Reverse(client_info.index));

    for (indx, (ip, client_info)) in clients.iter().enumerate() {

        let color = match client_info.state {
            state::ClientState::Downloading => state.theme.get().palette().primary,
            state::ClientState::Connected => state.theme.get().palette().success,
            state::ClientState::Disconnected => state.theme.get().palette().danger,
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
        
        if client_info.state == state::ClientState::Downloading {
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
            state::ClientState::Downloading => format!("Downloading at up to {}/s\nProgress: ({}/{})", size_string(client_info.max_speed), size_string(client_info.current_download_progress), size_string(client_info.current_downloads_size)),
            state::ClientState::Connected => {
                let last = if client_info.download_count > 0 {
                    format!("Last download {} ago \nat up to {}/s ", format_time(client_info.last_download.elapsed()), size_string(client_info.max_speed))
                } else {
                    "".to_owned()
                };
                format!("Connected.\n{}", last)
            },
            state::ClientState::Disconnected => {
                let last = if client_info.download_count > 0 {
                    format!("Last download {} ago \nat up to {}/s ", format_time(client_info.last_download.elapsed()), size_string(client_info.max_speed))
                } else {
                    "".to_owned()
                };
                format!("Last seen {} ago\n{}", format_time(client_info.last_connection.elapsed()), last)
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
        .style(CustomStyles::container_border(state.active_downloads > 0));

    let stats_text = text!("Stats")
        .size(H1_SIZE)
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

    let name_column = column![
        text("Active Downloads:").size(P_SIZE).width(iced::Length::Shrink),
        text("Active Clients:").size(P_SIZE).width(iced::Length::Shrink),
        text("Total Clients:").size(P_SIZE).width(iced::Length::Shrink),
        text("Total Downloads:").size(P_SIZE).width(iced::Length::Shrink),
        text("Current Upload:").size(P_SIZE).width(iced::Length::Shrink),
        text("Transmitted Data:").size(P_SIZE).width(iced::Length::Shrink),
    ]
    .spacing(5);

    let value_column = column![
        text!("{}", state.active_downloads).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.active_connections).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.clients.len()).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", state.total_downloads).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}/s", size_string(state.throughput)).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
        text!("{}", size_string(state.transmitted_data)).size(P_SIZE).align_x(iced::alignment::Horizontal::Right).width(iced::Length::Fill),
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

fn format_time(time: Duration) -> String {
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