use iced::{widget::{button, column}, Length};
use iced::widget::{text, vertical_space, Container};
use iced_font_awesome::{fa_icon_solid};
use crate::app::{Message, App};
use crate::app::update::icon_button_content;

pub fn nav_menu(app: &App) -> Container<Message> {
    let content_for_admin = column![
        button(icon_button_content(
            fa_icon_solid("address-card").style(move |_| text::base(&app.theme.target())),
            "Профиль"
        )).on_press(Message::GoToProfile).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("graduation-cap").style(move |_| text::base(&app.theme.target())),
            "Курсы"
        )).on_press(Message::GoToCourses).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("file-invoice-dollar").style(move |_| text::base(&app.theme.target())),
            "Платежи"
        )).on_press(Message::GoToPayment).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("user").style(move |_| text::base(&app.theme.target())),
            "Пользователи"
        )).on_press(Message::GoToUserList).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("users").style(move |_| text::base(&app.theme.target())),
            "Группы"
        )).on_press(Message::GoToGroupList).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("stamp").style(move |_| text::base(&app.theme.target())),
            "Сертификаты"
        )).on_press(Message::GoToCertificates).width(Length::Fill),
        vertical_space(),
        button(icon_button_content(
            fa_icon_solid("gear").style(move |_| text::base(&app.theme.target())),
            "Настройки"
        )).on_press(Message::GoToSettings).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("arrow-right-from-bracket").style(move |_| text::base(&app.theme.target())),
            "Выход"
        )).on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);
    
    let content_for_unconfirmed = column![
        button(icon_button_content(
            fa_icon_solid("address-card").style(move |_| text::base(&app.theme.target())),
            "Профиль"
        )).on_press(Message::GoToProfile).width(Length::Fill),
        vertical_space(),
        button(icon_button_content(
            fa_icon_solid("gear").style(move |_| text::base(&app.theme.target())),
            "Настройки"
        )).on_press(Message::GoToSettings).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("arrow-right-from-bracket").style(move |_| text::base(&app.theme.target())),
            "Выход"
        )).on_press(Message::Logout).width(Length::Fill),
    ]
        .spacing(10);

    let content_for_teacher = column![
        button(icon_button_content(
            fa_icon_solid("address-card").style(move |_| text::base(&app.theme.target())),
            "Профиль"
        )).on_press(Message::GoToProfile).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("person-chalkboard").style(move |_| text::base(&app.theme.target())),
            "Занятия"
        )).on_press(Message::GoToClasses).width(Length::Fill),
        vertical_space(),
        button(icon_button_content(
            fa_icon_solid("gear").style(move |_| text::base(&app.theme.target())),
            "Настройки"
        )).on_press(Message::GoToSettings).width(Length::Fill),
        button(icon_button_content(
            fa_icon_solid("arrow-right-from-bracket").style(move |_| text::base(&app.theme.target())),
            "Выход"
        )).on_press(Message::Logout).width(Length::Fill),
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
