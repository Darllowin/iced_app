use iced::{Element, Length};
use iced::widget::{Column, Container, Row};
use crate::app::state::Screen;
use crate::screens::{certificates_screen, classes_screen, courses_screen, groups_screen, login_screen, nav_menu, payment_screen, profile_screen, register_screen, settings_screen, user_list_screen};
use super::{App, Message};

impl App {
    pub fn view(&self) -> Row<Message> {
        Row::new()
            .spacing(20)
            .push(
                // Левое меню (sidebar)
                if self.current_screen != Screen::Login && self.current_screen != Screen::Register {
                    Container::new(nav_menu(self))
                        .width(Length::Fixed(200.0)) // Фиксированная ширина меню
                        .height(Length::Fill)
                        .padding(10)
                } else {
                    Container::new(Column::new()) // Пустой контейнер, если экран входа
                        .width(Length::Fixed(0.0))
                        .height(Length::Fill)
                }
            )
            .push(
                // Основной контент
                match &self.current_screen {
                    Screen::Login => login_screen(self),
                    Screen::Register => register_screen(self),
                    Screen::Profile => profile_screen(self),
                    Screen::Settings => settings_screen(self),
                    Screen::CoursesList => courses_screen(self),
                    Screen::UserList => user_list_screen(self),
                    Screen::GroupList => groups_screen(self),
                    Screen::Classes => classes_screen(self),
                    Screen::Payment => payment_screen(self),
                    Screen::Certificates => certificates_screen(self),
                }
                    .width(Length::Fill),
            )
            .into()
    }
}