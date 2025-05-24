use iced::{widget::{button, column}, Length};
use iced::widget::{vertical_space, Container};

use crate::app::{Message, App};

pub fn nav_menu(app: &App) -> Container<Message> {
    let content_for_admin = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        button("Курсы").on_press(Message::GoToCourses).width(Length::Fill),
        button("Платежи").on_press(Message::GOToPayment).width(Length::Fill),
        button("Пользователи").on_press(Message::GoToUserList).width(Length::Fill),
        button("Группы").on_press(Message::GoToGroupList).width(Length::Fill),
        vertical_space(),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    
    let content_for_unconfirmed = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        vertical_space(),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);

    let content_for_teacher = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        button("Занятия").on_press(Message::GoToClasses).width(Length::Fill),
        vertical_space(),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    
    match app.current_user.as_ref().unwrap().user_type.as_str() {
        "admin" => {
            Container::new(content_for_admin)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        }
        "unconfirmed" => {
            Container::new(content_for_unconfirmed)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        }
        "teacher" => {
            Container::new(content_for_teacher)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        }
        _ => {panic!()}
    }
    
}
