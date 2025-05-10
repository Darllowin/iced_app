use iced::{widget::{column, text, text_input, button, vertical_space, Container}, Length, Alignment, Center};
use crate::app::{App, Message};

pub fn login_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Вход в систему").size(30),
        vertical_space(),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(20)
            .width(Length::Fixed(350.0)),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(20)
            .width(Length::Fixed(350.0)),
        //vertical_space(),
        button("Войти")
            .on_press(Message::LoginPressed)
            .padding(10)
            .width(Length::Fixed(200.0)),
        vertical_space(),
        text(format!("Статус: {}", if app.value == 1 { "Успешно" } else { "Ожидание" }))
            .size(16),
        button("Регистрация")
            .on_press(Message::SwitchToRegister)
            .padding(10)
            .width(Length::Fixed(200.0)),
    ]
        .spacing(15)
        .width(Length::Fill)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
