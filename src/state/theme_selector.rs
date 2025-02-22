use std::sync::{Arc, RwLock};

use iced::{theme::{Custom, Palette}, Theme};

pub struct ThemeSelector {
    indx: usize,
    current: Arc<RwLock<Theme>>,
    themes: [iced::Theme; 20],
}

impl ThemeSelector {
    pub fn set_indx(&mut self, indx: usize) {
        self.indx = indx;
        self.current.write().unwrap().clone_from(&self.themes[self.indx]);
    }

    pub fn get(&self) -> iced::Theme {
        self.themes[self.indx].clone()
    }

    pub fn get_indx(&self) -> usize {
        self.indx
    }

    pub fn get_arc(&self) -> Arc<RwLock<Theme>> {
        self.current.clone()
    }

    pub fn next(&mut self) {
        self.indx = (self.indx + 1).min(self.themes.len() - 1);
        self.current.write().unwrap().clone_from(&self.themes[self.indx]);
    }

    pub fn previous(&mut self) {
        if let Some(val) = self.indx.checked_sub(1) {
            self.indx = val;
        }
        self.current.write().unwrap().clone_from(&self.themes[self.indx]);
    }

    pub fn available_themes(&self) -> &[iced::Theme] {
        &self.themes
    }

    pub fn set(&mut self, theme: &iced::Theme) {
        self.indx = self.themes.iter().position(|t| t == theme).unwrap_or(0);
        self.current.write().unwrap().clone_from(theme);
    }

    pub fn new() -> Self {
        let themes = [
            Theme::Custom(Arc::new(Custom::new("Dracula Light".to_string(), Palette {
                background: iced::Color::WHITE,
                text: iced::Color::BLACK,
                primary: iced::Color::from_rgb8(159, 99, 246),
                success: iced::Color::from_rgb8(20, 180, 20),
                danger: iced::Color::from_rgb8(255, 0, 0),
            }))),
            Theme::Light,
            Theme::SolarizedLight,
            Theme::CatppuccinLatte,
            Theme::GruvboxLight,
            Theme::TokyoNightLight,
            Theme::Nord,
            Theme::CatppuccinFrappe,
            Theme::CatppuccinMocha,
            Theme::Dracula,
            Theme::Dark,
            Theme::Ferra,
            Theme::GruvboxDark,
            Theme::Oxocarbon,
            Theme::TokyoNight,
            Theme::TokyoNightStorm,
            Theme::SolarizedDark,
            Theme::Nightfly,
            Theme::Moonfly,
            Theme::KanagawaDragon,
        ];
        let indx = 17;
        Self {
            indx,
            current: Arc::new(RwLock::new(themes[indx].clone())),
            themes
        }
    }
}
