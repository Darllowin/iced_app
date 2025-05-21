use iced::{widget::{column, text, button, Container}, ContentFit, Length};
use iced::widget::{image, row, Column};
use iced::widget::container::bordered_box;
use iced::widget::image::Handle;
use crate::app::{App, Message, DEFAULT_AVATAR};

pub fn profile_screen(app: &App) -> Container<Message> {

    // Получаем текущего пользователя из Option
    let user_data = app.current_user.as_ref(); // app.current_user имеет тип Option<UserInfo>

    // Аватар пользователя
    let avatar_widget = if let Some(user_info) = user_data {
        if let Some(ref data) = user_info.avatar_data { // Используем avatar_data из UserInfo
            let image_handle = Handle::from_bytes(data.clone());
            image(image_handle)
                .width(Length::Fixed(220.0))
                .height(Length::Fixed(220.0))
                .content_fit(ContentFit::Fill)
        } else {
            image(DEFAULT_AVATAR) // Если в UserInfo нет аватара
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Cover)
        }
    } else {
        image(DEFAULT_AVATAR) // Если user_data == None (пользователь не вошел)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .content_fit(ContentFit::Cover)
    };

    // Основное содержимое профиля
    let content = column![
        row![
            Container::new(avatar_widget)
                .style(move |_| bordered_box(&app.theme))
                .padding(10),
            column![
                // Используем данные из user_data. Если user_data None, показываем пустые строки или заглушки.
                text(format!("ФИО: {}", user_data.map_or("Неизвестно".to_string(), |u| u.name.clone()))).size(24),
                text(format!("Дата рождения: {}", user_data.map_or("Неизвестно".to_string(), |u| u.birthday.clone()))).size(24),
                text(format!("Почта: {}", user_data.map_or("Неизвестно".to_string(), |u| u.email.clone()))).size(24),
                text(format!("Тип профиля: {}", user_data.map_or("Неизвестно".to_string(), |u| u.user_type.clone()))).size(24),
            ]
            .spacing(10),
        ]
        .width(Length::Fill)
        .spacing(20),
        text(&app.error_message).size(10), // Это может быть у вас напрямую в App
        button("Выбрать аватар").on_press(Message::ChooseAvatar),
    ]
        .spacing(0);

    // Блок группы пользователя
    let user_group = column![
        if let Some(user_info) = user_data {
            if let Some(group_name) = &user_info.group { // Используем group из UserInfo
                Container::new(text(format!("Группа: {}", group_name)).size(24))
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(10)
                    .style(move |_| bordered_box(&app.theme))
            } else {
                Container::new(text("Группа отсутствует").size(24))
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(10)
                    .style(move |_| bordered_box(&app.theme))
            }
        } else {
            // Если user_data == None, показываем заглушку
            Container::new(text("Группа отсутствует (пользователь не вошел)").size(24))
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding(10)
                .style(move |_| bordered_box(&app.theme))
        }
    ];

    let user_info_widget = Container::new(content).style(move |_| bordered_box(&app.theme)).width(Length::Fill).padding(10);
    Container::new(
        Column::new()
            .push(user_info_widget)
            .push(user_group)
            .spacing(20)
            .padding(20)
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
}