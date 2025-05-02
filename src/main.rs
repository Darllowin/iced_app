use iced::{Alignment, Center, Length, Size, Theme};
use iced::widget::{Column, button, column, text, text_input, vertical_space, Text, Container};
use iced::widget::button::Style;

fn main() -> iced::Result {
    iced::application("Counter", App::update, App::view)
        .theme(|_| Theme::GruvboxDark)
        .window_size(Size::new(500.0, 700.0))
        .run()
}

#[derive(Default)]
struct App {
    value: i64,
    current_screen: Screen,
    user_name: String,
    user_surname: String,
    user_patronymic: String,
    user_email: String,
    user_phone: String,
    user_password: String,
    user_password_repeat: String,
}
#[derive(PartialEq, Default)]
enum Screen {
    #[default]
    Login,
    Register,
}

#[derive(Debug, Clone)]
enum Message {
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
}

impl App {
    fn new() -> Self {
        App {
            value: 0,
            current_screen: Screen::Login,
            user_password: String::from("1234"),
            user_password_repeat: String::new(),
            user_name: String::new(),
            user_surname: String::new(),
            user_patronymic: String::new(),
            user_email: String::new(),
            user_phone: String::new(),
        }
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::LoginPressed => {
                if self.user_email == "root" && self.user_password == "1234" {
                    self.value = 1;
                }
            },
            Message::SwitchToLogin => {
                self.current_screen = Screen::Login;
                self.clear_fields();
            },
            Message::SwitchToRegister => {
                self.current_screen = Screen::Register;
                self.clear_fields();
            },
            Message::FirstNameChanged(first_name) => {
                self.user_name = first_name
            },
            Message::LastNameChanged(last_name) => {
                self.user_surname = last_name
            },
            Message::MiddleNameChanged(middle_name) => {
                self.user_patronymic = middle_name
            },
            Message::PhoneChanged(phone) => {
                self.user_phone = phone
            },
            Message::EmailChanged(email) => {
                self.user_email = email
            },
            Message::PasswordChanged(password) => {
                self.user_password = password
            },
            Message::PasswordRepeatChanged(password_repeat) => {
                self.user_password_repeat = password_repeat
            },
            Message::RegisterPressed => {
                
            }
        }
    }

    fn view(&self) -> Container<Message> {
        match self.current_screen {
            Screen::Login => login_screen(self),
            Screen::Register => register_screen(self),
        }
    }

    fn clear_fields(&mut self) {
        self.user_name.clear();
        self.user_surname.clear();
        self.user_patronymic.clear();
        self.user_email.clear();
        self.user_phone.clear();
        self.user_password.clear();
        self.user_password_repeat.clear();
    }
}

// Вынесенная функция отрисовки
fn login_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Вход в систему")
            .size(30),
        vertical_space(),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(20)
            .width(Length::Fill),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(20)
            .width(Length::Fill),
        vertical_space(),
        button("Войти")
            .on_press(Message::LoginPressed)
            .padding(10)
            .width(Length::Fill),
        vertical_space(),
        text(format!("Статус: {}", if app.value == 1 { "Успешно" } else { "Ожидание" }))
            .size(16),
        button("Регистрация")
            .on_press(Message::SwitchToRegister)
            .padding(10)
            .width(Length::Fill),
    ]
        .spacing(15)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)

}

fn register_screen(app: &App) -> Container<Message> {
    let content = column![
        text("Регистрация").size(30),
        vertical_space(),
        text_input("Имя", &app.user_name)
            .on_input(Message::FirstNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        text_input("Фамилия", &app.user_surname)
            .on_input(Message::LastNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        text_input("Отчество", &app.user_patronymic)
            .on_input(Message::MiddleNameChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        vertical_space(),
        text_input("Телефон", &app.user_phone)
            .on_input(Message::PhoneChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        text_input("Почта", &app.user_email)
            .on_input(Message::EmailChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        vertical_space(),
        text_input("Пароль", &app.user_password)
            .on_input(Message::PasswordChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        text_input("Повторите пароль", &app.user_password_repeat)
            .on_input(Message::PasswordRepeatChanged)
            .padding(10)
            .size(18)
            .width(Length::Fill),
        vertical_space(),
        button("Зарегистрироваться")
            .on_press(Message::RegisterPressed)
            .padding(10)
            .width(Length::Fill),
        button("Назад ко входу")
            .on_press(Message::SwitchToLogin)
            .padding(10)
            .width(Length::Fill),
    ]
        .spacing(15)
        .align_x(Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}
