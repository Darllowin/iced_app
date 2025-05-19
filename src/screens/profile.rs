use iced::{widget::{column, text, button, Container}, ContentFit, Length};
use iced::widget::{image, row, Column};
use iced::widget::container::bordered_box;
use iced::widget::image::Handle;
use crate::app::{App, Message, DEFAULT_AVATAR};

pub fn profile_screen(app: &App) -> Container<Message> {
    
    let avatar_widget = if let Some(ref data) = app.user_avatar_data {
        let image_handle = Handle::from_bytes(data.clone());

        image(image_handle)
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(220.0))
            .content_fit(ContentFit::Fill)
    } else {
        // Возвращаемся к аватару по умолчанию, если данные отсутствуют
        image(DEFAULT_AVATAR)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .content_fit(ContentFit::Cover)
    };
    
    let content = column![
        //text("Профиль").size(30).align_x(Center).width(Length::Fill),
        row![
            Container::new(avatar_widget)
                .style(move |_| bordered_box(&app.theme))
                .padding(10),
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
        .spacing(0);
    
    let user_group = column![
        if let Some(group_name) = &app.user_group_name {
            Container::new(text(format!("Группа: {}", group_name)).size(24))
                .width(Length::Fill)
                //.height(Length::Fill)
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
    ];
    let user_info_wigget = Container::new(content).style(move |_| bordered_box(&app.theme)).width(Length::Fill).padding(10);
    Container::new(
        Column::new()
            .push(user_info_wigget)
            .push(user_group)
            .spacing(20)
            .padding(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        
}