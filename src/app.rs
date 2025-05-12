

use iced::{Length, Theme};
use iced::widget::{Column, Container, Row};
use crate::screens::{login_screen, register_screen, profile_screen, settings_screen, nav_menu};
use std::fs;
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection};
use sha2::{Sha256, Digest};
use crate::db;
use crate::screens::settings::theme_to_str;

const CONFIG_FILE: &str = "config.json";
//#[derive(Default)]
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
    pub user_phone: String,
    pub user_password: String,
    pub user_password_repeat: String,
    //
    pub theme: Theme,
    //
    pub register_error: Option<String>,
    pub registration_success: bool,
    pub logged_in_user: String,
    pub user_birthday: String,
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
}

#[derive(Debug, Clone)]
pub enum Message {
    LoginPressed,
    FirstNameChanged(String),
    LastNameChanged(String),
    MiddleNameChanged(String),
    PhoneChanged(String),
    EmailChanged(String),
    PasswordChanged(String),
    PasswordRepeatChanged(String),
    RegisterPressed,
    SwitchToLogin,
    SwitchToRegister,
    GoToProfile,
    GoToSettings,
    Logout,
    ThemeSelected(&'static str),
    ChooseDate,
    SubmitDate(Date),
    CancelDate,
    Er(String),
}
impl Default for App {
    fn default() -> Self {
        let selected_theme = load_theme().unwrap_or(Theme::Light);
        Self {
            user_birthday: "".to_string(),
            date: Date::today(),
            show_picker: false,
            current_screen: Default::default(),
            user_name: "".to_string(),
            user_surname: "".to_string(),
            user_patronymic: "".to_string(),
            user_email: "".to_string(),
            user_phone: "".to_string(),
            user_password: "".to_string(),
            user_password_repeat: "".to_string(),
            theme: selected_theme,
            register_error: None,
            registration_success: false,
            logged_in_user: "".to_string(),
        }
    }
}
impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LoginPressed => {
                let conn = Connection::open("db_platform").unwrap();
                let entered_hash = hash_password(&self.user_password);

                if let Some(name) = db::check_user_credentials(&conn, &self.user_email, &entered_hash) {
                    self.logged_in_user = name;
                    self.current_screen = Screen::Profile;
                    self.clear_fields();
                } else {
                    // Ошибка: неверные учетные данные
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
            Message::PhoneChanged(v) => self.user_phone = v,
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
                    self.logged_in_user = full_name;
                    self.current_screen = Screen::Profile;
                    self.clear_fields();
                }
            }
            Message::GoToProfile => self.current_screen = Screen::Profile,
            Message::GoToSettings => self.current_screen = Screen::Settings,
            Message::Logout => {
                self.clear_fields();
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
        }
    }

    pub fn view(&self) -> Row<Message> {
        Row::new()
            .spacing(20)
            .push(
                // Левое меню (sidebar)
                if self.current_screen != Screen::Login && self.current_screen != Screen::Register {
                    Container::new(nav_menu())
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
                }
                    .width(Length::Fill), // Занимает всё оставшееся пространство
            )
            .into()
    }

    fn clear_fields(&mut self) {
        self.user_name.clear();
        self.user_surname.clear();
        self.user_patronymic.clear();
        self.user_email.clear();
        self.user_phone.clear();
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