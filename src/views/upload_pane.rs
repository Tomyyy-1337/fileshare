use iced::{alignment, widget::{self, button, column, container, horizontal_rule, hover, row, text, tooltip, Space}};
use crate::{server::webpage_service::size_string, state::{file_manager::{CompressingZip, FileInfo}, state::State, update::Message}, views::styles::CustomStyles};

use super::root_view::{H1_SIZE, H2_SIZE, P_SIZE};

pub fn upload_pane(state: &State) -> iced::Element<Message> {
    let upload_files = text(state.language.upload_file())
        .size(H1_SIZE);

    let url_select_button = button(state.language.select_files())
        .on_press(Message::SelectFilesExplorer)
        .width(iced::Length::FillPortion(1));

    let url_select_button2 = button(state.language.select_folders())
        .on_press(Message::SelectFolderExplorer)
        .width(iced::Length::FillPortion(1));

    let url_select_button2 = tooltip(
        url_select_button2,
        container(text(state.language.select_folders_tooltip()).size(P_SIZE))
            .padding(10)
            .width(iced::Length::Fixed(200.0))
            .style(container::rounded_box),
        tooltip::Position::Bottom
    );

    let zip_select_button = button(state.language.zip_folder())
        .on_press(Message::SelectZipExplorer)
        .width(iced::Length::FillPortion(1));

    let zip_select_button = tooltip(
        zip_select_button,
        container(text(state.language.zip_folder_tooltip()).size(P_SIZE))
            .padding(10)
            .width(iced::Length::Fixed(200.0))
            .style(container::rounded_box),
        tooltip::Position::Bottom
    );

    let url_select_row = row![url_select_button, url_select_button2, zip_select_button]
        .spacing(5)
        .width(iced::Length::Fill);

    let mut pane = column![
        upload_files,
        horizontal_rule(5).style(CustomStyles::horizontal_rule),
    ]
    .padding(5)
    .spacing(10)
    .width(iced::Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);
    
    let file_path = state.file_manager.get_view();
    let zipping_files = state.file_manager.get_zip_compressing();

    if !file_path.is_empty() || !zipping_files.is_empty() {
        let uploaded_files = text(state.language.shared_files(file_path.len()))
            .size(H1_SIZE);

        let mut files_list = column![];
        for (color, (path, CompressingZip { num_files, progress, ..})) in zipping_files.iter().enumerate() {
            let text_file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            let text_file_name = text!("{}.zip", text_file_name)
                .size(H2_SIZE)
                .height(iced::Length::Fixed(32.0))
                .width(iced::Length::Fill);

            let cancle_button = button(state.language.cancel())
                .on_press(Message::ZipCancel((*path).clone()))
                .width(iced::Length::Shrink);

            let progress_bar = widget::progress_bar(0.0..=*num_files as f32, *progress as f32)
                .style(CustomStyles::progress_bar)
                .width(iced::Length::Fill)
                .height(iced::Length::Fixed(18.0));

            let progress_text = text!("{} / {}", progress, num_files)
                .size(P_SIZE)
                .align_y(alignment::Vertical::Center)
                .width(iced::Length::Shrink);

            let row = row![progress_bar, progress_text]
                .spacing(5)
                .width(iced::Length::Fill);

            let progress_row = container(
                row
            )
            .align_y(iced::alignment::Vertical::Center)
            .height(iced::Length::Fixed(32.0));
            
            let row = row![text_file_name, cancle_button]
                .spacing(5)
                .width(iced::Length::Fill)
                .align_y(iced::alignment::Vertical::Center);

            let col = column![row, progress_row]
                .spacing(5);

            let col = container(col)
                .padding(12)
                .style(CustomStyles::darker_background(if color & 1 == 0 { 0.9 } else { 0.7 }));
        
            files_list = files_list.push(col);
        } 

        for (color, (i, FileInfo{path, download_count, size, ..})) in file_path.iter().cloned().rev().enumerate() {
            let color = color + zipping_files.len();
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
                    
            let open_button = button(state.language.open())
                .on_press(Message::OpenFile(i))
                .width(iced::Length::FillPortion(1));

            let show_in_explorer_button = button(state.language.show())
                .on_press(Message::ShowInExplorer(i))
                .width(iced::Length::FillPortion(1));

            let delete_button = button(state.language.delete())
                .width(iced::Length::FillPortion(1));

            let delete_button: iced::Element<Message> = if state.client_manager.active_downloads() == 0 {
                delete_button.on_press(Message::DeleteFile(i)).into()
            } else {
                tooltip(delete_button, container(text(state.language.delete_tooltip()).size(P_SIZE))
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
                .style(CustomStyles::darker_background(if color & 1 == 0 { 0.9 } else { 0.7 }));

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

        let delete_all_button = button(state.language.remove_all())
            .width(iced::Length::FillPortion(1));

        let delete_all_button: iced::Element<Message> = if state.client_manager.active_downloads() == 0 {
            delete_all_button.on_press(Message::DeleteAllFiles).into()
        } else {
            tooltip(delete_all_button, container(text(state.language.delete_tooltip()).size(P_SIZE))
                .padding(10)
                .width(iced::Length::Fixed(200.0))
                .style(container::rounded_box),
                tooltip::Position::Right
            ).into()
        };
        
        pane = pane.push(url_select_row);
        pane = pane.push(uploaded_files);
        pane = pane.push(horizontal_rule(5).style(CustomStyles::horizontal_rule));
        pane = pane.push(files_list);
        pane = pane.push(delete_all_button);
    } else {
        pane = pane.push(text(state.language.no_file_selected())
            .size(H2_SIZE));
        pane = pane.push(text(state.language.drag_and_drop())
            .size(P_SIZE));
        pane = pane.push(url_select_row);
    }

    let upload_pane = container(pane)
        .style(CustomStyles::darker_background(0.8))
        .width(iced::Length::FillPortion(3))
        .height(iced::Length::FillPortion(1))
        .padding(5);

    upload_pane.into()
}