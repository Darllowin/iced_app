use iced::{widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable}, Alignment, Color, ContentFit, Length};
use iced::widget::{button, horizontal_space, image, row, text, Image, PickList};
use iced::widget::container::{background, bordered_box};
use rusqlite::Connection;
use crate::app::{App, Group, Message, UserInfo, DEFAULT_AVATAR};
use crate::db;

fn headerbar(group: Group) -> Row<'static, Message> {
    row![
        row![
            button("Редактировать").on_press(Message::StartEditingGroup(group.clone())),
            button("Состав").on_press(Message::OpenManageStudentsModal(group.id)),
        ].spacing(10),
        
        horizontal_space(),
        text(format!("{}", group.name)).size(26),
        horizontal_space(),
        button("X").on_press(Message::DeleteGroup(group.id)),
    ]
        .width(Length::Fill)
}

fn content(group: Group, app: &App) -> Column<Message> {
    let course_name = group.course_name.unwrap_or_else(|| "Неизвестно".to_string());
    let teacher_name = group.teacher_name.unwrap_or_else(|| "Неизвестно".to_string());

    let content_col = Column::new()
        .push(Text::new(format!("Курс: {}", course_name)).size(22))
        .push(Text::new(format!("Преподаватель: {}", teacher_name)).size(22))
        .push(Text::new(format!("Количество студентов: {}", group.student_count)).size(22));

    Column::new().push(
        Container::new(content_col)
            .padding(10)
            .width(Length::Fill)
            .style(move |_| bordered_box(&app.theme)) // Используем app.theme напрямую
    ).spacing(10)
}

