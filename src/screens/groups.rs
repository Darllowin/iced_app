use iced::{widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable}, Alignment, Color, ContentFit, Length, Theme};
use iced::widget::{button, horizontal_space, image, pick_list, row, text, PickList, Rule};
use iced::widget::container::{background, bordered_box};
use iced::widget::image::Handle;
use iced_font_awesome::fa_icon_solid;
use crate::app::{App, Message};
use crate::app::state::{Course, Group, GroupStatus, ReportType, UserInfo, DEFAULT_AVATAR};
use crate::app::update::icon_button_content;

fn headerbar<'a>(group: &Group, app: &'a App) -> Row<'a, Message> {
    row![
        row![
            button(icon_button_content(
                fa_icon_solid("pen-to-square").style(move |_| text::base(&app.theme.target())),
                "Редактировать"
            )).on_press(Message::StartEditingGroup(group.clone())),
            button(icon_button_content(
                fa_icon_solid("users").style(move |_| text::base(&app.theme.target())),
                "Состав"
            )).on_press(Message::OpenManageStudentsModal(group.id)),
            button(icon_button_content(
                fa_icon_solid("book").style(move |_| text::base(&app.theme.target())),
                "Занятия"
            )).on_press(Message::OpenGroupLessonsModal(group.id, group.course_id.unwrap_or(0))),
        ].spacing(10),

        horizontal_space(),
        text(format!("{}", group.name)).size(26),
        horizontal_space(),
        button(fa_icon_solid("xmark").style(move |_| text::base(&app.theme.target()))).on_press(Message::DeleteGroup(group.id)),
    ]
        .width(Length::Fill)
}

fn content(group: Group, app: &App) -> Column<Message> {
    let course_name = group.course_name.unwrap_or_else(|| "Неизвестно".to_string());
    let teacher_name = group.teacher_name.unwrap_or_else(|| "Неизвестно".to_string());

    let content_col = Column::new()
        .push(Text::new(format!("Курс: {}", course_name)).size(22))
        .push(Text::new(format!("Преподаватель: {}", teacher_name)).size(22))
        .push(Text::new(format!("Количество студентов: {}", group.student_count)).size(22))
        .push(Text::new(format!("Статус: {}", group.status)).size(22));

    Column::new().push(
        Container::new(content_col)
            .padding(10)
            .width(Length::Fill)
            .style(move |_| bordered_box(&app.theme.target()))
    ).spacing(10)
}

