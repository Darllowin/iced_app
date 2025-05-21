use iced::{Color, ContentFit};
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::advanced::image::Handle;
use iced::widget::container::{background, bordered_box};
use iced::widget::{horizontal_space, image, text, PickList};
use crate::app::{App, Message, DEFAULT_AVATAR};
use crate::db;

pub fn user_list_screen(app: &App) -> Container<Message> {
    let conn = rusqlite::Connection::open("db_platform").unwrap();
    let users = db::get_all_users_for_list(&conn, app.user_type_filter.as_deref()).unwrap_or_default();

    let mut list = Column::new().spacing(15);

    let filter_options = vec![
        "Все".to_string(), // Option to show all users
        "student".to_string(),
        "parent".to_string(),
        "admin".to_string(),
    ];

    let current_filter_selection = app.user_type_filter.clone().unwrap_or_else(|| "Все".to_string());

    let filter_picklist = PickList::new(
        filter_options,
        Some(current_filter_selection),
        |selection| {
            if selection == "Все" {
                Message::UserTypeFilterChanged(None) // If "All" selected, set filter to None
            } else {
                Message::UserTypeFilterChanged(Some(selection)) // Otherwise, use the selected type
            }
        },
    )
        .placeholder("Фильтровать по типу");

    let filter_row = Row::new()
        .push(text("Фильтр: "))
        .push(filter_picklist)
        .align_y(Alignment::Center)
        .spacing(10)
        .width(Length::Fill)
        .padding([0, 20]);

    for user in users {
        let avatar = if let Some(mut data) = user.avatar_data.clone() {
            data.extend_from_slice(user.email.as_bytes()); // Для уникальности
            let image_handle = Handle::from_bytes(data);

            image(image_handle)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Fill)
        } else {
            image(DEFAULT_AVATAR)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Cover)
        };

        let header = Row::new()
            .push(
                if user.user_type == "parent".to_string() {
                    Row::new()
                        .spacing(10)
                        .push(Button::new(Text::new("Редактировать")).on_press(Message::StartEditingUser(user.clone())))
                        .push(Button::new(Text::new("Дети")).on_press(Message::ShowParentChildren(user.email.clone())))
                } else {
                    Row::new()
                        .push(Button::new(Text::new("Редактировать")).on_press(Message::StartEditingUser(user.clone())))
                }
            )
            .push(horizontal_space())
            .push(Button::new(Text::new("X")).on_press(Message::DeleteUser(user.email.clone())))
            .width(Length::Fill);

        let mut info = Column::new()
            .spacing(5)
            .push(Text::new(user.name).size(18))
            .push(Text::new(format!("Email: {}", user.email)))
            .push(Text::new(format!("Дата рождения: {}", user.birthday)))
            .push(Text::new(format!("Тип: {}", user.user_type)));
        
        if user.user_type == "student".to_string() {
            if let Some(group_names) = &user.group_id { // child.group теперь Option<String> из БД
                // Отображаем строку с именами групп
                info = info.push(Text::new(format!("Группа: {}", group_names)));
            } else {
                // Если у ребенка нет групп
                info = info.push(Text::new("Группа: не указана"));
            }
        }

        if user.user_type == "teacher".to_string() {
            if let Some(group_names) = &user.group_id { // child.group теперь Option<String> из БД
                // Отображаем строку с именами групп
                info = info.push(Text::new(format!("Группа: {}", group_names)));
            } else {
                // Если у ребенка нет групп
                info = info.push(Text::new("Группа: не указана"));
            }
        }

        if user.user_type == "parent" {
            // user.child_count - это Option<i32> из БД
            if let Some(count) = user.child_count {
                // Отображаем количество, если оно > 0
                if count > 0 {
                    info = info.push(Text::new(format!("Количество детей: {}", count)));
                } 
            } else {
                // Этот случай должен быть редким для parent, но на всякий случай
                info = info.push(Text::new("Детей: нет данных"));
            }
        }
        
        let user_info_widget = Container::new(
            Row::new()
                .padding(10)
                .spacing(20)
                .push(avatar)
                .push(info)
        ).style(move |_| bordered_box(&app.theme)).width(Length::Fill);

        list = list.push(
            Container::new(Column::new()
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

    let base = Container::new(
        Column::new().push(filter_row.padding(10)).push(scrollable)
    )
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    // Модалка редактирования пользователя
    let ui_with_edit_modal = if app.show_edit_user_modal {
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
            modal_content = modal_content.push(Text::new(error_msg));
        }

        let modal = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme))
            .padding(20)
            .width(Length::Fixed(400.0));

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal).center(Length::Fill).padding(40))
                .on_press(Message::Er("".to_string()))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        Container::new(Stack::new().push(base).push(modal_overlay))
    } else {
        base
    };

    // Модалка детей
    if app.show_children_modal {
        let mut children_list = app.parent_children.iter().fold(Column::new().spacing(10).width(Length::Fill), |col, child| {
            let avatar = if let Some(mut data) = child.avatar_data.clone() {
                data.extend_from_slice(child.email.as_bytes());
                let image_handle = Handle::from_bytes(data);

                image(image_handle)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .content_fit(ContentFit::Cover)
            } else {
                image(DEFAULT_AVATAR)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .content_fit(ContentFit::Cover)
            };

            let info = Column::new()
                .spacing(5)
                .width(Length::Fill)
                .push(Text::new(&child.name).size(18))
                .push(Text::new(format!("Email: {}", &child.email)))
                .push(Text::new(format!("Дата рождения: {}", &child.birthday)))
                .push(
                    if let Some(group_names) = &child.group_id {
                        // Отображаем строку с именами групп
                        Text::new(format!("Группа: {}", group_names))
                    } else {
                        // Если у студента нет групп
                        Text::new("Группа: не указана")
                    }
                );

            let row = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .push(avatar)
                .push(info)
                .push(Button::new(Text::new("X")).on_press(Message::DeleteChild {
                    parent_email: app.edit_user_email.clone(),
                    child_email: child.email.clone(),
                }));

            col.push(Container::new(row).style(move |_| bordered_box(&app.theme)).padding(10).width(Length::Fill))
        });

        if app.parent_children.is_empty() {
            children_list = children_list.push(Text::new("Нет детей."));
        }

        let picklist = PickList::new(
            &app.available_children[..],
            app.selected_child_to_add.clone(),
            Message::SelectedChildToAddChanged,
        )
            .placeholder("Выберите ребёнка");

        let add_button = Button::new(Text::new("Добавить"))
            .on_press(Message::AddChildToParent);

        let add_row = Row::new().spacing(10).push(picklist).push(add_button);

        let modal_content = Column::new()
            .spacing(15)
            .push(Text::new("Список детей").size(24))
            .push(Scrollable::new(children_list))
            .push(add_row)
            .push(Button::new(Text::new("Закрыть")).on_press(Message::CloseParentChildrenModal));

        let modal = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme))
            .padding(20)
            .height(Length::Fixed(500.0))
            .width(Length::Fixed(800.0));

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal).center(Length::Fill).padding(40))
                .on_press(Message::Er("".to_string()))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        Container::new(Stack::new().push(ui_with_edit_modal).push(modal_overlay))
    } else {
        ui_with_edit_modal
    }
}