pub fn groups_screen(app: &App) -> Container<Message> {
    let conn = Connection::open("db_platform").unwrap();
    let groups = db::get_groups(&conn).unwrap_or_default();
    let courses = db::get_courses(&conn).unwrap_or_default();
    let users = db::get_all_users(&conn).unwrap_or_else(|e| {
        eprintln!("ERROR (groups_screen): Не удалось загрузить пользователей для PickList: {}", e);
        // Если критично, можно показать ошибку пользователю или вернуть пустой экран
        // Например: return Container::new(Text::new(format!("Ошибка: {}", e))).center_x().center_y();
        vec![] // Вернем пустой Vec, чтобы приложение не падало
    });
    //println!("DEBUG (groups_screen): Количество пользователей для PickList: {}", users.len());
    let students_without_group = db::get_students_without_group(&conn).unwrap_or_default();

    

    let filter = app.group_filter_text.to_lowercase();
    let filtered_groups: Vec<Group> = groups
        .into_iter()
        .filter(|g| {
            g.name.to_lowercase().contains(&filter)
                || g.course_name.clone() // Используем новое поле course_name для фильтрации
                .map_or(false, |title| title.to_lowercase().contains(&filter))
                || g.teacher_name.clone() // Используем новое поле teacher_name для фильтрации
                .map_or(false, |name| name.to_lowercase().contains(&filter))
        })
        .collect();

    let mut group_column = Column::new().spacing(20).padding(20);

    group_column = group_column
        .push(
            Row::new()
                .push(
                    Button::new(Text::new("Добавить группу"))
                        .on_press(Message::ToggleAddGroupModal(true))
                )
                .push(
                    TextInput::new("Поиск по группам...", &app.group_filter_text)
                        .on_input(Message::GroupFilterChanged)
                        .padding(10)
                        .size(18)
                        .width(Length::Fixed(400.0))
                ).spacing(10).align_y(Alignment::Center)
        );

    for group in filtered_groups {
        let group_content = Column::new().push(
            Container::new(
                Column::new()
                    .push(Container::new(headerbar(group.clone())).padding(10).style(move |_| bordered_box(&app.theme))) // `headerbar` все еще принимает Group
                .push(Container::new(content(group, app)).style(move |_| bordered_box(&app.theme))) // <--- Передаем `&app`
        )
            .padding(10)
            .style(move |_| bordered_box(&app.theme))
            .width(Length::Fill)
        );
        let group_card = Container::new(group_content)
            .width(Length::Fill);
        group_column = group_column.push(group_card);
    }

    let scrollable_groups = Scrollable::new(group_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let base = Container::new(scrollable_groups)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    if app.show_add_group_modal || app.is_manage_students_modal_open {
        let mut modal_column = Column::new().spacing(10).width(Length::Fill);

        if app.is_manage_students_modal_open {
            modal_column = modal_column
                .push(text("Студенты в группе").size(24))
                .push(
                    Container::new(
                        Scrollable::new(
                            Column::with_children(
                                app.group_students.iter().map(|student| {
                                    let avatar = if let Some(mut data) = student.avatar_data.clone() {
                                        data.extend_from_slice(student.email.as_bytes());
                                        Image::new(image::Handle::from_bytes(data))
                                            .width(100)
                                            .height(100)
                                            .content_fit(ContentFit::Fill)
                                    } else {
                                        Image::new(DEFAULT_AVATAR)
                                            .width(100)
                                            .height(100)
                                            .content_fit(ContentFit::Fill)
                                    };

                                    row![
                                        Container::new(avatar)
                                            .padding(5)
                                            .style(move |_| bordered_box(&app.theme)),
                                        Container::new(
                                            Column::new()
                                                .push(text(format!("ФИО: {}", &student.name)).size(20))
                                                .push(text(format!("Дата рождения: {}", &student.birthday)).size(20))
                                                .push(text(format!("Email: {}", &student.email)).size(20))
                                                .spacing(5)
                                        ),
                                        horizontal_space(),
                                        button("X").on_press(Message::RemoveStudent(student.name.clone()))
                                    ]
                                        .spacing(10)
                                        .width(Length::Fill)
                                        .into()
                                }).collect::<Vec<_>>()
                            )
                                .spacing(5)
                                .padding(10)
                        )
                            .height(Length::Fixed(300.0)) // ограничь высоту для скролла
                            .width(Length::Fill)
                    )
                        .style(move |_| bordered_box(&app.theme))
                )
                .push(
                    PickList::new(
                        students_without_group.clone(),
                        app.selected_student_to_add.clone(),
                        |s| Message::StudentToAddSelected(Some(s))
                    ).placeholder("Выберите студента")
                )
                .push(
                    button("Добавить")
                        .on_press(Message::AddStudent)
                )
                .push(button("Закрыть").on_press(Message::CloseManageStudentsModal));
        } else {
            let is_editing = app.editing_group.is_some();
            let modal_title = if is_editing { "Редактировать группу" } else { "Новая группа" };
            let submit_button_text = if is_editing { "Сохранить" } else { "Добавить" };
            let submit_message = if is_editing { Message::SubmitEditedGroup } else { Message::SubmitNewGroup };
            let cancel_message = if is_editing { Message::CancelEditingGroup } else { Message::ToggleAddGroupModal(false) };

            let (name_value, course_value, teacher_value, name_changed_msg, course_changed_msg, teacher_changed_msg): (
                &String,
                Option<i32>,
                Option<i32>,
                Box<dyn Fn(String) -> Message>,
                Box<dyn Fn(Option<i32>) -> Message>,
                Box<dyn Fn(Option<i32>) -> Message>
            ) = if is_editing {
                (
                    &app.edit_group_name,
                    app.edit_group_course,
                    app.edit_group_teacher,
                    Box::new(Message::EditGroupNameChanged),
                    Box::new(Message::EditGroupCourseChanged),
                    Box::new(Message::EditGroupTeacherChanged),
                )
            } else {
                (
                    &app.new_group_name,
                    app.new_group_course.clone(),
                    app.new_group_teacher.clone(),
                    Box::new(Message::NewGroupNameChanged),
                    Box::new(Message::NewGroupCourseChanged),
                    Box::new(Message::NewGroupTeacherChanged),
                )
            };



            modal_column = modal_column
                .push(Text::new(modal_title).size(24))
                .push(TextInput::new("Название группы", name_value)
                    .on_input(move |s| name_changed_msg(s)))
                .push(PickList::new(
                    courses.clone(),
                    course_value.and_then(|id| courses.iter().find(|c| c.id == id).cloned()),
                    move |course_selected: crate::app::Course| course_changed_msg(Some(course_selected.id)),
                ).placeholder("Выберите курс"))
                .push(PickList::new(
                    users.clone(),
                    teacher_value.and_then(|id| users.iter().find(|u| u.id == id).cloned()),
                    move |user_selected_from_picklist: UserInfo| { // <--- UserInfo (без crate::app::)
                        teacher_changed_msg(Some(user_selected_from_picklist.id))
                    },
                ).placeholder("Выберите преподавателя"))
                .push(
                    Row::new()
                        .spacing(10)
                        .push(Button::new(Text::new("Отмена")).on_press(cancel_message))
                        .push(Button::new(Text::new(submit_button_text)).on_press(submit_message))
                );
        }
        
        let modal = Container::new(modal_column)
            .style(move |_| bordered_box(&app.theme))
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

        Container::new(
            Stack::new()
                .push(base)
                .push(modal_overlay)
        )
    } else {
        base
    }
}
