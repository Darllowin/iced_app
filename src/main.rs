mod app;
mod db;
mod screens;

use iced_aw::iced_fonts::{REQUIRED_FONT_BYTES};
use app::App;

#[tokio::main]
async fn main() -> iced::Result {
    iced::application("Platform", App::update, App::view)
        .theme(|app: &App| app.theme.clone())
        .font(REQUIRED_FONT_BYTES)
        .window_size(iced::Size::new(1400.0, 800.0))
        .run()
}
