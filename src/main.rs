use std::fs::File;
use std::io::Read;
use std::net::IpAddr;
use std::path::Path;
use std::process::Command;
use warp::http::header::{self, CACHE_CONTROL, PRAGMA, EXPIRES};

use iced::{keyboard, Size, Subscription, Task, Theme, window};
use iced::theme::palette;
use iced::widget::{self, button, column, combo_box, container, row, text, text_input};
use local_ip_address::local_ip;
use qrcode_generator::QrCodeEcc;
use copypasta::{ClipboardContext, ClipboardProvider};
use warp::reply::Response;
use warp::Filter;
use webbrowser;
use rfd::FileDialog;

fn main() -> iced::Result { 
    iced::application("Fileshare", State::update, State::view)
        .subscription(subscription)
        .window(iced::window::Settings {
            resizable: true,
            size: (Size::new(800.0, 580.0)),
            min_size: Some(Size::new(620.0, 400.0)),
            ..iced::window::Settings::default()
        })
        .theme(|state| match state.dark_mode {
            true => Theme::Dracula,
            false => Theme::Light,
        })
        .antialiasing(true)
        .run()
        
}

fn subscription(state: &State) -> Subscription<Message> {
    let keyboard = keyboard_input(state);
    let window = window_events();

    Subscription::batch([
        keyboard, 
        window
    ])
}

fn window_events() -> Subscription<Message> {
    window::events().map(|(_, event)| match event {
        window::Event::FileDropped(path) => Message::FileDropped(path),
        _ => Message::None,
    })
}

fn keyboard_input(_state: &State) -> Subscription<Message> {
    keyboard::on_key_press(|key, _modifyer| {
        match key {
            _ => None,
        }
    })
}

#[derive(Debug, Clone)]
enum Message {
    ToggleDarkMode,
    CopyUrl,
    None,
    OpenInBrowser,
    FileDropped(std::path::PathBuf),
    ServerStopped,
    DeleteFile,
    OpenFile,
    ShowInExplorer,
    SelectPath,
}

struct State {
    dark_mode: bool,
    ip_adress: Option<IpAddr>,
    port: u16,
    file_path: Option<String>,
    qr_code: widget::image::Handle,
    server_handle: Option<iced::task::Handle>,
}

impl Default for State {
    fn default() -> Self {
        let ip = local_ip().ok();
        let qr_code = create_qr_code(&format!("http://{}:{}/download", ip.unwrap(), 8080), 1200);
        Self {
            dark_mode: true,
            ip_adress: ip,
            port: 8080,
            file_path: None,
            qr_code,
            server_handle: None,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
            }
            Message::CopyUrl => {
                let mut ctx = ClipboardContext::new().unwrap();
                ctx.set_contents(format!("http://{}:{}/download", self.ip_adress.unwrap(), self.port)).unwrap();	
            }
            Message::FileDropped(path) => {
                self.file_path = Some(path.to_str().unwrap().to_string());
                if let Some(handle) = &self.server_handle {
                    handle.abort();
                    self.server_handle = None;
                }
                if let Some(path) = &self.file_path {
                    let task =  Task::perform(server(self.ip_adress.unwrap(), self.port, path.clone()), |_result| Message::ServerStopped);
                    let (task, handle): (Task<Message>, iced::task::Handle) = Task::abortable(task);
                    self.server_handle = Some(handle);
                    return task;
                }
            }
            Message::ServerStopped => {
                self.server_handle = None;
            }
            Message::OpenInBrowser => {
                webbrowser::open(&format!("http://{}:{}/download", self.ip_adress.unwrap(), self.port)).unwrap();
            }
            Message::DeleteFile => {
                self.file_path = None;
                if let Some(handle) = &self.server_handle {
                    handle.abort();
                    self.server_handle = None;
                }
            }
            Message::OpenFile => {
                if let Some(path) = &self.file_path {
                    Command::new( "explorer" )
                        .arg(path)
                        .spawn( )
                        .unwrap( );
                }
            }
            Message::ShowInExplorer => {
                if let Some(path) = &self.file_path {
                    Command::new( "explorer" )
                        .arg("/select,")
                        .arg(path)
                        .spawn( )
                        .unwrap( );
                }
            }
            Message::SelectPath => {
                let path = FileDialog::new()
                    .add_filter("Any", &["*"])
                    .pick_file();
                if let Some(path) = path {
                    return Task::done(Message::FileDropped(path));
                }
            }
            Message::None => {}
        }

        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let h1_size = 30;
        let h2_size = 20;
        let p_size = 15;
        
        let settings_text = text!("Settings:")
            .size(h2_size);

        let theme_button = button("Toggle Dark Mode")
            .on_press(Message::ToggleDarkMode);

        let image = widget::image(&self.qr_code)
            .width(iced::Length::Fill);

        let url_text = text!("Download URL:")
            .size(h1_size);

        let url_string = format!("http://{}:{}/download", self.ip_adress.unwrap(), self.port);
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

        match &self.file_path {
            Some(path) => {
                let text_current_file = text_input("", path)
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
        let main = row![
            left,
            right
        ]
        .width(iced::Length::Fixed(1200.0))
        .height(iced::Length::Fill)
        .padding(5)
        .spacing(5);

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

fn create_qr_code(url: &String, size: usize) -> widget::image::Handle {
    let data = qrcode_generator::to_image(url, QrCodeEcc::Medium, size).expect("Couldn't generate QR code.")
        .into_iter()
        .flat_map(|pixel| {
            vec![pixel, pixel, pixel, 255]
        }).collect::<Vec<u8>>();
    
    widget::image::Handle::from_rgba(size as u32, size as u32, data)
}

async fn server(ip: IpAddr, port: u16, path: String) {
    let download_route = warp::path("download")
        .and_then( move || { 
            let path_clone = path.clone();
            async move {
                let file_name = Path::new(&path_clone).file_name().unwrap().to_str().unwrap();
                let mut file = match File::open(&path_clone) {
                    Ok(file) => file,
                    Err(_) => return Err(warp::reject::not_found()),
                };
                
                let mut buffer = Vec::new();
                if let Err(_) = file.read_to_end(&mut buffer) {
                    return Err(warp::reject::not_found());
                }
              
                Ok(warp::reply::with_header(
                    buffer,
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", file_name),
                ))
            }
        });

    warp::serve(download_route)
        .run((ip, port))
        .await;
}
