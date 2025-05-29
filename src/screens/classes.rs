use iced::{Color, Alignment, Length, Theme, Element, Renderer};
use iced::widget::{Column, Container, Row, Text, Button, PickList, Scrollable, horizontal_space, text, Stack, Checkbox};
use iced::widget::container::{background, bordered_box};
use crate::app::{App, Message};

pub fn classes_screen(app: &App) -> Container<Message> {
    let mut main_column = Column::new()
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    // --- Фильтр/Выбор групп ---
    let mut group_options_ui = Column::new().spacing(10);
    if app.teacher_groups.is_empty() {
        group_options_ui = group_options_ui.push(Text::new("Нет доступных групп."));
    } else {
        let group_picklist = PickList::new(
            app.teacher_groups.clone(), // Клонируем группы для PickList
            app.selected_group_for_classes.clone(),
            Message::SelectGroupForClasses,
        )
            .placeholder("Выберите группу");

        group_options_ui = group_options_ui.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(Text::new("Мои группы:"))
                .push(group_picklist)
                .width(Length::Fill)
        );
    }
    main_column = main_column.push(group_options_ui);

    // --- Список уроков с заданиями для выбранной группы ---
    if let Some(selected_group) = &app.selected_group_for_classes {
        main_column = main_column.push(
            Text::new(format!("Занятия для группы: {}", selected_group.name))
                .size(24)
        );

        let mut lessons_with_assignments_list_column = Column::new()
            .width(Length::Fill)
            .spacing(10);
        if app.selected_group_lessons_with_assignments.is_empty() {
            lessons_with_assignments_list_column = lessons_with_assignments_list_column.push(Text::new("В этом курсе пока нет уроков."));
        } else {
            for lesson_with_assignments in &app.selected_group_lessons_with_assignments {
                // Создаем отдельную колонку для каждого урока
                let mut lesson_card_content = Column::new()
                    .spacing(5) // Пространство между элементами внутри карточки урока
                    .width(Length::Fill);

                // Заголовок урока и кнопка "Провести занятие"
                let mut lesson_header_row = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(Text::new(format!("{}. {}",
                                            lesson_with_assignments.number,
                                            lesson_with_assignments.title))
                        .width(Length::FillPortion(10))
                        .size(20)
                    )
                    .push(horizontal_space());

                // Определяем, какая кнопка или текст будет отображаться
                let conduct_button_or_text =
                    Button::new(Text::new("Провести занятие"))
                        .on_press(Message::OpenConductLessonModal(lesson_with_assignments.id, selected_group.id));

                lesson_header_row = lesson_header_row.push(conduct_button_or_text);

                // Добавляем заголовок урока в колонку содержимого карточки урока
                lesson_card_content = lesson_card_content.push(lesson_header_row);

                // Отображение заданий для текущего урока (если они есть)
                if !lesson_with_assignments.assignments.is_empty() {
                    lesson_card_content = lesson_card_content.push(
                        Text::new("Задания:").size(18).color(Color::from_rgb8(142, 192, 124)) // Немного темнее зеленый
                    );
                    for assignment in &lesson_with_assignments.assignments {
                        let assignment_display = Row::new()
                            .spacing(10) // Пространство между элементами задания
                            .align_y(Alignment::Center)
                            .push(
                                // Можно добавить отступ для вложенности
                                Text::new(format!("  - {} ({})", assignment.title, assignment.assignment_type)).size(16)
                                    .width(Length::FillPortion(3))
                            )
                            .push(
                                Container::new(text(&assignment.description).size(14))
                                    .width(Length::FillPortion(5))
                                    .height(Length::Shrink) // Высота по содержимому
                                    .align_y(Alignment::Center)
                            );
                        lesson_card_content = lesson_card_content.push(
                            Container::new(assignment_display)
                                .padding(5)
                                .width(Length::Fill)
                                .style(move |_| bordered_box(&app.theme.target())) // Отдельная рамка для каждого задания
                        );
                    }
                } else {
                    // Если заданий нет, уже отобразили текст "Нет заданий..." в conduct_button_or_text
                    // Можно добавить пустой спейсер, если нужен отступ
                    lesson_card_content = lesson_card_content.push(horizontal_space().height(Length::Fixed(5.0)));
                }

                // Добавляем готовую карточку урока (заголовок + задания) в основной список уроков
                lessons_with_assignments_list_column = lessons_with_assignments_list_column.push(
                    Container::new(lesson_card_content)
                        .padding(10)
                        .width(Length::Fill)
                        .style(move |_| bordered_box(&app.theme.target())) // Рамка для всей карточки урока
                );
            }
        }
        main_column = main_column.push(
            Scrollable::new(
                Container::new(lessons_with_assignments_list_column)
                    .padding(10)
                    //.style(move |_| bordered_box(&app.theme)) // Общая рамка для всего прокручиваемого списка
            )
                .height(Length::FillPortion(1))
        );
    } else {
        main_column = main_column.push(
            Text::new("Выберите группу, чтобы увидеть её уроки и задания.")
                .size(20)
        );
    }

    let base_ui = Container::new(main_column)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut ui_stack = Stack::new().push(base_ui);

    if app.show_conduct_lesson_modal {
        // Фон модального окна
        ui_stack = ui_stack.push(
            Container::new(
                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill)

            ).style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }))
        );

        ui_stack = ui_stack.push(
            Container::new(
                Column::new()
                    .spacing(15)
                    .padding(20)
                    .width(Length::Fixed(400.0))
                    .height(Length::Shrink)
                    .push(Text::new("Отметить посещаемость").size(24))
                    .push(Text::new(format!("Урок: {}", app.current_lesson_to_conduct.as_ref().map_or("N/A".to_string(), |l| l.title.clone()))))
                    .push(Text::new(format!("Группа: {}", app.current_group_for_attendance.as_ref().map_or("N/A".to_string(), |g| g.name.clone()))))
                    .push(
                        Scrollable::new(
                            Container::new(
                                Column::new()
                                    .spacing(10)
                                    // Создаем вектор элементов для студентов
                                    .extend({ 
                                        let mut student_rows: Vec<Element<'_, Message, Theme, Renderer>> = Vec::new();

                                        if app.students_for_attendance.is_empty() {
                                            student_rows.push(Text::new("Нет студентов в этой группе.").into());
                                        } else {
                                            // Итерируемся и добавляем каждую строку студента в вектор
                                            for student in &app.students_for_attendance {
                                                student_rows.push(
                                                    Row::new()
                                                        .spacing(10)
                                                        .align_y(Alignment::Center)
                                                        .push(Checkbox::new(
                                                            format!("{}", student.name),
                                                            student.present,
                                                        )
                                                            .on_toggle(move |_is_checked| Message::ToggleStudentAttendance(student.id))
                                                        )
                                                        .into() // Преобразуем Row в Element
                                                );
                                            }
                                        }
                                        student_rows // Возвращаем вектор элементов
                                    })
                            )
                                .padding(10)
                        )
                            .height(Length::Fixed(200.0)) // Фиксированная высота для прокручиваемого списка студентов
                    )
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(
                                Button::new(Text::new("Сохранить"))
                                    .on_press(Message::SaveAttendance)
                            )
                            .push(
                                Button::new(Text::new("Отмена"))
                                    .on_press(Message::ConductLessonClicked(0,0)) // Заглушки, просто для закрытия модального окна
                                // Или лучше, новое сообщение типа Message::CancelAttendance
                            )
                    )
            )
                .style(move |_| bordered_box(&app.theme.target()))
        );
    }

    Container::new(ui_stack)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}