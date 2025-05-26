use iced::{Color, ContentFit, Theme}; // Добавляем Theme
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::advanced::image::Handle;
use iced::widget::container::{background, bordered_box};
use iced::widget::{horizontal_space, image, text, PickList, Rule, button as button_widget}; // Импортируем button как button_widget, чтобы не конфликтовать с Button
use crate::app::{App, Message};
use crate::app::state::{DEFAULT_AVATAR, PATH_TO_DB};
use crate::db;

pub fn user_list_screen(app: &App) -> Container<Message> {
    let conn = rusqlite::Connection::open(PATH_TO_DB).unwrap();
    let users = db::get_all_users_for_list(&conn, app.user_type_filter.as_deref()).unwrap_or_default();

    let mut list = Column::new().spacing(15);

    let filter_options = vec![
        "Все".to_string(),
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
                Message::UserTypeFilterChanged(None)
            } else {
                Message::UserTypeFilterChanged(Some(selection))
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
        let avatar_user_list = Container::new(
            if let Some(mut data) = user.avatar_data.clone() {
                data.extend_from_slice(user.email.as_bytes());
                let image_handle = Handle::from_bytes(data);

                image(image_handle)
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(120.0))
                    .content_fit(ContentFit::Cover)
            } else {
                image(DEFAULT_AVATAR)
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(120.0))
                    .content_fit(ContentFit::Cover)
            }
        )
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .clip(true);

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
            if let Some(group_names) = &user.group_id {
                info = info.push(Text::new(format!("Группа: {}", group_names)));
            } else {
                info = info.push(Text::new("Группа: не указана"));
            }
        }

        if user.user_type == "teacher".to_string() {
            if let Some(group_names) = &user.group_id {
                info = info.push(Text::new(format!("Группа: {}", group_names)));
            } else {
                info = info.push(Text::new("Группа: не указана"));
            }
        }

        if user.user_type == "parent" {
            if let Some(count) = user.child_count {
                if count > 0 {
                    info = info.push(Text::new(format!("Количество детей: {}", count)));
                }
            } else {
                info = info.push(Text::new("Детей: нет данных"));
            }
        }

        let user_info_widget = Container::new(
            Row::new()
                .padding(10)
                .spacing(20)
                .push(avatar_user_list)
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

    let base_ui = Container::new(
        Column::new().push(filter_row.padding(10)).push(scrollable)
    )
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut current_ui_stack = Stack::new().push(base_ui);

    // Модалка редактирования пользователя
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
            modal_content = modal_content.push(Text::new(error_msg));
        }

        let modal = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme))
            .padding(20)
            .width(Length::Fixed(400.0));

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal).center(Length::Fill).padding(40))
                .on_press(Message::CancelEditingUser)
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        current_ui_stack = current_ui_stack.push(modal_overlay);
    }

    // Модалка детей 
    if app.show_children_modal {
        // Заголовок модального окна, можно получить из имени родителя или типа "Список детей"
        let modal_title_text = "Список детей";

        let mut children_list_col: Column<'_, Message, Theme> = Column::new().spacing(5);

        // Проверяем, есть ли дети для отображения
        if app.parent_children.is_empty() {
            children_list_col = children_list_col.push(
                Text::new("У этого пользователя нет детей.").size(16)
            );
        } else {
            for child in &app.parent_children {
                let avatar_widget_child = if let Some(mut data) = child.avatar_data.clone() {
                    data.extend_from_slice(child.email.as_bytes());
                    let image_handle = Handle::from_bytes(data);

                    image(image_handle)
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(100.0))
                        .content_fit(ContentFit::Fill) // или Fill, как в примере со студентами
                } else {
                    image(DEFAULT_AVATAR)
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(100.0))
                        .content_fit(ContentFit::Fill)
                };

                // Оборачиваем виджет изображения в контейнер с обрезкой
                let avatar_container_child = Container::new(avatar_widget_child)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .clip(true); // <--- Это ключевой момент для обрезки

                let info = Column::new()
                    .spacing(5)
                    .width(Length::Fill)
                    .push(Text::new(format!("ФИО: {}", &child.name)).size(18)) // Как в примере со студентами
                    .push(Text::new(format!("Email: {}", &child.email)))
                    .push(Text::new(format!("Дата рождения: {}", &child.birthday)))
                    .push(
                        if let Some(group_names) = &child.group_id {
                            Text::new(format!("Группа: {}", group_names))
                        } else {
                            Text::new("Группа: не указана")
                        }
                    );

                let row = Row::new()
                    .padding(10) // Padding для каждого элемента списка, как в примере со студентами
                    .width(Length::Fill)
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(avatar_container_child)
                    .push(info)
                    .push(horizontal_space()) // Для выравнивания кнопки справа
                    .push(
                        button_widget(Text::new("Удалить")) // Используем button_widget из-за конфликта имен
                            .on_press(Message::DeleteChild {
                                parent_email: app.edit_user_email.clone(), // Полагаемся на email родителя
                                child_email: child.email.clone(),
                            })
                    );

                children_list_col = children_list_col.push(
                    Container::new(row)
                        .style(move |_| bordered_box(&app.theme))
                        .width(Length::Fill)
                );
            }
        }

        let scrollable_children = Scrollable::new(
            Container::new(children_list_col).padding(5) // Padding для контейнера внутри скролла
        ).height(Length::FillPortion(1)); // Высота, как в примере со студентами

        // Логика добавления ребенка
        let add_child_row = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(Text::new("Добавить ребенка:").size(18))
            .push(
                PickList::new(
                    app.available_children.clone(), // Список доступных детей
                    app.selected_child_to_add.clone(),
                    Message::SelectedChildToAddChanged,
                ).placeholder("Выберите ребенка")
            )
            .push(
                button_widget(Text::new("Добавить"))
                    .on_press(Message::AddChildToParent) // Убедитесь, что это сообщение обрабатывается
                    .width(Length::Shrink)
            );

        let modal_content = Column::new()
            .spacing(15)
            .align_x(Alignment::Start) // Выравнивание по левому краю
            .push(Text::new(modal_title_text).size(22))
            .push(scrollable_children)
            .push(Rule::horizontal(10)) // Разделитель
            .push(add_child_row)
            .push(Text::new(app.edit_user_error.clone().unwrap_or_default()).color(Color::from_rgb(1.0, 0.0, 0.0))) // Если есть специфичная ошибка для этой модалки, используйте ее
            .push(
                button_widget(Text::new("Закрыть"))
                    .on_press(Message::CloseParentChildrenModal)
            );

        let modal_container = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme))
            .padding(20)
            .height(Length::Fixed(600.0)) // Увеличил высоту, чтобы было место для всего
            .width(Length::Fixed(900.0)); // Увеличил ширину для содержимого

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal_container).center(Length::Fill))
                .on_press(Message::CloseParentChildrenModal) // Закрытие по клику вне модалки
        )
            .width(Length::Fill).height(Length::Fill)
            .center_y(Length::Fill) // Добавлено явное центрирование
            .center_x(Length::Fill) // Добавлено явное центрирование
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        current_ui_stack = current_ui_stack.push(modal_overlay);
    }

    // Возвращаем итоговый стек UI
    Container::new(current_ui_stack)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}