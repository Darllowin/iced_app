use iced::{widget::{button, row, column, Row}, Length};
use iced::widget::{vertical_space, Column, Container};
use iced::widget::container::bordered_box;
use crate::app::{Message, App};

pub fn nav_menu(app: &App) -> Container<Message> {
    let content_for_admin = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        button("Курсы").on_press(Message::GoToCourses).width(Length::Fill),
        button("Пользователи").on_press(Message::GoToUserList).width(Length::Fill),
        button("Группы").on_press(Message::GoToGroupList).width(Length::Fill),
        vertical_space(),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    
    let content_for_student = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        vertical_space(),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    
    match app.type_user.as_str() { 
        "admin" => {
            Container::new(content_for_admin)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        }
        "student" => {
            Container::new(content_for_student)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        }
        _ => {panic!()}
    }
    
}
