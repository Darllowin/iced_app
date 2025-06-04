use crate::config::{get_last_backup_time, load_config, start_backup_scheduler, theme_from_str};
use iced::Theme;
use iced::widget::text_editor;
use iced_anim::{Animated, spring};
use iced_aw::date_picker::Date;
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlError, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

pub const PATH_TO_DB: &str = "db_platform";
pub const CONFIG_FILE: &str = "config.json";
pub const DEFAULT_AVATAR: &str = "assets/images/default_avatar.jpg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInterval {
    pub display: &'static str,
    pub value: &'static str,
}
impl fmt::Display for BackupInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display)
    }
}

impl BackupInterval {
    pub fn duration(&self) -> Option<Duration> {
        match self.value {
            "daily" => Some(Duration::from_secs(60 * 60 * 24)),
            "weekly" => Some(Duration::from_secs(60 * 60 * 24 * 7)),
            "monthly" => Some(Duration::from_secs(60 * 60 * 24 * 30)),
            "never" => None,
            _ => None,
        }
    }
}
// Возможные интервалы (отображение → значение)
pub const BACKUP_INTERVALS: [BackupInterval; 4] = [
    BackupInterval {
        display: "Никогда",
        value: "never",
    },
    BackupInterval {
        display: "Каждый день",
        value: "daily",
    },
    BackupInterval {
        display: "Каждую неделю",
        value: "weekly",
    },
    BackupInterval {
        display: "Каждый месяц",
        value: "monthly",
    },
];

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
    pub user_password: String,
    pub user_password_repeat: String,
    //
    pub theme: Animated<Theme>,
    //
    pub register_error: Option<String>,
    pub registration_success: bool,
    pub logged_in_user: String,
    pub error_message: String,
    pub choose_avatar_message: String,
    //
    pub user_avatar_data: Option<Vec<u8>>,
    //
    pub show_add_course_modal: bool,
    pub new_course_title: String,
    pub new_course_description: String,
    pub new_course_total_seats: i32,
    pub new_course_seats: i32,
    pub new_course_price: f64,
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
    // Новые поля для строкового ввода чисел в модальном окне курса
    pub new_course_total_seats_str: String,
    pub new_course_seats_str: String,
    pub new_course_price_str: String,

    pub edit_course_total_seats_str: String,
    pub edit_course_seats_str: String,
    pub edit_course_price_str: String,
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
    pub new_group_status: GroupStatus,

    pub editing_group: Option<Group>,
    pub edit_group_name: String,
    pub edit_group_course: Option<i32>,
    pub edit_group_teacher: Option<i32>,
    pub edit_group_status: GroupStatus,

    pub group_filter_text: String,

    pub is_manage_students_modal_open: bool,
    pub selected_student_to_add: Option<UserInfo>,

    pub show_children_modal: bool,
    pub parent_children: Vec<UserInfo>,
    pub available_children: Vec<UserInfo>,
    pub selected_child_to_add: Option<UserInfo>,

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
    pub show_teacher_assignment_modal: bool,
    pub teacher_lesson_assignments: Vec<Assignment>, // Задания, связанные с текущим запланированным уроком
    pub editing_teacher_assignment: Option<Assignment>, // Редактируемое задание

    pub selected_group_lessons_with_assignments: Vec<LessonWithAssignments>,
    pub course_id_to_title: std::collections::HashMap<i32, String>,

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
    pub new_payment_type: String,        // Например, "enrollment" или "monthly"
    pub courses_with_seats: Vec<Course>,
    pub groups_for_selected_course: Vec<Group>,
    // Для picklist'ов может понадобиться отслеживать выбранный индекс
    pub selected_payment_type_idx: Option<usize>,
    //
    pub show_conduct_lesson_modal: bool, // Для управления видимостью модального окна
    pub students_for_attendance: Vec<StudentAttendance>, // Для хранения данных о студентах в модальном окне
    pub current_lesson_to_conduct: Option<LessonWithAssignments>, // Хранит урок, который проводится
    pub current_group_for_attendance: Option<Group>,     // Хранит группу для отметки посещаемости
    //
    pub students_with_certificates: Vec<UserInfo>,
    pub show_student_certificates_modal: bool, // Флаг для показа модалки сертификатов студента
    //
    pub selected_student_for_certificates: Option<UserInfo>,
    pub selected_student_certs: Vec<Certificate>, // Сертификаты выбранного студента
    pub is_loading_student_certs: bool,           // Флаг загрузки сертификатов студента
    //
    pub date_picker_open: DatePickerOpen,
    pub show_report_modal: bool,
    pub report_period_start: Date,
    pub report_period_end: Date,
    pub selected_report_type: Option<ReportType>,
    //
    pub show_certificate_report_modal: bool,
    pub show_group_report_modal: bool,
    //
    pub backup_interval: Option<BackupInterval>,
    pub backup_folder: Option<String>,
    pub max_backup_count: Option<usize>,
    pub last_backup_time: Option<String>,
}
impl Default for App {
    fn default() -> Self {
        let config = load_config();

        let selected_theme = config
            .as_ref()
            .and_then(|c| theme_from_str(&c.theme_name))
            .unwrap_or(Theme::GruvboxDark);

        let backup_interval = config.as_ref().and_then(|c| {
            let val = c.backup_interval.as_deref().unwrap_or("");
            BACKUP_INTERVALS
                .iter()
                .find(|interval| interval.value.eq_ignore_ascii_case(val))
                .cloned()
        });

        let backup_folder = config.as_ref().and_then(|c| c.backup_folder.clone());

        let max_backup_count = config.as_ref().and_then(|c| c.max_backup_count);

        let interval = config.as_ref().and_then(|c| {
            let val = c.backup_interval.as_deref().unwrap_or("never");
            BACKUP_INTERVALS.iter().find(|i| i.value == val).cloned()
        });

        let last_backup_time = get_last_backup_time("backup");

        start_backup_scheduler(
            interval,
            config.clone().unwrap().backup_folder.clone(),
            config.unwrap().max_backup_count,
        );
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
            theme: Animated::new(selected_theme, spring::Motion::SMOOTH),
            backup_interval,
            backup_folder,
            max_backup_count,
            last_backup_time,
            register_error: None,
            registration_success: false,
            logged_in_user: "".to_string(),
            show_add_course_modal: false,
            new_course_title: "".to_string(),
            user_birthday: "".to_string(),
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
            new_course_total_seats_str: "".to_string(),
            new_course_seats_str: "".to_string(),
            new_course_price_str: "".to_string(),
            edit_course_total_seats_str: "".to_string(),
            edit_course_seats_str: "".to_string(),
            edit_course_price_str: "".to_string(),
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
            new_group_status: GroupStatus::Active,
            editing_group: None,
            edit_group_name: "".to_string(),
            edit_group_course: None,
            edit_group_teacher: None,
            edit_group_status: GroupStatus::Active,
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
            teacher_lesson_assignments: vec![],
            editing_teacher_assignment: None,
            editing_assignment_description_text_input: "".to_string(),
            selected_group_lessons_with_assignments: vec![],
            course_id_to_title: Default::default(),
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
            show_conduct_lesson_modal: false,
            students_for_attendance: vec![],
            current_lesson_to_conduct: None,
            new_payment_group: None,
            current_group_for_attendance: None,
            students_with_certificates: vec![],
            show_student_certificates_modal: false,
            selected_student_for_certificates: None,
            selected_student_certs: vec![],
            is_loading_student_certs: false,
            date_picker_open: DatePickerOpen::None,
            show_report_modal: false,
            report_period_start: Default::default(),
            report_period_end: Default::default(),
            selected_report_type: None,
            show_certificate_report_modal: false,
            show_group_report_modal: false,
            choose_avatar_message: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    PDF,
    Excel,
}

impl fmt::Display for ReportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportType::PDF => write!(f, "PDF"),
            ReportType::Excel => write!(f, "Excel"),
        }
    }
}

