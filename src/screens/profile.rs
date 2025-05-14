use iced::{widget::{column, text, button, Container}, Center, ContentFit, Length};
use iced::widget::{image, row};
use crate::app::{App, Message};

pub fn profile_screen(app: &App) -> Container<Message> {
    let avatar = if let Some(ref path) = app.user_avatar_path {
        image(path.clone()).width(Length::Fixed(220.0)).height(Length::Fixed(220.0)).content_fit(ContentFit::Fill)
    } else {
        image("default_avatar.png").width(Length::Fixed(120.0)).height(Length::Fixed(120.0))
    };

    let content = column![
        text("Профиль").size(30).align_x(Center).width(Length::Fill),
        row![
            avatar,
            column![
                text(format!("ФИО: {}",&app.logged_in_user)).size(24),
                text(format!("Дата рождения: {}",app.user_birthday)).size(24),
                text(format!("Почта: {}", &app.user_email)).size(24),
                text(format!("Тип профиля: {}", &app.type_user)).size(24),
            ]
            .spacing(10),
        ]
        .width(Length::Fill)
        .spacing(20),
        text(&app.error_message).size(10),
        button("Выбрать аватар").on_press(Message::ChooseAvatar),
    ]
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
