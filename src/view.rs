use iced::{theme::palette, widget::{self, button, column, container, row, text, text_input}, Theme};
use crate::{state::State, update::Message};

pub fn view(state: &State) -> iced::Element<Message> {
    let h1_size = 30;
    let h2_size = 20;
    let p_size = 15;
    
    let settings_text = text!("Settings:")
        .size(h2_size);

    let theme_button = button("Toggle Dark Mode")
        .on_press(Message::ToggleDarkMode);

    let image = widget::image(&state.qr_code)
        .width(iced::Length::Fill);

    let url_text = text!("Download URL:")
        .size(h1_size);

    let url_string =  state.create_url_string();
    let url_text_field = text_input("", &url_string)
        .size(p_size)
        .on_input(|_| Message::None);

    let copy_button = button("Copy URL")
        .on_press(Message::CopyUrl);

    let browser_button = button("Download")
        .on_press(Message::OpenInBrowser);
    
    let uploaded_files = text!("Uploaded Files:")
        .size(h1_size);

    let text_current_file_header = text!("Current File:")
        .size(h2_size);

    let url_select_button = button("Select File")
        .on_press(Message::SelectPath);

    // === Layout ===
    let mut left = column![
        uploaded_files,
        text_current_file_header
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill)
    .height(iced::Length::Fill);

    match &state.file_path {
        Some(path) => {
            let text_current_file = text_input("", path.to_str().unwrap())
                .size(p_size)
                .on_input(|_| Message::None);

            let open_button = button("Open File")
                .on_press(Message::OpenFile)
                .width(iced::Length::FillPortion(1));

            let show_in_explorer_button = button("Show in Explorer")
                .on_press(Message::ShowInExplorer)
                .width(iced::Length::FillPortion(1));

            let row = row![
                show_in_explorer_button.width(iced::Length::FillPortion(1)),
                open_button.width(iced::Length::FillPortion(1)),
            ]
            .spacing(5);
        
            let text_stop_share = text!("Stop Sharing")
                .size(h2_size);
            
            let delete_button = button("Stop Server")
                .on_press(Message::DeleteFile);

            let text_new_file = text!("Select new File:")
                .size(h2_size);

            left = left.push(text_current_file);
            // if file is image, show preview
            left = left.push(row);
            left = left.push(text_new_file);
            left = left.push(url_select_button.width(iced::Length::Fill));
            left = left.push(text_stop_share);
            left = left.push(delete_button.width(iced::Length::Fill));
        }
        None => {
            left = left.push(text!("No file selected!")
                .size(p_size));
            left = left.push(text!("Drag and drop a file inside the window or click the button below to select a file.")
                .size(p_size));
            left = left.push(url_select_button.width(iced::Length::Fill));
        }
    }

    let left = container(left)
        .style(modify_style(0.8))
        .width(iced::Length::FillPortion(1))
        .height(iced::Length::Fill)
        .padding(5);
    
    let url_buttons_row = row![
        copy_button.width(iced::Length::FillPortion(1)),
        browser_button.width(iced::Length::FillPortion(1))
        ]
        .spacing(5);
    
    let right = column![
        url_text,
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
        .height(iced::Length::Fill)
        .padding(5);

    // footer
    let footer = row![
        settings_text,
        theme_button,
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
    .spacing(5);
    
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
