use iced::{theme::palette, widget::{self, button, column, container, row, scrollable, text, text_input::{self, default}, Theme}};
use crate::{state::State, update::Message};

pub fn view(state: &State) -> iced::Element<Message> {
    let h1_size = 30;
    let h2_size = 20;
    let p_size = 15;
     
    let settings_text = text!("Theme:")
        .size(h2_size);

    let theme_button = button("Toggle Dark Mode")
        .on_press(Message::ToggleDarkMode);

    let image = widget::image(&state.qr_code)
        .width(iced::Length::Fill);

    let url_text = text!("Download URL:")
        .size(h1_size);

    let url_string =  state.create_url_string();
    let url_text_field = widget::text_input("", &url_string)
        .size(p_size)
        .on_input(|_| Message::None);

    let copy_button = button("Copy URL")
        .on_press(Message::CopyUrl);

    let browser_button = button("Download")
        .on_press(Message::OpenInBrowser);
    
    let upload_files = text!("Upload File:")
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
    
    let update_port_button = button("Update Port")
        .on_press(Message::ChangePort);

    let mut port_text = widget::text_input("Port", &state.port_buffer)
        .on_input(Message::PortTextUpdate)
        .on_submit(Message::ChangePort)
        .width(iced::Length::Fixed(100.0));

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

   if !state.file_path.lock().unwrap().is_empty() {
        let uploaded_files = text!("Shared Files:")
            .size(h1_size);

        let mut files_list = column![]
            .spacing(10)
            .padding(12);

        for (i, path) in state.file_path.lock().unwrap().iter().cloned().enumerate() {
            let text_file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            let text_file_name = text(text_file_name)
                .size(h2_size)
                .height(iced::Length::Fixed(30.0));

            let text_current_file = widget::text_input("", path.to_str().unwrap())
                .size(p_size)
                .on_input(|_| Message::None);
                    
            let open_button = button("Open")
                .on_press(Message::OpenFile(i))
                .width(iced::Length::FillPortion(1));

            let show_in_explorer_button = button("Show")
                .on_press(Message::ShowInExplorer(i))
                .width(iced::Length::FillPortion(1));

            let delete_button = button("Delete")
                .on_press(Message::DeleteFile(i))
                .width(iced::Length::FillPortion(1));

            let row = row![
                open_button,
                show_in_explorer_button,
                delete_button
            ]
            .spacing(5);
            
            let col = column![
                text_file_name,
                text_current_file,
                row
            ]
            .spacing(5);

            files_list = files_list.push(col);
        }

        let files_list = scrollable(files_list)
            .height(iced::Length::Fill);
    
        let delete_all_button = button("Delete All")
            .on_press(Message::DeleteAllFiles)
            .width(iced::Length::FillPortion(1));
        
        let text_new_file = text!("Select new File:")
            .size(h2_size);
        
        left = left.push(text_new_file);
        left = left.push(url_select_row.width(iced::Length::Fill));
        left = left.push(uploaded_files);
        left = left.push(files_list);
        left = left.push(delete_all_button);
    } else {
        left = left.push(text!("No file selected!")
            .size(p_size));
        left = left.push(text!("Drag and drop a file inside the window or click the button below to select a file.")
            .size(p_size));
        left = left.push(url_select_row.width(iced::Length::Fill));
        
    }

    let left = container(left)
        .style(modify_style(0.8))
        .width(iced::Length::FillPortion(1))
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
        select_row,
        url_text_field.width(iced::Length::Fill),
        url_buttons_row,
        image
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill)
    .height(iced::Length::Fill);

    let right = container(right)
        .style(modify_style(0.8))
        .width(iced::Length::FillPortion(1))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    // footer
    let footer = row![
        settings_text,
        theme_button,
        port_title,
        port_text,
        update_port_button
    ]
    .spacing(20)
    .padding(10)
    .width(iced::Length::Fixed(1200.0));

    let footer = container(footer)
        .style(modify_style(0.8))
        .width(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

    // Main
    let mut main = row![
        left
    ]
    .width(iced::Length::Fixed(1200.0))
    .height(iced::Length::Fill)
    .padding(5)
    .spacing(10);
    
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
