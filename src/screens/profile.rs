use iced::{widget::{column, text, button, Container, vertical_space}, Length, Center};
use crate::app::{App, Message};
use super::nav_menu;


pub fn profile_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Профиль").size(30),
        vertical_space(),
        text(format!("Имя: {}", app.user_name)),
        text(format!("Фамилия: {}", app.user_surname)),
        text(format!("Почта: {}", app.user_email)),
        vertical_space(),
        button("Выйти")
            .on_press(Message::SwitchToLogin)
            .padding(10)
            .width(Length::Fill)
    ]
        .spacing(15)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
