mod app;
mod db;
mod doc_gen;
mod screens;

use iced::{window, Size};
use iced_aw::iced_fonts::{REQUIRED_FONT_BYTES};
use app::App;

#[tokio::main]
async fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: Size::new(1400.0, 800.0), // Начальный размер
        min_size: Some(Size::new(1400.0, 800.0)), // Минимальный размер
        ..Default::default() // Заполнить остальные поля значениями по умолчанию
    };
    
    iced::application("Platform", App::update, App::view)
        .theme(|app: &App| app.theme.clone())
        .font(REQUIRED_FONT_BYTES)
        .window(window_settings)
        .run()
}
