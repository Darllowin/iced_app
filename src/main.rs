mod app;
mod db;
mod doc_gen;
pub mod config;
mod screens;

use iced::{window, Settings, Size};
use iced_aw::iced_fonts::{REQUIRED_FONT_BYTES};
use app::App;

#[tokio::main]
async fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: Size::new(1400.0, 800.0), // Начальный размер
        min_size: Some(Size::new(1400.0, 800.0)), // Минимальный размер
        ..Default::default() // Заполнить остальные поля значениями по умолчанию
    };
    let settings = Settings {
        antialiasing: true,
        ..Settings::default()
    };
    
    iced::application("Platform", App::update, App::view)
        .theme(|app: &App| app.theme.value().clone())
        .font(REQUIRED_FONT_BYTES)
        .window(window_settings)
        .settings(settings)
        .centered()
        .run()
    
    
}
