use iced::{border, widget, Border, Theme};

pub struct CustomStyles;

impl CustomStyles {
    pub fn progress_bar(theme: &Theme) -> widget::progress_bar::Style {
        widget::progress_bar::Style {
            background: iced::Background::Color(color_multiply(theme.palette().background, 0.5)),
            bar: iced::Background::Color(theme.palette().primary),
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: border::Radius::from(5.0)
            },
        }
    }

    pub fn horizontal_rule(theme: &Theme) -> widget::rule::Style {
        widget::rule::Style {
            fill_mode: widget::rule::FillMode::Full,
            color: theme.palette().primary,
            width: 1,
            radius: border::Radius::default(),
        }
    }

    pub fn scrollable(theme: &Theme, _status: widget::scrollable::Status) -> widget::scrollable::Style {
        widget::scrollable::Style {
            container: widget::container::Style::default(),
            vertical_rail: widget::scrollable::Rail {
                background: None,
                border: Border { 
                    color: iced::Color::TRANSPARENT, 
                    width: 0.0, 
                    radius: border::Radius::default() 
                },
                scroller: widget::scrollable::Scroller { 
                    color: theme.palette().primary,
                    border: iced::Border { 
                        color: iced::Color::TRANSPARENT, 
                        width:  2.0, 
                        radius: border::Radius::from(5) 
                    } 
                },
            },
            horizontal_rail: iced::widget::scrollable::Rail {
                background: None,
                border: iced::Border { 
                    color: iced::Color::TRANSPARENT, 
                    width: 0.0, 
                    radius: border::Radius::default() 
                },
                scroller: iced::widget::scrollable::Scroller { 
                    color: iced::Color::TRANSPARENT, 
                    border: iced::Border { 
                        color: iced::Color::TRANSPARENT, 
                        width: 0.0, 
                        radius: border::Radius::default() 
                    } 
                },
            },
            gap: None,
        }
    }

    pub fn darker_background(factor: f32) -> impl Fn(&Theme) -> widget::container::Style {
        move | theme | {
            let darker_background = color_multiply(theme.palette().background, factor);
            widget::container::Style {
                background: Some(iced::Background::Color(darker_background)),
                ..widget::container::Style::default()
            }
        }
    }

    pub fn textfield_background(color: iced::Color) -> impl Fn(&Theme, widget::text_input::Status) -> widget::text_input::Style {
        move |theme, status| {
            iced::widget::text_input::Style {
                background: iced::Background::Color(color),
                ..widget::text_input::default(theme, status)               
            }
        }
    } 

    pub fn container_border(active: bool) -> impl Fn(&Theme) -> widget::container::Style {
        move |theme: &Theme| {
            let mut style = iced::widget::container::bordered_box(theme);
            style.border.width = 1.0;
            style.border.radius = border::Radius::from(0.0);
            style.border.color = if active {
                theme.palette().primary
            } else {
                color_multiply(theme.palette().primary, 0.5)
            };
            style.background = Some(iced::Background::Color(color_multiply(theme.palette().background,0.6)));
            style
        }
    }
}

pub fn color_multiply(color: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: color.a,
    }
}