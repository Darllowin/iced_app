use iced::{Length, Theme};
use iced::widget::{Column, Container, Row};
use crate::screens::{login_screen, register_screen, profile_screen, settings_screen, nav_menu, courses_screen, user_list_screen, groups_screen};
use std::fs;
use std::str::FromStr;
use iced_aw::date_picker::Date;
use regex::Regex;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection};
use sha2::{Sha256, Digest};
use crate::db;
use crate::screens::settings::theme_to_str;

const CONFIG_FILE: &str = "config.json";
pub const DEFAULF_AVATAR: &str = "default_avatar.jpg";

pub struct App {
    pub date: Date,
    pub show_picker: bool,
    //
    pub current_screen: Screen,
    //
    pub user_name: String,
    pub user_surname: String,
    pub user_patronymic: String,
    pub user_email: String,
    pub user_birthday: String,
    pub type_user: String,
    pub user_password: String,
    pub user_password_repeat: String,
    //
    pub theme: Theme,
    //
    pub register_error: Option<String>,
    pub registration_success: bool,
    pub logged_in_user: String,
    pub error_message: String,
    //
    pub user_avatar_data: Option<Vec<u8>>,
    //
    pub show_add_course_modal: bool,
    pub new_course_title: String,
    pub new_course_description: String,
    pub new_course_instructor: Option<String>,
    pub new_course_level: Level,
    // Добавлены поля для редактирования курса
    pub editing_course: Option<Course>,
    pub edit_course_title: String,
    pub edit_course_description: String,
    pub edit_course_instructor: Option<String>,
    pub edit_course_level: Level,
    //
    pub editing_user: Option<UserInfo>,
    pub edit_user_error: Option<String>,
    pub show_edit_user_modal: bool,
    pub edit_user_name: String,
    pub edit_user_email: String,
    pub edit_user_birthday: String,
    pub edit_user_type: String,
    //
    pub course_filter_text: String,
    // Группы
    pub show_add_group_modal: bool,
    pub new_group_name: String,
    pub new_group_course: Option<String>,
    pub new_group_teacher: Option<String>,

    pub editing_group: Option<Group>,
    pub edit_group_name: String,
    pub edit_group_course: Option<String>,
    pub edit_group_teacher: Option<String>,

    pub group_filter_text: String,

    pub selected_group_id: Option<i32>,
    pub is_manage_students_modal_open: bool,
    pub group_students: Vec<String>,
    pub all_students: Vec<String>,
    pub selected_student_to_add: Option<String>,
    
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub birthday: String,
    pub user_type: String,
    pub avatar_data: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub course: Option<String>,
    pub teacher: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Course {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub instructor: Option<String>,
    pub level: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

impl Level {
    pub const ALL: &'static [Level] = &[
        Level::Beginner,
        Level::Intermediate,
        Level::Advanced,
    ];
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Level::Beginner => "Начальный",
            Level::Intermediate => "Средний",
            Level::Advanced => "Продвинутый",
        })
    }
}

impl FromStr for Level {
    type Err = ();

