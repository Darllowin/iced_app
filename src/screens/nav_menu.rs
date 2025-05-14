use iced::{widget::{button, row, column,Row}, Length};
use iced::widget::{vertical_space, Column, Container};
use crate::app::Message;

pub fn nav_menu() -> Container<'static, Message> {
    let content = column![
        button("Профиль").on_press(Message::GoToProfile).width(Length::Fill),
        button("Курсы").on_press(Message::GoToCourses).width(Length::Fill),
        button("Настройки").on_press(Message::GoToSettings).width(Length::Fill),
        vertical_space(),
        button("Выход").on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        
}
