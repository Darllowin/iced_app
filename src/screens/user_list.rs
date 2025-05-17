use iced::{Color, ContentFit};
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::advanced::image::Handle;
use iced::widget::container::{background, bordered_box};
use iced::widget::{button, horizontal_space, image, row, text};
use crate::app::{App, Message, UserInfo, DEFAULF_AVATAR};
use crate::db;

pub fn user_list_screen(app: &App) -> Container<Message> {
    let conn = rusqlite::Connection::open("db_platform").unwrap();
    let users = db::get_all_users_for_list(&conn).unwrap_or_default();

    let mut list = Column::new().spacing(15);

    for user in users {
        let avatar = if let Some(mut data) = user.avatar_data.clone() {
            data.extend_from_slice(user.email.as_bytes()); // Для уникальности
            let image_handle = Handle::from_bytes(data);

            image(image_handle)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Fill)
        } else {
            image(DEFAULF_AVATAR)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Cover)
        };

        let header = Row::new()
            .push(Button::new(Text::new("Редактировать")).on_press(Message::StartEditingUser(user.clone())))
            .push(horizontal_space())
            .push(Button::new(Text::new("X")).on_press(Message::DeleteUser(user.email.clone())))
            .width(Length::Fill);

        let info = Column::new()
            .spacing(5)
            .push(Text::new(user.name).size(18))
            .push(Text::new(format!("Email: {}", user.email)))
            .push(Text::new(format!("Дата рождения: {}", user.birthday)))
            .push(Text::new(format!("Тип: {}", user.user_type)));

        let user_info_widget = Row::new().spacing(20).push(avatar).push(info);

        list = list.push(
            Container::new(Column::new()
                .spacing(10)
                .push(Container::new(header).style(move |_| bordered_box(&app.theme)).padding(10))
                .push(user_info_widget))
                .style(move |_| bordered_box(&app.theme))
                .width(Length::Fill)
                .padding(10)
        );
    }

    let scrollable = Scrollable::new(list.padding(20))
        .width(Length::Fill)
        .height(Length::Fill);

    let base = Container::new(scrollable)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    if app.show_edit_user_modal {
        let mut modal_content = Column::new()
            .spacing(10)
            .push(Text::new("Редактировать пользователя").size(24))
            
            .push(TextInput::new("Имя", &app.edit_user_name)
                .on_input(Message::EditUserNameChanged))
            .push(TextInput::new("Email", &app.edit_user_email)
                .on_input(Message::EditUserEmailChanged))
            .push(TextInput::new("Дата рождения", &app.edit_user_birthday)
                .on_input(Message::EditUserBirthdayChanged))
            .push(TextInput::new("Тип", &app.edit_user_type)
                .on_input(Message::EditUserTypeChanged))
            .push(Row::new()
                .spacing(10)
                .push(Button::new(Text::new("Отмена")).on_press(Message::CancelEditingUser))
                .push(Button::new(Text::new("Сохранить")).on_press(Message::SubmitEditedUser)));

        if let Some(error_msg) = &app.edit_user_error {
            modal_content = modal_content.push(Text::new(error_msg))
        }
        let modal = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme))
            .padding(20)
            .width(Length::Fixed(400.0));

        let modal_overlay = Container::new(
            mouse_area(
                Container::new(modal)
                    .center(Length::Fill)
                    .padding(40),
            )
                .on_press(Message::Er("".to_string()))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        Container::new(Stack::new().push(base).push(modal_overlay))
    } else {
        base
    }
}
