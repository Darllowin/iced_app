use iced::{widget::{column, text, text_input, button, vertical_space, Container}, Length, Center};
use iced::widget::{row, Text};
use iced_aw::{date_picker};
use iced_font_awesome::fa_icon_solid;
use crate::app::{App, Message};
use crate::app::update::icon_button_content;

pub fn register_screen(app: &App) -> Container<Message> {
    let but = button(icon_button_content(
        fa_icon_solid("calendar-check").style(move |_| text::base(&app.theme.target())),
        "Дата рождения"
    )).on_press(Message::ChooseDate);
    let content = column![
        text("Регистрация").size(30),
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
        row![
            text_input("Date", &app.date.to_string())
                .on_input(Message::Er)
                .padding(10)
                .size(18)
                .width(Length::Fixed(175.0)), 
            date_picker(app.show_picker, app.date, but, Message::CancelDate, Message::SubmitDate),
        ]
            .spacing(10)
            .align_y(Center),
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
            .secure(true)
            .width(Length::Fixed(350.0)),
        text_input("Повторите пароль", &app.user_password_repeat)
            .on_input(Message::PasswordRepeatChanged)
            .padding(10)
            .size(18)
            .secure(true)
            .width(Length::Fixed(350.0)),
        vertical_space(),
        if let Some(err) = &app.register_error {
            Text::new(err)
                .size(16)
        } else {
            Text::new("")
        },
        button(icon_button_content(
            fa_icon_solid("id-card").style(move |_| text::base(&app.theme.target())),
            "Зарегистрироваться"
        )).on_press(Message::RegisterPressed).padding(10),
        vertical_space(),
        button(icon_button_content(
            fa_icon_solid("arrow-left").style(move |_| text::base(&app.theme.target())),
            "Назад ко входу"
        )).on_press(Message::SwitchToLogin).padding(10)
    ]
        .spacing(15)
        .width(Length::Fill)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
