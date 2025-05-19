// В course.rs

use iced::{Color, ContentFit}; // Добавляем ContentFit если используем изображения
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::widget::container::{background, bordered_box};
use iced::widget::{button, horizontal_space, row, text, PickList, Rule};
use rusqlite::Connection;
use crate::app::{App, Course, Level, Message, Lesson, Screen, AssignmentType}; // Импортируем Lesson
use crate::db;

// <-- Обновляем headrbar, чтобы кнопка "Занятия" отправляла правильное сообщение
fn headrbar(course: Course) -> Row<'static, Message> { // Передаем тему
    row![
        row![
            button("Редактировать").on_press(Message::StartEditingCourse(course.clone())),
            button("Занятия").on_press(Message::ShowLessonsModal(course.clone())),
        ].spacing(10),
        horizontal_space(),
        text(format!("{}", course.title)).size(26),
        horizontal_space(),
        button("X").on_press(Message::DeleteCourse(course.id)),
    ]
        .width(Length::Fill)
        .align_y(Alignment::Center)
}

fn content(course: Course, app: &App) -> Column<Message> {
    let content_col = Column::new()
        .spacing(5)
        .push(Text::new(format!("Преподаватель: {}", course.instructor.unwrap_or_default())).size(20))
        .push(Text::new(format!("Уровень: {}", course.level.as_ref().map_or(String::new(), |l| l.to_string()))).size(18))
        .push(Text::new(format!("Занятий: {}", course.lesson_count)).size(18))
        .push(Text::new(""))
        .push(Text::new(format!("{}", course.description)).size(16))
        .padding(10);

    Column::new().push(
        Container::new(content_col)
            .width(Length::Fill)
            .style(move |_| bordered_box(&app.theme))
    )
}

