use iced::{widget::{column, text, button, Container}, Alignment, Color, ContentFit, Length, Theme};
use iced::widget::{image, mouse_area, row, Column, Row, Rule, Scrollable, Stack, Text};
use iced::widget::container::{background, bordered_box};
use iced::widget::image::Handle;
use crate::app::{App, Message, DEFAULT_AVATAR};

pub fn profile_screen(app: &App) -> Container<Message> {

    // Получаем текущего пользователя из Option
    let user_data = app.current_user.as_ref();

    // Аватар пользователя
    let avatar_widget = if let Some(user_info) = user_data {
        if let Some(ref data) = user_info.avatar_data {
            let image_handle = Handle::from_bytes(data.clone());
            image(image_handle)
                .width(Length::Fixed(220.0))
                .height(Length::Fixed(220.0))
                .content_fit(ContentFit::Fill)
        } else {
            image(DEFAULT_AVATAR)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Cover)
        }
    } else {
        image(DEFAULT_AVATAR)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .content_fit(ContentFit::Cover)
    };

    // Основное содержимое профиля
    let main_profile_content = column![
        row![
            Container::new(avatar_widget)
                .style(move |_| bordered_box(&app.theme))
                .padding(10),
            column![
                text(format!("ФИО: {}", user_data.map_or("Неизвестно".to_string(), |u| u.name.clone()))).size(24),
                text(format!("Дата рождения: {}", user_data.map_or("Неизвестно".to_string(), |u| u.birthday.clone()))).size(24),
                text(format!("Почта: {}", user_data.map_or("Неизвестно".to_string(), |u| u.email.clone()))).size(24),
                text(format!("Тип профиля: {}", user_data.map_or("Неизвестно".to_string(), |u| u.user_type.clone()))).size(24),
            ]
            .spacing(10),
        ]
        .width(Length::Fill)
        .spacing(20),
        text(&app.error_message).size(10),
        button("Выбрать аватар").on_press(Message::ChooseAvatar),
    ]
        .spacing(0);

    let user_info_widget = Container::new(main_profile_content)
        .style(move |_| bordered_box(&app.theme))
        .width(Length::Fill)
        .padding(10);

    let mut role_specific_content = Column::new()
        .spacing(20)
        .width(Length::Fill);

    if let Some(user_info) = user_data {
        match user_info.user_type.as_str() {
            "student" => {
                // Если пользователь - студент
                if let Some(_group_id) = &user_info.group_id { // Предполагаем, что у студента есть _group_id
                    if let Some(student_group) = &app.student_group_info {
                        role_specific_content = role_specific_content.push(
                            Container::new(
                                column![
                                    text(format!("Моя группа: {}", student_group.name)).size(24),
                                    text(format!("Курс: {}", student_group.course_name.as_deref().unwrap_or("Неизвестно"))).size(24),
                                ]
                                    .spacing(5)
                            )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(10)
                                .style(move |_| bordered_box(&app.theme))
                        );
                    } else {
                        role_specific_content = role_specific_content.push(
                            Container::new(text("Данные группы загружаются...").size(24))
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(10)
                                .style(move |_| bordered_box(&app.theme))
                        );
                    }
                } else {
                    role_specific_content = role_specific_content.push(
                        Container::new(text("Вы пока не прикреплены к группе.").size(24))
                            .width(Length::Fill)
                            .center_x(Length::Fill)
                            .padding(10)
                            .style(move |_| bordered_box(&app.theme))
                    );
                }
            },
            "teacher" => {
                // Если пользователь - преподаватель
                role_specific_content = role_specific_content.push(
                    Text::new("Мои группы:").size(24)
                );

                if app.teacher_groups.is_empty() {
                    role_specific_content = role_specific_content.push(
                        Text::new("Вы не преподаете ни в одной группе.").size(18)
                    );
                } else {
                    for group in &app.teacher_groups {
                        let group_row = row![
                            text(format!("Группа: {} (Курс: {})", group.name, group.course_name.as_deref().unwrap_or("Неизвестно"))).size(20).width(Length::Fill),
                            button("Показать состав")
                                .on_press(Message::ShowGroupStudents(group.id)), // Новое сообщение
                        ]
                            .spacing(10)
                            .align_y(Alignment::Center);

                        role_specific_content = role_specific_content.push(
                            Container::new(group_row)
                                .width(Length::Fill)
                                .padding(5)
                                .style(move |_| bordered_box(&app.theme))
                        );
                    }
                }
            },
            "admin" => {
                // Если пользователь - администратор, ничего дополнительного не отображаем
                // Здесь можно добавить какую-то специфичную для админа информацию, если потребуется
            },
            _ => {
                // Неизвестный тип пользователя
                role_specific_content = role_specific_content.push(
                    Text::new("Неизвестный тип пользователя.").size(24).color(Color::from_rgb8(255, 0, 0))
                );
            }
        }
    } else {
        // Пользователь не вошел в систему
        role_specific_content = role_specific_content.push(
            Text::new("Для просмотра информации войдите в систему.").size(24)
        );
    }

    let base_ui = Column::new()
        .push(user_info_widget)
        .push(role_specific_content)
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut ui_stack = Stack::new().push(Container::new(base_ui).center_x(Length::Fill).center_y(Length::Fill));

    // --- Модальное окно для просмотра студентов группы (для преподавателя) ---
    if app.show_group_students_modal {
        if let Some(group_name) = &app.selected_group_for_students_name {
            let modal_title_text = format!("Состав группы: {}", group_name);

            let mut students_list_col: Column<'_, Message, Theme> = Column::new().spacing(5);
            if app.selected_group_students.is_empty() {
                students_list_col = students_list_col.push(
                    Text::new("В этой группе пока нет студентов.").size(16)
                );
            } else {
                for student in &app.selected_group_students {
                    // Логика отображения аватара
                    let avatar = if let Some(mut data) = student.avatar_data.clone() {
                        // Добавление email для уникальности Handle, если data может быть одинаковой
                        data.extend_from_slice(student.email.as_bytes());
                        let image_handle = Handle::from_bytes(data);

                        image(image_handle)
                            .width(Length::Fixed(100.0)) // Меньший размер для списка студентов группы
                            .height(Length::Fixed(100.0))
                            .content_fit(ContentFit::Fill)
                    } else {
                        image(DEFAULT_AVATAR)
                            .width(Length::Fixed(100.0)) // Меньший размер
                            .height(Length::Fixed(100.0))
                            .content_fit(ContentFit::Cover)
                    };

                    let student_row_content = Row::new()
                        .padding(10)
                        .width(Length::Fill)
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(avatar) // Добавляем аватар
                        .push(Column::new()
                            .spacing(5) // Уменьшим spacing для компактности
                            .push(Text::new(format!("ФИО: {}", student.name.clone()))) // Используем bold для "ФИО"
                            .push(Text::new(format!("Email: {}", student.email.clone())))
                            .push(Text::new(format!("Дата рождения: {}", student.birthday.clone())))
                        );
                    // .push(horizontal_space()) // <-- УДАЛЕНО: нет кнопки для выравнивания
                    // .push(button("X").on_press(Message::RemoveStudentFromGroup(student.id, current_group_id))); // <-- УДАЛЕНО

                    students_list_col = students_list_col.push(
                        Container::new(student_row_content)
                            .style(move |_| bordered_box(&app.theme))
                            .width(Length::Fill)
                    );
                }
            }

            let scrollable_students = Scrollable::new(
                Container::new(students_list_col).padding(5)
            ).height(Length::FillPortion(1));

            let modal_content = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(modal_title_text).size(22))
                .push(scrollable_students)
                .push(Rule::horizontal(10))
                .push(
                    button(Text::new("Закрыть"))
                        .on_press(Message::CloseGroupStudentsModal)
                );

            let modal_container = Container::new(modal_content)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Fixed(500.0))
                .width(Length::Fixed(600.0));

            let modal_overlay = Container::new(
                mouse_area(Container::new(modal_container).center(Length::Fill))
                    .on_press(Message::CloseGroupStudentsModal) // Теперь можно закрывать по клику вне модалки
            )
                .width(Length::Fill).height(Length::Fill)
                .center_y(Length::Fill)
                .center_x(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(modal_overlay);
        }
    }


    Container::new(ui_stack)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}