    fn from_str(input: &str) -> Result<Level, Self::Err> {
        match input {
            "Начальный" => Ok(Level::Beginner),
            "Средний" => Ok(Level::Intermediate),
            "Продвинутый" => Ok(Level::Advanced),
            _ => Err(()),
        }
    }
}
#[derive(Serialize, Deserialize)]
struct Config {
    theme_name: String,
}
#[derive(PartialEq, Default)]
pub enum Screen {
    #[default]
    Login,
    Register,
    Profile,
    Settings,
    Courses,
    UserList,
    GroupList,
}
#[derive(Debug, Clone)]
pub enum Message {
    LoginPressed,
    RegisterPressed,
    //
    FirstNameChanged(String),
    LastNameChanged(String),
    MiddleNameChanged(String),
    EmailChanged(String),
    PasswordChanged(String),
    PasswordRepeatChanged(String),
    //
    SwitchToLogin,
    SwitchToRegister,
    GoToProfile,
    GoToSettings,
    GoToCourses,
    GoToUserList,
    GoToGroupList,
    Logout,
    //
    ThemeSelected(&'static str),
    //
    ChooseDate,
    SubmitDate(Date),
    CancelDate,
    Er(String),
    //
    ChooseAvatar,
    //
    NewCourseInstructorChanged(Option<String>),
    NewCourseLevelChanged(Level),
    ToggleAddCourseModal(bool),
    NewCourseTitleChanged(String),
    NewCourseDescriptionChanged(String),
    SubmitNewCourse,
    DeleteCourse(i32),
    // Редактирование курса
    StartEditingCourse(Course),
    EditCourseTitleChanged(String),
    EditCourseDescriptionChanged(String),
    EditCourseInstructorChanged(Option<String>),
    EditCourseLevelChanged(Level),
    SubmitEditedCourse,
    CancelEditingCourse,
    // Редактирование пользователя
    StartEditingUser(UserInfo),
    CancelEditingUser,
    SubmitEditedUser,
    DeleteUser(String),
    EditUserNameChanged(String),
    EditUserEmailChanged(String),
    EditUserBirthdayChanged(String),
    EditUserTypeChanged(String),
    //
    CourseFilterChanged(String),
    // Для групп
    ToggleAddGroupModal(bool),
    NewGroupNameChanged(String),
    NewGroupCourseChanged(Option<String>),
    NewGroupTeacherChanged(Option<String>),

    EditGroupNameChanged(String),
    EditGroupCourseChanged(Option<String>),
    EditGroupTeacherChanged(Option<String>),

    SubmitNewGroup,
    SubmitEditedGroup,
    StartEditingGroup(Group),
    CancelEditingGroup,
    DeleteGroup(i32),
    GroupFilterChanged(String),

    OpenManageStudentsModal(i32),
    CloseManageStudentsModal,
    StudentToAddSelected(Option<String>),
    AddStudent,
    RemoveStudent(String),
}
impl Default for App {
    fn default() -> Self {
        let selected_theme = load_theme().unwrap_or(Theme::Light);
        Self {
            error_message: "".to_string(),
            date: Date::today(),
            show_picker: false,
            current_screen: Default::default(),
            user_name: "".to_string(),
            user_surname: "".to_string(),
            user_patronymic: "".to_string(),
            user_email: "".to_string(),
            user_password: "".to_string(),
            user_password_repeat: "".to_string(),
            theme: selected_theme,
            register_error: None,
            registration_success: false,
            logged_in_user: "".to_string(),
            show_add_course_modal: false,
            new_course_title: "".to_string(),
            user_birthday: "".to_string(),
            type_user: "".to_string(),
            new_course_description: "".to_string(),
            new_course_instructor: None,
            new_course_level: Level::Beginner,
            editing_course: None,
            edit_course_title: "".to_string(),
            edit_course_description: "".to_string(),
            edit_course_instructor: None,
            user_avatar_data: None,
            edit_course_level: Level::Beginner,
            editing_user: None,
            edit_user_error: None,
            show_edit_user_modal: false,
            edit_user_name: "".to_string(),
            edit_user_email: "".to_string(),
            edit_user_birthday: "".to_string(),
            edit_user_type: "".to_string(),
            course_filter_text: "".to_string(),
            show_add_group_modal: false,
            new_group_name: "".to_string(),
            new_group_course: None,
            new_group_teacher: None,
            editing_group: None,
            edit_group_name: "".to_string(),
            edit_group_course: None,
            edit_group_teacher: None,
            group_filter_text: "".to_string(),
            selected_group_id: None,
            is_manage_students_modal_open: false,
            group_students: vec![],
            all_students: vec![],
            selected_student_to_add: None,
        }
    }
}
impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LoginPressed => {
                if self.user_email.trim().is_empty() || self.user_password.trim().is_empty() {
                    self.error_message = "Пожалуйста, заполните все поля.".to_string();
                    return;
                }

                let conn = Connection::open("db_platform").unwrap();
                let entered_hash = hash_password(&self.user_password);

                match db::check_user_credentials(&conn, &self.user_email, &entered_hash) {
                    Ok((name, avatar_data, birthday, type_user)) => {
                        self.logged_in_user = name;
                        self.user_birthday = birthday;
                        self.type_user = type_user;
                        self.user_avatar_data = avatar_data;
                        self.current_screen = Screen::Profile;
                        self.error_message.clear();
                    }
                    Err(db::LoginError::UserNotFound) => {
                        self.error_message = "Пользователь с таким email не найден.".to_string();
                    }
                    Err(db::LoginError::WrongPassword) => {
                        self.error_message = "Неверный пароль. Попробуйте снова.".to_string();
                    }
                    Err(db::LoginError::DatabaseError(_)) => {
                        self.error_message = "Ошибка базы данных. Попробуйте позже.".to_string();
                    }
                }
            }
            Message::SwitchToLogin => {
                self.current_screen = Screen::Login;
                self.clear_fields();
            }
            Message::SwitchToRegister => {
                self.current_screen = Screen::Register;
                self.clear_fields();
            }
            Message::FirstNameChanged(v) => self.user_name = v,
            Message::LastNameChanged(v) => self.user_surname = v,
            Message::MiddleNameChanged(v) => self.user_patronymic = v,
            Message::EmailChanged(v) => self.user_email = v,
            Message::PasswordChanged(v) => self.user_password = v,
            Message::PasswordRepeatChanged(v) => self.user_password_repeat = v,
            Message::RegisterPressed => {
                // Проверка пустых полей ФИО
                if self.user_name.trim().is_empty() || self.user_surname.trim().is_empty() || self.user_patronymic.trim().is_empty() {
                    self.register_error = Some("Пожалуйста, заполните Фамилию, Имя и Отчество".to_string());
                    return;
                }

                // Проверка ФИО: только русские буквы, пробелы и дефисы
                let fio_re = Regex::new(r"^[А-Яа-яЁё\s-]+$").unwrap();
                if !fio_re.is_match(&self.user_name) || !fio_re.is_match(&self.user_surname) || !fio_re.is_match(&self.user_patronymic) {
                    self.register_error = Some("ФИО может содержать только русские буквы, пробелы и дефисы".to_string());
                    return;
                }

                // Проверка паролей на пустоту
                if self.user_password.trim().is_empty() || self.user_password_repeat.trim().is_empty() {
                    self.register_error = Some("Пароль не может быть пустым".to_string());
                    return;
                }

                // Проверка совпадения паролей
                if self.user_password != self.user_password_repeat {
                    self.register_error = Some("Пароли не совпадают".to_string());
                    return;
                }

                let password = &self.user_password;

                // Проверка длины
                if password.len() < 8 {
                    self.register_error = Some("Пароль должен содержать минимум 8 символов".to_string());
                    return;
                }

                // Проверка наличия хотя бы одной заглавной буквы
                let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
                if !has_uppercase {
                    self.register_error = Some("Пароль должен содержать хотя бы одну заглавную букву".to_string());
                    return;
                }

                // Проверка наличия хотя бы одной цифры
                let has_digit = password.chars().any(|c| c.is_ascii_digit());
                if !has_digit {
                    self.register_error = Some("Пароль должен содержать хотя бы одну цифру".to_string());
                    return;
                }

                // Далее — твоя существующая проверка email:
                let email = self.user_email.trim();

                if email.is_empty() {
                    self.register_error = Some("Email не может быть пустым.".to_string());
                    return;
                }
                if !email.contains('@') {
                    self.register_error = Some("Email должен содержать символ '@'.".to_string());
                    return;
                }
                let parts: Vec<&str> = email.split('@').collect();
                if parts.len() != 2 {
                    self.register_error = Some("Email должен содержать только один символ '@'.".to_string());
                    return;
                }
                if parts[0].is_empty() {
                    self.register_error = Some("Email должен содержать имя пользователя перед '@'.".to_string());
                    return;
                }
                if parts[1].is_empty() {
                    self.register_error = Some("Email должен содержать домен после '@'.".to_string());
                    return;
                }
                if !parts[1].contains('.') {
                    self.register_error = Some("Домен email должен содержать хотя бы одну точку (например: example.com).".to_string());
                    return;
                }
                let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                if !email_re.is_match(email) {
                    self.register_error = Some("Email содержит недопустимые символы или некорректный формат.".to_string());
                    return;
                }

                // Проверка уникальности email
                let conn = Connection::open("db_platform").unwrap();
                match db::is_email_taken(&conn, email) {
                    Ok(true) => {
                        self.register_error = Some("Пользователь с таким email уже существует.".to_string());
                        return;
                    }
                    Ok(false) => {}
                    Err(err) => {
                        self.register_error = Some(format!("Ошибка при проверке email: {}", err));
                        return;
                    }
                }

                // Если все проверки пройдены — регистрируем пользователя
                let full_name = format!("{} {} {}", self.user_surname, self.user_name, self.user_patronymic);
                let password_hash = hash_password(&self.user_password);

                if let Err(_) = db::register_user(
                    &conn,
                    &full_name,
                    format!("{:02}.{:02}.{}", self.date.day, self.date.month, self.date.year).as_str(),
                    email,
                    &password_hash,
                ) {
                    self.register_error = Some("Ошибка при сохранении пользователя в базу данных.".to_string());
                } else {
                    self.register_error = None;
                    self.registration_success = true;
                    self.type_user = "student".to_string();
                    self.user_avatar_data = None;
                    self.user_email = email.to_string();
                    self.logged_in_user = full_name;
                    self.current_screen = Screen::Profile;
                    self.clear_fields();
                }
            }
            Message::GoToProfile => self.current_screen = Screen::Profile,
            Message::GoToSettings => self.current_screen = Screen::Settings,
            Message::GoToCourses => self.current_screen = Screen::Courses,
            Message::GoToUserList => self.current_screen = Screen::UserList,
            Message::GoToGroupList => self.current_screen = Screen::GroupList,
            Message::Logout => {
                self.clear_fields();
                self.user_avatar_data = None;
                self.user_email.clear();
                self.current_screen = Screen::Login;
            }
            Message::ThemeSelected(name) => {
                if let Some(theme) = theme_from_str(name) {
                    let _ = save_theme(&theme);
                    self.theme= theme;
                }
            }
            Message::ChooseDate => {
                self.show_picker = true;
            }
            Message::SubmitDate(date) => {
                self.date = date;
                self.show_picker = false;
            }
            Message::CancelDate => {
                self.show_picker = false;
            }
            Message::Er(_v) => {

            }
            Message::ChooseAvatar => {
                if self.user_email.trim().is_empty() {
                    self.error_message = "Вы не вошли в систему. Email неизвестен.".to_string();
                    return;
                }

                if let Some(path_buf) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg"]).pick_file() {
                    match fs::read(&path_buf) {
                        Ok(image_data) => {
                            let conn = Connection::open("db_platform").unwrap();
                            if let Err(err) = db::update_user_avatar(&conn, &self.user_email, &image_data) {
                                self.error_message = format!("Ошибка сохранения аватара: {}", err);
                            } else {
                                self.user_avatar_data = Some(image_data);
                                self.error_message.clear(); // Очищаем ошибку при успехе
                            }
                        }
                        Err(err) => {
                            self.error_message = format!("Ошибка чтения файла аватара: {}", err);
                        }
                    }
                }
            },
            Message::ToggleAddCourseModal(show) => {
                self.show_add_course_modal = show;
            },
            Message::NewCourseTitleChanged(title) => {
                self.new_course_title = title;
            },
            Message::NewCourseDescriptionChanged(desc) => {
                self.new_course_description = desc;
            },
            Message::SubmitNewCourse => {
                let conn = Connection::open("db_platform").unwrap();

                let level_str = Some(self.new_course_level.to_string());
                let instructor_str = self.new_course_instructor.clone();

                if db::add_course(
                    &conn,
                    &self.new_course_title,
                    &self.new_course_description,
                    instructor_str.as_deref(),
                    level_str.as_deref(),
                ).is_ok() {
                    self.show_add_course_modal = false;
                    self.new_course_title.clear();
                    self.new_course_description.clear();
                    self.new_course_instructor = None;
                    self.new_course_level = Level::Beginner;
                }
            }

            Message::DeleteCourse(course_id) => {
                let conn = Connection::open("db_platform").unwrap();
                db::delete_course(&conn, course_id).unwrap();
            },
            Message::NewCourseInstructorChanged(instructor) => {
                self.new_course_instructor = instructor;
            }
            Message::NewCourseLevelChanged(level) => {
                self.new_course_level = level;
            }
            // Обработка сообщений для редактирования курса
            Message::StartEditingCourse(course) => {
                self.edit_course_title = course.title.clone();
                self.edit_course_description = course.description.clone();
                self.edit_course_instructor = course.instructor.clone();
                // Преобразуем Option<String> уровня в Level
                self.edit_course_level = course.level.clone()
                    .and_then(|level_str| Level::from_str(&level_str).ok())
                    .unwrap_or(Level::Beginner); // Уровень по умолчанию, если не удалось распарсить
                self.editing_course = Some(course);
                // Открываем модальное окно (будет использоваться то же, но с другими данными)
                self.show_add_course_modal = true; // Переиспользуем флаг для отображения модалки
            },
            Message::EditCourseTitleChanged(title) => {
                self.edit_course_title = title;
            },
            Message::EditCourseDescriptionChanged(desc) => {
                self.edit_course_description = desc;
            },
            Message::EditCourseInstructorChanged(instructor) => {
                self.edit_course_instructor = instructor;
            }
            Message::EditCourseLevelChanged(level) => {
                self.edit_course_level = level;
            }
            Message::SubmitEditedCourse => {
                if let Some(original_course) = &self.editing_course {
                    let conn = Connection::open("db_platform").unwrap();

                    let updated_course = Course {
                        id: original_course.id,
                        title: self.edit_course_title.clone(),
                        description: self.edit_course_description.clone(),
                        instructor: self.edit_course_instructor.clone(),
                        level: Some(self.edit_course_level.to_string()),
                    };

                    if db::update_course(&conn, &updated_course).is_ok() {
                        // Закрываем модальное окно и сбрасываем состояние редактирования
                        self.show_add_course_modal = false;
                        self.editing_course = None;
                        self.edit_course_title.clear();
                        self.edit_course_description.clear();
                        self.edit_course_instructor = None;
                        self.edit_course_level = Level::Beginner;
                        // Возможно, здесь нужно обновить список курсов
                    } else {
                        // Обработка ошибки сохранения
                        self.error_message = "Ошибка сохранения курса.".to_string();
                    }
                }
            },
            Message::CancelEditingCourse => {
                self.show_add_course_modal = false; // Закрываем модальное окно
                self.editing_course = None; // Сбрасываем состояние редактирования
                // Сбрасываем поля редактирования на всякий случай
                self.edit_course_title.clear();
                self.edit_course_description.clear();
                self.edit_course_instructor = None;
                self.edit_course_level = Level::Beginner;
            },
            Message::StartEditingUser(user) => {
                self.editing_user = Some(user.clone());
                self.edit_user_name = user.name;
                self.edit_user_email = user.email;
                self.edit_user_birthday = user.birthday;
                self.edit_user_type = user.user_type;
                self.show_edit_user_modal = true;
            }

            Message::CancelEditingUser => {
                self.editing_user = None;
                self.show_edit_user_modal = false;
                self.edit_user_name.clear();
                self.edit_user_email.clear();
                self.edit_user_birthday.clear();
                self.edit_user_type.clear();
            }

            Message::EditUserNameChanged(value) => {
                self.edit_user_name = value;
            }

            Message::EditUserEmailChanged(value) => {
                self.edit_user_email = value;
            }

            Message::EditUserBirthdayChanged(value) => {
                self.edit_user_birthday = value;
            }

            Message::EditUserTypeChanged(value) => {
                self.edit_user_type = value;
            }

            Message::SubmitEditedUser => {
                if let Some(ref original_user) = self.editing_user {
                    let email = self.edit_user_email.trim();

                    // Пошаговая валидация
                    if email.is_empty() {
                        self.edit_user_error = Some("Email не может быть пустым.".to_string());
                        return;
                    }
                    if !email.contains('@') {
                        self.edit_user_error = Some("Email должен содержать символ '@'.".to_string());
                        return;
                    }

                    let parts: Vec<&str> = email.split('@').collect();
                    if parts.len() != 2 {
                        self.edit_user_error = Some("Email должен содержать только один символ '@'.".to_string());
                        return;
                    }

                    if parts[0].is_empty() {
                        self.edit_user_error = Some("Email должен содержать имя пользователя перед '@'.".to_string());
                        return;
                    }
                    if parts[1].is_empty() {
                        self.edit_user_error = Some("Email должен содержать домен после '@'.".to_string());
                        return;
                    }
                    if !parts[1].contains('.') {
                        self.edit_user_error = Some("Домен email должен содержать хотя бы одну точку (например: example.com).".to_string());
                        return;
                    }

                    // Дополнительная проверка через регулярку (общая структура)
                    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                    if !email_re.is_match(email) {
                        self.edit_user_error = Some("Email содержит недопустимые символы или некорректный формат.".to_string());
                        return;
                    }

                    let conn = Connection::open("db_platform").unwrap();

                    match db::is_email_taken_except(&conn, email, &original_user.email) {
                        Ok(true) => {
                            self.edit_user_error = Some("Email уже используется другим пользователем.".to_string());
                            return;
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
                            self.edit_user_error = Some(format!("Ошибка при проверке email: {}", err));
                        }
                    }
                }
            }

            Message::DeleteUser(email) => {
                let conn = Connection::open("db_platform").unwrap();
                let _ = db::delete_user(&conn, &email);
            }
            Message::CourseFilterChanged(text) => {
                self.course_filter_text = text;
            }
            Message::ToggleAddGroupModal(show) => {
                if !show {
                    self.new_group_name.clear();
                    self.new_group_course = None;
                    self.new_group_teacher = None;
                }
                self.show_add_group_modal = show;
            }
            Message::NewGroupNameChanged(name) => self.new_group_name = name,
            Message::NewGroupCourseChanged(course) => self.new_group_course = course,
            Message::NewGroupTeacherChanged(teacher) => self.new_group_teacher = teacher,

            Message::EditGroupNameChanged(name) => self.edit_group_name = name,
            Message::EditGroupCourseChanged(course) => self.edit_group_course = course,
            Message::EditGroupTeacherChanged(teacher) => self.edit_group_teacher = teacher,

            Message::StartEditingGroup(group) => {
                self.editing_group = Some(group.clone());
                self.edit_group_name = group.name.clone();
                self.edit_group_course = group.course.clone();
                self.edit_group_teacher = group.teacher.clone();
                self.show_add_group_modal = true;
            }
            Message::CancelEditingGroup => {
                self.editing_group = None;
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.show_add_group_modal = false;
            }
            Message::SubmitNewGroup => {
                let conn = Connection::open(".db_platform").unwrap();
                if let (Some(course), Some(teacher)) = (self.new_group_course.clone(), self.new_group_teacher.clone()) {
                    if let Err(err) = db::insert_group(&conn, &self.new_group_name, &course, &teacher) {
                        eprintln!("Ошибка добавления группы: {:?}", err);
                    }
                }
                self.new_group_name.clear();
                self.new_group_course = None;
                self.new_group_teacher = None;
                self.show_add_group_modal = false;
            }
            Message::SubmitEditedGroup => {
                if let Some(group) = self.editing_group.take() {
                    let conn = Connection::open("db_platform").unwrap();
                    if let (Some(course), Some(teacher)) = (self.edit_group_course.clone(), self.edit_group_teacher.clone()) {
                        if let Err(err) = db::update_group(&conn, group.id, &self.edit_group_name, &course, &teacher) {
                            eprintln!("Ошибка обновления группы: {:?}", err);
                        }
                    }
                }
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.show_add_group_modal = false;
            }
            Message::DeleteGroup(id) => {
                let conn = Connection::open("db_platform").unwrap();
                if let Err(err) = db::delete_group(&conn, id) {
                    eprintln!("Ошибка удаления группы: {:?}", err);
                }
            }
            Message::GroupFilterChanged(text) => self.group_filter_text = text,
            Message::OpenManageStudentsModal(group_id) => {
                let conn = Connection::open("db_platform").unwrap();
                let students = db::get_students_for_group(&conn, group_id);
                let all_students = db::get_all_student_names(&conn);

                self.selected_group_id = Some(group_id);
                self.is_manage_students_modal_open = true;
                self.group_students = students.expect("REASON");
                self.all_students = all_students.expect("REASON");
                self.selected_student_to_add = None;
            }
            Message::CloseManageStudentsModal => {
                self.is_manage_students_modal_open = false;
                self.selected_group_id = None;
                self.group_students.clear();
                self.selected_student_to_add = None;
            }

            Message::StudentToAddSelected(student_opt) => {
                self.selected_student_to_add = student_opt;
            }

            Message::AddStudent => {
                let conn = Connection::open("db_platform").unwrap();
                if let (Some(group_id), Some(student_name)) = (self.selected_group_id, &self.selected_student_to_add) {
                    db::add_student_to_group(&conn, group_id, student_name).unwrap();
                    self.group_students = db::get_students_for_group(&conn, group_id).expect("REASON");
                    self.selected_student_to_add = None;
                }
            }

            Message::RemoveStudent(student_name) => {
                let conn = Connection::open("db_platform").unwrap();
                if let Some(group_id) = self.selected_group_id {
                    db::remove_student_from_group(&conn, group_id, &student_name).unwrap();
                    self.group_students = db::get_students_for_group(&conn, group_id).expect("REASON");
                }
            }

            
        }
    }

    pub fn view(&self) -> Row<Message> {
        Row::new()
            .spacing(20)
            .push(
                // Левое меню (sidebar)
                if self.current_screen != Screen::Login && self.current_screen != Screen::Register {
                    Container::new(nav_menu(self))
                        .width(Length::Fixed(200.0)) // Фиксированная ширина меню
                        .height(Length::Fill)
                        .padding(10)
                } else {
                    Container::new(Column::new()) // Пустой контейнер, если экран входа
                        .width(Length::Fixed(0.0)) 
                        .height(Length::Fill)
                }
            )
            .push(
                // Основной контент
                match &self.current_screen {
                    Screen::Login => login_screen(self),
                    Screen::Register => register_screen(self),
                    Screen::Profile => profile_screen(self),
                    Screen::Settings => settings_screen(self),
                    Screen::Courses => courses_screen(self),
                    Screen::UserList => user_list_screen(self),
                    Screen::GroupList => groups_screen(self),
                }
                    .width(Length::Fill),
            )
            .into()
    }

    fn clear_fields(&mut self) {
        self.user_name.clear();
        self.user_surname.clear();
        self.user_patronymic.clear();
        self.user_email.clear();
        self.user_password.clear();
        self.user_password_repeat.clear();
        self.register_error = None;
        self.registration_success = false;
    }
}

pub fn theme_from_str(name: &str) -> Option<Theme> {
    Theme::ALL
        .iter()
        .find(|t| theme_to_str(t).eq_ignore_ascii_case(name))
        .cloned()
}

pub fn save_theme(theme: &Theme) -> std::io::Result<()> {
    let config = Config {
        theme_name: theme_to_str(theme).to_string(),
    };
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(CONFIG_FILE, json)?;
    Ok(())
}

pub fn load_theme() -> Option<Theme> {
    let contents = fs::read_to_string(CONFIG_FILE).ok()?;
    let config: Config = serde_json::from_str(&contents).ok()?;
    theme_from_str(&config.theme_name)
}
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    format!("{:x}", hasher.finalize())
}