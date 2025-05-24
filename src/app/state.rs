use std::fmt;
use std::str::FromStr;
use iced::Theme;
use iced::widget::text_editor;
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};
use crate::app::update::load_theme;

pub const PATH_TO_DB: &str = "db_platform";
pub const CONFIG_FILE: &str = "config.json";
pub const DEFAULT_AVATAR: &str = "default_avatar.jpg";

pub struct App {
    pub date: Date,
    pub show_picker: bool,
    //
    pub current_screen: Screen,
    pub current_user: Option<UserInfo>,
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
    pub new_course_total_seats: i32,
    pub new_course_seats: i32,
    pub new_course_price: f64,
    //pub new_course_instructor: Option<String>,
    pub new_course_level: Level,
    // Добавлены поля для редактирования курса
    pub editing_course: Option<Course>,
    pub edit_course_title: String,
    pub edit_course_description: String,
    pub edit_course_instructor: Option<String>,
    pub edit_course_level: Level,
    pub edit_course_total_seats: i32,
    pub edit_course_seats: i32,
    pub edit_course_price: f64,
    //
    pub show_lessons_modal: bool,
    pub editing_lessons_course: Option<Course>,
    pub course_lessons: Vec<LessonWithAssignments>,
    pub new_lesson_number_text: String,
    pub new_lesson_title: String,
    pub lesson_error_message: Option<String>,
    //
    pub editing_user: Option<UserInfo>,
    pub edit_user_error: Option<String>,
    pub show_edit_user_modal: bool,
    pub edit_user_name: String,
    pub edit_user_email: String,
    pub edit_user_birthday: String,
    pub edit_user_type: String,
    pub user_type_filter: Option<String>,
    //
    pub course_filter_text: String,
    // Группы
    pub show_add_group_modal: bool,
    pub new_group_name: String,
    pub new_group_course: Option<i32>,
    pub new_group_teacher: Option<i32>,

    pub editing_group: Option<Group>,
    pub edit_group_name: String,
    pub edit_group_course: Option<i32>,
    pub edit_group_teacher: Option<i32>,

    pub group_filter_text: String,

    //pub selected_group_id: Option<i32>,
    pub is_manage_students_modal_open: bool,
    //pub group_students: Vec<UserInfo>,
    //pub all_students: Vec<String>,
    pub selected_student_to_add: Option<UserInfo>,
    //pub user_group_name: Option<String>,
    //pub logged_in_user_id: Option<i32>,

    pub show_children_modal: bool,
    pub parent_children: Vec<UserInfo>,
    pub available_children: Vec<UserInfo>,
    pub selected_child_to_add: Option<UserInfo>,

    //pub course_modal_view: CourseModalView,
    pub course_error_message: Option<String>, // Ошибки, специфичные для модалки курса

    pub show_assignments_modal: bool,
    pub current_lesson_for_assignments: Option<LessonWithAssignments>,
    pub lesson_assignments: Vec<Assignment>, // Если будете загружать задания
    pub assignment_error_message: Option<String>,
    // --- Поля для формы нового задания ---
    pub new_assignment_title: String,
    pub new_assignment_description: String,
    pub new_assignment_type: Option<AssignmentType>, // Для текстового ввода типа

    pub show_assignment_detail_modal: bool,
    pub selected_assignment_for_detail: Option<Assignment>,
    pub editing_assignment_title: String,
    pub editing_assignment_description_content: text_editor::Content, // Для TextEditor (лекция, практика)

    pub editing_assignment_description_text_input: String,
    pub assignment_edit_error_message: Option<String>,

    // --- Состояние экрана занятий ---
    pub teacher_groups: Vec<Group>,
    pub selected_group_for_classes: Option<Group>,

    // Состояние модального окна заданий преподавателя
    pub selected_proven_lesson_for_assignments: Option<ProvenLesson>,
    pub show_teacher_assignment_modal: bool,
    pub teacher_lesson_assignments: Vec<Assignment>, // Задания, связанные с текущим запланированным уроком
    pub editing_teacher_assignment: Option<Assignment>, // Редактируемое задание
    pub editing_teacher_assignment_title: String,
    pub editing_teacher_assignment_description_text_input: String, // Для TextInput
    pub editing_teacher_assignment_description_content: text_editor::Content, // Для TextEditor
    pub teacher_assignment_edit_error_message: Option<String>,
    pub available_assignments: Vec<Assignment>, // Список всех заданий для выбора
    pub selected_assignment_to_add_to_lesson: Option<Assignment>,

    pub selected_group_lessons_with_assignments: Vec<LessonWithAssignments>,
    pub course_id_to_title: std::collections::HashMap<i32, String>,
    
    pub edit_course_teacher_id: Option<i32>,
    pub all_courses: Vec<Course>,
    pub past_sessions_for_group: Vec<PastSession>, // Для отображения списка прошедших занятий

    pub show_group_lessons_modal: bool,
    pub group_lessons_modal_lessons: Vec<LessonWithAssignments>, // Список уроков для отображения в модальном окне
    pub group_lessons_modal_past_sessions: Vec<PastSession>, // Список пройденных занятий для отображения
    pub group_lessons_modal_group_name: String,
    pub current_manage_students_group_id: Option<i32>,
    pub students_without_group: Vec<UserInfo>,

