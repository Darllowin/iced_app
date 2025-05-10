mod app;
mod screens;
use app::App;

fn main() -> iced::Result {
    iced::application("Platform", App::update, App::view)
        .theme(|app: &App| app.theme.clone())
        .window_size(iced::Size::new(1400.0, 800.0))
        .run()
}