pub fn groups_screen(app: &App) -> Container<Message> {
    let filter = app.group_filter_text.to_lowercase();
    let filtered_groups: Vec<Group> = app.all_groups
        .iter()
        .filter(|g| {
            g.name.to_lowercase().contains(&filter)
                || g.course_name.clone()
                .map_or(false, |title| title.to_lowercase().contains(&filter))
                || g.teacher_name.clone()
                .map_or(false, |name| name.to_lowercase().contains(&filter))
        })
        .cloned()
        .collect();

    let mut group_column = Column::new().spacing(20).padding(20);

    group_column = group_column
        .push(
            Row::new()
                .push(
                    Button::new(icon_button_content(
                        fa_icon_solid("plus").style(move |_| text::base(&app.theme.target())), // Изменено здесь
                        "Добавить группу"
                    ))
                        .on_press(Message::ToggleAddGroupModal(true))
                )
                .push(
                    TextInput::new("Поиск по группам...", &app.group_filter_text)
                        .on_input(Message::GroupFilterChanged)
                        .padding(10)
                        .size(18)
                        .width(Length::Fixed(400.0))
                ).spacing(10).align_y(Alignment::Center)
                .push(
                    Button::new(icon_button_content(
                        fa_icon_solid("certificate").style(move |_| text::base(&app.theme.target())), // Изменено здесь
                        "Отчёт"
                    )).on_press(Message::ToggleGroupReportModal)
                )
        );

    for group in filtered_groups {
        let group_content = Column::new().push(
            Container::new(
                Column::new()
                    .push(Container::new(headerbar(&group, &app)).padding(10).style(move |_| bordered_box(&app.theme.target())))
                    .push(Container::new(content(group.clone(), app)).style(move |_| bordered_box(&app.theme.target())))
            )
                .padding(10)
                .style(move |_| bordered_box(&app.theme.target()))
                .width(Length::Fill)
        );
        let group_card = Container::new(group_content)
            .width(Length::Fill);
        group_column = group_column.push(group_card);
    }

    let scrollable_groups = Scrollable::new(group_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let base_ui = Container::new(scrollable_groups)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut ui_stack = Stack::new().push(base_ui);

    // Модальное окно управления студентами
    if app.show_group_students_modal {
        if let Some(group_name) = &app.selected_group_for_students_name {
            // current_group_id нужен для добавления/удаления
            let current_group_id = app.current_manage_students_group_id.unwrap_or(0);
            let modal_title_text = format!("Состав группы: {}", group_name);

            let mut students_list_col: Column<'_, Message, Theme> = Column::new().spacing(5);
            println!("DEBUG VIEW: app.selected_group_students.len() = {}", app.selected_group_students.len());
            if app.is_loading_group_students { // <-- Проверяем флаг загрузки
                students_list_col = students_list_col.push(
                    Text::new("Загрузка студентов...").size(16).color(Color::from_rgb8(100, 100, 200)) // Добавлен цвет
                );
            } else if app.selected_group_students.is_empty() {
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
                        )
                        .push(horizontal_space())
                        .push(
                            button(fa_icon_solid("xmark")
                                .style(move |_| text::base(&app.theme.target())))
                                .on_press(Message::RemoveStudentFromGroup(student.id, current_group_id))
                        );

                    students_list_col = students_list_col.push(
                        Container::new(student_row_content)
                            .style(move |_| bordered_box(&app.theme.target()))
                            .width(Length::Fill)
                    );
                }
            }

            let scrollable_students = Scrollable::new(
                Container::new(students_list_col).padding(5)
            ).height(Length::FillPortion(1));

            // Логика добавления студента
            let add_student_row = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(Text::new("Добавить студента:").size(18))
                .push(
                    PickList::new(
                        app.students_without_group.clone(),
                        app.selected_student_to_add.clone(),
                        |student| Message::SelectedStudentToAddChanged(Some(student)), // Сообщение при выборе студента
                    ).placeholder("Выберите студента")
                )
                .push(
                    button(icon_button_content(
                        fa_icon_solid("plus").style(move |_| text::base(&app.theme.target())),
                        "Добавить"
                    ))
                        .on_press(Message::AddStudentToGroup(
                            app.selected_student_to_add.as_ref().map_or(0, |s| s.id), // ID выбранного студента
                            current_group_id // ID текущей группы
                        ))
                        .width(Length::Shrink)
                );

            let modal_content = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(modal_title_text).size(22))
                .push(scrollable_students)
                .push(Rule::horizontal(10))
                .push(add_student_row) // Добавляем строку для добавления студента
                .push(Text::new(app.group_error_message.clone().unwrap_or_default()).color(Color::from_rgb(1.0, 0.0, 0.0))) // Сообщение об ошибке
                .push(
                    button(icon_button_content(
                        fa_icon_solid("arrow-left").style(move |_| text::base(&app.theme.target())),
                        "Закрыть"
                    ))
                        .on_press(Message::CloseGroupStudentsModal)
                );

            let modal_container = Container::new(modal_content)
                .style(move |_| bordered_box(&app.theme.target()))
                .padding(20)
                .height(Length::Fixed(500.0))
                .width(Length::Fixed(600.0));

            let modal_overlay = Container::new(
                mouse_area(Container::new(modal_container).center(Length::Fill))
            )
                .width(Length::Fill).height(Length::Fill)
                .center_y(Length::Fill)
                .center_x(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(modal_overlay);
        }
    }

    if app.show_group_report_modal {
        let report_formats = vec![ReportType::PDF, ReportType::Excel];
        let selected_format = app.selected_report_type;

        let format_picklist = pick_list(
            report_formats.clone(),
            selected_format,
            |selected: ReportType| Message::ReportTypeSelected(Some(selected)),
        );


        let modal_content = Column::new()
            .spacing(15)
            .padding(20)
            .push(Text::new("Генерация отчёта по группам").size(24))
            .push(Text::new("Выберите период:"))
            .push(
                Row::new()
                    .spacing(15)
                    .align_y(Alignment::Center)
                    .push(format_picklist)
                    .push(
                        Button::new(icon_button_content(
                            fa_icon_solid("stamp").style(move |_| text::base(&app.theme.target())), 
                            "Отчёт"
                        ))
                            .on_press(Message::GenerateGroupReport),
                    )
                    .push(
                        Button::new(icon_button_content(
                            fa_icon_solid("arrow-left").style(move |_| text::base(&app.theme.target())), 
                            "Отмена"
                        ))
                            .on_press(Message::ToggleGroupReportModal),
                    ),
            )
            .push(Text::new(&app.error_message).size(24));

        let modal_container = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme.target()))
            .padding(20)
            .width(Length::Fixed(500.0))
            .height(Length::Shrink);

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal_container).center(Length::Fill))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .center_x(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        ui_stack = ui_stack.push(modal_overlay);
    }

    // Модальное окно добавления/редактирования группы
    if app.show_add_group_modal {
        let is_editing = app.editing_group.is_some();
        let modal_title = if is_editing { "Редактировать группу" } else { "Новая группа" };
        let submit_button_text = if is_editing { "Сохранить" } else { "Добавить" };
        let submit_message = if is_editing { Message::SubmitEditedGroup } else { Message::SubmitNewGroup };
        let cancel_message = if is_editing { Message::CancelEditingGroup } else { Message::ToggleAddGroupModal(false) };

        let (name_value, course_selected_value, teacher_selected_value, status_selected_value, name_changed_msg, course_changed_msg, teacher_changed_msg, status_changed_msg): (
            &String,
            Option<Course>,
            Option<UserInfo>,
            GroupStatus,
            Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(Option<Course>) -> Message>,
            Box<dyn Fn(Option<UserInfo>) -> Message>,
            Box<dyn Fn(GroupStatus) -> Message>
        ) = if is_editing {
            (
                &app.edit_group_name,
                app.edit_group_course.and_then(|course_id| {
                    app.courses_for_picklist.iter().find(|c| c.id == course_id).cloned()
                }),
                app.edit_group_teacher.and_then(|teacher_id| {
                    app.users_for_picklist.iter().find(|u| u.id == teacher_id).cloned()
                }),
                app.edit_group_status,
                Box::new(Message::EditGroupNameChanged),
                Box::new(Message::EditGroupCourseChanged),
                Box::new(Message::EditGroupTeacherChanged),
                Box::new(Message::EditGroupStatusChanged),
            )
        } else {
            (
                &app.new_group_name,
                app.new_group_course.and_then(|course_id| {
                    app.courses_for_picklist.iter().find(|c| c.id == course_id).cloned()
                }),
                app.new_group_teacher.and_then(|teacher_id| {
                    app.users_for_picklist.iter().find(|u| u.id == teacher_id).cloned()
                }),
                app.new_group_status,
                Box::new(Message::NewGroupNameChanged),
                Box::new(Message::NewGroupCourseChanged),
                Box::new(Message::NewGroupTeacherChanged),
                Box::new(Message::NewGroupStatusChanged),
            )
        };
        let status_options = vec![GroupStatus::Active, GroupStatus::Inactive];
        let modal_column = Column::new().spacing(10).width(Length::Fill)
            .push(Text::new(modal_title).size(24))
            .push(TextInput::new("Название группы", name_value)
                .on_input(move |s| name_changed_msg(s)))
            .push(PickList::new(
                app.courses_for_picklist.clone(),
                course_selected_value,
                move |course_selected: Course| course_changed_msg(Some(course_selected)),
            ).placeholder("Выберите курс"))
            .push(PickList::new(
                app.users_for_picklist.clone(),
                teacher_selected_value,
                move |user_selected_from_picklist: UserInfo| {
                    teacher_changed_msg(Some(user_selected_from_picklist))
                },
            ).placeholder("Выберите преподавателя"))
            .push(PickList::new(
                status_options, // Наши варианты статусов
                Some(status_selected_value), // Текущее выбранное значение
                move |status: GroupStatus| status_changed_msg(status), // Сообщение при изменении
            ).placeholder("Выберите статус"))
            .push(Text::new(app.group_error_message.clone().unwrap_or_default()))
            .push(
                Row::new()
                    .spacing(10)
                    .push(Button::new(icon_button_content(
                        fa_icon_solid("arrow-left").style(move |_| text::base(&app.theme.target())), // Изменено здесь
                        "Отмена"
                    )).on_press(cancel_message))
                    .push(Button::new(icon_button_content(
                        fa_icon_solid("bookmark").style(move |_| text::base(&app.theme.target())), // Изменено здесь
                        submit_button_text
                    )).on_press(submit_message))
            );
        let modal = Container::new(modal_column)
            .style(move |_| bordered_box(&app.theme.target()))
            .padding(20)
            .height(Length::Fixed(500.0))
            .width(Length::Fixed(800.0));

        let modal_overlay = Container::new(
            mouse_area(
                Container::new(modal)
                    .center(Length::Fill)
                    .padding(40),
            )
                .on_press(Message::ToggleAddGroupModal(false))
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        ui_stack = ui_stack.push(modal_overlay);
    }

    // Модальное окно "Занятия группы"
    if app.show_group_lessons_modal {
        let modal_title = format!("Занятия группы: {}", app.group_lessons_modal_group_name);
        let mut lessons_col = Column::new().spacing(10);

        if !app.group_lessons_modal_lessons.is_empty() {
            lessons_col = lessons_col.push(Text::new("Доступные занятия").size(20).color(Color::from_rgb8(142, 192, 124)));
            for lesson in &app.group_lessons_modal_lessons {
                lessons_col = lessons_col.push(
                    Container::new(
                        Row::new()
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .push(Text::new(format!("{}. {}", lesson.number, lesson.title)).width(Length::FillPortion(1)))
                            .push(Text::new("Статус: Предстоит").color(Color::from_rgb8(0, 150, 0)))
                    )
                        .padding(20)
                        .width(Length::Fill)
                        .style(move |_| bordered_box(&app.theme.target()))
                );
            }
        } else {
            lessons_col = lessons_col.push(Text::new("Нет доступных занятий.").size(16));
        }

        lessons_col = lessons_col.push(Rule::horizontal(10));

        if !app.group_lessons_modal_past_sessions.is_empty() {
            lessons_col = lessons_col.push(Text::new("Пройденные занятия").size(20).color(Color::from_rgb8(200, 0, 0)));
            for past_session in &app.group_lessons_modal_past_sessions {
                let lesson_title_display = past_session.lesson_title
                    .as_ref()
                    .map_or("Не указано".to_string(), |s| s.clone());

                let mut session_detail_col = Column::new()
                    .spacing(5)
                    .push(
                        Row::new()
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .push(Text::new(format!("{}. {} ({})",
                                                    past_session.lesson_number.unwrap_or(0),
                                                    lesson_title_display,
                                                    past_session.date)).width(Length::FillPortion(1)))
                            .push(Text::new("Статус: Пройдено").color(Color::from_rgb8(204, 36, 29)))
                    );

                // ЛОГИКА ОТОБРАЖЕНИЯ ПОСЕЩАЕМОСТИ СТУДЕНТОВ ---
                if !past_session.attendance_records.is_empty() {
                    session_detail_col = session_detail_col.push(
                        Text::new("Посещаемость студентов:").center()
                    );
                    for record in &past_session.attendance_records {
                        let status_color = if record.present_status == "Present" {
                            Color::from_rgb8(142, 192, 124) // Зеленый для "Присутствовал"
                        } else {
                            Color::from_rgb8(204, 36, 29) // Красный для "Отсутствовал"
                        };
                        session_detail_col = session_detail_col.push(
                            Text::new(format!("  • {}: {}", record.student_name, record.present_status))
                                .size(14)
                                .color(status_color)
                        );
                    }
                } else {
                    session_detail_col = session_detail_col.push(
                        Text::new("  Нет данных о посещаемости для этого занятия.").size(14)
                    );
                }

                lessons_col = lessons_col.push(
                    Container::new(session_detail_col) // Используем Column для деталей сессии
                        .padding(20)
                        .width(Length::Fill)
                        .style(move |_| bordered_box(&app.theme.target()))
                );
            }
        } else {
            lessons_col = lessons_col.push(Text::new("Нет пройденных занятий.").size(16));
        }

        let modal_content = Column::new()
            .spacing(15)
            .align_x(Alignment::Start)
            .push(Text::new(modal_title).size(24))
            .push(Scrollable::new(lessons_col).height(Length::FillPortion(1)))
            .push(Button::new(icon_button_content(
                fa_icon_solid("arrow-left").style(move |_| text::base(&app.theme.target())), // Изменено здесь
                "Закрыть"
            )).on_press(Message::CloseGroupLessonsModal));

        let modal = Container::new(modal_content)
            .style(move |_| bordered_box(&app.theme.target()))
            .padding(20)
            .height(Length::Fixed(600.0))
            .width(Length::Fixed(700.0));

        let modal_overlay = Container::new(
            mouse_area(Container::new(modal).center(Length::Fill))
                .on_press(Message::CloseGroupLessonsModal)
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .center_x(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));

        ui_stack = ui_stack.push(modal_overlay);
    }

    Container::new(ui_stack)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}