    pub courses_for_picklist: Vec<Course>, // Для PickList курсов в модалке групп
    pub users_for_picklist: Vec<UserInfo>,
    pub group_error_message: Option<String>,

    pub student_group_info: Option<Group>,
    pub show_group_students_modal: bool,
    pub selected_group_for_students_name: Option<String>, // Для отображения названия группы в модалке
    pub selected_group_students: Vec<UserInfo>,
    pub is_loading_group_students: bool,
    pub all_groups: Vec<Group>,
    // Payment
    pub payments: Vec<Payment>,
    pub show_add_payment_modal: bool,
    pub new_payment_student: Option<StudentPickListItem>,
    pub new_payment_course: Option<CoursePickListItem>,
    pub new_payment_group: Option<GroupPickListItem>,
    pub new_payment_amount: Option<f64>, // Будет заполняться автоматически
    pub new_payment_type: String, // Например, "enrollment" или "monthly"
    pub courses_with_seats: Vec<Course>,
    pub groups_for_selected_course: Vec<Group>,
    // Для picklist'ов может понадобиться отслеживать выбранный индекс
    pub selected_payment_type_idx: Option<usize>,
    
}

impl Default for App {
    fn default() -> Self {
        let selected_theme = load_theme().unwrap_or(Theme::Light);
        Self {
            error_message: "".to_string(),
            date: Date::today(),
            show_picker: false,
            current_screen: Default::default(),
            current_user: None,
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
            new_course_total_seats: 0,
            new_course_seats: 0,
            new_course_price: 0f64,
            new_course_level: Level::Beginner,
            editing_course: None,
            edit_course_title: "".to_string(),
            edit_course_description: "".to_string(),
            edit_course_instructor: None,
            user_avatar_data: None,
            edit_course_level: Level::Beginner,
            edit_course_total_seats: 0,
            edit_course_seats: 0,
            edit_course_price: 0f64,
            show_lessons_modal: false,
            editing_lessons_course: None,
            course_lessons: vec![],
            new_lesson_number_text: "".to_string(),
            new_lesson_title: "".to_string(),
            lesson_error_message: None,
            editing_user: None,
            edit_user_error: None,
            show_edit_user_modal: false,
            edit_user_name: "".to_string(),
            edit_user_email: "".to_string(),
            edit_user_birthday: "".to_string(),
            edit_user_type: "".to_string(),
            user_type_filter: None,
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
            is_manage_students_modal_open: false,
            selected_student_to_add: None,
            show_children_modal: false,
            parent_children: vec![],
            available_children: vec![],
            selected_child_to_add: None,
            course_error_message: None,
            show_assignments_modal: false,
            current_lesson_for_assignments: None,
            lesson_assignments: vec![],
            assignment_error_message: None,
            new_assignment_title: "".to_string(),
            new_assignment_description: "".to_string(),
            show_assignment_detail_modal: false,
            selected_assignment_for_detail: None,
            editing_assignment_title: "".to_string(),
            editing_assignment_description_content: Default::default(),
            new_assignment_type: None,
            assignment_edit_error_message: None,
            teacher_groups: vec![],
            selected_group_for_classes: None,
            //proven_lessons: vec![],
            show_teacher_assignment_modal: false,
            selected_proven_lesson_for_assignments: None,
            teacher_lesson_assignments: vec![],
            editing_teacher_assignment: None,
            editing_teacher_assignment_title: "".to_string(),
            editing_teacher_assignment_description_text_input: "".to_string(),
            editing_teacher_assignment_description_content: Default::default(),
            teacher_assignment_edit_error_message: None,
            available_assignments: vec![],
            editing_assignment_description_text_input: "".to_string(),
            selected_assignment_to_add_to_lesson: None,
            selected_group_lessons_with_assignments: vec![],
            course_id_to_title: Default::default(),
            edit_course_teacher_id: None,
            all_courses: vec![],
            past_sessions_for_group: vec![],
            show_group_lessons_modal: false,
            group_lessons_modal_lessons: vec![],
            group_lessons_modal_past_sessions: vec![],
            group_lessons_modal_group_name: "".to_string(),
            current_manage_students_group_id: None,
            students_without_group: vec![],
            courses_for_picklist: vec![],
            users_for_picklist: vec![],
            group_error_message: None,
            student_group_info: None,
            show_group_students_modal: false,
            selected_group_for_students_name: None,
            selected_group_students: vec![],
            is_loading_group_students: false,
            all_groups: vec![],
            payments: vec![],
            show_add_payment_modal: false,
            new_payment_student: None,
            new_payment_course: None,
            new_payment_amount: None,
            new_payment_type: "".to_string(),
            courses_with_seats: vec![],
            groups_for_selected_course: vec![],
            selected_payment_type_idx: None,
            new_payment_group: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Payment {
    pub id: i32,
    pub student_id: i32,
    pub date: String, 
    pub amount: f64,
    pub payment_type: String, 
    pub course_id: i32,
    pub group_id: i32,
    pub student_name: String,
    pub course_title: String,
    pub group_name: String,
}


#[derive(Debug, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub birthday: String,
    pub user_type: String, // Теперь соответствует полю "Type" в БД
    pub avatar_data: Option<Vec<u8>>,
    pub group_id: Option<String>,      // Это поле может быть, но оно всегда будет None из Users
    pub child_count: Option<i32>,   // Это поле может быть, но оно всегда будет None из Users
}


// Для PickList:
// Чтобы PickList мог отобразить пользователя
impl fmt::Display for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// Чтобы PickList мог сравнивать выбранный элемент
// Обновите PartialEq для UserInfo, чтобы сравнивать по ID
impl PartialEq for UserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // <--- Сравниваем по ID
    }
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub course_id: Option<i32>,       // Сохраняем ID курса
    pub course_name: Option<String>,  // Новое поле для названия курса
    pub teacher_id: Option<i32>,      // Сохраняем ID преподавателя
    pub teacher_name: Option<String>, // Новое поле для имени преподавателя
    pub student_count: u8,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Начинаем с имени группы
        write!(f, "{}", self.name)?;

