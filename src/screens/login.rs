use iced::{widget::{column, text, text_input, button, vertical_space, Container}, Length, Alignment, Center, Theme};
use crate::app::{App, Message};

pub fn login_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Вход").size(30),
        vertical_space(),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        button("Войти")
            .on_press(Message::LoginPressed)
            .padding(10),
        text(&app.error_message).size(20),
        vertical_space(),
        button("Регистрация")
            .on_press(Message::SwitchToRegister)
            .padding(10),
    ]
        .spacing(15)
        .width(Length::Fill)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
