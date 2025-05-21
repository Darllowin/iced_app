use iced::{widget::{column, text, Container, vertical_space}, Length, Center, Theme};
use iced::widget::{checkbox, pick_list};
use crate::app::{App, Message};

pub fn settings_screen(_app: &App) -> Container<Message> {
    let current_name = theme_to_str(&_app.theme);
    let theme_names: Vec<&'static str> = Theme::ALL.iter().map(theme_to_str).collect();
    let content = column![
        text("Настройки").size(30),
        vertical_space(),
        pick_list(theme_names, Some(current_name), Message::ThemeSelected)
        .placeholder("Выберите тему"),
    ]
        .spacing(15)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
pub fn theme_to_str(theme: &Theme) -> &'static str {
    match theme {
        Theme::Light => "Light",
        Theme::Dark => "Dark",
        Theme::Dracula => "Dracula",
        Theme::Nord => "Nord",
        Theme::SolarizedLight => "SolarizedLight",
        Theme::SolarizedDark => "SolarizedDark",
        Theme::GruvboxLight => "GruvboxLight",
        Theme::GruvboxDark => "GruvboxDark",
        Theme::CatppuccinLatte => "CatppuccinLatte",
        Theme::CatppuccinFrappe => "CatppuccinFrappe",
        Theme::CatppuccinMacchiato => "CatppuccinMacchiato",
        Theme::CatppuccinMocha => "CatppuccinMocha",
        Theme::TokyoNight => "TokyoNight",
        Theme::TokyoNightStorm => "TokyoNightStorm",
        Theme::TokyoNightLight => "TokyoNightLight",
        Theme::KanagawaWave => "KanagawaWave",
        Theme::KanagawaDragon => "KanagawaDragon",
        Theme::KanagawaLotus => "KanagawaLotus",
        Theme::Moonfly => "Moonfly",
        Theme::Nightfly => "Nightfly",
        Theme::Oxocarbon => "Oxocarbon",
        Theme::Ferra => "Ferra",
        _ => "Unknown",
    }
}
