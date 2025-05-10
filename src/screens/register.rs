use iced::{widget::{column, text, text_input, button, vertical_space, Container}, Length, Alignment, Center};
use crate::app::{App, Message};

pub fn register_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Регистрация").size(30),
        //vertical_space(),
        text_input("Имя", &app.user_name)
            .on_input(Message::FirstNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Фамилия", &app.user_surname)
            .on_input(Message::LastNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Отчество", &app.user_patronymic)
            .on_input(Message::MiddleNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        vertical_space(),
        text_input("Телефон", &app.user_phone)
            .on_input(Message::PhoneChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        vertical_space(),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Повторите пароль", &app.user_password_repeat)
            .on_input(Message::PasswordRepeatChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        button("Зарегистрироваться")
            .on_press(Message::RegisterPressed)
            .padding(10)
            .width(Length::Fixed(200.0)),
        vertical_space(),
        button("Назад ко входу")
            .on_press(Message::SwitchToLogin)
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
