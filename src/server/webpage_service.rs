use std::{collections::HashMap, sync::{Arc, RwLock}};
use iced::Theme;
use serde::Serialize;
use warp::Filter;
use tera::Tera;

use crate::{state::file_manager, views::styles::color_multiply};

#[derive(Serialize)]
struct SendColor {
    r: u8,
    g: u8,
    b: u8,
}

impl SendColor {
    fn from_iced(color: iced::Color) -> Self {
        let [r,g,b,_] = color.into_rgba8();
        SendColor { r, g, b }
    }
}

#[derive(Serialize)]
struct UpdateData {
    html: String,
    size: String,
    primary: SendColor,
    secondary: SendColor,
    background: SendColor,
    dark_background: SendColor,
    text: SendColor,
    text_secondary: SendColor,
    footer: SendColor,
}

#[derive(Serialize)]
struct DisplayFileInfo {
    name: String,
    index: usize,
    size: String,
}

pub fn static_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("static")
        .and(warp::fs::dir("./static"))
}

pub fn index_route(
    path: Arc<RwLock<HashMap<usize, file_manager::FileInfo>>>, 
    theme: Arc<RwLock<Theme>>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    let html_route = warp::path("index")
        .map(move || {
            let path = path.clone();
            let html_str = fill_template(path, "index.html", theme.clone());
            let response = warp::reply::html(html_str);
            response
        });
    html_route
}

pub fn refresh_route(
    path: Arc<RwLock<HashMap<usize, file_manager::FileInfo>>>,
    theme: Arc<RwLock<Theme>>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone 
{
    let refresh_route = warp::path("update-content")
        .map(move || {
            let html = fill_template(path.clone(), "file_list.html", theme.clone());
            let theme = theme.read().unwrap();
            let (primary, secondary, background, dark_background, text, text_secondary, footer) = colors(&theme);
            let size = size_string(path.read().unwrap().iter().map(|(_, file_manager::FileInfo{size, ..})| size).sum());
            let response = warp::reply::json(&UpdateData {html, size, primary, secondary, background, dark_background, text, text_secondary, footer  });
            response
        });
    refresh_route
}

pub fn fill_template(
    path: Arc<RwLock<HashMap<usize, file_manager::FileInfo>>>, 
    template: &'static str,
    theme: Arc<RwLock<Theme>>
) -> String {
    let tera: Tera = Tera::new("template/*.html").unwrap();
    let mut context = tera::Context::new();

    let mut path = path.read().unwrap()
        .iter()
        .map(|(i, f)| (*i, f.clone()))
        .collect::<Vec<_>>();
    path.sort_by_key(|(indx, _)| *indx);

    let files: Vec<DisplayFileInfo> = path.iter().rev().map(|(i, file_manager::FileInfo{path, size, ..})| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        let size_string = size_string(*size);
        DisplayFileInfo { name, index: *i, size: size_string }
    }).collect();
    context.insert("files", &files);

    let all_size: usize = path.iter().map(|(_, file_manager::FileInfo{size, ..})| size).sum();
    context.insert("all_size", &size_string(all_size));

    let theme = theme.read().unwrap();
    let (primary, secondary, background, dark_background, text, text_secondary, footer) = colors(&theme);
    context.insert("primary", &to_rgb_string(primary));
    context.insert("secondary", &to_rgb_string(secondary));
    context.insert("background", &to_rgb_string(background));
    context.insert("dark_background", &to_rgb_string(dark_background));
    context.insert("text", &to_rgb_string(text));
    context.insert("text_secondary", &to_rgb_string(text_secondary));
    context.insert("footer", &to_rgb_string(footer));

    tera.render(template, &context).unwrap()
}

fn to_rgb_string(color: SendColor) -> String {
    let SendColor { r, g, b } = color;
    format!("rgb({}, {}, {})", r, g, b)
}

pub fn size_string(size: usize) -> String {
    match size {
        s if s < 1000 => format!("{} B", s),
        s if s < 1024 * 1000 => format!("{:.1} KB", s as f64 / 1024.0),
        s if s < 1024 * 1024 * 1000 => format!("{:.1} MB", s as f64 / 1024.0 / 1024.0),
        s if s < 1024 * 1024 * 1024 * 1000 => format!("{:.1} GB", s as f64 / 1024.0 / 1024.0 / 1024.0),
        s => format!("{:.1} TB", s as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0),
    }
}

fn colors(theme: &Theme) -> (SendColor, SendColor, SendColor, SendColor, SendColor, SendColor, SendColor) {
    let primary = SendColor::from_iced(theme.palette().primary);
    let secondary = SendColor::from_iced(color_multiply(theme.palette().primary, 0.8));
    let background = SendColor::from_iced(theme.palette().background);
    let dark_background = SendColor::from_iced(color_multiply(theme.palette().background, 0.8));
    let footer = SendColor::from_iced(color_multiply(theme.palette().background, 0.6));
    let text = SendColor::from_iced(theme.palette().text);
    let text_secondary = SendColor::from_iced({
        let iced::Color { r, g, b, .. } = theme.palette().primary;
        if r + g + b > 1.5 {
            iced::Color::BLACK
        } else {
            iced::Color::WHITE
        }
    });
    (primary, secondary, background, dark_background, text, text_secondary, footer)
}
