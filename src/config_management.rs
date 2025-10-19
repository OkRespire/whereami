use iced::overlay::menu::Catalog;

pub struct Config {
    theme: iced::Theme,
    size: iced::Size,
    font: iced::Font,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: iced::Theme::GruvboxDark,
            size: iced::Size::new(600.0, 400.0),
            font: iced::Font::default(),
        }
    }
}
