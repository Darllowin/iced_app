use iced::{Color, };
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::widget::container::{background, bordered_box};
use iced::widget::{button, horizontal_space, row, text, PickList, Rule, TextEditor};
use rusqlite::Connection;
use crate::app::{App, Message};
use crate::app::state::{AssignmentType, Course, Level, TextInputOrEditorInput, PATH_TO_DB};
// Импортируем Lesson
use crate::db;

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
        .push(Text::new(format!("Уровень: {}", course.level.as_ref().map_or(String::new(), |l| l.to_string()))).size(18))
        .push(Text::new(format!("Занятий: {}", course.lesson_count)).size(18))
        .push(Text::new(format!("Запланированные места: {}", course.total_seats.unwrap())).size(18))
        .push(Text::new(format!("Свободные места: {}", course.seats.unwrap())).size(18))
        .push(Text::new(format!("Цена: {}₽", course.price.unwrap())).size(18))
        .push(Text::new(""))
        .push(Text::new(format!("{}", course.description.unwrap())).size(16))
        .padding(10);

    Column::new().push(
        Container::new(content_col)
            .width(Length::Fill)
            .style(move |_| bordered_box(&app.theme))
    )
}

pub fn courses_screen(app: &App) -> Container<Message> {
    // Соединение с БД для загрузки курсов. Операции изменения данных должны идти через App::update
    let conn = Connection::open(PATH_TO_DB).unwrap(); // Для чтения списка курсов
    let courses = db::get_courses(&conn).unwrap_or_else(|e| {
        println!("!!! Ошибка при загрузке курсов из БД: {:?}", e);
        vec![]
    });
    let filter = app.course_filter_text.to_lowercase();
    let filtered_courses: Vec<Course> = courses
        .into_iter()
        .filter(|c| {
            c.title.to_lowercase().contains(&filter)
                || c.description.clone().expect("REASON").to_lowercase().contains(&filter)
                || c.level.clone().unwrap_or_default().to_lowercase().contains(&filter)
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
            Container::new(Column::new()
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
                    .push(Text::new(format!("{}. {}", lesson.number, lesson.title)))
                    .push(horizontal_space())
                    .push(button("Задания").on_press(Message::ShowAssignmentsModal(lesson.clone())))
                    .push(button("X").on_press(Message::DeleteLesson(lesson.id)));
                col.push(Container::new(lesson_row).padding(5).width(Length::Fill).style(move |_| bordered_box(&app.theme)))
            });

            let scrollable_lessons = Scrollable::new(
                Container::new(lessons_list_col).style(move |_| bordered_box(&app.theme)).padding(10).width(Length::Fill)
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
            let assignments_modal_title_text = format!("Задания для: {} {}", lesson.number, lesson.title);

            let mut assignments_list_col = Column::new().spacing(5);
            if app.lesson_assignments.is_empty() {
                assignments_list_col = assignments_list_col.push(Text::new("Для этого занятия еще нет заданий.").size(16));
            } else {
                for assignment in &app.lesson_assignments {
                    let assignment_row = Row::new()
                        .spacing(10)
                        .width(Length::Fill)
                        .align_y(Alignment::Center)
                        .push(Text::new(format!("{} ({})", assignment.title, assignment.assignment_type)).width(Length::FillPortion(3)))
                        .push(Text::new(&assignment.description).width(Length::FillPortion(5)).height(Length::Fixed(30.0))) // Описание может быть длинным
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
                .height(Length::Fixed(600.0)) // Высота модалки заданий
                .width(Length::Fixed(800.0));  // Ширина модалки заданий

            let assignments_modal_overlay = Container::new(
                Container::new(assignments_modal_container).center(Length::Fill).width(Length::Fill)
            )
                .width(Length::Fill).height(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(assignments_modal_overlay);
        }
    }

    // Модальное окно для ДЕТАЛЕЙ ЗАДАНИЯ 
    if app.show_assignment_detail_modal {
        if let Some(selected_assignment) = &app.selected_assignment_for_detail {
            let detail_modal_title = format!("Редактирование: {}", app.editing_assignment_title); // Используем редактируемый заголовок

            let mut content_specific_to_type = Column::new().spacing(10);

            // Поле для редактирования названия задания (общее для всех типов)
            content_specific_to_type = content_specific_to_type
                .push(Text::new("Название задания:").size(16))
                .push(TextInput::new("Введите название...", &app.editing_assignment_title)
                    .on_input(Message::EditingAssignmentTitleChanged));

            // Определяем, какой тип задания редактируется (сравниваем строку из БД с .to_string() от enum)
            let assignment_type_str = &selected_assignment.assignment_type; // Это String

            if *assignment_type_str == AssignmentType::Lecture.to_string() {
                content_specific_to_type = content_specific_to_type
                    .push(Text::new("Текст лекции:").size(16))
                    // *** ИЗМЕНЕНО: Используем TextEditor для Лекции ***
                    .push(Scrollable::new(
                        TextEditor::new(&app.editing_assignment_description_content) // Привязываем к TextEditor контенту
                            .placeholder("Введите текст лекции...") // Placeholder
                            // Используем on_action и оборачиваем в TextInputOrEditorInput::TextEditor
                            .on_action(|action| Message::EditingAssignmentDescriptionChanged(TextInputOrEditorInput::TextEditor(action)))
                        ).height(Length::Fixed(300.0)) 
                    )
            } else if *assignment_type_str == AssignmentType::Practice.to_string() {
                content_specific_to_type = content_specific_to_type
                    .push(Text::new("Описание практического задания:").size(16))
                    // *** ИЗМЕНЕНО: Используем TextEditor для Практики ***
                    .push(Scrollable::new(
                        TextEditor::new(&app.editing_assignment_description_content) // Привязываем к TextEditor контенту
                            .placeholder("Введите описание...") // Placeholder
                            // Используем on_action и оборачиваем в TextInputOrEditorInput::TextEditor
                            .on_action(|action| Message::EditingAssignmentDescriptionChanged(TextInputOrEditorInput::TextEditor(action)))
                        ).height(Length::Fixed(300.0))
                    );
            } else if *assignment_type_str == AssignmentType::Test.to_string() {
                content_specific_to_type = content_specific_to_type
                    .push(Text::new("Общее описание/инструкции к тесту:").size(16))
                    // *** ИЗМЕНЕНО: Используем поле для TextInput и оборачиваем ввод ***
                    .push(TextInput::new("Введите описание...", &app.editing_assignment_description_text_input) // Привязываем к TextInput String полю
                        // Используем on_input и оборачиваем в TextInputOrEditorInput::TextInput
                        .on_input(|s| Message::EditingAssignmentDescriptionChanged(TextInputOrEditorInput::TextInput(s))))
                    .push(Rule::horizontal(5))
                    .push(Text::new("Примечание: Детальное создание вопросов и ответов для тестов будет доступно в следующих версиях.").size(14));
            } else {
                // Для неизвестных или других типов просто показываем описание
                content_specific_to_type = content_specific_to_type
                    .push(Text::new("Описание:").size(16))
                    // *** ИЗМЕНЕНО: Используем поле для TextInput и оборачиваем ввод ***
                    .push(TextInput::new("...", &app.editing_assignment_description_text_input) // Привязываем к TextInput String полю
                        // Используем on_input и оборачиваем в TextInputOrEditorInput::TextInput
                        .on_input(|s| Message::EditingAssignmentDescriptionChanged(TextInputOrEditorInput::TextInput(s))));
            }

            let mut detail_modal_col = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(detail_modal_title).size(22))
                .push(Container::new(Scrollable::new(content_specific_to_type)).padding(5).style(move |_| bordered_box(&app.theme)))
                .push(Rule::horizontal(10));

            if let Some(error_msg) = &app.assignment_edit_error_message {
                detail_modal_col = detail_modal_col.push(Text::new(error_msg).size(16));
            }

            let buttons_row = Row::new()
                .spacing(10)
                .push(button("Отмена").on_press(Message::CloseAssignmentDetailModal))
                .push(button("Сохранить").on_press(Message::SaveEditedAssignment));

            detail_modal_col = detail_modal_col.push(buttons_row.align_y(Alignment::End)); // Выравнивание ряда кнопок

            let detail_modal_container = Container::new(detail_modal_col)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Shrink) // Автоматическая высота, но не более экрана
                .width(Length::Fixed(700.0)); // Ширина модалки редактирования

            let detail_modal_overlay = Container::new(
                Container::new(detail_modal_container).center(Length::Fill) // Центрируем модалку
            )
                .width(Length::Fill).height(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
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

        let (title_val, desc_val, level_val, total_seats_val, seats_val, price_val, title_ch_msg, desc_ch_msg, level_ch_msg, total_seats_ch_msg, seats_ch_msg, price_ch_msg) : (
            &String,
            &String,
            Option<Level>,
            i32,
            i32,
            f64,
            Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(Level) -> Message>,
            Box<dyn Fn(i32) -> Message>,
            Box<dyn Fn(i32) -> Message>,
            Box<dyn Fn(f64) -> Message>
        ) = if is_editing {
            (
                &app.edit_course_title,
                &app.edit_course_description,
                Some(app.edit_course_level),
                app.edit_course_total_seats,
                app.edit_course_seats,
                app.edit_course_price,
                Box::new(Message::EditCourseTitleChanged),
                Box::new(Message::EditCourseDescriptionChanged),
                Box::new(Message::EditCourseLevelChanged),
                Box::new(Message::EditCourseTotalSeatsChanged),
                Box::new(Message::EditCourseSeatsChanged),
                Box::new(Message::EditCoursePriceChanged),
            )
        } else {
            (
                &app.new_course_title,
                &app.new_course_description,
                Some(app.new_course_level),
                app.new_course_total_seats,
                app.new_course_seats,
                app.new_course_price,
                Box::new(Message::NewCourseTitleChanged),
                Box::new(Message::NewCourseDescriptionChanged),
                Box::new(Message::NewCourseLevelChanged),
                Box::new(Message::NewCourseTotalSeatsChanged),
                Box::new(Message::NewCourseSeatsChanged),
                Box::new(Message::NewCoursePriceChanged),
            )
        };

        let modal_content_col = Column::new()
            .spacing(10)
            .push(Text::new(modal_title_text).size(24))
            .push(TextInput::new("Название курса", title_val).on_input(move |s| title_ch_msg(s)))
            .push(TextInput::new("Описание курса", desc_val).on_input(move |s| desc_ch_msg(s)))
            .push(TextInput::new("Запланированные в курсе", &total_seats_val.to_string()).on_input(move |s| total_seats_ch_msg(s.parse().unwrap_or(total_seats_val))))
            .push(TextInput::new("Свободные места в курсе", &seats_val.to_string()).on_input(move |s| seats_ch_msg(s.parse().unwrap_or(seats_val))))
            .push(TextInput::new("Цена курса", &price_val.to_string()).on_input(move |s| {price_ch_msg(s.parse().unwrap_or(price_val))}))
            .push(PickList::new(Level::ALL.to_vec(), level_val, move |level| level_ch_msg(level)).placeholder("Выберите уровень"))
            .push(Text::new(app.course_error_message.clone().unwrap_or_default()))
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
                .on_press(Message::Er("".to_string()))
        )
            .width(Length::Fill).height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
        ui_stack = ui_stack.push(course_modal_overlay);
    }
    Container::new(ui_stack)
        .center_x(Length::Fill).center_y(Length::Fill) // Центрируем Stack
}