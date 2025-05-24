use iced::{
    widget::{Button, Column, Container, Row, Scrollable, Space, Text, pick_list, horizontal_space, Stack, mouse_area},
    Alignment, Color, Element, Length
};
use iced::widget::container::{background, bordered_box};
use crate::app::{App, Message};
use crate::app::state::{StudentPickListItem, CoursePickListItem, GroupPickListItem};

pub fn payment_screen(app: &App) -> Container<Message> {
    let add_button = Button::new(Text::new("Добавить платеж"))
        .on_press(Message::ToggleAddPaymentModal)
        .padding(10);

    let header_section = Column::new()
        .spacing(15)
        .push(Text::new("Список платежей").size(30))
        .push(add_button)
        .push(Space::with_height(10));

    let mut payment_cards = Column::new().spacing(15);

    for payment in &app.payments {
        let header = Row::new()
            .push(Text::new(format!("Платёж #{}", payment.id)).size(20))
            .push(horizontal_space())
            .push(Button::new("X").on_press(Message::DeletePayment(payment.id)))
            .width(Length::Fill)
            .spacing(10);

        let info = Column::new()
            .spacing(5)
            .push(Text::new(format!("Студент: {}", payment.student_name)))
            .push(Text::new(format!("Курс: {}", payment.course_title)))
            .push(Text::new(format!("Группа: {}", payment.group_name)))
            .push(Text::new(format!("Дата: {}", payment.date)))
            .push(Text::new(format!("Сумма: {:.2} €", payment.amount)))
            .push(Text::new(format!("Тип: {}", payment.payment_type)));

        let payment_card = Container::new(
            Column::new()
                .push(Container::new(header).style(move |_| bordered_box(&app.theme)).padding(10))
                .push(
                    Container::new(
                        Row::new()
                            .spacing(20)
                            .push(info),
                    )
                        .padding(10),
                ),
        )
            .style(move |_| bordered_box(&app.theme))
            .width(Length::Fill)
            .padding(10);

        payment_cards = payment_cards.push(payment_card);
    }

    let scrollable_list: Element<_> = Scrollable::new(payment_cards)
        .height(Length::Fill)
        .into();

    let base_ui = Container::new(
        Column::new()
            .spacing(15)
            .padding(20)
            .push(header_section)
            .push(
                Container::new(scrollable_list)
                    .height(Length::Fill)
                    .width(Length::Fill),
            ),
    )
        .width(Length::Fill)
        .height(Length::Fill);

    let mut ui_stack = Stack::new().push(base_ui);

    if app.show_add_payment_modal {
        let students_options: Vec<StudentPickListItem> = app.students_without_group.iter()
            .map(|s| StudentPickListItem { id: s.id, name: s.name.clone() })
            .collect();

        let courses_options: Vec<CoursePickListItem> = app.courses_with_seats.iter()
            .map(|c| {
                let price_display = c.price
                    .map(|p| format!("{:.2} €", p))
                    .unwrap_or_else(|| "Цена не указана".to_string());
                CoursePickListItem { id: c.id, title: c.title.clone(), price_display }
            })
            .collect();
        
        let payment_types = vec!["Карта".to_string(), "QR-Код".to_string()];

        let student_pick_list = pick_list(
            students_options,
            app.new_payment_student.as_ref(),
            |selected_item| Message::NewPaymentFormStudentSelected(selected_item),
        )
            .placeholder("Выберите студента");

        let course_pick_list = pick_list(
            courses_options.clone(),
            app.new_payment_course.as_ref(),
            |course| Message::NewPaymentFormCourseSelected(course.clone()),
        )
            .placeholder("Выберите курс");

        // Условное создание PickList для групп
        let group_pick_list_widget: Element<Message> = if app.new_payment_course.is_some() {
            let groups_options_active: Vec<GroupPickListItem> = app.groups_for_selected_course.iter()
                .map(|g| GroupPickListItem { id: g.id, name: g.name.clone() })
                .collect();

            pick_list(
                groups_options_active,
                app.new_payment_group.as_ref(),
                |selected_group| Message::NewPaymentFormGroupSelected(selected_group),
            )
                .placeholder("Выберите группу")
                .into()
        } else {
            pick_list(
                vec![], // Пустой список
                // <-- ИСПРАВЛЕНИЕ ЗДЕСЬ: Явно указываем тип для Option::<&T>::None
                Option::<&GroupPickListItem>::None,
                |_: GroupPickListItem| Message::NoOp, // Тип параметра замыкания остается явно указанным
            )
                .placeholder("Выберите группу (сначала выберите курс)")
                .into()
        };

        let type_pick_list = pick_list(
            payment_types.clone(),
            // 1. Для `selected`: Используем `and_then` с `get()` для получения `Option<&String>`
            app.selected_payment_type_idx.and_then(|i| payment_types.get(i).cloned()),
            // 2. Для `on_selected`: Используем замыкание, которое принимает String и возвращает Message
            |selected_type_string: String| Message::NewPaymentFormTypeChanged(selected_type_string),
        )
            .placeholder("Выберите тип платежа");

        let amount = app.new_payment_amount.unwrap_or(0.0);
        let amount_text = Text::new(format!("Сумма: {:.2} ₽", amount));
        let date_text = Text::new(format!("Дата: {}", chrono::Local::now().format("%Y-%m-%d")));

        // <--- СТАРТ ИСПРАВЛЕНИЙ ДЛЯ МОДАЛЬНОГО ОКНА ---
        // 1. Содержимое формы (Column)
        let modal_form_content = Column::new()
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center)
            .push(Text::new("Добавить новый платеж").size(24))
            .push(student_pick_list)
            .push(course_pick_list)
            .push(group_pick_list_widget)
            .push(type_pick_list)
            .push(amount_text)
            .push(date_text)
            .push(
                Row::new()
                    .spacing(10)
                    .push(Button::new(Text::new("Добавить")).on_press(Message::AddPaymentConfirmed))
                    .push(Button::new(Text::new("Отмена")).on_press(Message::ToggleAddPaymentModal)),
            );

        // 2. Контейнер для модального окна с фиксированной шириной и стилем
        let modal_container = Container::new(modal_form_content)
            .style(move |_| bordered_box(&app.theme))
            .width(Length::Fixed(500.0)); // <-- Здесь применяется фиксированная ширина
        // .center_x/.y не нужны здесь, центрирование будет в родительском mouse_area

        // 3. Фон-оверлей с mouse_area для кликов вне модального окна
        let modal_overlay_element: Element<Message> = Container::new(
            mouse_area(modal_container) // Оборачиваем контейнер модального окна в mouse_area
                .on_press(Message::ToggleAddPaymentModal) // Клик по mouse_area закрывает модальное окно
        )
            .width(Length::Fill)  // Сам оверлей занимает всю доступную ширину/высоту
            .height(Length::Fill)
            .center_x(Length::Fill) // Центрируем mouse_area (которая содержит модальное окно)
            .center_y(Length::Fill) // по центру родителя (Stack)
            .style(move |_| background(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 })) // Полупрозрачный черный фон
            .into();
        // <--- КОНЕЦ ИСПРАВЛЕНИЙ ДЛЯ МОДАЛЬНОГО ОКНА ---

        ui_stack = ui_stack.push(modal_overlay_element);
    }

    Container::new(ui_stack)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}