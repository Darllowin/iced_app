use iced::{widget::{column, text, button, Container}, Alignment, Length};
use crate::app::{App, Message};

pub fn profile_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Профиль").size(30),
        text(format!("Добро пожаловать, {}!", app.logged_in_user)).size(24),
    ]
        .spacing(20)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