pub enum DatePickerOpen {
    None,
    Start,
    End,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Copy позволит избежать лишних клонирований
pub enum GroupStatus {
    Active,
    Inactive,
}

// Реализация Display для GroupStatus, чтобы PickList мог его отображать
impl fmt::Display for GroupStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupStatus::Active => write!(f, "Активна"),
            GroupStatus::Inactive => write!(f, "Неактивна"),
        }
    }
}
// Реализация FromSql для GroupStatus
impl FromSql for GroupStatus {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        // Получаем значение из БД как строку
        let s = value.as_str()?;
        // Пытаемся сопоставить строку с вариантами нашего enum
        match s {
            "Активна" => Ok(GroupStatus::Active),
            "Неактивна" => Ok(GroupStatus::Inactive),
            _ => Err(FromSqlError::Other(
                format!("Неизвестный статус группы: {}", s).into(),
            )),
        }
    }
}
// Реализация ToSql для GroupStatus
impl ToSql for GroupStatus {
    fn to_sql(&self) -> Result<ToSqlOutput, rusqlite::Error> {
        // Преобразуем наш enum в строку для записи в БД
        Ok(self.to_string().into())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Certificate {
    pub id: i32,
    pub student_id: i32,
    pub student_name: String, // Для отображения имени студента
    pub course_id: i32,
    pub course_title: String, // Для отображения названия курса
    pub issue_date: String,
    pub grade: String,
}
#[derive(Debug, Clone)]
pub struct StudentAttendanceStatus {
    pub student_id: i32,
    pub student_name: String,   // Имя студента
    pub present_status: String, // "Present" или "Absent"
}

#[derive(Debug, Clone)]
pub struct StudentAttendance {
    pub id: i32,
    pub name: String,
    pub present: bool, // true, если присутствует; false, если отсутствует
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
    pub group_id: Option<String>, // Это поле может быть, но оно всегда будет None из Users
    pub child_count: Option<i32>, // Это поле может быть, но оно всегда будет None из Users
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

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub course_id: Option<i32>,       // Сохраняем ID курса
    pub course_name: Option<String>,  // Новое поле для названия курса
    pub teacher_id: Option<i32>,      // Сохраняем ID преподавателя
    pub teacher_name: Option<String>, // Новое поле для имени преподавателя
    pub student_count: u8,
    pub status: GroupStatus,
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

#[derive(Clone, Debug, PartialEq)]
pub struct GroupForReport {
    pub id: i32,
    pub name: String,
    pub course_id: Option<i32>,       // Сохраняем ID курса
    pub course_name: Option<String>,  // Новое поле для названия курса
    pub teacher_id: Option<i32>,      // Сохраняем ID преподавателя
    pub teacher_name: Option<String>, // Новое поле для имени преподавателя
    pub student_count: u8,
    pub status: GroupStatus,
    pub students: Vec<String>,
}

impl fmt::Display for GroupForReport {
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
    Practice,
    // Добавьте другие типы по необходимости
}

impl AssignmentType {
    pub const ALL: &'static [AssignmentType] = &[AssignmentType::Lecture, AssignmentType::Practice];
}

// Реализация Display для отображения в PickList и преобразования в строку
impl fmt::Display for AssignmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssignmentType::Lecture => "Лекция",
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
    pub id: i32, // ID урока из таблицы Lessons
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
    pub attendance_records: Vec<StudentAttendanceStatus>,
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

#[derive(Debug, Clone, PartialEq)]
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
    pub const ALL: &'static [Level] = &[Level::Beginner, Level::Intermediate, Level::Advanced];
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::Beginner => "Начальный",
                Level::Intermediate => "Средний",
                Level::Advanced => "Продвинутый",
            }
        )
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

impl fmt::Display for StudentPickListItem {
    // Используем импортированный fmt
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme_name: String,
    pub backup_interval: Option<String>,
    pub backup_folder: Option<String>,
    pub max_backup_count: Option<usize>,
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
    Payment,
    Certificates,
}
