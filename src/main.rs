mod app;
mod db;
mod doc_gen;
pub mod config;
mod screens;

use iced::{window, Size};
use app::App;

fn main() -> iced::Result {
    let window_settings = window::Settings {
        size: Size::new(1400.0, 800.0),
        min_size: Some(Size::new(1400.0, 800.0)),
        ..Default::default()
    };

    iced::application(App::default, App::update, App::view)
        .title("Platform")
        .theme(|app: &App| app.theme.value().clone())
        .window(window_settings)
        .centered()
        .run()
}
