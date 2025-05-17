use std::borrow::Cow;
use iced::{widget::{column, text, button, Container}, Center, ContentFit, Length};
use iced::widget::{image, row, TextInput};
use iced::widget::container::bordered_box;
use iced::widget::image::Handle;
use crate::app::{App, Message};

pub fn profile_screen(app: &App) -> Container<Message> {
    let avatar_widget = if let Some(ref data) = app.user_avatar_data {
        let image_handle = Handle::from_bytes(data.clone());

        image(image_handle)
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(220.0))
            .content_fit(ContentFit::Fill)
    } else {
        // Возвращаемся к аватару по умолчанию, если данные отсутствуют
        image("default_avatar.jpg")
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .content_fit(ContentFit::Cover)
    };
    
    let content = column![
        //text("Профиль").size(30).align_x(Center).width(Length::Fill),
        row![
            avatar_widget,
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
        .spacing(20);

    let user_info_wigget = Container::new(content).style(move |_| bordered_box(&app.theme)).width(Length::Fill).padding(10);
    Container::new(user_info_wigget)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        
}