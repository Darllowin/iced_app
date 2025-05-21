// src/classes.rs

use iced::{Color, Alignment, Length, Theme};
use iced::widget::{Column, Container, Row, Text, Button, PickList, Scrollable, TextInput, Rule, TextEditor, mouse_area, horizontal_space, text, Stack};
use iced::widget::container::{background, bordered_box};
use crate::app::{App, Message, TextInputOrEditorInput};

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
        // Создать PickList для групп
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
                let lesson_header = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(Text::new(format!("{}. {}",
                                            lesson_with_assignments.number.unwrap_or(0),
                                            lesson_with_assignments.title))
                        .width(Length::FillPortion(10))
                        .size(20)
                    )
                    .push(horizontal_space())
                    .push(
                        if !lesson_with_assignments.assignments.is_empty() {
                            Button::new(Text::new("Провести занятие"))
                                .on_press(Message::ConductLesson(lesson_with_assignments.id, selected_group.id))
                        } else {
                            Button::new(Text::new("Провести занятие"))
                        }
                    );

                let mut lesson_item_content = Column::new()
                    .spacing(5)
                    .width(Length::Fill)
                    .push(lesson_header);

                // Отображение заданий для текущего урока
                if lesson_with_assignments.assignments.is_empty() {
                    lesson_item_content = lesson_item_content.push(
                        Text::new("Заданий для этого урока нет.").size(18).color(Color::from_rgb8(204, 36, 29))
                    );
                } else {
                    lesson_item_content = lesson_item_content.push(
                        Text::new("Задания:").size(20)
                    );
                    for assignment in &lesson_with_assignments.assignments {
                        let assignment_display = Row::new()
                            .spacing(5)
                            .push(
                                Container::new(Row::new()
                                    .push(
                                        Text::new(format!("  - {} ({})", assignment.title, assignment.assignment_type)).size(14)
                                    )
                                    .push(
                                        Container::new(text(&assignment.description).size(14))
                                            .width(Length::FillPortion(10))
                                            .height(Length::Fixed(100.0))
                                            .align_y(Alignment::Center)
                                    )
                                    .spacing(10).align_y(Alignment::Center)
                                ).style(move |_| bordered_box(&app.theme))
                            );
                        lesson_item_content = lesson_item_content.push(assignment_display);
                    }
                }

                lessons_with_assignments_list_column = lessons_with_assignments_list_column.push(
                    Container::new(lesson_item_content)
                        .padding(10)
                        .width(Length::Fill)
                        .style(move |_| bordered_box(&app.theme))
                );
            }
        }
        main_column = main_column.push(
            Scrollable::new(
                Container::new(lessons_with_assignments_list_column)
                    .padding(10)
                    .style(move |_| bordered_box(&app.theme))
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
        //.style(move |_| bordered_box(&app.theme));

    let mut ui_stack = Stack::new().push(base_ui);

    // --- Модальное окно заданий преподавателя ---
    if app.show_teacher_assignment_modal {
        if let Some(proven_lesson) = &app.selected_proven_lesson_for_assignments {
            let modal_title_text = format!("Задания для: {} ({}) - {}",
                                           proven_lesson.lesson_title,
                                           proven_lesson.topic,
                                           proven_lesson.date
            );

            let mut assignments_list_col: Column<'_, _, Theme> = Column::new().spacing(5);
            if app.teacher_lesson_assignments.is_empty() {
                assignments_list_col = assignments_list_col.push(
                    Text::new("Для этого занятия еще нет заданий.").size(16)
                );
            } else {
                for assignment in &app.teacher_lesson_assignments {
                    let assignment_row = Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(Text::new(format!("{} ({})", assignment.title, assignment.assignment_type))
                            .width(Length::FillPortion(3)))
                        .push(text(&assignment.description) // Используем text() вместо Text::new() для корректного отображения длинного текста
                                  .width(Length::FillPortion(5))
                                  .height(Length::Shrink) // Высота по содержимому
                        )
                        .push(horizontal_space())
                        .push(
                            Button::new(Text::new("Редактировать"))
                                .on_press(Message::StartEditingTeacherAssignment(assignment.clone()))
                        )
                        .push(
                            Button::new(Text::new("X"))
                                .on_press(Message::DeleteProvenLessonAssignment(proven_lesson.id, assignment.id)) // Передаем proven_lesson_id и assignment_id
                        );
                    assignments_list_col = assignments_list_col.push(
                        Container::new(assignment_row).padding(5).width(Length::Fill).style(move |_| bordered_box(&app.theme))
                    );
                }
            }

            let scrollable_assignments = Scrollable::new(
                Container::new(assignments_list_col).padding(5)
            ).height(Length::FillPortion(2));

            // Форма редактирования задания
            let mut editing_form = Column::new().spacing(10);
            if app.editing_teacher_assignment.is_some() {
                let assignment_type = app.editing_teacher_assignment.as_ref().unwrap().assignment_type.clone();

                editing_form = editing_form
                    .push(Text::new("Редактировать задание").size(18))
                    .push(TextInput::new("Название задания", &app.editing_teacher_assignment_title)
                        .on_input(Message::EditingTeacherAssignmentTitleChanged));

                if assignment_type == "Lecture" || assignment_type == "Practice" {
                    editing_form = editing_form
                        .push(Text::new("Описание/Текст:"))
                        .push(Scrollable::new(
                            TextEditor::new(&app.editing_teacher_assignment_description_content)
                                .placeholder("Введите описание...")
                                .on_action(|action| Message::EditingTeacherAssignmentDescriptionChanged(TextInputOrEditorInput::TextEditor(action)))
                        ).height(Length::Fixed(150.0))
                        );
                } else {
                    editing_form = editing_form
                        .push(Text::new("Описание/Инструкции:"))
                        .push(TextInput::new("Введите описание...", &app.editing_teacher_assignment_description_text_input)
                            .on_input(|s| Message::EditingTeacherAssignmentDescriptionChanged(TextInputOrEditorInput::TextInput(s))));
                }

                editing_form = editing_form.push(
                    Button::new(Text::new("Сохранить изменения"))
                        .on_press(Message::SaveEditedTeacherAssignment)
                );
            } else {
                editing_form = editing_form.push(Text::new("Выберите задание для редактирования или добавьте новое."));
            }

            // Добавить существующее задание
            let add_existing_assignment_form = Column::new()
                .spacing(10)
                .push(Text::new("Добавить существующее задание к занятию").size(18))
                .push(
                    PickList::new(
                        app.available_assignments.clone(),
                        app.selected_assignment_to_add_to_lesson.clone(),
                        Message::SelectedAssignmentToAddToLesson,
                    )
                        .placeholder("Выберите задание")
                )
                .push(
                    Button::new(Text::new("Добавить выбранное задание"))
                        .on_press(Message::AddExistingAssignmentToProvenLesson)
                );


            let mut assignments_modal_col = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(modal_title_text).size(22))
                .push(scrollable_assignments)
                .push(Rule::horizontal(10))
                .push(editing_form)
                .push(Rule::horizontal(10))
                .push(add_existing_assignment_form);


            if let Some(error_msg) = &app.teacher_assignment_edit_error_message {
                assignments_modal_col = assignments_modal_col.push(Text::new(error_msg).size(16).color(Color::from_rgb8(255, 0, 0)));
            }
            assignments_modal_col = assignments_modal_col.push(
                Button::new(Text::new("Закрыть"))
                    .on_press(Message::CloseTeacherAssignmentModal)
            );


            let assignments_modal_container = Container::new(assignments_modal_col)
                .style(move |_| bordered_box(&app.theme))
                .padding(20)
                .height(Length::Fixed(700.0)) // Увеличена высота для большего содержимого
                .width(Length::Fixed(800.0));

            let assignments_modal_overlay = Container::new(
                mouse_area(Container::new(assignments_modal_container).center(Length::Fill))
                    .on_press(Message::Er("".to_string())) // Рассмотрите более конкретное сообщение или кнопку закрытия
            )
                .width(Length::Fill).height(Length::Fill)
                .center_y(Length::Fill)
                .center_x(Length::Fill)
                .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 }));
            ui_stack = ui_stack.push(assignments_modal_overlay);
        }
    }

    Container::new(ui_stack)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}