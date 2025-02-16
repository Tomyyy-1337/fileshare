use std::usize;

use iced::{theme::palette, widget::{self, button, column, container, row, scrollable, text, text_input::default, Theme}};
use crate::{server::size_string, state::{self, ClientInfo, State}, update::Message};

pub fn view(state: &State) -> iced::Element<Message> {
    let h1_size = 30;
    let h2_size = 20;
    let p_size = 13;
     
    let settings_text = text!("Theme:")
        .size(h2_size);

    let theme_button = button("Toggle Dark Mode")
        .on_press(Message::ToggleDarkMode);

    let image = widget::image(&state.qr_code)
        .width(iced::Length::Fill);

    let url_text = text!("Download")
        .size(h1_size);

    let url_string =  state.create_url_string();
    let url_text_field = widget::text_input("", &url_string)
        .size(p_size)
        .on_input(|_| Message::None);

    let copy_button = button("Copy URL")
        .on_press(Message::CopyUrl);

    let browser_button = button("Download")
        .on_press(Message::OpenInBrowser);
    
    let upload_files = text!("Upload File")
        .size(h1_size);

    let url_select_button = button("Select File")
        .on_press(Message::SelectFilesExplorer)
        .width(iced::Length::FillPortion(1));
    
    let url_select_button2 = button("Select Folder")
        .on_press(Message::SelectFolderExplorer)
        .width(iced::Length::FillPortion(1));

    let url_select_row = row![url_select_button, url_select_button2]
        .spacing(5);

    let port_title = text!("Port:")
        .size(h2_size);

    let mut port_text = widget::text_input("Port", &state.port_buffer)
        .on_input(Message::PortTextUpdate)
        .on_submit(Message::ChangePort)
        .width(iced::Length::Fixed(100.0));

    let connection_info = match state.show_connections {
        true => "Hide Connections",
        false => "Show Connections"
    };
    let button_connection_info = button(connection_info)
        .on_press(Message::ToggleConnectionsView);

    let text_mode = match state.local_host {
        true => "Mode: Localhost",
        false => "Mode: Public IP"
    };
    let text_mode = text(text_mode)
        .size(h2_size);

    let text_connection_info = text("Connection Info")
        .size(h2_size);

    let text_view = text!("Show Connections:")
        .size(h2_size);

    match state.port_buffer.parse::<u16>() {
        Err(_) => port_text = port_text.style(|theme, status| {
            iced::widget::text_input::Style {
                background: iced::Background::Color(iced::Color::from_rgb8(255, 0, 0)),
                ..default(theme, status)                
            }
        }),
        Ok(n) if state.port != n => 
            port_text = port_text.style(|theme, status| {
                iced::widget::text_input::Style {
                    background: iced::Background::Color(iced::Color::from_rgb8(0, 0, 255)),
                    ..default(theme, status)                
                }
            }
        ),
        _ => {}
    }

    // === Layout ===
    let mut left = column![
        upload_files,
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill);
    {
    let file_path = state.file_path.read().unwrap();

    if !file_path.is_empty() {
        let shared_files_text = match file_path.len() {
            1 => "Shared File:".to_owned(),
            _ => format!("Shared Files [{}]:", file_path.len())   
        };

        let uploaded_files = text(shared_files_text)
            .size(h1_size);

        let text_num_send_files = text!("Total Downloads: {}", file_path.iter().map(|f| f.download_count).sum::<usize>())
            .size(h2_size); 

        let mut files_list = column![]
            .spacing(10)
            .padding(12);

        for (i, state::FileInfo{path, download_count, size}) in file_path.iter().cloned().enumerate() {
            let text_file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            let text_file_name = text(text_file_name)
                .size(h2_size)
                .height(iced::Length::Fixed(30.0))
                .width(iced::Length::Fill);

            let text_current_file = widget::text_input("", path.to_str().unwrap())
                .size(p_size)
                .on_input(|_| Message::None);
                    
            let open_button = button("Open")
                .on_press(Message::OpenFile(i))
                .width(iced::Length::FillPortion(1));

            let show_in_explorer_button = button("Show")
                .on_press(Message::ShowInExplorer(i))
                .width(iced::Length::FillPortion(1));

            let delete_button = button("Remove")
                .on_press(Message::DeleteFile(i))
                .width(iced::Length::FillPortion(1));

            let text_download_count = text!("Downloads: {}", download_count)
                .size(p_size)
                .width(iced::Length::Shrink);

            let text_size = text!("{}", size_string(&size))
                .size(p_size)
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

            let title_row = row![text_file_name, meta_col]
                .spacing(5)
                .width(iced::Length::Fill)
                .align_y(iced::alignment::Vertical::Center);
            
            let col = column![
                title_row,
                text_current_file,
                row
            ]
            .spacing(5);

            files_list = files_list.push(col);
        }

        let files_list = scrollable(files_list)
            .height(iced::Length::Fill);

        let files_list = container(files_list)
            .style(modify_style(1.0));
    
        let delete_all_button = button("Remove All")
            .on_press(Message::DeleteAllFiles)
            .width(iced::Length::FillPortion(1));
        
        let text_new_file = text!("Select new File:")
            .size(h2_size);
        
        left = left.push(text_new_file);
        left = left.push(url_select_row.width(iced::Length::Fill));
        left = left.push(uploaded_files);
        left = left.push(files_list);
        left = left.push(text_num_send_files);
        left = left.push(delete_all_button);
        left = left.align_x(iced::alignment::Horizontal::Center);
    } else {
        left = left.push(text!("No file selected!")
            .size(p_size));
        left = left.push(text!("Drag and drop a file inside the window or click the button below to select a file.")
            .size(p_size));
        left = left.push(url_select_row.width(iced::Length::Fill));
    }
    }

    let max_width = 1000.0 + if state.show_connections { 280.0 } else { 0.0 };

    let mut main = row![]
    .width(iced::Length::Fixed(max_width))
    .height(iced::Length::Fill)
    .padding(5)
    .spacing(5);

    if state.show_connections {
        let text_connections = text!("Connections")
            .size(h1_size)
            .align_x(iced::alignment::Horizontal::Center)
            .width(iced::Length::Fill);

        let mut connections = column![]
            .spacing(5)
            .padding(5);

        let ip_heading = text!("IP")
            .size(h2_size)
            .width(iced::Length::Fill);

        let count_heading = text!("Downloads")
            .size(h2_size)
            .width(iced::Length::Fixed(100.0))
            .align_x(iced::alignment::Horizontal::Right);

        let heading = row![ip_heading, count_heading]
            .spacing(5)
            .padding(5);

        for (ip, ClientInfo {download_count, last_connection, download_size}) in state.clients.iter() {
            let is_active = last_connection.elapsed().as_millis() < 3500;

            let text_ip = text!("{}", ip.to_string())
                .size(p_size)
                .width(iced::Length::Fill)
                .color(if is_active { iced::Color::from_rgb8(0, 255, 0) } else { iced::Color::from_rgb8(255, 0, 0) });

            let text_count = text!("{} ({})", download_count , size_string(download_size))
                .size(p_size)
                .width(iced::Length::Fixed(100.0))
                .align_x(iced::alignment::Horizontal::Right);


            let conection = row![text_ip, text_count];

            connections = connections.push(conection);
        }

        let connections = scrollable(connections)
            .height(iced::Length::Fill);

        let connections = container(connections)
            .width(iced::Length::Fill)
            .style(modify_style(1.0));

        let text_transmitted_data = text!("Transmitted Data")
            .size(h2_size);

        let text_transmitted_data_value = text!("{}", size_string(&state.transmitted_data))
            .size(h2_size);

        let connections = column![text_connections, heading,connections, text_transmitted_data, text_transmitted_data_value]
            .padding(5)
            .spacing(5);

        let connections = container(connections)
            .style(modify_style(0.8))
            .width(iced::Length::Fixed(230.0))
            .height(iced::Length::FillPortion(1))
            .padding(5);

        main = main.push(connections);
    }  

    let left = container(left)
        .style(modify_style(0.8))
        .width(iced::Length::FillPortion(3))
        .height(iced::Length::FillPortion(1))
        .padding(5);
    
    let url_buttons_row = row![
        copy_button.width(iced::Length::FillPortion(1)),
        browser_button.width(iced::Length::FillPortion(1))
        ]
        .spacing(5);
    
    let select_row = row![
        button("Localhost").on_press(Message::Localhost).width(iced::Length::FillPortion(1)),
        button("Public IP").on_press(Message::PublicIp).width(iced::Length::FillPortion(1))
    ]
    .spacing(5);

    let right = column![
        url_text,
        text_mode,
        select_row,
        text_connection_info,
        url_text_field.width(iced::Length::Fill),
        url_buttons_row,
        image
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    let right = container(right)
        .style(modify_style(0.8))
        .width(iced::Length::Fixed(230.0))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    // footer
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
    .width(iced::Length::Fixed(1000.0))
    .align_y(iced::alignment::Vertical::Center);

    let footer = container(footer)
        .style(modify_style(0.8))
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

    main = main.push(left);
    
    if state.server_handle.is_some() {
        main = main.push(right);
    }

    let main = container(main)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);
        
    let main = column!(
        main,
        footer
    );

    main.into()
}

fn color_multiply(color: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: color.a,
    }
}

fn modify_style(mult: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme: &Theme| {
        let p: palette::Palette = theme.palette();
        let darker_background = color_multiply(p.background, mult);
        container::Style {
            background: Some(iced::Background::Color(darker_background)),
            ..container::Style::default()
        }
    }
}