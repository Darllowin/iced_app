use iced::{Length, Theme};
use iced::widget::{Column, Container, Row};
use crate::screens::{login_screen, register_screen, profile_screen, settings_screen, nav_menu, courses_screen};
use std::fs;
use std::str::FromStr;
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection};
use sha2::{Sha256, Digest};
use crate::db;
use crate::screens::settings::theme_to_str;

const CONFIG_FILE: &str = "config.json";

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

impl std::str::FromStr for Level {
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
    //AvatarSelected(Option<String>),
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
            //user_avatar_path: None,
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
                if self.user_password != self.user_password_repeat {
                    self.register_error = Some("Пароли не совпадают".to_string());
                    return;
                }

                let full_name = format!("{} {} {}", self.user_surname, self.user_name, self.user_patronymic);
                let password_hash = hash_password(&self.user_password);

                let conn = Connection::open("db_platform").unwrap();

                // Используем функцию для регистрации пользователя
                if let Err(_) = db::register_user(&conn,
                                                  &full_name,
                                                  format!("{:02}.{:02}.{}", &self.date.day, &self.date.month, &self.date.year).as_str(),
                                                  &self.user_email,
                                                  &password_hash) {
                    self.register_error = Some("Ошибка регистрации".to_string());
                } else {
                    self.register_error = None;
                    self.registration_success = true;
                    self.type_user = "student".to_string();
                    self.user_avatar_data = None;
                    self.logged_in_user = full_name;
                    self.current_screen = Screen::Profile;
                    self.clear_fields();
                }
            }
            Message::GoToProfile => self.current_screen = Screen::Profile,
            Message::GoToSettings => self.current_screen = Screen::Settings,
            Message::GoToCourses => self.current_screen = Screen::Courses,
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
            Message::Er(v) => {

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
                        .width(Length::Fixed(0.0)) // Меню скрыто
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