use iced::{widget::{button, row, Row}, Length};
use iced::widget::{vertical_space, Column, Container};
use crate::app::Message;

pub fn nav_menu() -> Container<'static, Message> {
    let content = iced::widget::column![
        button("Профиль").on_press(Message::GoToProfile),
        button("Настройки").on_press(Message::GoToSettings),
        vertical_space(),
        button("Выход").on_press(Message::Logout)
    ]
        .spacing(10);
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        
}
