use crate::app::{App, Message};
use iced::{widget::{button, column, text, text_input, vertical_space, Container}, Center, Length};
use iced_font_awesome::fa_icon_solid;
use crate::app::update::icon_button_content;

pub fn login_screen(app: &App) -> Container<Message> {
    let content = column![
        vertical_space(),
        text("Вход").size(30),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(18)
            .width(Length::Fixed(350.0)),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(18)
            .secure(true)
            .width(Length::Fixed(350.0)),
        button(icon_button_content(
            fa_icon_solid("right-to-bracket").style(move |_| text::base(&app.theme.target())),
            "Войти"
        )).on_press(Message::LoginPressed).padding(10),
        text(&app.error_message).size(20),
        vertical_space(),
        button(icon_button_content(
            fa_icon_solid("id-card").style(move |_| text::base(&app.theme.target())),
            "Регистрация"
        )).on_press(Message::SwitchToRegister).padding(10),
    ]
        .spacing(15)
        .width(Length::Fill)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
