use iced::{widget::{Button, Column, Container, Row, Stack, Text, mouse_area, Scrollable}, Alignment, Color, ContentFit, Length, Theme};
use iced::widget::{button, horizontal_space, image, text}; // Удален PickList и Rule, если они не используются
use iced::widget::container::{background, bordered_box};
use iced::widget::image::Handle;
use crate::app::{App, Message};
use crate::app::state::{DEFAULT_AVATAR, Certificate, UserInfo}; // Импортируйте UserInfo
// Удален 'rusqlite::Connection' так как теперь данные из app.state

pub fn certificates_screen(app: &App) -> Container<Message> {
    let mut main_column = Column::new().spacing(20).padding(20);

    // Заголовок
    main_column = main_column.push(
        Row::new()
            .push(Text::new("Сертификаты студентов").size(26))
            .push(horizontal_space())
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding([0, 20])
    );

    // Список студентов с сертификатами
    let mut student_list_column = Column::new().spacing(15);

    if app.students_with_certificates.is_empty() {
        student_list_column = student_list_column.push(Text::new("Нет студентов с сертификатами.").size(18).color(Color::from_rgb8(150, 150, 150)));
    } else {
        // Теперь итерируемся по UserInfo
        for student_info in &app.students_with_certificates {
            let avatar = if let Some(data) = &student_info.avatar_data { // Используем &data для ссылки
                // data.extend_from_slice(student_info.email.as_bytes()); // Нельзя изменять &data
                let mut data_clone = data.clone(); // Клонируем, чтобы добавить email
                data_clone.extend_from_slice(student_info.email.as_bytes()); // Добавляем email для уникальности Handle
                let image_handle = Handle::from_bytes(data_clone);

                image(image_handle)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .content_fit(ContentFit::Fill)
            } else {
                image(DEFAULT_AVATAR)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .content_fit(ContentFit::Cover)
            };

            let student_card_content = Row::new()
                .padding(10)
                .spacing(20)
                .push(avatar)
                .push(
                    Column::new()
                        .spacing(5)
                        .push(Text::new(format!("Имя: {}", student_info.name)).size(20))
                        .push(Text::new(format!("Email: {}", student_info.email)).size(16))
                        // Используем child_count для отображения количества сертификатов
                        .push(Text::new(format!("Количество сертификатов: {}", student_info.child_count.unwrap_or(0))).size(16))
                )
                .push(horizontal_space())
                .push(
                    // Передаем UserInfo студента при нажатии кнопки
                    button(Text::new("Посмотреть сертификаты"))
                        .on_press(Message::OpenStudentCertificatesModal(student_info.clone()))
                );

            student_list_column = student_list_column.push(
                Container::new(student_card_content)
                    .style(move |_| bordered_box(&app.theme))
                    .width(Length::Fill)
            );
        }
    }

    let scrollable_students = Scrollable::new(student_list_column)
        .width(Length::Fill)
        .height(Length::FillPortion(1));

    main_column = main_column.push(scrollable_students);

    let base_ui = Container::new(main_column)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    let mut ui_stack = Stack::new().push(base_ui);

    // --- Модальное окно сертификатов студента ---
    if app.show_student_certificates_modal {
        if let Some(student) = &app.selected_student_for_certificates {
            let modal_title = format!("Сертификаты {}", student.name);
            let mut certs_list_col: Column<'_, Message, Theme> = Column::new().spacing(10);

            if app.is_loading_student_certs {
                certs_list_col = certs_list_col.push(
                    Text::new("Загрузка сертификатов...").size(16).color(Color::from_rgb8(100, 100, 200))
                );
            } else if app.selected_student_certs.is_empty() {
                certs_list_col = certs_list_col.push(
                    Text::new("У этого студента пока нет сертификатов.").size(16)
                );
            } else {
                for cert in &app.selected_student_certs {
                    let cert_clone = cert.clone();
                    let student_clone = student.clone();
                    certs_list_col = certs_list_col.push(
                        Container::new(
                            Column::new()
                                .spacing(5)
                                .push(Text::new(format!("Курс: {}", cert.course_title)).size(18))
                                .push(Text::new(format!("Дата выдачи: {}", cert.issue_date)).size(16))
                                .push(Text::new(format!("Оценка: {}", cert.grade)).size(16).color(
                                    match cert.grade.as_str() {
                                        "Отлично" => Color::from_rgb(0.0, 0.7, 0.0),
                                        "Хорошо" => Color::from_rgb(0.0, 0.5, 0.8),
                                        _ => Color::from_rgb(0.8, 0.4, 0.0),
                                    }
                                ))
                                .push(
                                    // НОВАЯ КНОПКА: Генерировать сертификат
                                    button(Text::new("Сгенерировать сертификат"))
                                        .on_press(Message::GenerateCertificatePdf(cert_clone, student_clone))
                                )
                                
                        )
                            .padding(10)
                            .width(Length::Fill)
                            .style(move |_| bordered_box(&app.theme))
                    );
                }
            }

            let scrollable_certs = Scrollable::new(
                Container::new(certs_list_col).padding(5)
            ).height(Length::FillPortion(1));

            let modal_content = Column::new()
                .spacing(15)
                .align_x(Alignment::Start)
                .push(Text::new(modal_title).size(22))
                .push(scrollable_certs)
                .push(Text::new(app.error_message.clone()).color(Color::from_rgb(1.0, 0.0, 0.0)))
                .push(
                    button(Text::new("Закрыть"))
                        .on_press(Message::CloseStudentCertificatesModal)
                );

            let modal_container = Container::new(modal_content)
                .style(move |_| bordered_box(&app.theme))
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

    Container::new(ui_stack)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}
