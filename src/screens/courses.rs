use iced::{Color};
use iced::{
    widget::{Button, Column, Container, Row, Stack, Text, TextInput, mouse_area, Scrollable},
    Alignment, Length
};
use iced::widget::container::{background, bordered_box};
use iced::widget::{button, horizontal_space, row, text, PickList};
use rusqlite::Connection;
use crate::app::{App, Course, Level, Message};
use crate::db;

fn headrbar(course: Course) -> Row<'static, Message> {
    row![
        button("Редактировать").on_press(Message::StartEditingCourse(course.clone())),
        horizontal_space(),
        text(format!("{}", course.title)).size(26),
        horizontal_space(),
        button("X").on_press(Message::DeleteCourse(course.id)),
    ]
        .width(Length::Fill)
}

fn content(course: Course, app: &App) -> Column<Message> {
    let content = Column::new()
        .push(Text::new(format!("Преподаватель: {}", course.instructor.unwrap_or(String::from("")))).size(24))
        .push(Text::new(format!("Уровень: {}", course.level.unwrap_or(String::from("")))).size(22))
        .push(Text::new(""))
        .push(Text::new(format!("{}", course.description)).size(18));
    
    Column::new().push(
        Container::new(content)
            .width(Length::Fill)
            .style(move |_| bordered_box(&app.theme))
    )
}
pub fn courses_screen(app: &App) -> Container<Message> {
    let conn = Connection::open("db_platform").unwrap();
    let courses = db::get_courses(&conn).unwrap_or_default();
    let instructors = db::get_all_users(&conn).unwrap_or_default();

    let mut courses_column = Column::new().spacing(20).padding(20);

    courses_column = courses_column.push(
        Button::new(Text::new("Добавить курс"))
            .on_press(Message::ToggleAddCourseModal(true))
            .width(Length::Shrink)
    );

    for course in courses {
        // Создаем контент карточки курса
        let course_content = Column::new().push(
            Container::new(
                Column::new()
                    .push(headrbar(course.clone()))
                    .push(content(course, &app))
                    .height(Length::Shrink)
            )
                //.style(move |_| background(Color::default()))
                .style(move |_| bordered_box(&app.theme))
                .width(Length::Fill)
        );
        
        // Оборачиваем контент в Container и применяем стиль, похожий на Card
        let course_card_like = Container::new(course_content)
            .width(Length::Fill); // Карточка занимает всю доступную ширину

        courses_column = courses_column.push(course_card_like); // Теперь просто пушим контейнер-карточку
    }

    // Оборачиваем колонку с курсами в Scrollable
    let scrollable_courses = Scrollable::new(courses_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let base = Container::new(scrollable_courses)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill);


    // Модальное окно для добавления ИЛИ редактирования курса
    if app.show_add_course_modal {
        let is_editing = app.editing_course.is_some();
        let modal_title = if is_editing { "Редактировать курс" } else { "Новый курс" };
        let submit_button_text = if is_editing { "Сохранить" } else { "Добавить" };
        let submit_message = if is_editing { Message::SubmitEditedCourse } else { Message::SubmitNewCourse };
        let cancel_message = if is_editing { Message::CancelEditingCourse } else { Message::ToggleAddCourseModal(false) };

        // Используем поля редактирования, если редактируем, иначе поля добавления
        // Используем Box<dyn Fn> для унификации типов функций-обработчиков
        let (title_value, description_value, instructor_value, level_value, title_changed_msg, description_changed_msg, instructor_changed_msg, level_changed_msg) : (
            &String,
            &String,
            Option<String>,
            Option<Level>,
            Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(String) -> Message>,
            Box<dyn Fn(Option<String>) -> Message>,
            Box<dyn Fn(Level) -> Message>
        ) = if is_editing {
            (
                &app.edit_course_title,
                &app.edit_course_description,
                app.edit_course_instructor.clone(),
                Some(app.edit_course_level),
                Box::new(Message::EditCourseTitleChanged),
                Box::new(Message::EditCourseDescriptionChanged),
                Box::new(Message::EditCourseInstructorChanged),
                Box::new(Message::EditCourseLevelChanged),
            )
        } else {
            (
                &app.new_course_title,
                &app.new_course_description,
                app.new_course_instructor.clone(),
                Some(app.new_course_level),
                Box::new(Message::NewCourseTitleChanged),
                Box::new(Message::NewCourseDescriptionChanged),
                Box::new(Message::NewCourseInstructorChanged),
                Box::new(Message::NewCourseLevelChanged),
            )
        };


        let modal_content = Column::new()
            .spacing(10)
            .push(Text::new(modal_title).size(24))
            .push(TextInput::new("Название курса", title_value)
                .on_input(move |s| title_changed_msg(s))) // Используем boxed closure
            .push(TextInput::new("Описание курса", description_value)
                .on_input(move |s| description_changed_msg(s))) // Используем boxed closure
            .push(PickList::new(
                instructors.clone(),
                instructor_value, // Используем соответствующее поле
                move |instructor_name: String| instructor_changed_msg(Some(instructor_name)), // Используем boxed closure
            ).placeholder("Выберите преподавателя"))
            .push(PickList::new(
                Level::ALL,
                level_value, // Используем соответствующее поле
                move |level| level_changed_msg(level), // Используем boxed closure
            ).placeholder("Выберите уровень"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(Button::new(Text::new("Отмена")).on_press(cancel_message))
                    .push(Button::new(Text::new(submit_button_text)).on_press(submit_message))
            );


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
                // Клик вне окна закрывает только если НЕ в режиме редактирования.
                // В режиме редактирования отмена только по кнопке "Отмена" для предотвращения случайной потери данных.
                .on_press(if is_editing { Message::Er("".to_string()) /* Пустое сообщение, чтобы ничего не происходило */ } else { Message::ToggleAddCourseModal(false) })
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 })); // Затемнение фона


        Container::new(
            Stack::new()
                .push(base)
                .push(modal_overlay)
        )
    } else {
        base
    }
}