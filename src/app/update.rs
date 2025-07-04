use super::{App, Message};
use crate::app::state::{
    Assignment, AssignmentType, Course, DEFAULT_AVATAR, DatePickerOpen, Group,
    LessonWithAssignments, Level, PATH_TO_DB, ReportType, Screen, StudentAttendance,
    TextInputOrEditorInput, UserInfo,
};
use crate::config::{
    backup_database_now_with_config, get_last_backup_time, save_config, theme_from_str,
};
use crate::db;
use crate::doc_gen::{
    generate_certificate_excel_report, generate_certificate_html, generate_certificate_report,
    generate_group_excel_report, generate_group_report, generate_payment_excel_report,
    generate_payment_report, generate_pdf_from_html,
};
use chrono::{Local, NaiveDate};
use iced::widget::{Row, text, text_editor};
use iced::{Alignment, Element, Renderer, Task, Theme};
use regex::Regex;
use rfd::FileDialog;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tokio::task::spawn_blocking;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let current_user_for_task_clone = self.current_user.clone();
        match message {
            Message::LoginPressed => {
                if self.user_email.trim().is_empty() || self.user_password.trim().is_empty() {
                    self.error_message = "Пожалуйста, заполните все поля.".to_string();
                    return Task::none(); // Возвращаем Task::none()
                }
                let email_clone = self.user_email.clone();
                let password_clone = self.user_password.clone();

                Task::perform(
                    db::authenticate_and_get_user_data(email_clone, hash_password(&password_clone)),
                    Message::UserLoggedIn,
                )
            }
            Message::UserLoggedIn(result) => {
                match result {
                    Ok(user_info_data) => {
                        self.current_user = Some(UserInfo {
                            id: user_info_data.id,
                            name: user_info_data.name,
                            email: user_info_data.email,
                            birthday: user_info_data.birthday,
                            user_type: user_info_data.user_type.clone(),
                            group_id: user_info_data.group_id,
                            avatar_data: user_info_data.avatar_data,
                            child_count: user_info_data.child_count,
                        });
                        self.error_message = "".to_string();
                        self.current_screen = Screen::Profile;

                        if let Some(user) = &self.current_user {
                            if user.user_type == "admin" {
                                println!(
                                    "DEBUG: Пользователь является АДМИНИСТРАТОРОМ. Запускаем загрузку ВСЕХ групп."
                                );
                                // !!! АДМИНИСТРАТОР: отправляем LoadAllGroups
                                return self.update(Message::LoadAllGroups);
                            } else if user.user_type == "teacher" {
                                println!(
                                    "DEBUG: Пользователь является ПРЕПОДАВАТЕЛЕМ. Запускаем загрузку его групп."
                                );
                                let teacher_id_for_task = user.id;
                                // !!! ПРЕПОДАВАТЕЛЬ: отправляем LoadTeacherGroups с ID
                                return self
                                    .update(Message::LoadTeacherGroups(teacher_id_for_task));
                            } else if user.user_type == "student" {
                                // <-- ДОБАВЛЕН ВАРИАНТ ДЛЯ СТУДЕНТА
                                println!(
                                    "DEBUG: Пользователь является СТУДЕНТОМ. Запускаем загрузку его группы."
                                );
                                // !!! СТУДЕНТ: отправляем LoadStudentGroupInfo
                                return self.update(Message::LoadStudentGroupInfo);
                            } else {
                                // Если это родитель или другой тип пользователя, для которого нет специфичной загрузки
                                println!(
                                    "DEBUG: Пользователь {} (ID: {}) не преподаватель, не администратор и не студент. Группы не загружаются автоматически.",
                                    user.user_type, user.id
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = e.to_string();
                        eprintln!("Ошибка входа: {}", e);
                    }
                }
                Task::none()
            }
            Message::SwitchToLogin => {
                self.current_screen = Screen::Login;
                self.clear_fields();
                Task::none() // Возвращаем Task::none()
            }
            Message::SwitchToRegister => {
                self.current_screen = Screen::Register;
                self.clear_fields();
                Task::none() // Возвращаем Task::none()
            }
            Message::FirstNameChanged(v) => {
                self.user_name = v;
                Task::none()
            }
            Message::LastNameChanged(v) => {
                self.user_surname = v;
                Task::none()
            }
            Message::MiddleNameChanged(v) => {
                self.user_patronymic = v;
                Task::none()
            }
            Message::EmailChanged(v) => {
                self.user_email = v;
                Task::none()
            }
            Message::PasswordChanged(v) => {
                self.user_password = v;
                Task::none()
            }
            Message::PasswordRepeatChanged(v) => {
                self.user_password_repeat = v;
                Task::none()
            }
            Message::RegisterPressed => {
                if self.user_name.trim().is_empty()
                    || self.user_surname.trim().is_empty()
                    || self.user_patronymic.trim().is_empty()
                {
                    self.register_error =
                        Some("Пожалуйста, заполните Фамилию, Имя и Отчество".to_string());
                    return Task::none();
                }

                let fio_re = Regex::new(r"^[А-Яа-яЁё\s-]+$").unwrap();
                if !fio_re.is_match(&self.user_name)
                    || !fio_re.is_match(&self.user_surname)
                    || !fio_re.is_match(&self.user_patronymic)
                {
                    self.register_error = Some(
                        "ФИО может содержать только русские буквы, пробелы и дефисы".to_string(),
                    );
                    return Task::none();
                }

                if self.user_password.trim().is_empty()
                    || self.user_password_repeat.trim().is_empty()
                {
                    self.register_error = Some("Пароль не может быть пустым".to_string());
                    return Task::none();
                }

                if self.user_password != self.user_password_repeat {
                    self.register_error = Some("Пароли не совпадают".to_string());
                    return Task::none();
                }

                let password = &self.user_password;

                if password.len() < 8 {
                    self.register_error =
                        Some("Пароль должен содержать минимум 8 символов".to_string());
                    return Task::none();
                }

                let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
                if !has_uppercase {
                    self.register_error =
                        Some("Пароль должен содержать хотя бы одну заглавную букву".to_string());
                    return Task::none();
                }

                let has_digit = password.chars().any(|c| c.is_ascii_digit());
                if !has_digit {
                    self.register_error =
                        Some("Пароль должен содержать хотя бы одну цифру".to_string());
                    return Task::none();
                }

                let email = self.user_email.trim();

                if email.is_empty() {
                    self.register_error = Some("Email не может быть пустым.".to_string());
                    return Task::none();
                }
                if !email.contains('@') {
                    self.register_error = Some("Email должен содержать символ '@'.".to_string());
                    return Task::none();
                }
                let parts: Vec<&str> = email.split('@').collect();
                if parts.len() != 2 {
                    self.register_error =
                        Some("Email должен содержать только один символ '@'.".to_string());
                    return Task::none();
                }
                if parts[0].is_empty() {
                    self.register_error =
                        Some("Email должен содержать имя пользователя перед '@'.".to_string());
                    return Task::none();
                }
                if parts[1].is_empty() {
                    self.register_error =
                        Some("Email должен содержать домен после '@'.".to_string());
                    return Task::none();
                }
                if !parts[1].contains('.') {
                    self.register_error = Some(
                        "Домен email должен содержать хотя бы одну точку (например: example.com)."
                            .to_string(),
                    );
                    return Task::none();
                }
                let email_re =
                    Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                if !email_re.is_match(email) {
                    self.register_error = Some(
                        "Email содержит недопустимые символы или некорректный формат.".to_string(),
                    );
                    return Task::none();
                }

                let conn = Connection::open(PATH_TO_DB).unwrap();
                match db::is_email_taken(&conn, email) {
                    Ok(true) => {
                        self.register_error =
                            Some("Пользователь с таким email уже существует.".to_string());
                        return Task::none();
                    }
                    Ok(false) => {}
                    Err(err) => {
                        self.register_error = Some(format!("Ошибка при проверке email: {}", err));
                        return Task::none();
                    }
                }

                let full_name = format!(
                    "{} {} {}",
                    self.user_surname, self.user_name, self.user_patronymic
                );
                let password_hash = hash_password(&self.user_password);

                if let Err(_) = db::register_user(
                    &conn,
                    &full_name,
                    format!(
                        "{:02}.{:02}.{}",
                        self.date.day, self.date.month, self.date.year
                    )
                    .as_str(),
                    email,
                    &password_hash,
                ) {
                    self.register_error =
                        Some("Ошибка при сохранении пользователя в базу данных.".to_string());
                } else {
                    self.register_error = None;
                    self.registration_success = true;
                    self.user_email = email.to_string();
                    self.logged_in_user = full_name;
                    self.error_message = "".to_string();
                    db::update_user_avatar(
                        &conn,
                        &self.user_email,
                        fs::read(DEFAULT_AVATAR).unwrap().as_slice(),
                    )
                    .unwrap();
                    self.user_avatar_data = Some(fs::read(DEFAULT_AVATAR).unwrap());
                }
                Task::perform(
                    db::authenticate_and_get_user_data(self.user_email.clone(), password_hash),
                    Message::UserLoggedIn,
                )
            }
            Message::GoToProfile => {
                self.current_screen = Screen::Profile;
                Task::none()
            }
            Message::GoToSettings => {
                self.current_screen = Screen::Settings;
                Task::none()
            }
            Message::GoToCourses => {
                self.current_screen = Screen::CoursesList;
                Task::none()
            }
            Message::GoToUserList => {
                self.current_screen = Screen::UserList;
                Task::none()
            }
            Message::GoToGroupList => {
                self.current_screen = Screen::GroupList;
                Task::none()
            }
            Message::Logout => {
                self.clear_fields();
                self.user_avatar_data = None;
                self.user_email.clear();
                self.current_screen = Screen::Login;
                Task::none()
            }
            Message::ThemeSelected(name) => {
                if let Some(new_theme) = theme_from_str(name) {
                    self.theme.update(new_theme.into());
                    save_config(
                        &self.theme.target(), // Сохранить целевую тему
                        self.backup_interval.as_ref().map(|interval| interval.value),
                        self.backup_folder.clone(),
                        self.max_backup_count,
                    )
                    .ok();
                }
                Task::none()
            }
            Message::ChangeTheme(event) => {
                self.theme.update(event);
                Task::none()
            }
            Message::ChooseDate => {
                self.show_picker = true;
                Task::none()
            }
            Message::SubmitDate(date) => {
                self.date = date;
                self.show_picker = false;
                Task::none()
            }
            Message::CancelDate => {
                self.show_picker = false;
                Task::none()
            }
            Message::Er(_v) => Task::none(),
            Message::ChooseAvatar => {
                // Проверяем, что текущий пользователь существует (то есть email не пустой)
                // Используем current_user.email, а не user_email, для надежности
                let user_email_clone = if let Some(user) = &self.current_user {
                    user.email.clone()
                } else {
                    self.choose_avatar_message =
                        "Вы не вошли в систему. Email неизвестен.".to_string();
                    return Task::none(); // Используйте Command::none()
                };

                let db_path_for_task = PATH_TO_DB;

                // Запускаем асинхронную задачу для выбора аватара и обновления БД
                Task::perform(
                    async move {
                        let result: Result<Vec<u8>, String> = spawn_blocking(move || {
                            let Some(path_buf) = FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg"])
                                .pick_file()
                            else {
                                return Err("Выбор файла аватара отменен.".to_string());
                            };

                            let image_data = fs::read(&path_buf)
                                .map_err(|err| format!("Ошибка чтения файла аватара: {}", err))?;

                            let conn = Connection::open(&db_path_for_task).map_err(|err| {
                                format!("Не удалось открыть БД для сохранения аватара: {}", err)
                            })?;

                            // Обновляем аватар в БД по email
                            db::update_user_avatar(&conn, &user_email_clone, &image_data).map_err(
                                |err| format!("Ошибка сохранения аватара в БД: {}", err),
                            )?;

                            Ok(image_data) // Возвращаем новые данные аватара
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения задачи выбора аватара: {:?}",
                                join_err
                            ))
                        });

                        result
                    },
                    Message::AvatarChosen, // Отображаем результат выполнения этой задачи
                )
            }
            Message::AvatarChosen(result) => {
                match result {
                    Ok(new_avatar_data) => {
                        // Если аватар успешно загружен и сохранен:
                        // ОБНОВЛЯЕМ current_user
                        if let Some(user) = &mut self.current_user {
                            user.avatar_data = Some(new_avatar_data);
                            self.choose_avatar_message.clear(); // Очищаем предыдущие ошибки
                            println!("DEBUG: Аватар успешно обновлен в self.current_user.");
                        } else {
                            // Этот случай не должен наступать, если мы уже проверили user_email_clone
                            self.choose_avatar_message =
                                "Не удалось обновить аватар: пользователь не найден.".to_string();
                        }
                    }
                    Err(e) => {
                        // Если произошла ошибка
                        self.choose_avatar_message = e;
                        eprintln!(
                            "ERROR: Ошибка при выборе или сохранении аватара: {}",
                            self.choose_avatar_message
                        );
                    }
                }
                Task::none() // Возвращаем Command::none(), так как состояние обновлено
            }
            Message::ToggleAddCourseModal(show) => {
                self.show_add_course_modal = show;
                Task::none()
            }
            Message::NewCourseTitleChanged(title) => {
                self.new_course_title = title;
                Task::none()
            }
            Message::NewCourseDescriptionChanged(desc) => {
                self.new_course_description = desc;
                Task::none()
            }
            Message::NewCourseTotalSeatsChanged(s) => {
                self.new_course_total_seats_str = s;
                // Попытка парсинга
                if let Ok(value) = self.new_course_total_seats_str.parse::<i32>() {
                    self.new_course_total_seats = value;
                    self.course_error_message = None; // Очищаем ошибку, если парсинг успешен
                } else {
                    self.course_error_message =
                        Some("Запланированные места должны быть целым числом.".to_string());
                }
                Task::none()
            }
            Message::NewCourseSeatsChanged(s) => {
                self.new_course_seats_str = s;
                if let Ok(value) = self.new_course_seats_str.parse::<i32>() {
                    self.new_course_seats = value;
                    self.course_error_message = None;
                } else {
                    self.course_error_message =
                        Some("Свободные места должны быть целым числом.".to_string());
                }
                Task::none()
            }
            Message::NewCoursePriceChanged(s) => {
                self.new_course_price_str = s.clone();
                // Использование replace(',', '.') для поддержки как точки, так и запятой в качестве разделителя
                let s_cleaned = s.replace(',', ".");
                if let Ok(value) = s_cleaned.parse::<f64>() {
                    self.new_course_price = value;
                    self.course_error_message = None;
                } else {
                    self.course_error_message =
                        Some("Цена должна быть числом (например, 123.45).".to_string());
                }
                Task::none()
            }
            Message::LoadStudentGroupInfo => {
                Task::perform(
                    async move {
                        // 'move' здесь захватывает `current_user_for_task_clone`
                        let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                        // Теперь `current_user_for_task_clone` доступен, так как он был захвачен `move` замыканием
                        if let Some(user_id) = current_user_for_task_clone.as_ref().map(|u| u.id) {
                            // <-- Используем правильную клонированную переменную
                            db::get_student_group_by_user_id(&conn, user_id)
                                .map_err(|e| e.to_string())
                        } else {
                            Ok(None)
                        }
                    },
                    Message::StudentGroupInfoLoaded,
                )
            }
            Message::StudentGroupInfoLoaded(result) => {
                match result {
                    Ok(group_opt) => {
                        self.student_group_info = group_opt;
                        // Отладочный вывод
                        if self.student_group_info.is_some() {
                            println!("DEBUG: Student group loaded: {:?}", self.student_group_info);
                        } else {
                            println!("DEBUG: Student has no group or failed to load.");
                        }
                    }
                    Err(e) => {
                        self.error_message = e;
                        println!(
                            "ERROR: Failed to load student group: {}",
                            self.error_message
                        );
                    }
                }
                Task::none()
            }
            Message::ShowGroupStudents(group_id) => {
                self.show_group_students_modal = true;
                // Находим название группы, чтобы отобразить его в модальном окне
                if let Some(group) = self.teacher_groups.iter().find(|g| g.id == group_id) {
                    self.selected_group_for_students_name = Some(group.name.clone());
                } else {
                    self.selected_group_for_students_name = None;
                }
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            // 1. Открываем соединение, обрабатывая Result и преобразуя ошибку в String
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?; // <-- ИСПРАВЛЕНО

                            // 2. Вызываем функцию БД, обрабатывая Result и преобразуя ошибку в String
                            let students = db::get_students_in_group(&conn, group_id)
                                .map_err(|e| e.to_string())?; // <-- ИСПРАВЛЕНО

                            Ok((group_id, students))
                        })
                        .await // Ждем завершения блокирующей задачи
                        .map_err(|e| format!("Failed to run blocking task: {:?}", e))? // Обработка ошибки от tokio::task::JoinError
                    },
                    Message::GroupStudentsLoaded,
                )
            }
            Message::GroupStudentsLoaded(result) => {
                match result {
                    Ok((_group_id, students)) => {
                        self.selected_group_students = students; // Теперь 'students' будет Vec<UserInfo>
                        println!(
                            "DEBUG: Students for group loaded: {} students",
                            self.selected_group_students.len()
                        );
                    }
                    Err(e) => {
                        self.error_message = e;
                        println!(
                            "ERROR: Failed to load students for group: {}",
                            self.error_message
                        );
                        self.show_group_students_modal = false;
                    }
                }
                self.selected_student_to_add = None;
                Task::none()
            }
            Message::CloseGroupStudentsModal => {
                self.show_group_students_modal = false;
                self.selected_group_for_students_name = None;
                self.selected_group_students.clear();
                Task::none()
            }
            Message::LoadAllCourses => {
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД для курсов: {}", e))?;
                            db::get_courses(&conn)
                                .map_err(|e| format!("Ошибка загрузки курсов: {}", e))
                        })
                        .await
                        .map_err(|join_err| {
                            format!("Ошибка выполнения задачи загрузки курсов: {}", join_err)
                        })?
                    },
                    Message::AllCoursesLoaded, // <-- Когда задача завершится, отправь это сообщение
                )
            }
            Message::AllCoursesLoaded(result) => {
                match result {
                    Ok(courses) => {
                        self.all_courses = courses.clone();
                        self.course_id_to_title =
                            courses.into_iter().map(|c| (c.id, c.title)).collect();
                        println!(
                            "DEBUG: course_id_to_title заполнена: {:?}",
                            self.course_id_to_title
                        ); // <--- ВАЖНО!
                        self.error_message = "".to_string(); // Очищаем сообщение об ошибке
                    }
                    Err(e) => {
                        eprintln!("Ошибка при загрузке курсов: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none() // Возвращаем Task::none(), так как это конечный обработчик
            }
            Message::SubmitNewCourse => {
                // Проверки на пустые поля
                if self.new_course_title.is_empty() {
                    self.course_error_message =
                        Some("Название курса не может быть пустым.".to_string());
                    return Task::none();
                }
                if self.new_course_description.is_empty() {
                    self.course_error_message =
                        Some("Описание курса не может быть пустым.".to_string());
                    return Task::none();
                }
                // Проверка на ошибки парсинга, установленные ранее
                if self.course_error_message.is_some() {
                    // Если уже есть ошибка парсинга (например, введено нечисловое значение),
                    // то не продолжаем.
                    return Task::none();
                }
                // Дополнительные логические проверки числовых значений
                if self.new_course_total_seats <= 0 {
                    self.course_error_message =
                        Some("Запланированные места должны быть больше 0.".to_string());
                    return Task::none();
                }
                if self.new_course_seats < 0 {
                    self.course_error_message =
                        Some("Свободные места не могут быть отрицательными.".to_string());
                    return Task::none();
                }
                if self.new_course_seats > self.new_course_total_seats {
                    self.course_error_message =
                        Some("Свободные места не могут превышать запланированные.".to_string());
                    return Task::none();
                }
                if self.new_course_price <= 0.0 {
                    self.course_error_message =
                        Some("Цена курса должна быть больше 0.".to_string());
                    return Task::none();
                }

                // Если все проверки пройдены, очищаем сообщение об ошибке
                self.course_error_message = None;

                // Клонируем значения, которые будут перемещены в асинхронный блок
                let new_course_title_clone = self.new_course_title.clone();
                let new_course_description_clone = self.new_course_description.clone();
                let new_course_level_string = self.new_course_level.to_string(); // Преобразуем Level в String для БД
                let new_course_seats_val = self.new_course_seats; // Берем само значение i32
                let new_course_total_seats_val = self.new_course_total_seats; // Берем само значение i32
                let new_course_price_val = self.new_course_price; // Берем само значение f64

                // Очищаем поля формы после успешной проверки, но до выполнения Task
                self.new_course_title.clear();
                self.new_course_description.clear();
                self.new_course_level = Level::default(); // Сброс к дефолту. Используем Level::default()
                self.new_course_seats = 0;
                self.new_course_total_seats = 0;
                self.new_course_price = 0.0; // Сброс цены

                // Также очищаем строковые поля, чтобы они не сохраняли старые значения при повторном открытии модалки
                self.new_course_total_seats_str.clear();
                self.new_course_seats_str.clear();
                self.new_course_price_str.clear();

                self.show_add_course_modal = false; // Закрываем модалку

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;

                            db::add_course(
                                &conn,
                                &new_course_title_clone,
                                &new_course_description_clone,
                                &new_course_level_string, // level уже String
                                new_course_seats_val,     // i32
                                new_course_price_val,     // f64
                                new_course_total_seats_val, // i32
                            )
                            .map_err(|e| format!("Ошибка добавления курса: {}", e))?;
                            Ok(()) // Возвращаем Ok(()) если все успешно
                        })
                        .await
                        .map_err(|join_err| format!("Ошибка выполнения задачи: {:?}", join_err))?
                    },
                    |result: Result<(), String>| {
                        match result {
                            Ok(_) => Message::LoadAllCourses, // Обновить список курсов
                            Err(e) => Message::ErrorOccurred(e),
                        }
                    },
                )
            }
            Message::DeleteCourse(course_id) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open(PATH_TO_DB).unwrap();
                if let Err(err) = db::delete_course(&conn, course_id) {
                    eprintln!("Ошибка удаления группы: {:?}", err);
                }
                Task::none()
            }
            Message::NewCourseLevelChanged(level) => {
                self.new_course_level = level;
                Task::none()
            }
            Message::StartEditingCourse(course) => {
                self.edit_course_title = course.title.clone();
                self.edit_course_description = course.description.clone().expect("REASON");

                self.edit_course_level = course
                    .level
                    .clone()
                    .and_then(|level_str| Level::from_str(&level_str).ok())
                    .unwrap_or(Level::Beginner);
                self.edit_course_total_seats = course.total_seats.clone().expect("REASON");
                self.edit_course_seats = course.seats.clone().expect("REASON");
                self.edit_course_price = course.price.clone().expect("REASON");
                self.edit_course_total_seats_str = course
                    .total_seats
                    .map_or("0".to_string(), |val| val.to_string());
                // .unwrap_or_default().to_string();

                self.edit_course_seats_str =
                    course.seats.map_or("0".to_string(), |val| val.to_string());
                // .unwrap_or_default().to_string(); // Альтернативный вариант

                self.edit_course_price_str = course
                    .price
                    .map_or("0.0".to_string(), |val| val.to_string());
                self.editing_course = Some(course);
                self.show_add_course_modal = true;
                Task::none()
            }
            Message::EditCourseTitleChanged(title) => {
                self.edit_course_title = title;
                Task::none()
            }
            Message::EditCourseDescriptionChanged(desc) => {
                self.edit_course_description = desc;
                Task::none()
            }
            Message::EditCourseLevelChanged(level) => {
                self.edit_course_level = level;
                Task::none()
            }
            Message::EditCourseTotalSeatsChanged(s) => {
                self.edit_course_total_seats_str = s;
                if let Ok(value) = self.edit_course_total_seats_str.parse::<i32>() {
                    self.edit_course_total_seats = value;
                    self.course_error_message = None;
                } else {
                    self.course_error_message =
                        Some("Запланированные места должны быть целым числом.".to_string());
                }
                Task::none()
            }
            Message::EditCourseSeatsChanged(s) => {
                self.edit_course_seats_str = s;
                if let Ok(value) = self.edit_course_seats_str.parse::<i32>() {
                    self.edit_course_seats = value;
                    self.course_error_message = None;
                } else {
                    self.course_error_message =
                        Some("Свободные места должны быть целым числом.".to_string());
                }
                Task::none()
            }
            Message::EditCoursePriceChanged(s) => {
                self.edit_course_price_str = s.clone();
                let s_cleaned = s.replace(',', ".");
                if let Ok(value) = s_cleaned.parse::<f64>() {
                    self.edit_course_price = value;
                    self.course_error_message = None;
                } else {
                    self.course_error_message =
                        Some("Цена должна быть числом (например, 123.45).".to_string());
                }
                Task::none()
            }
            Message::SubmitEditedCourse => {
                // Эта операция ДОЛЖНА быть асинхронной.
                // Проверки на пустые поля и логические проверки
                if self.edit_course_title.is_empty() {
                    self.course_error_message =
                        Some("Название курса не может быть пустым.".to_string());
                    return Task::none();
                }
                if self.edit_course_description.is_empty() {
                    self.course_error_message =
                        Some("Описание курса не может быть пустым.".to_string());
                    return Task::none();
                }
                // Проверка на ошибки парсинга, установленные ранее в *Changed сообщениях
                if self.course_error_message.is_some() {
                    // Если уже есть ошибка парсинга (например, введено нечисловое значение),
                    // то не продолжаем.
                    return Task::none();
                }
                // Дополнительные логические проверки числовых значений, которые уже распарсены
                if self.edit_course_total_seats <= 0 {
                    // Теперь это соответствует вашей предыдущей логике
                    self.course_error_message =
                        Some("Запланированные места должны быть больше 0.".to_string());
                    return Task::none();
                }
                if self.edit_course_seats < 0 {
                    self.course_error_message =
                        Some("Свободные места не могут быть отрицательными.".to_string());
                    return Task::none();
                }
                if self.edit_course_seats > self.edit_course_total_seats {
                    self.course_error_message =
                        Some("Свободные места не могут превышать запланированные.".to_string());
                    return Task::none();
                }
                if self.edit_course_price <= 0.0 {
                    self.course_error_message =
                        Some("Цена курса должна быть больше 0.".to_string());
                    return Task::none();
                }

                // Если все проверки пройдены, очищаем сообщение об ошибке
                self.course_error_message = None;

                // Клонируем значения для перемещения в асинхронный блок
                let original_course_id = self.editing_course.as_ref().map(|c| c.id);

                // Получите значения, которые редактировались в состоянии UI
                let edited_course_title = self.edit_course_title.clone();
                let edited_course_description = self.edit_course_description.clone(); // Теперь это String
                let edited_course_level = self.edit_course_level.to_string(); // Level enum в String
                let edited_course_total_seats = self.edit_course_total_seats; // i32
                let edited_course_seats = self.edit_course_seats; // i32
                let edited_course_price = self.edit_course_price; // f64

                // Вам нужно убедиться, что другие поля Course доступны или имеют значения по умолчанию,
                // если они не редактируются в этом модальном окне.
                // Например, если Course имеет 'lesson_count', но оно не редактируется:
                let original_lesson_count = self
                    .editing_course
                    .as_ref()
                    .map(|c| c.lesson_count)
                    .unwrap_or(0); // Предоставьте значение по умолчанию, если None/не найдено

                // Очистите поля UI и сбросьте состояние редактирования ДО асинхронной задачи
                self.edit_course_title.clear();
                self.edit_course_description.clear();
                self.edit_course_level = Level::default();
                self.edit_course_total_seats = 0;
                self.edit_course_seats = 0;
                self.edit_course_price = 0.0;
                self.edit_course_total_seats_str.clear();
                self.edit_course_seats_str.clear();
                self.edit_course_price_str.clear();

                self.editing_course = None;
                self.show_add_course_modal = false;

                // Запустите асинхронную операцию
                Task::perform(
                    async move {
                        // Убедитесь, что course_id существует (из original_course_id, захваченного ранее)
                        let course_id_val = original_course_id.ok_or_else(|| {
                            "Ошибка: ID редактируемого курса не найден.".to_string()
                        })?;

                        // --- Создайте экземпляр структуры Course здесь ---
                        let updated_course_data = Course {
                            id: course_id_val, // Используйте ID из исходного курса
                            title: edited_course_title,
                            description: Some(edited_course_description),
                            level: Some(edited_course_level),
                            lesson_count: original_lesson_count, // Это, вероятно, не Option, так что оставляем как есть
                            total_seats: Some(edited_course_total_seats),
                            seats: Some(edited_course_seats),
                            price: Some(edited_course_price), // Теперь это f64, а не Option<f64>
                        };

                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;

                            // Вызовите db::update_course с новым экземпляром Course
                            db::update_course(&conn, &updated_course_data) // Передайте &updated_course_data
                                .map_err(|e| format!("Ошибка обновления курса: {}", e))
                        })
                        .await
                        .map_err(|join_err| format!("Ошибка выполнения задачи: {:?}", join_err))?
                    },
                    |result: Result<(), String>| match result {
                        Ok(_) => Message::LoadAllCourses,
                        Err(e) => Message::ErrorOccurred(e),
                    },
                )
            }
            Message::CancelEditingCourse => {
                self.show_add_course_modal = false;
                self.editing_course = None;
                self.edit_course_title.clear();
                self.edit_course_description.clear();
                self.edit_course_instructor = None;
                self.edit_course_level = Level::Beginner;
                Task::none()
            }
            Message::StartEditingUser(user) => {
                self.editing_user = Some(user.clone());
                self.edit_user_name = user.name;
                self.edit_user_email = user.email;
                self.edit_user_birthday = user.birthday;
                self.edit_user_type = user.user_type;
                self.show_edit_user_modal = true;
                Task::none()
            }
            Message::CancelEditingUser => {
                self.editing_user = None;
                self.show_edit_user_modal = false;
                self.edit_user_name.clear();
                self.edit_user_email.clear();
                self.edit_user_birthday.clear();
                self.edit_user_type.clear();
                Task::none()
            }
            Message::EditUserNameChanged(value) => {
                self.edit_user_name = value;
                Task::none()
            }
            Message::EditUserEmailChanged(value) => {
                self.edit_user_email = value;
                Task::none()
            }
            Message::EditUserBirthdayChanged(value) => {
                self.edit_user_birthday = value;
                Task::none()
            }
            Message::EditUserTypeChanged(value) => {
                self.edit_user_type = value;
                Task::none()
            }
            Message::UserTypeFilterChanged(selected_type) => {
                self.user_type_filter = selected_type;
                Task::none()
            }
            Message::SubmitEditedUser => {
                // Эта операция должна быть асинхронной
                if let Some(ref original_user) = self.editing_user {
                    let email = self.edit_user_email.trim();

                    if email.is_empty() {
                        self.edit_user_error = Some("Email не может быть пустым.".to_string());
                        return Task::none();
                    }
                    if !email.contains('@') {
                        self.edit_user_error =
                            Some("Email должен содержать символ '@'.".to_string());
                        return Task::none();
                    }

                    let parts: Vec<&str> = email.split('@').collect();
                    if parts.len() != 2 {
                        self.edit_user_error =
                            Some("Email должен содержать только один символ '@'.".to_string());
                        return Task::none();
                    }

                    if parts[0].is_empty() {
                        self.edit_user_error =
                            Some("Email должен содержать имя пользователя перед '@'.".to_string());
                        return Task::none();
                    }
                    if parts[1].is_empty() {
                        self.edit_user_error =
                            Some("Email должен содержать домен после '@'.".to_string());
                        return Task::none();
                    }
                    if !parts[1].contains('.') {
                        self.edit_user_error = Some("Домен email должен содержать хотя бы одну точку (например: example.com).".to_string());
                        return Task::none();
                    }

                    let email_re =
                        Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                    if !email_re.is_match(email) {
                        self.edit_user_error = Some(
                            "Email содержит недопустимые символы или некорректный формат."
                                .to_string(),
                        );
                        return Task::none();
                    }

                    let conn = Connection::open(PATH_TO_DB).unwrap();

                    match db::is_email_taken_except(&conn, email, &original_user.email) {
                        Ok(true) => {
                            self.edit_user_error =
                                Some("Email уже используется другим пользователем.".to_string());
                            return Task::none();
                        }
                        Ok(false) => {
                            let _ = db::update_user(
                                &conn,
                                &original_user.email,
                                &self.edit_user_name,
                                email,
                                &self.edit_user_birthday,
                                &self.edit_user_type,
                            );
                            self.editing_user = None;
                            self.show_edit_user_modal = false;
                            self.edit_user_error = None;
                        }
                        Err(err) => {
                            self.edit_user_error =
                                Some(format!("Ошибка при проверке email: {}", err));
                        }
                    }
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::DeleteUser(email) => {
                println!("DEBUG: Попытка удалить пользователя с email: {}", email);
                // Клонируем путь к БД для использования в асинхронной задаче
                let db_path_for_task = PATH_TO_DB;
                let user_email_for_task = email.clone(); // Клонируем email для использования в замыкании

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(&db_path_for_task).map_err(|e| {
                                format!("Не удалось открыть БД для удаления: {}", e)
                            })?;
                            db::delete_user(&conn, &user_email_for_task).map_err(|e| {
                                format!(
                                    "Ошибка удаления пользователя {}: {}",
                                    user_email_for_task, e
                                )
                            })?;
                            Ok(user_email_for_task) // Возвращаем email успешно удаленного пользователя
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения задачи удаления пользователя: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::UserDeleted, // Передаем результат этой асинхронной задачи в Message::UserDeleted
                )
                .into() // Преобразуем Task в Command
            }
            Message::UserDeleted(result) => {
                match result {
                    Ok(email) => {
                        println!(
                            "DEBUG: Пользователь {} успешно удален. Обновляем список.",
                            email
                        );
                        Task::perform(
                            async { Message::GoToUserList }, // Асинхронный блок, который просто возвращает нужное сообщение
                            |msg| msg, // Замыкание-маппер: просто возвращает сообщение как есть
                        )
                    }
                    Err(e) => {
                        self.error_message = e.clone(); // Сохраняем сообщение об ошибке для отображения
                        eprintln!("ERROR: Не удалось удалить пользователя: {}", e);
                        Task::none() // Ничего не делаем, ошибка отображена
                    }
                }
            }
            Message::CourseFilterChanged(text) => {
                self.course_filter_text = text;
                Task::none()
            }
            Message::CoursesForPicklistLoaded(result) => {
                match result {
                    Ok(courses) => {
                        self.courses_for_picklist = courses;
                        println!(
                            "DEBUG: Курсы для PickList загружены: {} шт.",
                            self.courses_for_picklist.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Не удалось загрузить курсы для PickList: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::UsersForPicklistLoaded(result) => {
                match result {
                    Ok(users) => {
                        // Для PickList преподавателей можно отфильтровать только учителей
                        self.users_for_picklist = users
                            .clone()
                            .into_iter()
                            .filter(|u| u.user_type == "teacher")
                            .collect();
                        println!(
                            "DEBUG: Пользователи для PickList загружены: {} шт. (из них преподавателей: {})",
                            users.len(),
                            self.users_for_picklist.len()
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "ERROR: Не удалось загрузить пользователей для PickList: {}",
                            e
                        );
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::ToggleAddGroupModal(open) => {
                self.show_add_group_modal = open;
                if open {
                    println!("DEBUG: Открываем модальное окно добавления/редактирования группы.");
                    // Загружаем данные для PickList'ов при открытии модалки
                    let task_courses = Task::perform(
                        async {
                            spawn_blocking(move || {
                                let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                    format!("Не удалось открыть БД для курсов: {}", e)
                                })?;
                                db::get_courses(&conn) // У вас должна быть db::get_courses
                                    .map_err(|e| format!("Ошибка загрузки курсов: {}", e))
                            })
                            .await
                            .unwrap_or_else(|join_err| {
                                Err(format!(
                                    "Ошибка выполнения задачи загрузки курсов: {:?}",
                                    join_err
                                ))
                            })
                        },
                        |result| Message::CoursesForPicklistLoaded(result),
                    );

                    let task_users = Task::perform(
                        async {
                            spawn_blocking(move || {
                                let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                    format!("Не удалось открыть БД для пользователей: {}", e)
                                })?;
                                db::get_all_users(&conn) // У вас должна быть db::get_all_users
                                    .map_err(|e| format!("Ошибка загрузки пользователей: {}", e))
                            })
                            .await
                            .unwrap_or_else(|join_err| {
                                Err(format!(
                                    "Ошибка выполнения задачи загрузки пользователей: {:?}",
                                    join_err
                                ))
                            })
                        },
                        |result| Message::UsersForPicklistLoaded(result),
                    );
                    Task::batch(vec![task_courses, task_users])
                } else {
                    // Очистка состояния модалки при закрытии
                    self.editing_group = None;
                    self.new_group_name.clear();
                    self.new_group_course = None;
                    self.new_group_teacher = None;
                    self.edit_group_name.clear();
                    self.edit_group_course = None;
                    self.edit_group_teacher = None;
                    self.group_error_message = None;
                    self.courses_for_picklist.clear(); // Очищаем данные PickList'ов
                    self.users_for_picklist.clear();
                    Task::none()
                }
            }
            Message::NewGroupNameChanged(name) => {
                self.new_group_name = name;
                Task::none()
            }
            Message::NewGroupCourseChanged(selected_course) => {
                self.new_group_course = selected_course.map(|c| c.id); // Сохраняем только ID
                Task::none()
            }
            Message::NewGroupTeacherChanged(selected_teacher) => {
                self.new_group_teacher = selected_teacher.map(|u| u.id); // Сохраняем только ID
                Task::none()
            }
            Message::NewGroupStatusChanged(status) => {
                self.new_group_status = status;
                Task::none()
            }
            Message::EditGroupNameChanged(name) => {
                self.edit_group_name = name;
                Task::none()
            }
            Message::EditGroupCourseChanged(selected_course) => {
                self.edit_group_course = selected_course.map(|c| c.id); // Сохраняем только ID
                Task::none()
            }
            Message::EditGroupTeacherChanged(selected_teacher) => {
                self.edit_group_teacher = selected_teacher.map(|u| u.id); // Сохраняем только ID
                Task::none()
            }
            Message::EditGroupStatusChanged(status) => {
                self.edit_group_status = status;
                Task::none()
            }
            Message::StartEditingGroup(group) => {
                self.editing_group = Some(group.clone());
                self.edit_group_name = group.name;
                self.edit_group_course = group.course_id; // Убедитесь, что group.course_id это Option<i32>
                self.edit_group_teacher = group.teacher_id; // Убедитесь, что group.teacher_id это Option<i32>
                self.edit_group_status = group.status;
                self.show_add_group_modal = true; // Открываем модалку
                // Вызываем загрузку списков для PickList
                Task::batch(vec![
                    Task::perform(
                        async {
                            db::get_courses(&Connection::open(PATH_TO_DB).unwrap())
                                .map_err(|e| e.to_string())
                        },
                        |r| Message::CoursesForPicklistLoaded(r),
                    ),
                    Task::perform(
                        async {
                            db::get_all_users(&Connection::open(PATH_TO_DB).unwrap())
                                .map_err(|e| e.to_string())
                        },
                        |r| Message::UsersForPicklistLoaded(r),
                    ),
                ])
            }
            Message::CancelEditingGroup => {
                self.editing_group = None;
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.show_add_group_modal = false;
                self.group_error_message = None;
                Task::none()
            }
            Message::SubmitEditedGroup => {
                // Проверки на пустые поля
                if self.edit_group_name.is_empty() {
                    self.group_error_message =
                        Some("Название группы не может быть пустым.".to_string());
                    return Task::none();
                }
                if self.edit_group_course.is_none() {
                    self.group_error_message =
                        Some("Необходимо выбрать курс для группы.".to_string());
                    return Task::none();
                }
                if self.edit_group_teacher.is_none() {
                    self.group_error_message =
                        Some("Необходимо выбрать преподавателя для группы.".to_string());
                    return Task::none();
                }
                if self.editing_group.is_none() {
                    self.group_error_message =
                        Some("Ошибка: группа для редактирования не выбрана.".to_string());
                    return Task::none();
                }

                // Если все поля заполнены, очищаем сообщение об ошибке
                self.group_error_message = None;

                let group_id = self.editing_group.as_ref().unwrap().id; // ID редактируемой группы
                let group_name_clone = self.edit_group_name.clone();
                let group_course_id = self.edit_group_course.unwrap_or_default(); // ID курса (уже i32)
                let group_teacher_id = self.edit_group_teacher.unwrap_or_default(); // ID преподавателя (уже i32)
                let group_status_string = self.edit_group_status;

                // Очищаем поля ввода и сбрасываем состояние редактирования до выполнения Task
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.group_error_message = None;
                self.editing_group = None; // Сброс редактируемой группы
                self.show_add_group_modal = false; // Закрываем модалку

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                            db::update_group(
                                &conn,
                                group_id,
                                &group_name_clone,
                                group_course_id,
                                group_teacher_id,
                                group_status_string,
                            )
                            .map_err(|e| format!("Ошибка обновления группы: {}", e))?;
                            Ok(()) // Возвращаем Ok(()) если все успешно
                        })
                        .await
                        .map_err(|join_err| format!("Ошибка выполнения задачи: {:?}", join_err))?
                    },
                    |result: Result<(), String>| {
                        match result {
                            Ok(_) => Message::LoadAllGroups, // Обновить список групп
                            Err(e) => Message::ErrorOccurred(e),
                        }
                    },
                )
            }
            Message::SubmitNewGroup => {
                // Проверки на пустые поля
                if self.new_group_name.is_empty() {
                    self.group_error_message =
                        Some("Название группы не может быть пустым.".to_string());
                    return Task::none();
                }
                if self.new_group_course.is_none() {
                    self.group_error_message =
                        Some("Необходимо выбрать курс для группы.".to_string());
                    return Task::none();
                }
                if self.new_group_teacher.is_none() {
                    self.group_error_message =
                        Some("Необходимо выбрать преподавателя для группы.".to_string());
                    return Task::none();
                }

                // Если все поля заполнены, очищаем сообщение об ошибке
                self.group_error_message = None;

                let group_name_clone = self.new_group_name.clone();
                // ИСПРАВЛЕНО: Просто unwrap() для Option<i32>
                let group_course_id = self.new_group_course.unwrap(); // new_group_course имеет тип Option<i32>
                let group_teacher_id = self.new_group_teacher.unwrap(); // new_group_teacher имеет тип Option<i32>
                let group_status_string = self.new_group_status;

                // Очищаем поля ввода до выполнения Task
                self.new_group_name.clear();
                self.new_group_course = None;
                self.new_group_teacher = None;
                self.group_error_message = None;
                self.show_add_group_modal = false; // Закрываем модалку

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                            // Вызов функции БД теперь корректен с i32
                            db::insert_group(
                                &conn,
                                &group_name_clone,
                                group_course_id,
                                group_teacher_id,
                                group_status_string,
                            )
                            .map_err(|e| format!("Ошибка добавления группы: {}", e))?;
                            Ok(()) // Возвращаем Ok(()) если все успешно
                        })
                        .await
                        .map_err(|join_err| format!("Ошибка выполнения задачи: {:?}", join_err))?
                    },
                    |result: Result<(), String>| {
                        match result {
                            Ok(_) => Message::LoadAllGroups, // Обновить список групп
                            Err(e) => Message::ErrorOccurred(e),
                        }
                    },
                )
            }
            Message::CourseLessonsLoaded(result) => {
                match result {
                    Ok(lessons) => {
                        self.course_lessons = lessons; // Предполагается, что у вас есть поле `course_lessons: Vec<LessonWithAssignments>` в структуре App
                        self.lesson_error_message = None; // Очищаем предыдущее сообщение об ошибке
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки уроков курса: {}", e);
                        self.course_lessons.clear(); // Очищаем уроки в случае ошибки
                        self.lesson_error_message = Some(e); // Отображаем сообщение об ошибке
                    }
                }
                Task::none() // Эта задача завершена
            }
            Message::DeleteGroup(id) => {
                let conn = Connection::open(PATH_TO_DB).unwrap();
                if let Err(err) = db::delete_group(&conn, id) {
                    eprintln!("Ошибка удаления группы: {:?}", err);
                    // Здесь можно установить сообщение об ошибке в состояние приложения
                } else {
                    // Группа успешно удалена. Теперь перезагрузите все группы.
                    match db::get_all_groups(&conn) {
                        // Предполагается, что у вас есть функция db::get_all_groups
                        Ok(groups) => {
                            self.all_groups = groups; // Обновляем состояние приложения
                        }
                        Err(e) => {
                            eprintln!("Ошибка при перезагрузке групп после удаления: {:?}", e);
                            // Установите сообщение об ошибке, если загрузка не удалась
                            self.group_error_message =
                                Some("Ошибка при обновлении списка групп.".to_string());
                        }
                    }
                    // Также очистите любые связанные выделения или модальные окна, если необходимо
                    self.selected_group_for_students_name = None;
                    self.current_manage_students_group_id = None;
                    self.show_group_students_modal = false;
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::GroupFilterChanged(text) => {
                self.group_filter_text = text;
                Task::none()
            }
            Message::OpenManageStudentsModal(group_id) => {
                println!(
                    "DEBUG: Открываем модальное окно 'Состав' для группы ID: {}",
                    group_id
                );
                self.is_manage_students_modal_open = true;
                self.show_group_students_modal = true;
                self.current_manage_students_group_id = Some(group_id); // Сохраняем ID текущей группы

                if let Some(group) = self.all_groups.iter().find(|g| g.id == group_id) {
                    self.selected_group_for_students_name = Some(group.name.clone());
                    println!("DEBUG: Имя группы для модального окна: {}", group.name);
                } else {
                    self.selected_group_for_students_name = Some("Неизвестная группа".to_string());
                    println!("DEBUG: Имя группы для модального окна: Неизвестная группа");
                }

                self.selected_group_students = Vec::new(); // Очищаем список перед новой загрузкой
                self.is_loading_group_students = true;

                // Запускаем асинхронную задачу для загрузки студентов В ТЕКУЩЕЙ ГРУППЕ
                let group_id_for_task = group_id; // Копируем для async move
                let task_students_in_group = Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для студентов группы: {}", e)
                            })?;
                            // Вызываем функцию для загрузки студентов конкретной группы
                            db::get_students_in_group(&conn, group_id_for_task)
                                .map_err(|e| format!("Ошибка загрузки студентов в группе: {}", e))
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения задачи студентов в группе: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::StudentsInGroupLoaded, // Сообщение, когда студенты в группе загружены
                );

                // Запускаем асинхронную задачу для загрузки студентов БЕЗ ГРУППЫ (для PickList)
                let task_students_without_group = Task::perform(
                    async {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для студентов без группы: {}", e)
                            })?;
                            // Вызываем функцию для загрузки студентов без группы
                            db::get_students_without_group(&conn)
                                .map_err(|e| format!("Ошибка загрузки студентов без группы: {}", e))
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения задачи студентов без группы: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::StudentsWithoutGroupLoaded, // Сообщение, когда студенты без группы загружены
                );

                // Возвращаем обе задачи, чтобы они выполнялись параллельно
                Task::batch(vec![task_students_in_group, task_students_without_group])
            }
            Message::StudentsInGroupLoaded(result) => {
                self.is_loading_group_students = false;
                match result {
                    Ok(students) => {
                        self.selected_group_students = students;
                        //self.students_in_current_group_modal = students;
                        println!(
                            "DEBUG: Студенты в текущей группе загружены: {} шт.",
                            self.selected_group_students.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Не удалось загрузить студентов в группе: {}", e);
                        self.group_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::StudentsWithoutGroupLoaded(result) => {
                match result {
                    Ok(students) => {
                        self.students_without_group = students;
                        println!(
                            "DEBUG: Студенты без группы загружены: {} шт.",
                            self.students_without_group.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Не удалось загрузить студентов без группы: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::AddStudentToGroup(student_id, group_id) => {
                Task::perform(
                    async move {
                        let mut conn = Connection::open(PATH_TO_DB).unwrap();
                        db::add_student_to_group(&mut conn, student_id, group_id)
                            .map_err(|e| format!("Ошибка добавления студента: {}", e))
                    },
                    move |result| {
                        if let Err(e) = result {
                            Message::ErrorOccurred(e) // Или ваше сообщение об ошибке
                        } else {
                            println!(
                                "DEBUG: Студент успешно добавлен/удален. Запускаем перезагрузку..."
                            );
                            Message::StudentsAndGroupsReloaded(group_id, 0) // Отправляем новое сообщение
                        }
                    },
                )
            }
            Message::SelectedStudentToAddChanged(student_opt) => {
                if let Some(student) = &student_opt {
                    println!("DEBUG: Выбран студент для добавления: {:?}", student.name);
                }
                self.selected_student_to_add = student_opt;
                self.group_error_message = None;
                Task::none()
            }
            Message::RemoveStudentFromGroup(student_id, group_id) => {
                println!(
                    "DEBUG: Попытка удалить студента ID: {} из группы ID: {}",
                    student_id, group_id
                );

                let group_id_for_async_task = group_id; // Для асинхронной задачи db
                let teacher_id = self.current_user.as_ref().map(|u| u.id).unwrap_or(0);

                Task::perform(
                    async move {
                        // 'move' здесь гарантирует, что student_id и group_id_for_async_task перемещаются в этот async блок
                        spawn_blocking(move || {
                            // 'move' здесь гарантирует, что student_id и group_id_for_async_task перемещаются в этот blocking блок
                            let mut conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для удаления студента: {}", e)
                            })?;
                            db::remove_student_from_group(
                                &mut conn,
                                student_id,
                                group_id_for_async_task,
                            ) // Используем переданные значения
                            .map_err(|e| format!("Ошибка удаления студента из группы: {}", e))
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения задачи удаления студента: {:?}",
                                join_err
                            ))
                        })
                    },
                    move |result| {
                        match result {
                            Ok(_) => {
                                println!("DEBUG: Студент успешно удален. Перезагружаем списки.");
                                // Используем перемещенную копию group_id
                                //Message::OpenManageStudentsModal(group_id_for_result_closure)
                                println!(
                                    "DEBUG: Получено StudentsAndGroupsReloaded для group_id: {}, teacher_id: {}",
                                    group_id, teacher_id
                                );
                                Message::StudentsAndGroupsReloaded(group_id, 0)
                            }
                            Err(e) => {
                                eprintln!("ERROR: Не удалось удалить студента: {}", e);
                                // ИСПРАВЛЕНИЕ: используем Message::ErrorOccurred
                                Message::ErrorOccurred(e)
                            }
                        }
                    },
                )
            }
            Message::StudentsAndGroupsReloaded(group_id, teacher_id) => {
                // 1. Перезагрузка студентов в модальном окне
                let command1 = Task::perform(
                    async move {
                        let conn = Connection::open(PATH_TO_DB)
                            .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                        db::get_students_in_group(&conn, group_id)
                            .map(|students| (group_id, students))
                            .map_err(|e| format!("Ошибка загрузки студентов группы: {}", e))
                    },
                    Message::GroupStudentsLoaded,
                );

                // 2. Перезагрузка списка всех групп учителя
                let command2 = Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                            db::get_all_groups(&conn)
                                .map_err(|e| format!("Ошибка загрузки всех групп: {}", e))
                        })
                        .await
                        .unwrap_or_else(|j| Err(format!("Join error: {:?}", j)))
                    },
                    Message::AllGroupsLoaded,
                );

                // ✅ 3. Перезагрузка студентов без группы
                let command3 = Task::perform(
                    async move {
                        let conn = Connection::open(PATH_TO_DB)
                            .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                        db::get_students_without_group(&conn)
                            .map_err(|e| format!("Ошибка загрузки студентов без группы: {}", e))
                    },
                    Message::StudentsWithoutGroupLoaded,
                );

                Task::batch(vec![command1, command2, command3])
            }
            Message::ShowParentChildren(parent_email) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open(PATH_TO_DB).unwrap();

                match db::get_children_for_parent(&conn, &parent_email) {
                    Ok(children) => self.parent_children = children,
                    Err(err) => {
                        println!("Ошибка при получении детей родителя: {:?}", err);
                        self.parent_children = vec![];
                    }
                }

                match db::get_unassigned_children(&conn) {
                    Ok(children) => self.available_children = children,
                    Err(err) => {
                        println!("Ошибка при получении неназначенных детей: {:?}", err);
                        self.available_children.clear();
                    }
                }

                self.edit_user_email = parent_email;
                self.show_children_modal = true;
                Task::none() // Возвращаем Task::none()
            }
            Message::CloseParentChildrenModal => {
                self.show_children_modal = false;
                self.parent_children.clear();
                Task::none()
            }
            Message::DeleteChild {
                parent_email,
                child_email,
            } => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open(PATH_TO_DB).unwrap();
                if let Err(e) = db::delete_child_for_parent(&conn, &parent_email, &child_email) {
                    println!("Ошибка при удалении ребенка: {:?}", e);
                }

                match db::get_children_for_parent(&conn, &parent_email) {
                    Ok(children) => self.parent_children = children,
                    Err(_) => self.parent_children.clear(),
                }
                match db::get_unassigned_children(&conn) {
                    Ok(children) => self.available_children = children,
                    Err(err) => {
                        println!("Ошибка при получении неназначенных детей: {:?}", err);
                        self.available_children.clear();
                    }
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::AddChildToParent => {
                // Эта операция должна быть асинхронной
                let parent_email = self.edit_user_email.clone();

                if let Some(child) = self.selected_child_to_add.clone() {
                    println!(
                        "Attempting to add child with email: {} to parent with email: {}",
                        child.email, parent_email
                    );
                    let conn = Connection::open(PATH_TO_DB).unwrap();

                    if let Err(e) = db::add_child_to_parent(&conn, &parent_email, &child.email) {
                        println!("Ошибка при добавлении ребёнка: {}", e);
                    } else {
                        self.parent_children =
                            db::get_children_for_parent(&conn, &parent_email).unwrap_or_default();
                        self.available_children =
                            db::get_unassigned_children(&conn).unwrap_or_default();
                        self.selected_child_to_add = None;
                    }
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::SelectedChildToAddChanged(child) => {
                self.selected_child_to_add = Some(child);
                Task::none()
            }
            Message::ShowLessonsModal(course) => {
                self.editing_lessons_course = Some(course.clone()); // Клонируем course, чтобы использовать его в асинхронном блоке
                self.new_lesson_number_text = String::new();
                self.new_lesson_title = String::new();
                self.lesson_error_message = None;
                self.show_lessons_modal = true; // Открываем модалку сразу

                let course_id_clone = course.id;

                Task::perform(
                    async move {
                        let blocking_result = spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для загрузки уроков: {}", e)
                            })?;

                            // 1. Получаем все базовые уроки для этого курса
                            let mut lessons = db::get_lessons_for_course(&conn, course_id_clone)
                                .map_err(|e| format!("Ошибка загрузки уроков курса: {}", e))?;

                            for lesson in &mut lessons {
                                let assignments = db::get_assignments_for_lesson(&conn, lesson.id)
                                    .map_err(|e| {
                                        format!(
                                            "Ошибка загрузки заданий для урока {}: {}",
                                            lesson.id, e
                                        )
                                    })?;
                                lesson.assignments = assignments;
                            }
                            Ok(lessons)
                        })
                        .await;

                        blocking_result.unwrap_or_else(|join_err| {
                            eprintln!(
                                "Блокирующая задача для уроков завершилась ошибкой: {:?}",
                                join_err
                            );
                            Err(format!("Ошибка выполнения операции: {}", join_err))
                        })
                    },
                    |result: Result<Vec<LessonWithAssignments>, String>| {
                        // Отправляем результат в новое сообщение, чтобы обновить App.course_lessons
                        Message::CourseLessonsLoaded(result)
                    },
                )
            }
            Message::CloseLessonsModal => {
                self.show_lessons_modal = false;
                self.editing_lessons_course = None;
                self.course_lessons.clear();
                self.new_lesson_number_text = String::new();
                self.new_lesson_title = String::new();
                Task::none()
            }
            Message::NewLessonNumberChanged(text) => {
                self.new_lesson_number_text = text;
                Task::none()
            }
            Message::NewLessonTitleChanged(text) => {
                self.new_lesson_title = text;
                Task::none()
            }
            Message::AddLesson => {
                self.lesson_error_message = None;

                if let Some(course) = &self.editing_lessons_course {
                    let course_id = course.id;
                    let lesson_number = self.new_lesson_number_text.parse::<i32>().ok();
                    let lesson_title = self.new_lesson_title.trim();

                    if lesson_title.is_empty() && lesson_number.is_none() {
                        self.lesson_error_message =
                            Some("Название занятия не может быть пустым.".to_string());
                        println!("Ошибка добавления занятия: Название не может быть пустым.");
                        return Task::none(); // Возвращаем Task::none()
                    }

                    // Эта операция должна быть асинхронной
                    let conn = Connection::open(PATH_TO_DB).unwrap();
                    match db::add_lesson(
                        &conn,
                        course_id,
                        Some(lesson_number.unwrap_or(0)),
                        lesson_title,
                    ) {
                        Ok(_) => {
                            println!("Занятие успешно добавлено.");
                            self.new_lesson_number_text.clear();
                            self.new_lesson_title.clear();
                            self.lesson_error_message = None;

                            match db::get_lessons_for_course(&conn, course_id) {
                                Ok(lessons) => self.course_lessons = lessons,
                                Err(e) => println!("Ошибка при обновлении списка занятий: {:?}", e),
                            }
                        }
                        Err(e) => {
                            println!("Ошибка при добавлении занятия в БД: {:?}", e);
                            self.lesson_error_message =
                                Some(format!("Ошибка БД при добавлении занятия: {:?}", e));
                        }
                    }
                } else {
                    println!("Ошибка: Не выбран курс для добавления занятия.");
                    self.lesson_error_message =
                        Some("Не выбран курс для добавления занятия.".to_string());
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::GoToClasses => {
                self.current_screen = Screen::Classes;
                if let Some(user) = &self.current_user {
                    if user.user_type == "teacher" {
                        println!(
                            "Teacher logged in, attempting to load groups for: {}",
                            user.email
                        );
                        return Task::perform(
                            load_teacher_groups(user.email.clone()),
                            Message::TeacherGroupsLoaded,
                        );
                    }
                }
                Task::none()
            }
            Message::GoToPayment => {
                self.current_screen = Screen::Payment;
                Task::perform(
                    async {
                        let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                        db::get_all_payments_with_details(&conn).map_err(|e| e.to_string())
                    },
                    Message::PaymentsFetched,
                )
            }
            Message::GoToCertificates => {
                self.current_screen = Screen::Certificates;
                self.error_message = "".to_string();
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД для загрузки студентов с сертификатами: {}", e))?;
                            // Вызываем функцию, которая возвращает Vec<UserInfo>
                            db::get_students_with_certificates_info(&conn)
                                .map_err(|e| format!("Не удалось загрузить студентов с сертификатами: {}", e))
                        }).await.unwrap_or_else(|join_err| {
                            Err(format!("Блокирующая задача (загрузка студентов с сертификатами) завершилась ошибкой: {:?}", join_err))
                        })
                    },
                    Message::StudentsWithCertificatesLoaded,
                )
            }
            Message::StudentsWithCertificatesLoaded(result) => {
                match result {
                    Ok(students) => {
                        self.students_with_certificates = students;
                        self.error_message = "".to_string();
                        println!(
                            "DEBUG: Студенты с сертификатами успешно загружены: {}",
                            self.students_with_certificates.len()
                        );
                    }
                    Err(e) => {
                        self.error_message = e;
                        eprintln!(
                            "Ошибка загрузки студентов с сертификатами: {}",
                            self.error_message
                        );
                    }
                }
                Task::none()
            }
            Message::OpenStudentCertificatesModal(student_info) => {
                println!(
                    "DEBUG: Открытие модалки сертификатов для студента: {}",
                    student_info.name
                );
                self.selected_student_for_certificates = Some(student_info.clone());
                self.show_student_certificates_modal = true;
                self.selected_student_certs.clear();
                self.is_loading_student_certs = true;

                let student_id = student_info.id; // ID студента все еще доступен
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД для загрузки сертификатов студента: {}", e))?;
                            db::get_certificates_for_student(&conn, student_id)
                                .map_err(|e| format!("Не удалось загрузить сертификаты для студента {}: {}", student_id, e))
                        }).await.unwrap_or_else(|join_err| {
                            Err(format!("Блокирующая задача (загрузка сертификатов студента) завершилась ошибкой: {:?}", join_err))
                        })
                    },
                    Message::StudentCertificatesLoaded,
                )
            }
            Message::StudentCertificatesLoaded(result) => {
                self.is_loading_student_certs = false;
                match result {
                    Ok(certs) => {
                        self.selected_student_certs = certs;
                        self.error_message = "".to_string();
                        println!(
                            "DEBUG: Сертификаты студента успешно загружены: {}",
                            self.selected_student_certs.len()
                        );
                    }
                    Err(e) => {
                        self.error_message = e;
                        eprintln!(
                            "Ошибка загрузки сертификатов студента: {}",
                            self.error_message
                        );
                    }
                }
                Task::none()
            }
            Message::CloseStudentCertificatesModal => {
                self.show_student_certificates_modal = false;
                self.selected_student_for_certificates = None;
                self.selected_student_certs.clear();
                self.error_message = "".to_string();
                Task::none()
            }
            Message::SelectGroupForClasses(group) => {
                self.selected_group_for_classes = Some(group.clone());
                self.show_teacher_assignment_modal = false;
                self.editing_teacher_assignment = None;
                self.teacher_lesson_assignments = Vec::new();

                let group_id_clone = group.id;
                // Переименовали переменную, чтобы показать, что это Option
                let course_id_for_group_option = group.course_id;

                Task::batch([
                    Task::perform(
                        async move {
                            let blocking_result = spawn_blocking(move || {
                                let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                    format!("Не удалось открыть БД для уроков/заданий: {}", e)
                                })?;
                                let course_id = course_id_for_group_option.ok_or_else(|| {
                                    "У выбранной группы нет связанного курса".to_string()
                                })?;
                                db::get_lessons_for_course_and_group(
                                    &conn,
                                    course_id,
                                    group_id_clone,
                                ) // `course_id` теперь i32
                                .map_err(|e| format!("Ошибка загрузки уроков для группы: {}", e))
                            })
                            .await;

                            blocking_result.unwrap_or_else(|join_err| {
                                eprintln!("Блокирующая задача для уроков/заданий завершилась ошибкой: {:?}", join_err);
                                Err(format!("Ошибка выполнения операции: {}", join_err))
                            })
                        },
                        Message::GroupLessonsWithAssignmentsLoaded,
                    ),
                    Task::perform(
                        async move {
                            let blocking_result = spawn_blocking(move || {
                                let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                    format!("Не удалось открыть БД для PastSessions: {}", e)
                                })?;
                                db::get_past_sessions_for_group(&conn, group_id_clone).map_err(
                                    |e| format!("Ошибка загрузки проведенных занятий: {}", e),
                                )
                            })
                            .await;

                            blocking_result.unwrap_or_else(|join_err| {
                                eprintln!(
                                    "Блокирующая задача для PastSessions завершилась ошибкой: {:?}",
                                    join_err
                                );
                                Err(format!("Ошибка выполнения операции: {}", join_err))
                            })
                        },
                        Message::PastSessionsLoaded,
                    ),
                ])
            }
            Message::ConductLessonClicked(lesson_id, group_id) => {
                println!(
                    "DEBUG: Handling ConductLesson for lesson_id: {}, group_id: {}",
                    lesson_id, group_id
                );
                let group_id_clone = group_id;
                let lesson_id_clone = lesson_id;

                Task::perform(
                    async move {
                        // 1. Попытка добавить PastSession
                        let add_result = spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для добавления PastSession: {}", e)
                            })?;
                            db::add_past_session(&conn, group_id_clone, lesson_id_clone).map_err(
                                |e| {
                                    format!("Ошибка добавления записи о проведенном занятии: {}", e)
                                },
                            )
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            // Если блокирующая задача упала или не смогла запуститься
                            Err(format!(
                                "Блокирующая задача (добавление) завершилась ошибкой: {:?}",
                                join_err
                            ))
                        });

                        match add_result {
                            Ok(_) => {
                                // 2. Если добавление успешно, пытаемся перезагрузить PastSessions
                                spawn_blocking(move || {
                                    let conn = Connection::open(PATH_TO_DB)
                                        .map_err(|e| format!("Не удалось открыть БД для перезагрузки PastSessions: {}", e))?;
                                    db::get_past_sessions_for_group(&conn, group_id_clone)
                                        .map_err(|e| format!("Ошибка перезагрузки PastSessions: {}", e))
                                }).await.unwrap_or_else(|join_err| {
                                    // Если блокирующая задача упала или не смогла запуститься
                                    Err(format!("Блокирующая задача (загрузка) завершилась ошибкой: {:?}", join_err))
                                })
                            }
                            Err(e) => Err(e), // Если была ошибка при добавлении, передаем её дальше
                        }
                    },
                    Message::ConductLessonResult,
                )
            }
            Message::ConductLessonResult(result) => {
                println!("DEBUG: Handling ConductLessonResult: {:?}", result.is_ok());
                if result.is_err() {
                    println!(
                        "DEBUG: ConductLessonResult error: {:?}",
                        result.clone().unwrap_err()
                    );
                }
                match result {
                    Ok(past_sessions) => {
                        println!(
                            "DEBUG: Successfully conducted lesson. Past sessions loaded: {}",
                            past_sessions.len()
                        );
                        self.past_sessions_for_group = past_sessions;

                        if let Some(group) = &self.selected_group_for_classes {
                            println!(
                                "DEBUG: Sending SelectGroupForClasses for group ID: {}",
                                group.id
                            );
                            let group_clone = group.clone(); // Клонируем здесь, чтобы владеть данными
                            Task::perform(
                                async move {
                                    Message::SelectGroupForClasses(group_clone) // Используем клонированную переменную
                                },
                                |msg| msg,
                            )
                        } else {
                            println!("DEBUG: No selected group, cannot re-select.");
                            Task::none()
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка проведения занятия или перезагрузки списка: {}", e);
                        self.error_message = e.to_string();
                        Task::none()
                    }
                }
            }
            Message::GroupLessonsWithAssignmentsLoaded(result) => match result {
                Ok(lessons) => {
                    self.selected_group_lessons_with_assignments = lessons;
                    Task::none()
                }
                Err(e) => {
                    eprintln!("Ошибка загрузки уроков для группы: {}", e);
                    self.error_message = e.to_string();
                    Task::none()
                }
            },
            Message::PastSessionsLoaded(result) => match result {
                Ok(past_sessions) => {
                    self.past_sessions_for_group = past_sessions;
                    Task::none()
                }
                Err(e) => {
                    eprintln!("Ошибка загрузки проведенных занятий: {}", e);
                    self.error_message = e.to_string();
                    Task::none()
                }
            },
            Message::DeleteLesson(lesson_id) => {
                // Проверяем, какой курс сейчас открыт
                if let Some(course) = &self.editing_lessons_course {
                    let course_id = course.id;
                    let conn = Connection::open(PATH_TO_DB).unwrap();
                    // Удаляем занятие из БД
                    match db::delete_lesson(&conn, lesson_id) {
                        Ok(_) => {
                            println!("Занятие успешно удалено: {}", lesson_id);
                            // После успешного удаления, нужно:
                            // 1. Обновить список занятий в модалке
                            self.lesson_error_message = None;
                            match db::get_lessons_for_course(&conn, course_id) {
                                Ok(lessons) => {
                                    self.course_lessons = lessons;
                                    Task::none()
                                }
                                Err(e) => {
                                    println!(
                                        "Ошибка при обновлении списка занятий после удаления: {:?}",
                                        e
                                    );
                                    Task::none()
                                }
                            }
                            // 2. Обновить основной список курсов, чтобы обновился счетчик занятий (см. комментарий в AddLesson)
                            // Пропускаем для простоты примера.
                        }
                        Err(e) => {
                            println!("Ошибка при удалении занятия {} из БД: {:?}", lesson_id, e);
                            Task::none()
                            // Отобразить ошибку в UI
                        }
                    }
                } else {
                    println!("Ошибка: Не выбран курс для удаления занятия.");
                    Task::none()
                }
            }
            Message::ShowAssignmentsModal(lesson_with_assignments) => {
                let conn = Connection::open(PATH_TO_DB).unwrap();
                self.current_lesson_for_assignments = Some(lesson_with_assignments.clone());

                match db::get_assignments_for_lesson(&conn, lesson_with_assignments.id) {
                    Ok(assignments) => {
                        self.lesson_assignments = assignments;
                        self.assignment_error_message = None;
                    }
                    Err(e) => {
                        self.lesson_assignments = vec![];
                        self.assignment_error_message =
                            Some(format!("Не удалось загрузить задания: {}", e));
                    }
                }
                self.show_assignments_modal = true;
                self.new_assignment_title.clear();
                self.new_assignment_description.clear();
                self.new_assignment_type = None;
                Task::none()
            }
            Message::CloseAssignmentsModal => {
                self.show_assignments_modal = false;
                self.current_lesson_for_assignments = None;
                self.lesson_assignments = vec![];
                self.assignment_error_message = None;
                self.new_assignment_title.clear();
                self.new_assignment_description.clear();
                self.new_assignment_type = None;
                Task::none()
            }
            Message::NewAssignmentTitleChanged(title) => {
                self.new_assignment_title = title;
                Task::none()
            }
            Message::NewAssignmentDescriptionChanged(description) => {
                self.new_assignment_description = description;
                Task::none()
            }
            Message::NewAssignmentTypeSelected(assignment_type) => {
                self.new_assignment_type = Some(assignment_type);
                Task::none()
            }
            Message::AddAssignment => {
                // Проверка наличия выбранного занятия
                let Some(current_lesson) = &self.current_lesson_for_assignments else {
                    self.assignment_error_message =
                        Some("Нет выбранного занятия для добавления задания.".to_string());
                    return Task::none();
                };

                // Проверка на пустые поля перед добавлением
                if self.new_assignment_title.is_empty() {
                    self.assignment_error_message =
                        Some("Название задания не может быть пустым.".to_string());
                    return Task::none();
                }
                if self.new_assignment_description.is_empty() {
                    self.assignment_error_message =
                        Some("Описание задания не может быть пустым.".to_string());
                    return Task::none();
                }
                let Some(assignment_type_enum) = self.new_assignment_type else {
                    self.assignment_error_message =
                        Some("Необходимо выбрать тип задания.".to_string());
                    return Task::none();
                };

                // Очищаем предыдущее сообщение об ошибке, если все проверки пройдены
                self.assignment_error_message = None;

                let lesson_id = current_lesson.id;
                let new_assignment_title_clone = self.new_assignment_title.clone();
                let new_assignment_description_clone = self.new_assignment_description.clone();
                let assignment_type_str = assignment_type_enum.to_string(); // Преобразуем enum в String

                // Очищаем поля формы до выполнения Task
                self.new_assignment_title.clear();
                self.new_assignment_description.clear();
                self.new_assignment_type = None; // Сброс выбранного типа

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                            db::add_assignment(
                                &conn,
                                lesson_id,
                                &new_assignment_title_clone,
                                &new_assignment_description_clone,
                                &assignment_type_str,
                            )
                            .map_err(|e| format!("Ошибка добавления задания: {}", e))?;
                            // После успешного добавления, загружаем обновленный список заданий
                            db::get_assignments_for_lesson(&conn, lesson_id).map_err(|e| {
                                format!("Не удалось перезагрузить задания после добавления: {}", e)
                            })
                        })
                        .await
                        .map_err(|join_err| format!("Ошибка выполнения задачи: {:?}", join_err))?
                    },
                    |result: Result<Vec<Assignment>, String>| {
                        // Ожидаем Result<Vec<Assignment>, String>
                        match result {
                            Ok(assignments) => {
                                Message::AssignmentsLoaded(Ok(assignments)) // Отправляем новое сообщение с загруженными заданиями
                            }
                            Err(e) => Message::ErrorOccurred(e),
                        }
                    },
                )
            }
            Message::DeleteAssignment(assignment_id) => {
                let conn = Connection::open(PATH_TO_DB).unwrap();
                match db::delete_assignment(&conn, assignment_id) {
                    Ok(_) => {
                        if let Some(current_lesson) = &self.current_lesson_for_assignments {
                            match db::get_assignments_for_lesson(&conn, current_lesson.id) {
                                Ok(assignments) => self.lesson_assignments = assignments,
                                Err(e) => {
                                    self.assignment_error_message =
                                        Some(format!("Не удалось перезагрузить задания: {}", e))
                                }
                            }
                        }
                        self.assignment_error_message = None;
                        Task::none()
                    }
                    Err(e) => {
                        self.assignment_error_message =
                            Some(format!("Ошибка удаления задания: {}", e));
                        Task::none()
                    }
                }
            }
            Message::ShowAssignmentDetailModal(assignment) => {
                self.selected_assignment_for_detail = Some(assignment.clone()); // Клонируем, чтобы работать с owned data
                self.show_assignment_detail_modal = true;

                // --- ДОБАВЛЕНО: Инициализация полей редактирования ---
                self.editing_assignment_title = assignment.title.clone(); // Заголовок всегда строковый

                // В зависимости от типа задания, инициализируем либо TextEditor, либо TextInput
                if assignment.assignment_type == AssignmentType::Lecture.to_string()
                    || assignment.assignment_type == AssignmentType::Practice.to_string()
                {
                    // Для TextEditor: создаем новое содержимое из описания
                    self.editing_assignment_description_content =
                        text_editor::Content::with_text(&assignment.description);
                    // Очищаем поле TextInput, если оно используется для другого типа
                    self.editing_assignment_description_text_input = String::new();
                } else {
                    // Для TextInput: устанавливаем описание
                    self.editing_assignment_description_text_input = assignment.description.clone();
                    // Очищаем TextEditor
                    self.editing_assignment_description_content = text_editor::Content::new();
                }
                self.assignment_edit_error_message = None; // Очищаем старые ошибки
                // --- КОНЕЦ ДОБАВЛЕННОГО ---

                Task::none()
            }
            Message::CloseAssignmentDetailModal => {
                self.show_assignment_detail_modal = false;
                self.selected_assignment_for_detail = None; // Очищаем выбранное задание
                Task::none()
            }
            Message::EditingAssignmentDescriptionChanged(input) => {
                match input {
                    TextInputOrEditorInput::TextEditor(action) => {
                        // Если пришло действие из TextEditor, примените его к TextEditor контенту
                        self.editing_assignment_description_content.perform(action);
                        Task::none()
                        // Опционально: если вам нужно, чтобы String поле всегда отражало TextEditor, синхронизируйте их:
                        // self.editing_assignment_description_text_input = self.editing_assignment_description_content.text();
                    }
                    TextInputOrEditorInput::TextInput(text) => {
                        // Если пришла строка из TextInput, обновите String поле
                        self.editing_assignment_description_text_input = text;
                        Task::none()
                        // Опционально: если вам нужно, чтобы TextEditor контент отражал String поле (например, при переключении типов), синхронизируйте их:
                        // self.editing_assignment_description_content = text_editor::Content::with_text(&text); // Осторожно: это сбрасывает историю действий TextEditor
                    }
                }
            }
            Message::EditingAssignmentTitleChanged(title) => {
                self.editing_assignment_title = title;
                Task::none()
            }
            Message::SaveEditedAssignment => {
                let conn = Connection::open(PATH_TO_DB).unwrap();
                if let Some(selected_assignment) = &self.selected_assignment_for_detail {
                    if self.editing_assignment_title.is_empty() {
                        self.assignment_edit_error_message =
                            Some("Название задания не может быть пустым.".to_string());
                        return Task::none(); // <-- Правильный возврат!
                    }

                    let description_to_save = if selected_assignment.assignment_type
                        == AssignmentType::Lecture.to_string()
                        || selected_assignment.assignment_type
                            == AssignmentType::Practice.to_string()
                    {
                        self.editing_assignment_description_content.text()
                    } else {
                        self.editing_assignment_description_text_input.clone()
                    };

                    let updated_assignment = Assignment {
                        id: selected_assignment.id,
                        lesson_id: selected_assignment.lesson_id,
                        title: self.editing_assignment_title.clone(),
                        description: description_to_save,
                        assignment_type: selected_assignment.assignment_type.clone(),
                    };

                    match db::update_assignment(&conn, &updated_assignment) {
                        Ok(_) => {
                            self.selected_assignment_for_detail = Some(updated_assignment.clone());
                            self.assignment_edit_error_message = None;
                            self.show_assignment_detail_modal = false;

                            // Важно: Обновить список заданий, чтобы изменения отобразились.
                            // Возвращаем Task, чтобы выполнить асинхронную операцию
                            if self.show_assignments_modal {
                                if let Some(lesson) = &self.current_lesson_for_assignments {
                                    let lesson_id_clone = lesson.id;
                                    return Task::perform(
                                        async move {
                                            // <-- Асинхронный блок - это Future, передаваемый в Task::perform
                                            // Этот фьючер spawn_blocking сам по себе возвращает Result<Result<Vec<Assignment>, String>, JoinError>
                                            let blocking_result = spawn_blocking(move || {
                                                let conn_task = Connection::open(PATH_TO_DB)
                                                    .map_err(|e| format!("Не удалось открыть БД для загрузки заданий: {}", e))?;
                                                db::get_assignments_for_lesson(&conn_task, lesson_id_clone)
                                                    .map_err(|e| format!("Ошибка загрузки заданий: {}", e))
                                            }).await; // <-- Ожидаем завершения блокирующей задачи, чтобы получить ее Result<T, JoinError>

                                            // Теперь разворачиваем внешний Result от spawn_blocking.
                                            // Если spawn_blocking сам по себе завершился с ошибкой (например, паникой),
                                            // преобразуем JoinError в String.
                                            // В противном случае у нас есть внутренний Result<Vec<Assignment>, String>.
                                            blocking_result.unwrap_or_else(|join_err| {
                                                // Обрабатываем JoinError (например, если блокирующая задача запаниковала)
                                                eprintln!("Блокирующая задача запаниковала или была отменена: {:?}", join_err);
                                                Err(format!("Ошибка выполнения операции: {}", join_err))
                                            })
                                        },
                                        Message::AssignmentsLoaded, // <-- Теперь это сообщение корректно ожидает Result<Vec<Assignment>, String>
                                    );
                                }
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.assignment_edit_error_message =
                                Some(format!("Ошибка сохранения задания: {}", e));
                            Task::none()
                        }
                    }
                } else {
                    eprintln!("Попытка сохранить детали задания, но задание не выбрано.");
                    self.assignment_edit_error_message =
                        Some("Ошибка: Не выбрано задание для сохранения.".to_string());
                    Task::none()
                }
            }
            Message::AssignmentsLoaded(result) => {
                match result {
                    Ok(assignments) => {
                        self.lesson_assignments = assignments;
                        self.assignment_error_message = None; // Очищаем ошибки после успешной загрузки
                    }
                    Err(e) => {
                        self.assignment_error_message =
                            Some(format!("Ошибка загрузки заданий: {}", e));
                    }
                }
                Task::none()
            }
            Message::LoadTeacherGroups(teacher_id_to_load) => {
                println!(
                    "DEBUG: Запущена асинхронная загрузка групп для преподавателя ID: {}",
                    teacher_id_to_load
                );
                Task::perform(
                    async move {
                        // Вызываем `spawn_blocking` напрямую из `task`
                        spawn_blocking(move || {
                            // <-- Меняем на `task::spawn_blocking`
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?;
                            db::get_teacher_groups_with_details(&conn, teacher_id_to_load)
                                .map_err(|e| format!("Ошибка загрузки групп из БД: {}", e))
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения блокирующей задачи: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::TeacherGroupsLoaded,
                )
            }
            Message::LoadAllGroups => {
                println!("DEBUG: Запущена асинхронная загрузка ВСЕХ групп (для администратора).");
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для загрузки ВСЕХ групп: {}", e)
                            })?;
                            db::get_all_groups(&conn) // <--- ВЫЗЫВАЕМ НОВУЮ ФУНКЦИЮ
                                .map_err(|e| format!("Ошибка загрузки ВСЕХ групп из БД: {}", e))
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Ошибка выполнения блокирующей задачи загрузки ВСЕХ групп: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::AllGroupsLoaded, // <--- ИСПОЛЬЗУЕМ НОВОЕ СООБЩЕНИЕ
                )
            }
            Message::AllGroupsLoaded(result) => {
                // <--- НОВЫЙ ОБРАБОТЧИК
                match result {
                    Ok(groups) => {
                        self.all_groups = groups; // Обновляем новое поле
                        self.group_error_message = None; // Очищаем ошибку, если она была
                        println!(
                            "DEBUG: AllGroupsLoaded успешно. Загружено ВСЕХ групп: {} шт.",
                            self.all_groups.len()
                        );
                    }
                    Err(e) => {
                        self.group_error_message =
                            Some(format!("Ошибка загрузки ВСЕХ групп: {}", e));
                        eprintln!("Ошибка загрузки ВСЕХ групп для администратора: {}", e);
                    }
                }
                Task::none()
            }
            Message::TeacherGroupsLoaded(result) => {
                match result {
                    Ok(groups) => {
                        println!(
                            "DEBUG: TeacherGroupsLoaded успешно. Загружено групп: {}",
                            groups.len()
                        );
                        self.teacher_groups = groups; // <-- Теперь self.teacher_groups будет содержать либо группы преподавателя, либо ВСЕ группы
                        println!(
                            "DEBUG: Группы успешно загружены: {} шт.",
                            self.teacher_groups.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Не удалось загрузить группы: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::OpenGroupLessonsModal(group_id, course_id) => {
                self.show_group_lessons_modal = true;
                self.group_lessons_modal_lessons.clear();
                self.group_lessons_modal_past_sessions.clear();

                // Ищем название группы для заголовка модального окна
                if let Some(group_found) = self.all_groups.iter().find(|g| g.id == group_id) {
                    self.group_lessons_modal_group_name = group_found.name.clone();
                } else {
                    self.group_lessons_modal_group_name = "Неизвестная группа".to_string();
                }

                Task::perform(
                    async move {
                        let result = spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                format!("Не удалось открыть БД для загрузки занятий группы: {}", e)
                            })?;

                            // Загружаем уроки, которые ЕЩЕ НЕ ПРОВЕДЕНЫ для этой группы
                            let available_lessons =
                                db::get_lessons_for_course_and_group(&conn, course_id, group_id)
                                    .map_err(|e| {
                                        format!("Ошибка загрузки доступных уроков: {}", e)
                                    })?;

                            // Загружаем уроки, которые УЖЕ ПРОВЕДЕНЫ для этой группы
                            let past_sessions = db::get_past_sessions_for_group(&conn, group_id)
                                .map_err(|e| format!("Ошибка загрузки прошедших занятий: {}", e))?;

                            Ok((available_lessons, past_sessions))
                        })
                        .await;

                        // Обработка ошибок из spawn_blocking
                        result.unwrap_or_else(|join_err| {
                            Err(format!(
                                "Блокирующая задача завершилась ошибкой: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::GroupLessonsModalLoaded, // Отправляем результат в новое сообщение
                )
            }
            Message::GroupLessonsModalLoaded(result) => {
                match result {
                    Ok((lessons, past_sessions)) => {
                        self.group_lessons_modal_lessons = lessons;
                        self.group_lessons_modal_past_sessions = past_sessions;
                        Task::none() // Больше никаких задач не нужно
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки занятий для модального окна: {}", e);
                        self.error_message = e.to_string();
                        self.show_group_lessons_modal = false; // Закрываем модальное окно при ошибке
                        Task::none()
                    }
                }
            }
            Message::CloseGroupLessonsModal => {
                self.show_group_lessons_modal = false;
                self.group_lessons_modal_lessons.clear();
                self.group_lessons_modal_past_sessions.clear();
                self.group_lessons_modal_group_name.clear();
                Task::none()
            }
            Message::ErrorOccurred(_) => Task::none(),
            Message::PaymentsFetched(Ok(payments)) => {
                self.payments = payments;
                Task::none()
            }
            Message::PaymentsFetched(Err(e)) => {
                eprintln!("Ошибка загрузки платежей: {}", e);
                // Тут можно показать ошибку пользователю
                Task::none()
            }
            Message::ToggleAddPaymentModal => {
                self.show_add_payment_modal = !self.show_add_payment_modal;
                // При открытии модального окна загружаем необходимые данные
                if self.show_add_payment_modal {
                    self.reset_new_payment_form(); // Сбросить форму
                    Task::batch(vec![
                        Task::perform(
                            async {
                                // Используйте tokio::task::spawn_blocking для блокирующих DB-операций
                                let conn =
                                    Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                                db::get_students_not_in_any_group(&conn).map_err(|e| e.to_string())
                            },
                            Message::StudentsWithoutGroupFetched,
                        ),
                        Task::perform(
                            async {
                                let conn =
                                    Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                                db::get_courses_with_available_seats(&conn)
                                    .map_err(|e| e.to_string())
                            },
                            Message::CoursesWithSeatsFetched,
                        ),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::StudentsWithoutGroupFetched(Ok(students)) => {
                self.students_without_group = students;
                Task::none()
            }
            Message::StudentsWithoutGroupFetched(Err(e)) => {
                eprintln!("Ошибка загрузки студентов без группы: {}", e);
                Task::none()
            }
            Message::CoursesWithSeatsFetched(Ok(courses)) => {
                self.courses_with_seats = courses;
                Task::none()
            }
            Message::CoursesWithSeatsFetched(Err(e)) => {
                eprintln!("Ошибка загрузки курсов со свободными местами: {}", e);
                Task::none()
            }
            Message::NewPaymentFormStudentSelected(selected_student_item) => {
                self.new_payment_student = Some(selected_student_item);
                Task::none()
            }
            Message::NewPaymentFormCourseSelected(selected_course_item) => {
                // Теперь мы получаем выбранный CoursePickListItem
                self.new_payment_course = Some(selected_course_item.clone());

                // Автоматически подтягиваем цену курса из выбранного элемента
                if let Some(course_price_str) =
                    selected_course_item.price_display.strip_suffix(" €")
                {
                    if let Ok(price) = course_price_str.parse::<f64>() {
                        self.new_payment_amount = Some(price);
                    }
                } else if selected_course_item.price_display == "Цена не указана" {
                    self.new_payment_amount = None; // Или 0.0, в зависимости от логики
                }

                // Загружаем группы для выбранного курса, используя его ID
                Task::perform(
                    async move {
                        let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                        db::get_groups_by_course_id(&conn, selected_course_item.id)
                            .map_err(|e| e.to_string())
                    },
                    Message::GroupsForCourseFetched,
                )
            }
            Message::GroupsForCourseFetched(Ok(groups)) => {
                self.groups_for_selected_course = groups;
                Task::none()
            }
            Message::GroupsForCourseFetched(Err(e)) => {
                eprintln!("Ошибка загрузки групп для курса: {}", e);
                Task::none()
            }
            Message::NewPaymentFormGroupSelected(selected_group_item) => {
                self.new_payment_group = Some(selected_group_item);
                Task::none()
            }
            Message::NewPaymentFormTypeChanged(selected_type_string) => {
                let payment_types_options = vec!["Карта".to_string(), "QR-Код".to_string()];

                self.selected_payment_type_idx = payment_types_options
                    .iter()
                    .position(|s| s == &selected_type_string);

                self.new_payment_type = selected_type_string;

                Task::none() // Возвращаем пустую команду
            }
            Message::AddPaymentConfirmed => {
                if let (Some(student), Some(course), Some(group), Some(amount)) = (
                    &self.new_payment_student,
                    &self.new_payment_course,
                    &self.new_payment_group,
                    self.new_payment_amount,
                ) {
                    let student_id = student.id;
                    let course_id = course.id;
                    let group_id = group.id;
                    let payment_type = self.new_payment_type.clone();
                    let current_date = Local::now().format("%Y-%m-%d").to_string();

                    Task::perform(
                        async move {
                            let mut conn =
                                Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;

                            // Добавление платежа
                            db::add_payment(
                                &conn,
                                student_id,
                                &current_date,
                                amount,
                                &payment_type,
                                course_id,
                                group_id,
                            )
                            .map_err(|e| e.to_string())?;

                            // Добавление студента в группу
                            db::add_student_to_group(&mut conn, student_id, group_id)
                                .map_err(|e| e.to_string())?;

                            Ok(())
                        },
                        Message::PaymentAdded,
                    )
                } else {
                    eprintln!("Не все поля для нового платежа заполнены.");
                    Task::none()
                }
            }
            Message::PaymentAdded(Ok(_)) => {
                println!("Платеж успешно добавлен.");
                self.show_add_payment_modal = false;
                self.reset_new_payment_form();
                self.error_message = "".to_string();
                Task::batch(vec![
                    Task::perform(
                        async {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                            db::get_all_payments_with_details(&conn).map_err(|e| e.to_string())
                        },
                        Message::PaymentsFetched,
                    ),
                    Task::perform(
                        async {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                            db::get_students_not_in_any_group(&conn) // Перезагружаем этот список
                                .map_err(|e| e.to_string())
                        },
                        Message::StudentsWithoutGroupFetched,
                    ),
                    Task::perform(
                        async {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                            db::get_courses_with_available_seats(&conn) // Перезагружаем курсы (места)
                                .map_err(|e| e.to_string())
                        },
                        Message::CoursesWithSeatsFetched,
                    ),
                    Task::perform(
                        async {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;

                            db::get_all_groups(&conn).map_err(|e| e.to_string())
                        },
                        Message::GroupsFetched, // Это новое сообщение
                    ),
                ])
            }
            Message::GroupsFetched(Ok(groups)) => {
                self.all_groups = groups;
                Task::none()
            }
            Message::GroupsFetched(Err(e)) => {
                eprintln!("Ошибка загрузки всех групп: {}", e);
                Task::none()
            }
            Message::PaymentAdded(Err(e)) => {
                eprintln!("Ошибка добавления платежа: {}", e);
                Task::none()
            }
            Message::DeletePayment(payment_id) => {
                let conn = Connection::open(PATH_TO_DB).unwrap();

                if let Err(err) = db::delete_payment(&conn, payment_id) {
                    eprintln!("Ошибка удаления платежа: {:?}", err);
                    Task::none()
                } else {
                    // Загружаем обновлённый список платежей асинхронно
                    Task::perform(
                        async {
                            let conn = Connection::open(PATH_TO_DB).map_err(|e| e.to_string())?;
                            db::load_payments(&conn).map_err(|e| e.to_string())
                        },
                        |result| match result {
                            Ok(payments) => Message::PaymentsUpdated(payments),
                            Err(e) => {
                                eprintln!("Ошибка загрузки платежей после удаления: {}", e);
                                Message::NoOp
                            }
                        },
                    )
                }
            }
            Message::PaymentsUpdated(new_list) => {
                self.payments = new_list;
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::OpenConductLessonModal(lesson_id, group_id) => {
                // Сохраняем контекст для модального окна
                self.current_lesson_to_conduct = self
                    .selected_group_lessons_with_assignments
                    .iter()
                    .find(|l| l.id == lesson_id)
                    .cloned();
                self.current_group_for_attendance = self.selected_group_for_classes.clone();
                self.show_conduct_lesson_modal = true;

                // Загружаем студентов для выбранной группы
                let group_id_clone = group_id;
                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            let conn = Connection::open(PATH_TO_DB)
                                .map_err(|e| format!("Не удалось открыть БД для загрузки студентов: {}", e))?;
                            db::get_students_in_group(&conn, group_id_clone) // Вам понадобится эта новая функция БД
                                .map_err(|e| format!("Ошибка загрузки студентов для посещаемости: {}", e))
                        }).await.unwrap_or_else(|join_err| {
                            Err(format!("Блокирующая задача для загрузки студентов завершилась ошибкой: {:?}", join_err))
                        })
                    },
                    |result: Result<Vec<UserInfo>, String>| {
                        // Явно указываем, что входной тип - Vec<UserInfo>
                        let converted_result = result.map(|user_infos| {
                            user_infos
                                .into_iter()
                                .map(|user_info| {
                                    StudentAttendance {
                                        id: user_info.id,
                                        name: user_info.name,
                                        present: true, // По умолчанию true
                                    }
                                })
                                .collect()
                        });
                        Message::StudentsForAttendanceLoaded(converted_result)
                    },
                )
            }
            Message::StudentsForAttendanceLoaded(result) => {
                match result {
                    Ok(students) => {
                        // Инициализируем всех студентов как присутствующих по умолчанию
                        self.students_for_attendance = students
                            .into_iter()
                            .map(|s| StudentAttendance {
                                id: s.id,
                                name: s.name,
                                present: true, // По умолчанию присутствуют
                            })
                            .collect();
                        Task::none()
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки студентов для посещаемости: {}", e);
                        self.error_message = e.to_string();
                        self.show_conduct_lesson_modal = false; // Закрываем модальное окно при ошибке
                        Task::none()
                    }
                }
            }
            Message::ToggleStudentAttendance(student_id) => {
                if let Some(student) = self
                    .students_for_attendance
                    .iter_mut()
                    .find(|s| s.id == student_id)
                {
                    student.present = !student.present;
                }
                Task::none()
            }
            Message::SaveAttendance => {
                if let (Some(lesson), Some(group)) = (
                    &self.current_lesson_to_conduct,
                    &self.current_group_for_attendance,
                ) {
                    let lesson_id = lesson.id;
                    let group_id = group.id;
                    let students_to_save = self.students_for_attendance.clone(); // Клонируем для перемещения в асинхронный блок

                    self.show_conduct_lesson_modal = false; // Немедленно закрываем модальное окно

                    Task::perform(
                        async move {
                            spawn_blocking(move || {
                                let mut conn = Connection::open(PATH_TO_DB) // Удалите 'mut', если не изменяете conn после открытия
                                    .map_err(|e| format!("Не удалось открыть БД для сохранения посещаемости: {}", e))?;

                                // Начинаем транзакцию для атомарности
                                let tx = conn.transaction()
                                    .map_err(|e| format!("Ошибка начала транзакции: {}", e))?; // <--- ИСПРАВЛЕНИЕ ЗДЕСЬ

                                // 1. Добавляем PastSession
                                let past_session_id = db::add_past_session(&tx, group_id, lesson_id)
                                    .map_err(|e| format!("Ошибка добавления PastSession: {}", e))?; // <--- ИСПРАВЛЕНИЕ ЗДЕСЬ (и для других db:: вызовов)

                                // 2. Добавляем записи о посещаемости
                                for student in students_to_save {
                                    let present_status = if student.present { "Present" } else { "Absent" };
                                    db::add_attendance(&tx, group_id, past_session_id, student.id, present_status)
                                        .map_err(|e| format!("Ошибка добавления записи посещаемости: {}", e))?; // <--- ИСПРАВЛЕНИЕ ЗДЕСЬ
                                }

                                tx.commit()
                                    .map_err(|e| format!("Ошибка фиксации транзакции: {}", e))?; // <--- ИСПРАВЛЕНИЕ ЗДЕСЬ

                                // 3. Перезагружаем PastSessions для группы
                                db::get_past_sessions_for_group(&conn, group_id)
                                    .map_err(|e| format!("Ошибка перезагрузки PastSessions после сохранения: {}", e))
                            }).await.unwrap_or_else(|join_err| {
                                Err(format!("Блокирующая задача (сохранение посещаемости) завершилась ошибкой: {:?}", join_err))
                            })
                        },
                        |result| Message::AttendanceSavedResult(result), // Используем замыкание
                    )
                } else {
                    eprintln!(
                        "Ошибка: Отсутствует информация об уроке или группе для сохранения посещаемости."
                    );
                    Task::none()
                }
            }
            Message::AttendanceSavedResult(result) => {
                println!(
                    "DEBUG: Обработка AttendanceSavedResult: {:?}",
                    result.is_ok()
                );
                if result.is_err() {
                    println!(
                        "DEBUG: Ошибка AttendanceSavedResult: {:?}",
                        result.clone().unwrap_err()
                    );
                }
                match result {
                    Ok(past_sessions) => {
                        println!(
                            "DEBUG: Успешно отмечена посещаемость. Проведенные занятия загружены: {}",
                            past_sessions.len()
                        );
                        self.past_sessions_for_group = past_sessions;

                        let group_id = self
                            .current_group_for_attendance
                            .as_ref()
                            .map_or(0, |g| g.id);
                        let course_id = self
                            .current_lesson_to_conduct
                            .as_ref()
                            .map_or(0, |l| l.course_id); // Используем course_id из урока

                        if group_id != 0 && course_id != 0 {
                            // После успешного сохранения посещаемости, выполняем проверку завершения курса
                            Task::perform(
                                async move {
                                    spawn_blocking(move || {
                                        let mut conn = Connection::open(PATH_TO_DB)
                                            .map_err(|e| format!("Не удалось открыть БД для проверки завершения курса: {}", e))?;

                                        let tx = conn.transaction() // Начинаем новую транзакцию для этой проверки
                                            .map_err(|e| format!("Ошибка начала транзакции для проверки сертификатов: {}", e))?;

                                        // Вызываем новую функцию для проверки и выдачи сертификатов
                                        db::check_course_completion_and_issue_certificates(&tx, group_id, course_id)
                                            .map_err(|e| format!("Ошибка проверки завершения курса и выдачи сертификатов: {}", e))?;

                                        tx.commit()
                                            .map_err(|e| format!("Ошибка фиксации транзакции для сертификатов: {}", e))?;

                                        Ok(()) // Возвращаем Ok(()) если все успешно
                                    }).await.unwrap_or_else(|join_err| {
                                        Err(format!("Блокирующая задача (проверка сертификатов) завершилась ошибкой: {:?}", join_err))
                                    })
                                },
                                |check_result| Message::CourseCompletionChecked(check_result), // Отправляем результат в новое сообщение
                            )
                        } else {
                            println!(
                                "DEBUG: Нет информации о группе или курсе для проверки завершения курса."
                            );
                            Task::none()
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Ошибка сохранения посещаемости или перезагрузки списка: {}",
                            e
                        );
                        self.error_message = e.to_string();
                        Task::none()
                    }
                }
            }
            Message::CourseCompletionChecked(result) => {
                match result {
                    Ok(_) => {
                        println!(
                            "DEBUG: Проверка завершения курса успешно выполнена. Сертификаты возможно выданы."
                        );
                        // Здесь можно добавить обновление UI, если необходимо
                        // Например, перевыбрать группу, чтобы обновить UI, если это требуется.
                        if let Some(group) = &self.selected_group_for_classes {
                            let group_clone = group.clone();
                            Task::perform(
                                async move { Message::SelectGroupForClasses(group_clone) },
                                |msg| msg,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка при проверке завершения курса: {}", e);
                        self.error_message = e.to_string();
                        Task::none()
                    }
                }
            }
            Message::GenerateCertificatePdf(cert, student) => {
                self.error_message = format!("Генерация сертификата для {}...", cert.course_title);
                println!("DEBUG: Запущена генерация PDF для сертификата: {}", cert.id);

                Task::perform(
                    async move {
                        spawn_blocking(move || {
                            // 1. Получаем путь к директории с исполняемым файлом
                            let exe_dir = std::env::current_exe()
                                .map_err(|e| {
                                    format!("Не удалось получить путь к исполняемому файлу: {}", e)
                                })?
                                .parent()
                                .ok_or("Не удалось получить директорию исполняемого файла")?
                                .to_path_buf();

                            // 2. Формируем абсолютные пути к HTML и PDF
                            let assets_path = exe_dir.join("assets");
                            let html_path =
                                exe_dir.join("certificates").join("temp_certificate.html");
                            let pdf_path =
                                exe_dir.join("certificates").join("certificate_output.pdf");

                            println!("DEBUG: HTML путь: {:?}", html_path);
                            println!("DEBUG: PDF путь: {:?}", pdf_path);

                            // 3. Генерируем HTML
                            if let Err(e) =
                                generate_certificate_html(&cert, &student, &html_path, &assets_path)
                            {
                                return Err(format!("Ошибка генерации HTML: {}", e));
                            }

                            // 4. Генерируем PDF из HTML
                            if let Err(e) = generate_pdf_from_html(&html_path, &pdf_path) {
                                return Err(format!("Ошибка конвертации в PDF: {}", e));
                            }

                            Ok(pdf_path)
                        })
                        .await
                        .unwrap_or_else(|join_err| {
                            Err(format!(
                                "Блокирующая задача завершилась ошибкой: {:?}",
                                join_err
                            ))
                        })
                    },
                    Message::CertificatePdfGenerated,
                )
            }
            Message::CertificatePdfGenerated(result) => {
                match result {
                    Ok(path) => {
                        self.error_message =
                            format!("Сертификат успешно сохранен: {:?}", path.display());
                        println!("DEBUG: Сертификат сохранен: {}", path.display());
                        // Опционально: открыть файл во внешней программе
                        #[cfg(target_os = "windows")]
                        {
                            if let Err(e) = open::that(path) {
                                self.error_message = format!("Ошибка при открытии PDF: {}", e);
                                eprintln!("Ошибка при открытии PDF: {}", e);
                            }
                        }
                        #[cfg(target_os = "macos")]
                        {
                            if let Err(e) = open::that(path) {
                                self.error_message = format!("Ошибка при открытии PDF: {}", e);
                                eprintln!("Ошибка при открытии PDF: {}", e);
                            }
                        }
                        #[cfg(target_os = "linux")]
                        {
                            // Для Linux может потребоваться 'xdg-open' или аналогичная утилита
                            // Это сложнее, обычно вы просто сообщаете пользователю путь.
                            // Или можно использовать крейт 'open'
                            if let Err(e) = open::that(path) {
                                self.error_message = format!("Ошибка при открытии PDF: {}", e);
                                eprintln!("Ошибка при открытии PDF: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = format!("Ошибка генерации сертификата: {}", e);
                        eprintln!("Ошибка генерации сертификата: {}", self.error_message);
                    }
                }
                Task::none()
            }
            Message::ToggleReportModal => {
                self.show_report_modal = !self.show_report_modal;
                Task::none()
            }
            Message::ChooseStartDate => {
                self.date_picker_open = DatePickerOpen::Start;
                Task::none()
            }
            Message::ChooseEndDate => {
                self.date_picker_open = DatePickerOpen::End;
                Task::none()
            }
            Message::SubmitStartDate(date) => {
                self.report_period_start = date;
                self.date_picker_open = DatePickerOpen::None;
                Task::none()
            }
            Message::SubmitEndDate(date) => {
                self.report_period_end = date;
                self.date_picker_open = DatePickerOpen::None;
                Task::none()
            }
            Message::CancelDatePicker => {
                self.date_picker_open = DatePickerOpen::None;
                Task::none()
            }

            Message::GeneratePaymentReport => {
                let output_dir = Path::new("reports");
                if let Err(e) = fs::create_dir_all(output_dir) {
                    println!("Failed to create dir: {}", e);
                    return Task::none();
                }

                if let (Some(from), Some(to)) = (
                    NaiveDate::from_ymd_opt(
                        self.report_period_start.year,
                        self.report_period_start.month,
                        self.report_period_start.day,
                    ),
                    NaiveDate::from_ymd_opt(
                        self.report_period_end.year,
                        self.report_period_end.month,
                        self.report_period_end.day,
                    ),
                ) {
                    println!("Date range valid: {} - {}", from, to);

                    match self.selected_report_type {
                        Some(ReportType::PDF) => {
                            println!("Date range valid: {} - {}", from, to);

                            let filtered_payments: Vec<_> = self
                                .payments
                                .iter()
                                .filter(|payment| {
                                    if let Ok(payment_date) =
                                        NaiveDate::parse_from_str(&payment.date, "%Y-%m-%d")
                                    {
                                        payment_date >= from && payment_date <= to
                                    } else {
                                        false
                                    }
                                })
                                .cloned()
                                .collect();

                            if filtered_payments.is_empty() {
                                println!("Нет платежей за указанный период");
                                return Task::none();
                            }

                            let from_str = from.format("%Y-%m-%d").to_string();
                            let to_str = to.format("%Y-%m-%d").to_string();
                            let output_dir = output_dir.to_path_buf();

                            Task::perform(
                                async move {
                                    spawn_blocking(move || {
                                        println!("Inside blocking task");
                                        let path = output_dir.join("payment_report.pdf");
                                        generate_payment_report(
                                            &filtered_payments,
                                            &from_str,
                                            &to_str,
                                            &output_dir,
                                        )
                                        .unwrap();
                                        Ok(path)
                                    })
                                    .await
                                    .unwrap_or_else(|e| {
                                        Err(format!("Ошибка блокирующей задачи: {e:?}"))
                                    })
                                },
                                Message::ReportGenerated,
                            )
                        }

                        Some(ReportType::Excel) => {
                            println!("Generating Excel report");
                            let from_copy = from.clone();
                            let to_copy = to.clone();
                            let output_dir = output_dir.to_path_buf();

                            Task::perform(
                                async move {
                                    spawn_blocking(move || {
                                        let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                            format!("Ошибка подключения к БД: {}", e)
                                        })?;
                                        let payments =
                                            db::get_payments_between(&conn, from_copy, to_copy)
                                                .map_err(|e| e.to_string())?;

                                        let file_name = format!(
                                            "payment_report_{}_{}.xlsx",
                                            from_copy.format("%Y-%m-%d"),
                                            to_copy.format("%Y-%m-%d")
                                        );
                                        let path = output_dir.join(file_name);

                                        generate_payment_excel_report(
                                            &payments, &from_copy, &to_copy, &path,
                                        )
                                        .unwrap();

                                        Ok(path)
                                    })
                                    .await
                                    .unwrap_or_else(|e| {
                                        Err(format!("Ошибка блокирующей задачи: {e:?}"))
                                    })
                                },
                                Message::ReportGenerated,
                            )
                        }

                        None => {
                            println!("Тип отчёта не выбран");
                            Task::none()
                        }
                    }
                } else {
                    println!("Неверный период даты");
                    Task::none()
                }
            }
            Message::ReportGenerated(result) => {
                match result {
                    Ok(path) => {
                        println!("Отчёт успешно сгенерирован: {}", path.display());
                        #[cfg(any(
                            target_os = "windows",
                            target_os = "macos",
                            target_os = "linux"
                        ))]
                        {
                            if let Err(e) = open::that(&path) {
                                let msg = format!("Ошибка при открытии отчёта: {}", e);
                                eprintln!("{msg}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
                self.show_report_modal = false;
                Task::none()
            }
            Message::ReportTypeSelected(selected) => {
                self.selected_report_type = selected;
                Task::none()
            }
            Message::ToggleCertificateReportModal => {
                self.show_certificate_report_modal = !self.show_certificate_report_modal;
                Task::none()
            }
            Message::ToggleGroupReportModal => {
                self.show_group_report_modal = !self.show_group_report_modal;
                Task::none()
            }
            Message::ChooseCertificateReportStartDate => {
                self.date_picker_open = DatePickerOpen::Start;
                Task::none()
            }
            Message::ChooseCertificateReportEndDate => {
                self.date_picker_open = DatePickerOpen::End;
                Task::none()
            }
            Message::SubmitCertificateReportStartDate(date) => {
                self.report_period_start = date;

                if let (Some(start), Some(end)) = (
                    NaiveDate::from_ymd_opt(date.year, date.month, date.day),
                    NaiveDate::from_ymd_opt(
                        self.report_period_end.year,
                        self.report_period_end.month,
                        self.report_period_end.day,
                    ),
                ) {
                    if start > end {
                        self.report_period_end = self.report_period_start;
                    }
                }

                self.date_picker_open = DatePickerOpen::None;
                Task::none()
            }
            Message::SubmitCertificateReportEndDate(date) => {
                self.report_period_end = date;

                if let (Some(start), Some(end)) = (
                    NaiveDate::from_ymd_opt(
                        self.report_period_start.year,
                        self.report_period_start.month,
                        self.report_period_start.day,
                    ),
                    NaiveDate::from_ymd_opt(date.year, date.month, date.day),
                ) {
                    if end < start {
                        self.report_period_start = self.report_period_end;
                    }
                }

                self.date_picker_open = DatePickerOpen::None;
                Task::none()
            }
            Message::GenerateCertificateReport => {
                let output_dir = Path::new("reports");
                if let Err(e) = fs::create_dir_all(output_dir) {
                    eprintln!("Ошибка создания директории отчётов: {}", e);
                    self.error_message = format!("Ошибка создания директории отчётов: {}", e);
                    return Task::none();
                }

                let from = NaiveDate::from_ymd_opt(
                    self.report_period_start.year,
                    self.report_period_start.month,
                    self.report_period_start.day,
                );
                let to = NaiveDate::from_ymd_opt(
                    self.report_period_end.year,
                    self.report_period_end.month,
                    self.report_period_end.day,
                );

                match (from, to) {
                    (Some(from), Some(to)) => match self.selected_report_type {
                        Some(ReportType::PDF) => {
                            println!("Генерация PDF отчёта");

                            let from_copy = from.clone();
                            let to_copy = to.clone();
                            let output_dir = output_dir.to_path_buf();

                            Task::perform(
                                async move {
                                    spawn_blocking(move || {
                                        let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                            format!("Ошибка подключения к БД: {}", e)
                                        })?;

                                        // Получаем сертификаты из БД за период
                                        let certificates =
                                            db::get_certificates_between(&conn, from_copy, to_copy)
                                                .map_err(|e| e.to_string())?;

                                        if certificates.is_empty() {
                                            return Err(
                                                "Нет сертификатов за указанный период".to_string()
                                            );
                                        }

                                        let from_str = from_copy.format("%Y-%m-%d").to_string();
                                        let to_str = to_copy.format("%Y-%m-%d").to_string();

                                        let path = output_dir.join("certificate_report.pdf");

                                        generate_certificate_report(
                                            &certificates,
                                            &from_str,
                                            &to_str,
                                            &output_dir,
                                        )
                                        .map(|_| path)
                                        .map_err(|e| format!("Ошибка генерации PDF: {}", e))
                                    })
                                    .await
                                    .unwrap_or_else(|e| {
                                        Err(format!("Ошибка блокирующей задачи: {:?}", e))
                                    })
                                },
                                Message::CertificateReportGenerated,
                            )
                        }
                        Some(ReportType::Excel) => {
                            let from_copy = from.clone();
                            let to_copy = to.clone();
                            let output_dir = output_dir.to_path_buf();

                            Task::perform(
                                async move {
                                    spawn_blocking(move || {
                                        let conn = Connection::open(PATH_TO_DB).map_err(|e| {
                                            format!("Ошибка подключения к БД: {}", e)
                                        })?;
                                        let certificates =
                                            db::get_certificates_between(&conn, from_copy, to_copy)
                                                .map_err(|e| e.to_string())?;

                                        // Передаём в функцию только путь к папке, а не полный путь к файлу!
                                        generate_certificate_excel_report(
                                            &certificates,
                                            &from_copy,
                                            &to_copy,
                                            &output_dir,
                                        )
                                        .map(|_| output_dir.clone())
                                        .map_err(|e| format!("Ошибка генерации Excel: {}", e))
                                    })
                                    .await
                                    .unwrap_or_else(|e| {
                                        Err(format!("Ошибка блокирующей задачи: {e:?}"))
                                    })
                                },
                                Message::CertificateReportGenerated,
                            )
                        }
                        None => {
                            self.error_message = "Тип отчёта не выбран".to_string();
                            Task::none()
                        }
                    },
                    _ => {
                        self.error_message = "Неверный период даты".to_string();
                        Task::none()
                    }
                }
            }
            Message::CertificateReportGenerated(result) => {
                self.show_certificate_report_modal = false;

                match result {
                    Ok(path) => {
                        let success_msg = format!("Отчёт успешно сгенерирован: {}", path.display());
                        println!("{}", success_msg);
                        self.error_message = success_msg;

                        #[cfg(any(
                            target_os = "windows",
                            target_os = "macos",
                            target_os = "linux"
                        ))]
                        {
                            if let Err(e) = open::that(&path) {
                                let msg = format!("Ошибка при открытии отчёта: {}", e);
                                self.error_message = msg.clone();
                                eprintln!("{msg}");
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = format!("Ошибка генерации отчёта: {}", e);
                        eprintln!("{}", self.error_message);
                    }
                }

                Task::none()
            }
            Message::GenerateGroupReport => {
                let output_dir = Path::new("reports");
                if let Err(e) = fs::create_dir_all(output_dir) {
                    eprintln!("Ошибка создания директории отчётов: {}", e);
                    //self.error_message = format!("Ошибка создания директории отчётов: {}", e);
                    return Task::none();
                }

                // Подключаемся к БД
                let conn = match Connection::open(PATH_TO_DB) {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("Ошибка подключения к БД: {}", e);
                        return Task::none();
                    }
                };

                // Получаем все группы
                let groups = match db::get_all_groups_for_report(&conn) {
                    Ok(groups) => groups,
                    Err(e) => {
                        eprintln!("Ошибка получения групп: {}", e);
                        return Task::none();
                    }
                };

                if groups.is_empty() {
                    eprintln!("Нет групп для отчёта");
                    return Task::none();
                }

                match self.selected_report_type {
                    Some(ReportType::PDF) => {
                        let groups = groups.clone();
                        let output_dir = output_dir.to_path_buf();
                        Task::perform(
                            async move {
                                spawn_blocking(move || {
                                    generate_group_report(&groups, &output_dir)
                                        .map(|_| output_dir.join("group_report.pdf"))
                                        .map_err(|e| format!("Ошибка генерации PDF: {}", e))
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    Err(format!("Ошибка блокирующей задачи: {:?}", e))
                                })
                            },
                            Message::GroupReportGenerated,
                        )
                    }
                    Some(ReportType::Excel) => {
                        let groups = groups.clone();
                        let output_dir = output_dir.to_path_buf();
                        Task::perform(
                            async move {
                                spawn_blocking(move || {
                                    generate_group_excel_report(&groups, &output_dir)
                                        .map(|_| output_dir.clone())
                                        .map_err(|e| format!("Ошибка генерации Excel: {}", e))
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    Err(format!("Ошибка блокирующей задачи: {:?}", e))
                                })
                            },
                            Message::GroupReportGenerated,
                        )
                    }
                    None => {
                        eprintln!("Тип отчёта не выбран");
                        Task::none()
                    }
                }
            }

            Message::GroupReportGenerated(result) => {
                match result {
                    Ok(path) => {
                        let success_msg =
                            format!("Отчёт по группам успешно сгенерирован: {}", path.display());
                        println!("{}", success_msg);
                        self.error_message = success_msg;

                        #[cfg(any(
                            target_os = "windows",
                            target_os = "macos",
                            target_os = "linux"
                        ))]
                        {
                            if let Err(e) = open::that(&path) {
                                let msg = format!("Ошибка при открытии отчёта: {}", e);
                                self.error_message = msg.clone();
                                eprintln!("{msg}");
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = format!("Ошибка генерации отчёта по группам: {}", e);
                        eprintln!("{}", self.error_message);
                    }
                }

                Task::none()
            }
            Message::BackupIntervalSelected(interval) => {
                self.backup_interval = interval;
                save_config(
                    &self.theme.target(),
                    self.backup_interval.as_ref().map(|i| i.value),
                    self.backup_folder.clone(),
                    self.max_backup_count,
                )
                .ok();
                Task::none()
            }
            Message::BackupNowPressed => {
                let result = backup_database_now_with_config(
                    self.backup_folder.clone(),
                    self.max_backup_count,
                );

                if let Err(e) = result {
                    println!("Ошибка резервного копирования: {}", e);
                } else if let Some(folder) = &self.backup_folder {
                    self.last_backup_time = get_last_backup_time(folder);
                }

                Task::none()
            }

            Message::SelectBackupFolder => {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    self.backup_folder = Some(folder.display().to_string());
                    save_config(
                        &self.theme.target(),
                        self.backup_interval.as_ref().map(|interval| interval.value),
                        self.backup_folder.clone(),
                        self.max_backup_count,
                    )
                    .ok();
                }
                Task::none()
            }
            Message::MaxBackupCountSelected(count_opt) => {
                self.max_backup_count = count_opt;
                save_config(
                    &self.theme.target(),
                    self.backup_interval.as_ref().map(|interval| interval.value),
                    self.backup_folder.clone(),
                    self.max_backup_count,
                )
                .ok();
                Task::none()
            }
            Message::OpenBackupFolder => {
                if let Some(folder) = &self.backup_folder {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer").arg(folder).spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open").arg(folder).spawn();
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open").arg(folder).spawn();
                    }
                }
                Task::none()
            }
        }
    }
    fn reset_new_payment_form(&mut self) {
        self.new_payment_student = None;
        self.new_payment_course = None;
        self.new_payment_group = None;
        self.new_payment_amount = None;
        self.new_payment_type = "".to_string();
        self.students_without_group.clear();
        self.courses_with_seats.clear();
        self.groups_for_selected_course.clear();
        self.selected_payment_type_idx = Some(0);
    }
    fn clear_fields(&mut self) {
        self.user_name.clear();
        self.user_surname.clear();
        self.user_patronymic.clear();
        self.user_email.clear();
        self.user_password.clear();
        self.user_birthday.clear();
        self.user_password_repeat.clear();
        self.register_error = None;
        self.registration_success = false;
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(password);
    format!("{:x}", hasher.finalize())
}
async fn load_teacher_groups(teacher_email: String) -> Result<Vec<Group>, String> {
    let conn = Connection::open(PATH_TO_DB)
        .map_err(|e| format!("Failed to open database connection: {}", e))?;

    let teacher_id = db::get_user_id_by_email(&conn, &teacher_email)
        .ok_or_else(|| format!("Teacher with email '{}' not found.", teacher_email))?;

    db::get_groups_for_teacher(&conn, teacher_id)
        .map_err(|e| format!("Failed to load groups for teacher {}: {}", teacher_id, e))
}
pub fn icon_button_content<'a>(
    icon_element: impl Into<Element<'a, Message, Theme, Renderer>>, // Виджет иконки
    label: &'a str,                                                 // Текст метки
) -> Row<'a, Message> {
    Row::new()
        .align_y(Alignment::Center)
        .spacing(5)
        .push(icon_element)
        .push(text(label))
}