pub fn courses_screen(app: &App) -> Container<Message> {
    // Соединение с БД для загрузки курсов. Операции изменения данных должны идти через App::update
    let conn = Connection::open("db_platform").unwrap(); // Для чтения списка курсов
    let courses = db::get_courses(&conn).unwrap_or_else(|e| {
        println!("!!! Ошибка при загрузке курсов из БД: {:?}", e);
        vec![]
    });
    let instructors = db::get_all_users(&conn).unwrap_or_default();


    let filter = app.course_filter_text.to_lowercase();
    let filtered_courses: Vec<Course> = courses
        .into_iter()
        .filter(|c| {
            c.title.to_lowercase().contains(&filter)
                || c.description.to_lowercase().contains(&filter)
                || c.instructor.clone().unwrap_or_default().to_lowercase().contains(&filter)
                || c.level.clone().unwrap_or_default().to_lowercase().contains(&filter) // Предполагаем, что level это String в Course
        })
        .collect();

    let mut courses_column = Column::new().spacing(15).padding(20);

    courses_column = courses_column
        .push(
            Row::new()
                .push(
                    Button::new(Text::new("Добавить курс"))
                        .on_press(Message::ToggleAddCourseModal(true))
                )
                .push(
                    TextInput::new("Поиск по курсам...", &app.course_filter_text)
                        .on_input(Message::CourseFilterChanged)
                        .padding(10)
                        .size(16)
                        .width(Length::Fixed(300.0))
                )
                .spacing(10)
                .align_y(Alignment::Center)
        );

    for course in filtered_courses {
        let course_content = Column::new().push(
            Container::new(
                Column::new()
                    .push(Container::new(headrbar(course.clone())).padding(10)).push(content(course.clone(), &app))).style(move |_| bordered_box(&app.theme))
                    
                )
                .padding(5)
                .width(Length::Fill);
        courses_column = courses_column.push(course_content);
    }

    let scrollable_courses = Scrollable::new(courses_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let base_ui = Container::new(scrollable_courses)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut ui_stack = Stack::new().push(base_ui);

    // --- Модальное окно для занятий ---
    if app.show_lessons_modal {
        if let Some(course_for_lessons) = &app.editing_lessons_course {
            let modal_title_text = format!("Занятия курса: {}", course_for_lessons.title);

            let lessons_list_col = app.course_lessons.iter().fold(Column::new().spacing(5), |col, lesson| {
                let lesson_row = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(Text::new(format!("{}. {}", lesson.number.unwrap_or(0), lesson.title)))
                    .push(horizontal_space())
                    .push(button("Задания").on_press(Message::ShowAssignmentsModal(lesson.clone())))
                    .push(button("X").on_press(Message::DeleteLesson(lesson.id)));
                col.push(Container::new(lesson_row).padding(5).width(Length::Fill).style(move |_| bordered_box(&app.theme)))
            });

            let scrollable_lessons = Scrollable::new(
                Container::new(lessons_list_col).style(move |_| bordered_box(&app.theme)).padding(10)
            ).height(Length::FillPortion(3)); // Больше места для списка

            let add_lesson_form = Column::new()
                .spacing(10)
                .push(Text::new("Добавить новое занятие").size(18))
                .push(TextInput::new("Номер", &app.new_lesson_number_text).on_input(Message::NewLessonNumberChanged).width(Length::Fixed(100.0)))
                .push(TextInput::new("Название", &app.new_lesson_title).on_input(Message::NewLessonTitleChanged).width(Length::Fill))
                .push(button("Добавить").on_press(Message::AddLesson))
                .width(Length::Fill);

            let mut lessons_modal_content_col = Column::new()
                .spacing(15)
                .push(Text::new(modal_title_text).size(24))
                .push(scrollable_lessons)
                .push(Rule::horizontal(10)) // Разделитель
                .push(add_lesson_form);

            if let Some(error_msg) = &app.lesson_error_message {
                lessons_modal_content_col = lessons_modal_content_col.push(Text::new(error_msg).size(16));
            }
            lessons_modal_content_col = lessons_modal_content_col.push(button("Закрыть").on_press(Message::CloseLessonsModal));

            let lessons_modal_container = Container::new(lessons_modal_content_col)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Fixed(600.0)) // Увеличил высоту
                .width(Length::Fixed(800.0));

            let lessons_modal_overlay = Container::new(
                Container::new(lessons_modal_container).center(Length::Fill)
            )
                .width(Length::Fill).height(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(lessons_modal_overlay);
        }
    }

    // --- Модальное окно для ЗАДАНИЙ ---
    if app.show_assignments_modal {
        if let Some(lesson) = &app.current_lesson_for_assignments {
            let assignments_modal_title_text = format!("Задания для: {} {}", lesson.number.unwrap_or(0), lesson.title);

            let mut assignments_list_col = Column::new().spacing(5);
            if app.lesson_assignments.is_empty() {
                assignments_list_col = assignments_list_col.push(Text::new("Для этого занятия еще нет заданий.").size(16));
            } else {
                for assignment in &app.lesson_assignments {
                    let assignment_row = Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(Text::new(format!("{} ({})", assignment.title, assignment.assignment_type)).width(Length::FillPortion(3)))
                        .push(Text::new(&assignment.description).width(Length::FillPortion(5))) // Описание может быть длинным
                        .push(horizontal_space())
                        .push(button("Открыть").on_press(Message::ShowAssignmentDetailModal(assignment.clone())))
                        .push(button("X").on_press(Message::DeleteAssignment(assignment.id)));
                    assignments_list_col = assignments_list_col.push(
                        Container::new(assignment_row).padding(5).width(Length::Fill).style(move |_| bordered_box(&app.theme))
                    );
                }
            }

            let scrollable_assignments = Scrollable::new(
                Container::new(assignments_list_col).padding(5) // Без рамки вокруг самого списка, рамки у каждого элемента
            ).height(Length::FillPortion(2)); // Доля высоты для списка заданий

            let add_assignment_form = Column::new()
                .spacing(10)
                .push(Text::new("Добавить новое задание").size(18))
                .push(TextInput::new("Название задания", &app.new_assignment_title).on_input(Message::NewAssignmentTitleChanged))
                .push(TextInput::new("Описание", &app.new_assignment_description).on_input(Message::NewAssignmentDescriptionChanged))
                .push(
                    PickList::new(
                        AssignmentType::ALL.to_vec(), // Преобразуем срез в Vec, так как PickList ожидает 'static lifetime или owned data
                        app.new_assignment_type,      // Выбранное значение (Option<AssignmentType>)
                        Message::NewAssignmentTypeSelected // Сообщение при выборе
                    )
                        .placeholder("Выберите тип задания")
                )
                .push(button("Добавить задание").on_press(Message::AddAssignment))
                .width(Length::Fill);

            let mut assignments_modal_col = Column::new()
                .spacing(15)
                .align_x(Alignment::Start) // Выравниваем по левому краю
                .push(Text::new(assignments_modal_title_text).size(22))
                .push(scrollable_assignments)
                .push(Rule::horizontal(10)) // Разделитель
                .push(add_assignment_form);

            if let Some(error_msg) = &app.assignment_error_message {
                assignments_modal_col = assignments_modal_col.push(Text::new(error_msg).size(16));
            }
            assignments_modal_col = assignments_modal_col.push(button("Закрыть").on_press(Message::CloseAssignmentsModal));


            let assignments_modal_container = Container::new(assignments_modal_col)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Fixed(550.0)) // Высота модалки заданий
                .width(Length::Fixed(700.0));  // Ширина модалки заданий

            let assignments_modal_overlay = Container::new(
                Container::new(assignments_modal_container).center(Length::Fill)
            )
                .width(Length::Fill).height(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(assignments_modal_overlay);
        }
    }

    // --- НОВОЕ: Модальное окно для ДЕТАЛЕЙ ЗАДАНИЯ ---
    if app.show_assignment_detail_modal {
        if let Some(selected_assignment) = &app.selected_assignment_for_detail {
            let detail_modal_title = format!("Детали задания: {}", selected_assignment.title);

            let assignment_details_content = Column::new()
                .spacing(10)
                .push(Text::new(format!("Тип: {}", selected_assignment.assignment_type)).size(18))
                .push(Rule::horizontal(5))
                .push(Text::new("Описание:").size(18))
                .push(Scrollable::new(Text::new(&selected_assignment.description).size(16)).height(Length::FillPortion(1))) // Описание может быть длинным
                .width(Length::Fill);

            let mut detail_modal_col = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(detail_modal_title).size(22))
                .push(Container::new(assignment_details_content).padding(10).style(move |_| bordered_box(&app.theme)))
                .push(Rule::horizontal(10));

            // Кнопка закрытия
            let close_button_row = Row::new().push(horizontal_space()).push(button("Закрыть").on_press(Message::CloseAssignmentDetailModal));
            detail_modal_col = detail_modal_col.push(close_button_row);

            let detail_modal_container = Container::new(detail_modal_col)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Fixed(400.0)) // Размер модального окна деталей
                .width(Length::Fixed(600.0));

            let detail_modal_overlay = Container::new(
                Container::new(detail_modal_container).center(Length::Fill)
            )
                .width(Length::Fill).height(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 })); // Еще темнее, так как поверх всех
            ui_stack = ui_stack.push(detail_modal_overlay);
        }
    }

    // --- Модальное окно для добавления/редактирования КУРСА ---
    if app.show_add_course_modal {
        let is_editing = app.editing_course.is_some();
        let modal_title_text = if is_editing { "Редактировать курс" } else { "Новый курс" };
        let submit_button_text = if is_editing { "Сохранить" } else { "Добавить" };
        let submit_message = if is_editing { Message::SubmitEditedCourse } else { Message::SubmitNewCourse };
        let cancel_message = if is_editing { Message::CancelEditingCourse } else { Message::ToggleAddCourseModal(false) };

        let (title_val, desc_val, instructor_val, level_val, title_ch_msg, desc_ch_msg, instr_ch_msg, level_ch_msg) : (
            &String, &String, Option<String>, Option<Level>,
            Box<dyn Fn(String) -> Message>, Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(Option<String>) -> Message>, Box<dyn Fn(Level) -> Message>
        ) = if is_editing {
            (
                &app.edit_course_title, &app.edit_course_description,
                app.edit_course_instructor.clone(), Some(app.edit_course_level), // app.edit_course_level уже Level
                Box::new(Message::EditCourseTitleChanged), Box::new(Message::EditCourseDescriptionChanged),
                Box::new(Message::EditCourseInstructorChanged), Box::new(Message::EditCourseLevelChanged),
            )
        } else {
            (
                &app.new_course_title, &app.new_course_description,
                app.new_course_instructor.clone(), Some(app.new_course_level), // app.new_course_level уже Level
                Box::new(Message::NewCourseTitleChanged), Box::new(Message::NewCourseDescriptionChanged),
                Box::new(Message::NewCourseInstructorChanged), Box::new(Message::NewCourseLevelChanged),
            )
        };

        let modal_content_col = Column::new()
            .spacing(10)
            .push(Text::new(modal_title_text).size(24))
            .push(TextInput::new("Название курса", title_val).on_input(move |s| title_ch_msg(s)))
            .push(TextInput::new("Описание курса", desc_val).on_input(move |s| desc_ch_msg(s)))
            .push(PickList::new(instructors.clone(), instructor_val, move |name| instr_ch_msg(Some(name))).placeholder("Выберите преподавателя"))
            .push(PickList::new(Level::ALL.to_vec(), level_val, move |level| level_ch_msg(level)).placeholder("Выберите уровень"))
            .push(
                Row::new().spacing(10)
                    .push(Button::new(Text::new("Отмена")).on_press(cancel_message.clone()))
                    .push(Button::new(Text::new(submit_button_text)).on_press(submit_message))
            );

        let course_modal_container = Container::new(modal_content_col)
            .style(move |_| bordered_box(&app.theme))
            .padding(20).width(Length::Fixed(400.0));

        let course_modal_overlay = Container::new(
            mouse_area(Container::new(course_modal_container).center(Length::Fill))
                .on_press(if is_editing { Message::Er("".to_string()) } else { cancel_message })
        )
            .width(Length::Fill).height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
        ui_stack = ui_stack.push(course_modal_overlay);
    }
    Container::new(ui_stack)
        .center_x(Length::Fill).center_y(Length::Fill) // Центрируем Stack
}