        let mut parts = Vec::new();

        // Добавляем название курса, если оно есть
        if let Some(course_name) = &self.course_name {
            parts.push(format!("Курс: {}", course_name));
        }

        // Добавляем название преподавателя, если оно есть
        if let Some(teacher_name) = &self.teacher_name {
            parts.push(format!("Преподаватель: {}", teacher_name));
        }

        // Если есть дополнительные части (курс или преподаватель), добавляем их в скобках
        if !parts.is_empty() {
            write!(f, " ({})", parts.join(", "))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub id: i32,
    pub lesson_id: i32,
    pub title: String,
    pub description: String,
    pub assignment_type: String, // В таблице колонка "type"
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Отображаем название задания и его тип
        write!(f, "{} ({})", self.title, self.assignment_type)
    }
}

#[derive(Debug, Clone)]
pub enum TextInputOrEditorInput {
    TextEditor(text_editor::Action), // Действие из TextEditor
    TextInput(String),               // Строка из TextInput
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] // Добавил Default
pub enum AssignmentType {
    #[default] // Задаем значение по умолчанию, если потребуется
    Lecture,
    Test,
    Practice,
    // Добавьте другие типы по необходимости
}

impl AssignmentType {
    pub const ALL: &'static [AssignmentType] = &[
        AssignmentType::Lecture,
        AssignmentType::Test,
        AssignmentType::Practice,
    ];
}

// Реализация Display для отображения в PickList и преобразования в строку
impl fmt::Display for AssignmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssignmentType::Lecture => "Лекция",
                AssignmentType::Test => "Тест",
                AssignmentType::Practice => "Практика",
            }
        )
    }
}

impl PartialEq for Lesson {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // Сравниваем по ID
    }
}

#[derive(Debug, Clone)]
pub struct LessonWithAssignments {
    pub id: i32,          // ID урока из таблицы Lessons
    pub course_id: i32,
    pub number: i32,
    pub title: String,
    pub assignments: Vec<Assignment>, // Список заданий для этого урока
}

#[derive(Debug, Clone)]
pub struct PastSession {
    pub id: i32,
    pub group_id: i32,
    pub date: String,
    pub lesson_id: i32,
    pub lesson_number: Option<i32>,
    pub lesson_title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProvenLesson {
    pub id: i32,
    pub group_id: i32,
    pub lesson_id: i32, // Новое поле для связи с Lesson
    pub date: String,
    pub topic: String,
    // Поля из связанного Lesson для отображения
    pub lesson_number: i32,
    pub lesson_title: String,
    pub assignments: Vec<Assignment>,
}


#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct Course {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub level: Option<String>,
    pub total_seats: Option<i32>,
    pub seats: Option<i32>, 
    pub price: Option<f64>,
    pub lesson_count: i32,
}

impl fmt::Display for Course {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title) // Что будет отображаться в PickList
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}
impl Default for Level {
    fn default() -> Self {
        Level::Beginner
    }
}
impl Level {
    pub const ALL: &'static [Level] = &[
        Level::Beginner,
        Level::Intermediate,
        Level::Advanced,
    ];
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

// --- СТРУКТУРЫ ДЛЯ PICKLIST ЭЛЕМЕНТОВ ---
#[derive(Debug, Clone, PartialEq)]
pub struct StudentPickListItem {
    pub id: i32,
    pub name: String,
}

impl fmt::Display for StudentPickListItem { // Используем импортированный fmt
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoursePickListItem {
    pub id: i32,
    pub title: String,
    pub price_display: String, // String representation of price
}

impl fmt::Display for CoursePickListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.price_display)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupPickListItem {
    pub id: i32,
    pub name: String,
}

impl fmt::Display for GroupPickListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme_name: String,
}
#[derive(PartialEq, Default)]
pub enum Screen {
    #[default]
    Login,
    Register,
    Profile,
    Settings,
    CoursesList,
    UserList,
    GroupList,
    Classes,
    Payment
}