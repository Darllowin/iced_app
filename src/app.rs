use iced::{Length, Task, Theme};
use iced::widget::{text_editor, Column, Container, Row};
use crate::screens::{login_screen, register_screen, profile_screen, settings_screen, nav_menu, courses_screen, user_list_screen, groups_screen, classes_screen};
use std::{fmt, fs};
use std::str::FromStr;
use chrono::Local;
use iced_aw::date_picker::Date;
use regex::Regex;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection};
use sha2::{Sha256, Digest};
use tokio::task;
use crate::db;
use crate::screens::settings::theme_to_str;

const CONFIG_FILE: &str = "config.json";
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
    pub new_course_instructor: Option<String>,
    pub new_course_level: Level,
    // Добавлены поля для редактирования курса
    pub editing_course: Option<Course>,
    pub edit_course_title: String,
    pub edit_course_description: String,
    pub edit_course_instructor: Option<String>,
    pub edit_course_level: Level,
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

    pub selected_group_id: Option<i32>,
    pub is_manage_students_modal_open: bool,
    pub group_students: Vec<UserInfo>,
    pub all_students: Vec<String>,
    pub selected_student_to_add: Option<String>,
    //pub user_group_name: Option<String>,
    //pub logged_in_user_id: Option<i32>,

    pub show_children_modal: bool,
    pub parent_children: Vec<UserInfo>,
    pub available_children: Vec<UserInfo>,
    pub selected_child_to_add: Option<UserInfo>,

    pub show_course_details_modal: bool,
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
    pub group_proven_lessons: Vec<ProvenLesson>,

    // Состояние модального окна заданий преподавателя
    //pub proven_lessons: Vec<ProvenLesson>,
    pub teacher_error_message: Option<String>,
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
    pub group_past_sessions: Vec<PastSession>,
    pub course_id_to_title: std::collections::HashMap<i32, String>,
    pub user_id_to_name: std::collections::HashMap<i32, String>,
    pub groups: Vec<Group>,
    pub editing_assignment_lesson: Option<LessonWithAssignments>,

    pub new_course_teacher: Option<UserInfo>,    // Уже есть, но теперь хранит UserInfo
    pub edit_course_teacher: Option<UserInfo>,   // Уже есть, но теперь хранит UserInfo
    pub new_course_teacher_id: Option<i32>,     // <--- ДОБАВЬТЕ ЭТО
    pub edit_course_teacher_id: Option<i32>,
    pub all_users: Vec<UserInfo>,
    pub all_groups: Vec<Group>,
    pub all_courses: Vec<Course>,
    pub past_sessions_for_group: Vec<PastSession>, // Для отображения списка прошедших занятий
}

#[derive(Debug, Clone, Eq)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub birthday: String,
    pub user_type: String, // Теперь соответствует полю "Type" в БД
    pub avatar_data: Option<Vec<u8>>,
    pub group: Option<String>,      // Это поле может быть, но оно всегда будет None из Users
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
        // Отображаем имя группы, опционально с курсом и преподавателем
        write!(f, "{} (", self.name)?;
        let mut parts = Vec::new();
        if let Some(course) = &self.course_id {
            parts.push(format!("Курс: {}", course));
        }
        if let Some(teacher) = &self.teacher_id {
            parts.push(format!("Преподаватель: {}", teacher));
        }
        write!(f, "{})", parts.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: i32,
    pub course_id: i32,
    pub number: Option<i32>, // Номер может быть опциональным
    pub title: String,
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
    pub number: Option<i32>,
    pub title: String,
    pub assignments: Vec<Assignment>, // Список заданий для этого урока
}

#[derive(Debug, Clone)]
pub struct PastSession {
    pub id: i32,
    pub group_id: i32,
    pub date: String,
    pub lesson_id: i32,
    // Можете добавить сюда информацию об уроке, если нужно:
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
    pub description: String,
    pub instructor_id: Option<i32>,   // <--- Новое поле для ID преподавателя
    pub instructor_name: Option<String>,
    pub level: Option<String>,
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
    CoursesList,
    UserList,
    GroupList,
    Classes,
}
#[derive(Debug, Clone)]
pub enum Message {
    LoginPressed,
    UserLoggedIn(Result<UserInfo, String>),
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
    GoToClasses,
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
    NewCourseInstructorChanged(Option<UserInfo>),
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
    EditCourseInstructorChanged(Option<UserInfo>),
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
    UserTypeFilterChanged(Option<String>),
    //
    CourseFilterChanged(String),
    // Для групп
    ToggleAddGroupModal(bool),
    NewGroupNameChanged(String),
    NewGroupCourseChanged(Option<i32>),
    NewGroupTeacherChanged(Option<i32>),

    EditGroupNameChanged(String),
    EditGroupCourseChanged(Option<i32>),
    EditGroupTeacherChanged(Option<i32>),

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

    ShowParentChildren(String), // email родителя
    CloseParentChildrenModal,
    DeleteChild { parent_email: String, child_email: String },
    AddChildToParent,
    SelectedChildToAddChanged(UserInfo),
    ShowLessonsModal(Course), 
    CloseLessonsModal,      
    NewLessonNumberChanged(String), 
    NewLessonTitleChanged(String),  
    AddLesson,                
    DeleteLesson(i32),

    ShowAssignmentsModal(LessonWithAssignments),
    CloseAssignmentsModal,
    // --- Сообщения для управления заданиями ---
    NewAssignmentTitleChanged(String),
    NewAssignmentDescriptionChanged(String),
    NewAssignmentTypeSelected(AssignmentType), // Для текстового ввода типа
    AddAssignment,
    DeleteAssignment(i32), // передаем ID задания
    
    ShowAssignmentDetailModal(Assignment),
    CloseAssignmentDetailModal,

    EditingAssignmentTitleChanged(String),
    EditingAssignmentDescriptionChanged(TextInputOrEditorInput),
    SaveEditedAssignment, // Для сохранения изменений
    // --- Сообщения, связанные с экраном занятий ---
    LoadTeacherGroups,
    TeacherGroupsLoaded(Result<Vec<Group>, String>), // Result для обработки ошибок
    SelectGroupForClasses(Group),
    LoadGroupProvenLessons,
    GroupProvenLessonsLoaded(Result<Vec<ProvenLesson>, String>),

    // Для модального окна заданий преподавателя
    ShowTeacherAssignmentModal(ProvenLesson),
    CloseTeacherAssignmentModal,
    LoadTeacherAssignmentForProvenLesson,
    TeacherAssignmentsLoaded(Result<Vec<Assignment>, String>),

    // Для редактирования задания в модальном окне преподавателя
    StartEditingTeacherAssignment(Assignment), // Для предварительного заполнения полей ввода
    EditingTeacherAssignmentTitleChanged(String),
    EditingTeacherAssignmentDescriptionChanged(TextInputOrEditorInput), // Может быть действием TextEditor или строкой TextInput
    SaveEditedTeacherAssignment,
    TeacherAssignmentSaved(Result<(), String>), // Result для обратной связи

    // Для добавления существующих заданий к запланированному уроку
    LoadAvailableAssignments,
    AvailableAssignmentsLoaded(Result<Vec<Assignment>, String>),
    SelectedAssignmentToAddToLesson(Assignment),
    AddExistingAssignmentToProvenLesson,
    ExistingAssignmentAdded(Result<(), String>),
    DeleteProvenLessonAssignment(i32, i32), // proven_lesson_id, assignment_id
    ProvenLessonAssignmentDeleted(Result<(), String>),

    SubmitEditedTeacherAssignment,
    CancelEditingTeacherAssignment,
    //ProvenLessonsLoaded(Result<Vec<ProvenLesson>, String>),
    SelectProvenLessonForAssignments(ProvenLesson),
    AssignmentsLoaded(Result<Vec<Assignment>, String>),
    GroupLessonsLoaded(Result<Vec<ProvenLesson>, String>),

    // Cообщение для загрузки уроков с заданиями
    GroupLessonsWithAssignmentsLoaded(Result<Vec<LessonWithAssignments>, String>),
    // Сообщение для загрузки проведенных занятий (если будете их отображать)
    PastSessionsLoaded(Result<Vec<PastSession>, String>),
    ConductLesson(i32, i32),

    LoadGroups,
    GroupsLoaded(Result<Vec<Group>, String>),
    CourseLessonsLoaded(Result<Vec<LessonWithAssignments>, String>),

    LoadAllUsers, // Запрос на загрузку всех пользователей
    AllUsersLoaded(Result<Vec<UserInfo>, String>),

    LoadAllCourses,
    AllCoursesLoaded(Result<Vec<Course>, String>), // Course должен быть импортирован

    NoOp,

    ConductLessonResult(Result<Vec<PastSession>, String>), // Результат добавления и загрузки PastSessions
    ShowError(String), // Более общее сообщение для отображения ошибок
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
            new_course_instructor: None,
            new_course_level: Level::Beginner,
            editing_course: None,
            edit_course_title: "".to_string(),
            edit_course_description: "".to_string(),
            edit_course_instructor: None,
            user_avatar_data: None,
            edit_course_level: Level::Beginner,
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
            selected_group_id: None,
            is_manage_students_modal_open: false,
            group_students: vec![],
            all_students: vec![],
            selected_student_to_add: None,
            //user_group_name: None,
            //logged_in_user_id: None,
            show_children_modal: false,
            parent_children: vec![],
            available_children: vec![],
            selected_child_to_add: None,
            show_course_details_modal: false,
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
            group_proven_lessons: vec![],
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
            teacher_error_message: None,
            group_past_sessions: vec![],
            course_id_to_title: Default::default(),
            user_id_to_name: Default::default(),
            groups: vec![],
            editing_assignment_lesson: None,
            new_course_teacher: None,
            edit_course_teacher: None,
            new_course_teacher_id: None,
            edit_course_teacher_id: None,
            all_users: vec![],
            all_groups: vec![],
            all_courses: vec![],
            past_sessions_for_group: vec![],
        }
    }
}
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message>{
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
                    Message::UserLoggedIn
                )
                
            }
            Message::UserLoggedIn(result) => {
                match result {
                    Ok(user_info_data) => { // <-- ИЗМЕНИТЕ ЭТО ЗДЕСЬ! Назовите переменную, например, user_info_data
                        self.current_user = Some(UserInfo {
                            id: user_info_data.id,
                            name: user_info_data.name, // <-- Теперь это будет работать!
                            email: user_info_data.email,
                            birthday: user_info_data.birthday,
                            user_type: user_info_data.user_type.clone(),
                            group: user_info_data.group, // <-- И здесь UserInfo.group (поле структуры)
                            avatar_data: user_info_data.avatar_data,
                            child_count: user_info_data.child_count,
                        });

                        self.current_screen = Screen::Profile;
                        self.error_message.clear();

                        if let Some(user) = &self.current_user {
                            if user.user_type == "teacher" {
                                return self.update(Message::LoadTeacherGroups);
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = e.clone();
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
                if self.user_name.trim().is_empty() || self.user_surname.trim().is_empty() || self.user_patronymic.trim().is_empty() {
                    self.register_error = Some("Пожалуйста, заполните Фамилию, Имя и Отчество".to_string());
                    return Task::none();
                }

                let fio_re = Regex::new(r"^[А-Яа-яЁё\s-]+$").unwrap();
                if !fio_re.is_match(&self.user_name) || !fio_re.is_match(&self.user_surname) || !fio_re.is_match(&self.user_patronymic) {
                    self.register_error = Some("ФИО может содержать только русские буквы, пробелы и дефисы".to_string());
                    return Task::none();
                }

                if self.user_password.trim().is_empty() || self.user_password_repeat.trim().is_empty() {
                    self.register_error = Some("Пароль не может быть пустым".to_string());
                    return Task::none();
                }

                if self.user_password != self.user_password_repeat {
                    self.register_error = Some("Пароли не совпадают".to_string());
                    return Task::none();
                }

                let password = &self.user_password;

                if password.len() < 8 {
                    self.register_error = Some("Пароль должен содержать минимум 8 символов".to_string());
                    return Task::none();
                }

                let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
                if !has_uppercase {
                    self.register_error = Some("Пароль должен содержать хотя бы одну заглавную букву".to_string());
                    return Task::none();
                }

                let has_digit = password.chars().any(|c| c.is_ascii_digit());
                if !has_digit {
                    self.register_error = Some("Пароль должен содержать хотя бы одну цифру".to_string());
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
                    self.register_error = Some("Email должен содержать только один символ '@'.".to_string());
                    return Task::none();
                }
                if parts[0].is_empty() {
                    self.register_error = Some("Email должен содержать имя пользователя перед '@'.".to_string());
                    return Task::none();
                }
                if parts[1].is_empty() {
                    self.register_error = Some("Email должен содержать домен после '@'.".to_string());
                    return Task::none();
                }
                if !parts[1].contains('.') {
                    self.register_error = Some("Домен email должен содержать хотя бы одну точку (например: example.com).".to_string());
                    return Task::none();
                }
                let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                if !email_re.is_match(email) {
                    self.register_error = Some("Email содержит недопустимые символы или некорректный формат.".to_string());
                    return Task::none();
                }

                let conn = Connection::open("db_platform").unwrap();
                match db::is_email_taken(&conn, email) {
                    Ok(true) => {
                        self.register_error = Some("Пользователь с таким email уже существует.".to_string());
                        return Task::none();
                    }
                    Ok(false) => {}
                    Err(err) => {
                        self.register_error = Some(format!("Ошибка при проверке email: {}", err));
                        return Task::none();
                    }
                }

                let full_name = format!("{} {} {}", self.user_surname, self.user_name, self.user_patronymic);
                let password_hash = hash_password(&self.user_password);

                // Здесь регистрация пользователя должна быть асинхронной операцией
                // Однако, чтобы не усложнять, пока оставим синхронно, но вернем Task::none()
                // В идеале, эта часть тоже должна быть Task::perform
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
                    self.user_email = email.to_string();
                    self.logged_in_user = full_name;
                    // Эти операции тоже могут быть асинхронными
                    db::update_user_avatar(&conn, &self.user_email, fs::read(DEFAULT_AVATAR).unwrap().as_slice()).unwrap();
                    self.user_avatar_data = Some(fs::read(DEFAULT_AVATAR).unwrap());
                    self.current_screen = Screen::Profile;
                }
                Task::none() // Возвращаем Task::none()
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
                if let Some(theme) = theme_from_str(name) {
                    let _ = save_theme(&theme);
                    self.theme= theme;
                }
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
            Message::Er(_v) => {
                Task::none()
            }
            Message::ChooseAvatar => {
                if self.user_email.trim().is_empty() {
                    self.error_message = "Вы не вошли в систему. Email неизвестен.".to_string();
                    return Task::none();
                }

                // Эта операция также должна быть асинхронной, но для простоты пока оставим синхронно
                // В идеале, это должен быть Task::perform(choose_avatar_and_update_db(self.user_email.clone()), Message::AvatarChosen)
                if let Some(path_buf) = rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg"]).pick_file() {
                    match fs::read(&path_buf) {
                        Ok(image_data) => {
                            let conn = Connection::open("db_platform").unwrap();
                            if let Err(err) = db::update_user_avatar(&conn, &self.user_email, &image_data) {
                                self.error_message = format!("Ошибка сохранения аватара: {}", err);
                            } else {
                                self.user_avatar_data = Some(image_data);
                                self.error_message.clear();
                            }
                        }
                        Err(err) => {
                            self.error_message = format!("Ошибка чтения файла аватара: {}", err);
                        }
                    }
                }
                Task::none() // Возвращаем Task::none()
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
            Message::LoadAllCourses => {
                Task::perform(
                    async move {
                        task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД для курсов: {}", e))?;
                            db::get_courses(&conn)
                                .map_err(|e| format!("Ошибка загрузки курсов: {}", e))
                        }).await
                            .map_err(|join_err| format!("Ошибка выполнения задачи загрузки курсов: {}", join_err))?
                    },
                    Message::AllCoursesLoaded // <-- Когда задача завершится, отправь это сообщение
                )
            }
            Message::LoadAllUsers => {
                Task::perform(
                    async move {
                        task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД для пользователей: {}", e))?;
                            db::get_all_users(&conn)
                                .map_err(|e| format!("Ошибка загрузки пользователей: {}", e))
                        }).await
                            .map_err(|join_err| format!("Ошибка выполнения задачи загрузки пользователей: {}", join_err))?
                    },
                    Message::AllUsersLoaded // <-- Когда задача завершится, отправь это сообщение
                )
            }
            Message::LoadGroups => { // Если вы еще не загружаете группы асинхронно
                Task::perform(
                    async move {
                        task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД для групп: {}", e))?;
                            db::get_groups(&conn) // Убедитесь, что db::get_groups существует
                                .map_err(|e| format!("Ошибка загрузки групп: {}", e))
                        }).await
                            .map_err(|join_err| format!("Ошибка выполнения задачи загрузки групп: {}", join_err))?
                    },
                    Message::GroupsLoaded
                )
            }


            // --- Обработка результатов загрузки данных ---
            Message::AllCoursesLoaded(result) => {
                match result {
                    Ok(courses) => {
                        self.all_courses = courses.clone();
                        self.course_id_to_title = courses.into_iter().map(|c| (c.id, c.title)).collect();
                        println!("DEBUG: course_id_to_title заполнена: {:?}", self.course_id_to_title); // <--- ВАЖНО!
                        self.error_message = "".to_string(); // Очищаем сообщение об ошибке
                    }
                    Err(e) => {
                        eprintln!("Ошибка при загрузке курсов: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none() // Возвращаем Task::none(), так как это конечный обработчик
            }
            Message::AllUsersLoaded(result) => {
                match result {
                    Ok(users) => {
                        self.all_users = users.clone();
                        self.user_id_to_name = users.into_iter().map(|u| (u.id, u.name)).collect();
                        println!("DEBUG: user_id_to_name заполнена: {:?}", self.user_id_to_name); // <--- ВАЖНО!
                        self.error_message = "".to_string();
                    }
                    Err(e) => {
                        eprintln!("Ошибка при загрузке пользователей: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::GroupsLoaded(result) => {
                match result {
                    Ok(groups) => {
                        self.all_groups = groups; // Предполагаем, что у вас есть `pub all_groups: Vec<Group>` в App
                        println!("DEBUG: all_groups заполнена: {:?}", self.all_groups);
                        self.error_message = "".to_string();
                    }
                    Err(e) => {
                        eprintln!("Ошибка при загрузке групп: {}", e);
                        self.error_message = e.to_string();
                    }
                }
                Task::none()
            }
            Message::SubmitNewCourse => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();

                let level_str = self.new_course_level.to_string(); // Level::to_string() возвращает String
                // instructor_str здесь больше не нужен, мы используем ID

                if db::add_course(
                    &conn,
                    &self.new_course_title,
                    &self.new_course_description,
                    self.new_course_teacher_id, // <--- ПЕРЕДАЁМ Option<i32> ID ПРЕПОДАВАТЕЛЯ
                    Some(&level_str),           // <--- Теперь передаём ссылку на String
                ).is_ok() {
                    self.show_add_course_modal = false;
                    self.new_course_title.clear();
                    self.new_course_description.clear();
                    self.new_course_teacher_id = None; // Очищаем ID преподавателя
                    self.new_course_level = Level::Beginner;
                }
                Task::none() // Возвращаем Task::none()
            }

            Message::DeleteCourse(course_id) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
                db::delete_course(&conn, course_id).unwrap();
                Task::none() // Возвращаем Task::none()
            }
            Message::NewCourseInstructorChanged(selected_user_info) => {
                // Сохраняем UserInfo для отображения, и ID для сохранения
                self.new_course_teacher = selected_user_info.clone();
                self.new_course_teacher_id = selected_user_info.map(|u| u.id); // Получаем ID
                Task::none()
            }
            Message::NewCourseLevelChanged(level) => {
                self.new_course_level = level;
                Task::none()
            }
            Message::StartEditingCourse(course) => {
                self.edit_course_title = course.title.clone();
                self.edit_course_description = course.description.clone();

                // <--- ИЗМЕНЕНИЯ ЗДЕСЬ
                // Теперь App должен хранить ID преподавателя, а не UserInfo или String имени
                self.edit_course_teacher_id = course.instructor_id; // Используем instructor_id из Course
                // self.edit_course_instructor_name = course.instructor_name.clone(); // Если нужно хранить имя тоже, но для PickList нужен только ID UserInfo, а PickList выведет имя сам

                self.edit_course_level = course.level.clone()
                    .and_then(|level_str| Level::from_str(&level_str).ok())
                    .unwrap_or(Level::Beginner);
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
            Message::EditCourseInstructorChanged(selected_user_info) => {
                // Сохраняем UserInfo для отображения, и ID для сохранения
                self.edit_course_teacher = selected_user_info.clone();
                self.edit_course_teacher_id = selected_user_info.map(|u| u.id); // Получаем ID
                Task::none()
            }
            Message::EditCourseLevelChanged(level) => {
                self.edit_course_level = level;
                Task::none()
            }
            Message::SubmitEditedCourse => {
                // Эта операция должна быть асинхронной
                if let Some(original_course) = &self.editing_course {
                    let conn = Connection::open("db_platform").unwrap();

                    // Получаем имя преподавателя для instructor_name, если оно нужно
                    // Если App.edit_course_teacher_id установлен, найдем соответствующего UserInfo
                    let instructor_name_for_course = self.edit_course_teacher_id.and_then(|id| {
                        // Поиск UserInfo в app.all_users
                        self.all_users.iter()
                            .find(|u| u.id == id)
                            .map(|u| u.name.clone())
                    });


                    let updated_course = Course {
                        id: original_course.id,
                        title: self.edit_course_title.clone(),
                        description: self.edit_course_description.clone(),
                        instructor_id: self.edit_course_teacher_id, // <--- ИСПОЛЬЗУЕМ instructor_id
                        instructor_name: instructor_name_for_course, // <--- ИСПОЛЬЗУЕМ instructor_name
                        level: Some(self.edit_course_level.to_string()),
                        lesson_count: original_course.lesson_count, // Сохраняем оригинальное количество занятий
                    };

                    if db::update_course(&conn, &updated_course).is_ok() {
                        self.show_add_course_modal = false;
                        self.editing_course = None;
                        self.edit_course_title.clear();
                        self.edit_course_description.clear();
                        // Очищаем поля, связанные с преподавателем, как мы их назвали
                        self.edit_course_teacher_id = None; // <--- Очищаем ID
                        // self.edit_course_instructor_name = None; // Если такое поле есть в App и вы его очищаете
                        self.edit_course_level = Level::Beginner;
                    } else {
                        self.error_message = "Ошибка сохранения курса.".to_string();
                    }
                }
                Task::none() // Возвращаем Task::none()
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
                        self.edit_user_error = Some("Email должен содержать символ '@'.".to_string());
                        return Task::none();
                    }

                    let parts: Vec<&str> = email.split('@').collect();
                    if parts.len() != 2 {
                        self.edit_user_error = Some("Email должен содержать только один символ '@'.".to_string());
                        return Task::none();
                    }

                    if parts[0].is_empty() {
                        self.edit_user_error = Some("Email должен содержать имя пользователя перед '@'.".to_string());
                        return Task::none();
                    }
                    if parts[1].is_empty() {
                        self.edit_user_error = Some("Email должен содержать домен после '@'.".to_string());
                        return Task::none();
                    }
                    if !parts[1].contains('.') {
                        self.edit_user_error = Some("Домен email должен содержать хотя бы одну точку (например: example.com).".to_string());
                        return Task::none();
                    }

                    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
                    if !email_re.is_match(email) {
                        self.edit_user_error = Some("Email содержит недопустимые символы или некорректный формат.".to_string());
                        return Task::none();
                    }

                    let conn = Connection::open("db_platform").unwrap();

                    match db::is_email_taken_except(&conn, email, &original_user.email) {
                        Ok(true) => {
                            self.edit_user_error = Some("Email уже используется другим пользователем.".to_string());
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
                            self.edit_user_error = Some(format!("Ошибка при проверке email: {}", err));
                        }
                    }
                }
                Task::none() // Возвращаем Task::none()
            }

            Message::DeleteUser(email) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
                let _ = db::delete_user(&conn, &email);
                Task::none() // Возвращаем Task::none()
            }
            Message::CourseFilterChanged(text) => {
                self.course_filter_text = text;
                Task::none()
            }
            Message::ToggleAddGroupModal(show) => {
                if !show {
                    self.new_group_name.clear();
                    self.new_group_course = None;
                    self.new_group_teacher = None;
                }
                self.show_add_group_modal = show;
                Task::none()
            }
            Message::NewGroupNameChanged(name) => {
                self.new_group_name = name;
                Task::none()
            }
            Message::NewGroupCourseChanged(course) => {
                self.new_group_course = course;
                Task::none()
            }
            Message::NewGroupTeacherChanged(teacher) => {
                self.new_group_teacher = teacher;
                Task::none()
            }

            Message::EditGroupNameChanged(name) => {
                self.edit_group_name = name;
                Task::none()
            }
            Message::EditGroupCourseChanged(course) => {
                self.edit_group_course = course;
                Task::none()
            }
            Message::EditGroupTeacherChanged(teacher) => {
                self.edit_group_teacher = teacher;
                Task::none()
            }

            Message::StartEditingGroup(group) => {
                self.editing_group = Some(group.clone());
                self.edit_group_name = group.name.clone();
                self.edit_group_course = group.course_id;
                self.edit_group_teacher = group.teacher_id.clone();
                self.show_add_group_modal = true;
                Task::none()
            }
            Message::CancelEditingGroup => {
                self.editing_group = None;
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.show_add_group_modal = false;
                Task::none()
            }
            Message::SubmitEditedGroup => {
                let edited_group_id = self.editing_group.as_ref().map(|g| g.id);
                let edited_group_name_clone = self.edit_group_name.clone();
                let edited_group_course_id = self.edit_group_course;
                let edited_group_teacher_id = self.edit_group_teacher;

                self.editing_group = None;
                self.edit_group_name.clear();
                self.edit_group_course = None;
                self.edit_group_teacher = None;
                self.show_add_group_modal = false;

                Task::perform(
                    async move {
                        task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?; // <-- map_err

                            let group_id = edited_group_id
                                .ok_or_else(|| "Не удалось получить ID редактируемой группы".to_string())?;
                            let course_id = edited_group_course_id
                                .ok_or_else(|| "Не выбран курс".to_string())?;
                            let teacher_id = edited_group_teacher_id
                                .ok_or_else(|| "Не выбран преподаватель".to_string())?;

                            // Теперь вызываем db::update_group с ID напрямую
                            db::update_group(&conn, group_id, &edited_group_name_clone, course_id, teacher_id) // <--- ПЕРЕДАЕМ ID
                                .map_err(|e| format!("Ошибка обновления группы: {}", e)) // <-- map_err
                        }).await // .await для tokio::task::spawn_blocking
                            .map_err(|join_err| { // <-- map_err для JoinHandle
                                eprintln!("Блокирующая задача для обновления группы завершилась ошибкой: {:?}", join_err);
                                format!("Ошибка выполнения операции: {}", join_err)
                            })? // <-- ? в конце
                    },
                    |result: Result<(), String>| { // Явно укажите тип результата
                        match result {
                            Ok(_) => Message::LoadGroups,
                            Err(e) => {
                                eprintln!("Ошибка при обновлении группы: {}", e);
                                Message::Er(e)
                            }
                        }
                    }
                )
            }
            Message::SubmitNewGroup => {
                let new_group_name_clone = self.new_group_name.clone();
                let new_group_course_id = self.new_group_course; // Это Option<i32>
                let new_group_teacher_id = self.new_group_teacher; // Это Option<i32>

                // Очищаем поля UI до выполнения асинхронной операции
                self.new_group_name.clear();
                self.new_group_course = None;
                self.new_group_teacher = None;
                self.show_add_group_modal = false;

                Task::perform(
                    async move {
                        task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД: {}", e))?; // <-- map_err здесь

                            let course_id = new_group_course_id
                                .ok_or_else(|| "Не выбран курс".to_string())?;
                            let teacher_id = new_group_teacher_id
                                .ok_or_else(|| "Не выбран преподаватель".to_string())?;

                            // Теперь вызываем db::insert_group с ID напрямую
                            db::insert_group(&conn, &new_group_name_clone, course_id, teacher_id) // <--- ПЕРЕДАЕМ ID
                                .map_err(|e| format!("Ошибка добавления группы: {}", e)) // <-- map_err здесь
                        }).await // .await для tokio::task::spawn_blocking
                            .map_err(|join_err| { // <-- map_err для JoinHandle
                                eprintln!("Блокирующая задача для добавления группы завершилась ошибкой: {:?}", join_err);
                                format!("Ошибка выполнения операции: {}", join_err)
                            })? // <-- ? в конце для обработки Result<Result<...>, ...>
                    },
                    |result: Result<(), String>| { // Явно укажите тип результата
                        match result {
                            Ok(_) => Message::LoadGroups,
                            Err(e) => {
                                eprintln!("Ошибка при добавлении группы: {}", e);
                                Message::Er(e)
                            }
                        }
                    }
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
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
                if let Err(err) = db::delete_group(&conn, id) {
                    eprintln!("Ошибка удаления группы: {:?}", err);
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::GroupFilterChanged(text) => {
                self.group_filter_text = text;
                Task::none()
            }
            Message::OpenManageStudentsModal(group_id) => {
                // Эти операции должны быть асинхронными
                let conn = Connection::open("db_platform").unwrap();
                let students = db::get_students_for_group(&conn, group_id);
                let all_students = db::get_all_student_names(&conn);

                self.selected_group_id = Some(group_id);
                self.is_manage_students_modal_open = true;
                self.group_students = students.expect("Failed to get students for group");
                self.all_students = all_students.expect("Failed to get all student names");
                self.selected_student_to_add = None;
                Task::none() // Возвращаем Task::none()
            }
            Message::CloseManageStudentsModal => {
                self.is_manage_students_modal_open = false;
                self.selected_group_id = None;
                self.group_students.clear();
                self.selected_student_to_add = None;
                Task::none()
            }

            Message::StudentToAddSelected(student_opt) => {
                self.selected_student_to_add = student_opt;
                Task::none()
            }

            Message::AddStudent => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
                if let (Some(group_id), Some(student_name)) = (self.selected_group_id, &self.selected_student_to_add) {
                    db::add_student_to_group(&conn, group_id, student_name).unwrap();
                    self.group_students = db::get_students_for_group(&conn, group_id).expect("Failed to get students for group after adding");
                    self.selected_student_to_add = None;
                }
                Task::none() // Возвращаем Task::none()
            }

            Message::RemoveStudent(student_name) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
                if let Some(group_id) = self.selected_group_id {
                    db::remove_student_from_group(&conn, group_id, &student_name).unwrap();
                    self.group_students = db::get_students_for_group(&conn, group_id).expect("Failed to get students for group after removing");
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::ShowParentChildren(parent_email) => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();

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
            Message::DeleteChild { parent_email, child_email } => {
                // Эта операция должна быть асинхронной
                let conn = Connection::open("db_platform").unwrap();
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
                    println!("Attempting to add child with email: {} to parent with email: {}", child.email, parent_email);
                    let conn = Connection::open("db_platform").unwrap();

                    if let Err(e) = db::add_child_to_parent(&conn, &parent_email, &child.email) {
                        println!("Ошибка при добавлении ребёнка: {}", e);
                    } else {
                        self.parent_children = db::get_children_for_parent(&conn, &parent_email).unwrap_or_default();
                        self.available_children = db::get_unassigned_children(&conn).unwrap_or_default();
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
                        let blocking_result = task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД для загрузки уроков: {}", e))?;

                            // 1. Получаем все базовые уроки для этого курса
                            let mut lessons = db::get_lessons_for_course(&conn, course_id_clone)
                                .map_err(|e| format!("Ошибка загрузки уроков курса: {}", e))?;


                            for lesson in &mut lessons {
                                let assignments = db::get_assignments_for_lesson(&conn, lesson.id)
                                    .map_err(|e| format!("Ошибка загрузки заданий для урока {}: {}", lesson.id, e))?;
                                lesson.assignments = assignments;
                            }
                            Ok(lessons)
                        }).await;

                        blocking_result.unwrap_or_else(|join_err| {
                            eprintln!("Блокирующая задача для уроков завершилась ошибкой: {:?}", join_err);
                            Err(format!("Ошибка выполнения операции: {}", join_err))
                        })
                    },
                    |result: Result<Vec<LessonWithAssignments>, String>| {
                        // Отправляем результат в новое сообщение, чтобы обновить App.course_lessons
                        Message::CourseLessonsLoaded(result)
                    }
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
                        self.lesson_error_message = Some("Название занятия не может быть пустым.".to_string());
                        println!("Ошибка добавления занятия: Название не может быть пустым.");
                        return Task::none(); // Возвращаем Task::none()
                    }

                    // Эта операция должна быть асинхронной
                    let conn = Connection::open("db_platform").unwrap();
                    match db::add_lesson(&conn, course_id, Some(lesson_number.unwrap_or(0)), lesson_title) {
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
                            self.lesson_error_message = Some(format!("Ошибка БД при добавлении занятия: {:?}", e));
                        }
                    }
                } else {
                    println!("Ошибка: Не выбран курс для добавления занятия.");
                    self.lesson_error_message = Some("Не выбран курс для добавления занятия.".to_string());
                }
                Task::none() // Возвращаем Task::none()
            }
            Message::StartEditingTeacherAssignment(assignment) => {
                self.editing_teacher_assignment = Some(assignment.clone());
                self.editing_teacher_assignment_title = assignment.title;
                if assignment.assignment_type == "Lecture" || assignment.assignment_type == "Practice" {
                    self.editing_teacher_assignment_description_content = text_editor::Content::with_text(&assignment.description);
                    self.editing_teacher_assignment_description_text_input.clear();
                } else {
                    self.editing_teacher_assignment_description_text_input = assignment.description;
                    self.editing_teacher_assignment_description_content = text_editor::Content::new();
                }
                self.teacher_assignment_edit_error_message = None;
                Task::none()
            }
            Message::EditingTeacherAssignmentDescriptionChanged(input) => {
                match input {
                    TextInputOrEditorInput::TextInput(s) => {
                        self.editing_teacher_assignment_description_text_input = s;
                    }
                    TextInputOrEditorInput::TextEditor(action) => {
                        self.editing_teacher_assignment_description_content.perform(action);
                    }
                }
                Task::none()
            }
            Message::TeacherAssignmentsLoaded(result) => {
                match result {
                    Ok(assignments) => {
                        self.teacher_lesson_assignments = assignments;
                    }
                    Err(e) => {
                        self.teacher_assignment_edit_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::AvailableAssignmentsLoaded(result) => {
                match result {
                    Ok(assignments) => {
                        self.available_assignments = assignments;
                    }
                    Err(e) => {
                        self.teacher_assignment_edit_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::SelectedAssignmentToAddToLesson(assignment) => {
                self.selected_assignment_to_add_to_lesson = Some(assignment);
                Task::none()
            }
            Message::AddExistingAssignmentToProvenLesson => {
                if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                    if let Some(assignment) = &self.selected_assignment_to_add_to_lesson {
                        return Task::perform(
                            add_existing_assignment_to_proven_lesson(proven_lesson.id, assignment.id),
                            Message::ExistingAssignmentAdded,
                        );
                    }
                }
                self.teacher_assignment_edit_error_message = Some("Не выбрано задание для добавления.".to_string());
                Task::none()
            }
            Message::ExistingAssignmentAdded(result) => {
                match result {
                    Ok(_) => {
                        self.teacher_assignment_edit_error_message = None;
                        // Перезагрузить задания для текущего занятия
                        if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                            return Task::perform(
                                load_teacher_assignments_for_proven_lesson(proven_lesson.id),
                                Message::TeacherAssignmentsLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        self.teacher_assignment_edit_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::DeleteProvenLessonAssignment(_proven_lesson_id, assignment_id) => {
                if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                    let proven_lesson_id = proven_lesson.id;
                    return Task::perform(
                        delete_proven_lesson_assignment(proven_lesson_id, assignment_id),
                        Message::ProvenLessonAssignmentDeleted,
                    );
                }
                self.teacher_assignment_edit_error_message = Some("Ошибка удаления задания: не выбрано занятие.".to_string());
                Task::none()
            }
            Message::ProvenLessonAssignmentDeleted(result) => {
                match result {
                    Ok(_) => {
                        self.teacher_assignment_edit_error_message = None;
                        // Перезагрузить задания для текущего занятия
                        if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                            return Task::perform(
                                load_teacher_assignments_for_proven_lesson(proven_lesson.id),
                                Message::TeacherAssignmentsLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        self.teacher_assignment_edit_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::SubmitEditedTeacherAssignment => {
                if let Some(mut assignment) = self.editing_teacher_assignment.take() {
                    assignment.title = self.editing_teacher_assignment_title.clone();
                    // Получаем описание в зависимости от того, какой тип ввода активен
                    assignment.description = if assignment.assignment_type == "Lecture" || assignment.assignment_type == "Practice" {
                        self.editing_teacher_assignment_description_content.text()
                    } else {
                        self.editing_teacher_assignment_description_text_input.clone()
                    };

                    return Task::perform(
                        update_assignment(assignment),
                        Message::TeacherAssignmentSaved,
                    );
                }
                self.teacher_assignment_edit_error_message = Some("Ошибка: Нет задания для сохранения.".to_string());
                Task::none()
            }
            Message::TeacherAssignmentSaved(result) => {
                match result {
                    Ok(_) => {
                        self.teacher_assignment_edit_error_message = None;
                        // Перезагрузить задания после сохранения
                        if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                            return Task::perform(
                                load_teacher_assignments_for_proven_lesson(proven_lesson.id),
                                Message::TeacherAssignmentsLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        self.teacher_assignment_edit_error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::CancelEditingTeacherAssignment => {
                self.editing_teacher_assignment = None;
                self.editing_teacher_assignment_title.clear();
                self.editing_teacher_assignment_description_content = text_editor::Content::new();
                self.editing_teacher_assignment_description_text_input.clear();
                self.teacher_assignment_edit_error_message = None;
                Task::none()
            }
            Message::GoToClasses => {
                self.current_screen = Screen::Classes;
                if let Some(user) = &self.current_user {
                    if user.user_type == "teacher" {
                        println!("Teacher logged in, attempting to load groups for: {}", user.email);
                        return Task::perform(load_teacher_groups(user.email.clone()), Message::TeacherGroupsLoaded);
                    }
                }
                Task::none()
            }
            // ... (остальные сообщения, добавленные вами ранее) ...
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
                            let blocking_result = task::spawn_blocking(move || {
                                let conn = Connection::open("db_platform")
                                    .map_err(|e| format!("Не удалось открыть БД для уроков/заданий: {}", e))?;

                                // --- ИСПРАВЛЕНИЕ ЗДЕСЬ ---
                                // Разворачиваем Option<i32> в i32. Если это None, возвращаем ошибку.
                                let course_id = course_id_for_group_option
                                    .ok_or_else(|| "У выбранной группы нет связанного курса".to_string())?;
                                // --- КОНЕЦ ИСПРАВЛЕНИЯ ---

                                db::get_lessons_for_course_and_group(&conn, course_id, group_id_clone) // `course_id` теперь i32
                                    .map_err(|e| format!("Ошибка загрузки уроков для группы: {}", e))

                            }).await;

                            blocking_result.unwrap_or_else(|join_err| {
                                eprintln!("Блокирующая задача для уроков/заданий завершилась ошибкой: {:?}", join_err);
                                Err(format!("Ошибка выполнения операции: {}", join_err))
                            })
                        },
                        Message::GroupLessonsWithAssignmentsLoaded
                    ),

                    // Задача 2: Загрузить проведенные сессии для группы (эта часть, скорее всего, в порядке)
                    Task::perform(
                        async move {
                            let blocking_result = task::spawn_blocking(move || {
                                let conn = Connection::open("db_platform")
                                    .map_err(|e| format!("Не удалось открыть БД для PastSessions: {}", e))?;
                                db::get_past_sessions_for_group(&conn, group_id_clone)
                                    .map_err(|e| format!("Ошибка загрузки проведенных занятий: {}", e))
                            }).await;

                            blocking_result.unwrap_or_else(|join_err| {
                                eprintln!("Блокирующая задача для PastSessions завершилась ошибкой: {:?}", join_err);
                                Err(format!("Ошибка выполнения операции: {}", join_err))
                            })
                        },
                        Message::PastSessionsLoaded
                    )
                ])
            }
            Message::ConductLesson(lesson_id, group_id) => {
                println!("DEBUG: Handling ConductLesson for lesson_id: {}, group_id: {}", lesson_id, group_id);
                let group_id_clone = group_id;
                let lesson_id_clone = lesson_id;

                Task::perform(
                    async move {
                        // 1. Попытка добавить PastSession
                        let add_result = task::spawn_blocking(move || {
                            let conn = Connection::open("db_platform")
                                .map_err(|e| format!("Не удалось открыть БД для добавления PastSession: {}", e))?;
                            db::add_past_session(&conn, group_id_clone, lesson_id_clone)
                                .map_err(|e| format!("Ошибка добавления записи о проведенном занятии: {}", e))
                        }).await.unwrap_or_else(|join_err| {
                            // Если блокирующая задача упала или не смогла запуститься
                            Err(format!("Блокирующая задача (добавление) завершилась ошибкой: {:?}", join_err))
                        });

                        match add_result {
                            Ok(_) => {
                                // 2. Если добавление успешно, пытаемся перезагрузить PastSessions
                                task::spawn_blocking(move || {
                                    let conn = Connection::open("db_platform")
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
                    // <--- ВОТ ГЛАВНОЕ ИЗМЕНЕНИЕ: Отправляем новое сообщение с результатом
                    Message::ConductLessonResult // Это будет callback, который преобразует Result<...> в Message
                )
            }
            Message::ShowError(error_msg) => {
                self.error_message = error_msg.to_string(); // Устанавливаем сообщение об ошибке
                Task::none() // Не запускаем никаких новых задач
            }
            Message::ConductLessonResult(result) => {
                println!("DEBUG: Handling ConductLessonResult: {:?}", result.is_ok());
                if result.is_err() {
                    println!("DEBUG: ConductLessonResult error: {:?}", result.clone().unwrap_err());
                }
                match result {
                    Ok(past_sessions) => {
                        println!("DEBUG: Successfully conducted lesson. Past sessions loaded: {}", past_sessions.len());
                        self.past_sessions_for_group = past_sessions;

                        if let Some(group) = &self.selected_group_for_classes {
                            println!("DEBUG: Sending SelectGroupForClasses for group ID: {}", group.id);
                            let group_clone = group.clone(); // Клонируем здесь, чтобы владеть данными
                            Task::perform(
                                async move { // <--- Добавляем `move` сюда
                                    Message::SelectGroupForClasses(group_clone) // Используем клонированную переменную
                                },
                                |msg| msg
                            )
                        } else {
                            println!("DEBUG: No selected group, cannot re-select.");
                            Task::none()
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка проведения занятия или перезагрузки списка: {}", e);
                        self.error_message = e.to_string(); // Убедитесь, что error_message: Option<String>
                        Task::none() // Никаких задач при ошибке
                    }
                }
            }

            Message::GroupLessonsWithAssignmentsLoaded(result) => {
                match result {
                    Ok(lessons) => {
                        self.selected_group_lessons_with_assignments = lessons;
                        Task::none()
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки уроков для группы: {}", e);
                        self.error_message = e.to_string();
                        Task::none()
                    }
                }
            }

            Message::PastSessionsLoaded(result) => {
                match result {
                    Ok(past_sessions) => {
                        self.past_sessions_for_group = past_sessions;
                        Task::none()
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки проведенных занятий: {}", e);
                        self.error_message = e.to_string();
                        Task::none()
                    }
                }
            }
            Message::GroupLessonsLoaded(result) => {
                match result {
                    Ok(lessons) => {
                        self.group_proven_lessons = lessons; // Обновляем список уроков с их заданиями
                        self.teacher_assignment_edit_error_message = None; // Очищаем ошибку
                    }
                    Err(e) => {
                        self.group_proven_lessons = Vec::new(); // Очищаем список при ошибке
                        self.teacher_assignment_edit_error_message = Some(e); // Показываем ошибку
                    }
                }
                Task::none()
            }
            Message::GroupProvenLessonsLoaded(result) => {
                match result {
                    Ok(lessons) => {
                        self.group_proven_lessons = lessons;
                        self.teacher_error_message = None;
                        println!("DEBUG: Загружено {} занятий для группы.", self.group_proven_lessons.len());
                    }
                    Err(e) => {
                        self.teacher_error_message = Some(e.clone());
                        self.group_proven_lessons.clear(); // Очистить список при ошибке
                        eprintln!("ERROR: Ошибка загрузки занятий для группы: {}", e);
                    }
                }
                Task::none()
            }
            Message::SelectProvenLessonForAssignments(lesson) => {
                self.selected_proven_lesson_for_assignments = Some(lesson.clone());
                // Открываем модальное окно с заданиями
                self.update(Message::ShowTeacherAssignmentModal(lesson)) // Вызов другого сообщения через update
            }
            Message::ShowTeacherAssignmentModal(proven_lesson) => {
                self.selected_proven_lesson_for_assignments = Some(proven_lesson.clone());
                self.show_teacher_assignment_modal = true;

                let lesson_id = proven_lesson.lesson_id;
                Task::batch(vec![
                    Task::perform(load_teacher_assignments_for_proven_lesson(lesson_id), Message::TeacherAssignmentsLoaded),
                    Task::perform(load_all_assignments(), Message::AvailableAssignmentsLoaded),
                ])
            }
            Message::CloseTeacherAssignmentModal => {
                self.show_teacher_assignment_modal = false;
                self.selected_proven_lesson_for_assignments = None;
                self.teacher_lesson_assignments.clear();
                self.available_assignments.clear();
                self.selected_assignment_to_add_to_lesson = None;
                self.editing_teacher_assignment = None;
                self.teacher_assignment_edit_error_message = None;
                Task::none()
            }
            Message::LoadTeacherAssignmentForProvenLesson => {
                if let Some(proven_lesson) = &self.selected_proven_lesson_for_assignments {
                    let proven_lesson_id = proven_lesson.id;
                    // Возвращаем Task::perform напрямую
                    Task::perform(load_teacher_assignments_for_proven_lesson(proven_lesson_id), Message::TeacherAssignmentsLoaded)
                } else {
                    // Если proven_lesson не выбран, возвращаем пустой Task и возможно, сообщение об ошибке
                    self.teacher_assignment_edit_error_message = Some("Не выбрано занятие для загрузки заданий.".to_string());
                    Task::none()
                }
            }
            Message::EditingTeacherAssignmentTitleChanged(value) => {
                self.editing_teacher_assignment_title = value;
                Task::none() // Никаких побочных эффектов
            }
            Message::SaveEditedTeacherAssignment => {
                if let Some(mut assignment_to_edit) = self.editing_teacher_assignment.clone() {
                    let new_title = self.editing_teacher_assignment_title.trim().to_string();
                    if new_title.is_empty() {
                        self.teacher_assignment_edit_error_message = Some("Название задания не может быть пустым.".to_string());
                        return Task::none(); // Если ошибка валидации, останавливаемся
                    }

                    assignment_to_edit.title = new_title;
                    assignment_to_edit.description = if assignment_to_edit.assignment_type == "Lecture" || assignment_to_edit.assignment_type == "Practice" {
                        self.editing_teacher_assignment_description_content.text()
                    } else {
                        self.editing_teacher_assignment_description_text_input.clone()
                    };

                    let updated_assignment = assignment_to_edit.clone();
                    self.editing_teacher_assignment = None;
                    self.editing_teacher_assignment_title.clear();
                    self.editing_teacher_assignment_description_text_input.clear();
                    self.editing_teacher_assignment_description_content = text_editor::Content::new();

                    // Возвращаем Task::perform
                    Task::perform(update_assignment(updated_assignment), Message::TeacherAssignmentSaved)
                } else {
                    self.teacher_assignment_edit_error_message = Some("Нет задания для сохранения.".to_string());
                    Task::none() // Если нет задания для редактирования
                }
            }
            Message::LoadAvailableAssignments => {
                // Возвращаем Task::perform
                Task::perform(load_all_assignments(), Message::AvailableAssignmentsLoaded)
            }
            Message::DeleteLesson(lesson_id) => {
                // Проверяем, какой курс сейчас открыт
                if let Some(course) = &self.editing_lessons_course {
                    let course_id = course.id;
                    let conn = Connection::open("db_platform").unwrap();
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
                                    println!("Ошибка при обновлении списка занятий после удаления: {:?}", e);
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
                let conn = Connection::open("db_platform").unwrap();
                self.current_lesson_for_assignments = Some(lesson_with_assignments.clone());

                match db::get_assignments_for_lesson(&conn, lesson_with_assignments.id) {
                    Ok(assignments) => {
                        self.lesson_assignments = assignments;
                        self.assignment_error_message = None;
                    }
                    Err(e) => {
                        self.lesson_assignments = vec![];
                        self.assignment_error_message = Some(format!("Не удалось загрузить задания: {}", e));
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
                let conn = Connection::open("db_platform").unwrap();
                if let Some(current_lesson) = &self.current_lesson_for_assignments {
                    if self.new_assignment_title.is_empty() || self.new_assignment_description.is_empty() || self.new_assignment_type.is_none() {
                        self.assignment_error_message = Some("Все поля (название, описание, тип) для нового задания должны быть заполнены.".to_string());
                    }

                    // Преобразуем выбранный AssignmentType в строку для сохранения в БД
                    let assignment_type_str = self.new_assignment_type.unwrap().to_string();
                    match db::add_assignment(&conn, current_lesson.id, &self.new_assignment_title, &self.new_assignment_description, &assignment_type_str) {
                        Ok(_) => {
                            match db::get_assignments_for_lesson(&conn, current_lesson.id) {
                                Ok(assignments) => self.lesson_assignments = assignments,
                                Err(e) => self.assignment_error_message = Some(format!("Не удалось перезагрузить задания: {}", e)),
                            }
                            self.new_assignment_title.clear();
                            self.new_assignment_description.clear();
                            self.new_assignment_type = None;
                            self.assignment_error_message = None;
                            Task::none()
                        }
                        Err(e) => {
                            self.assignment_error_message = Some(format!("Ошибка добавления задания: {}", e));
                            Task::none()
                        }
                    }
                } else {
                    self.assignment_error_message = Some("Нет выбранного занятия для добавления задания.".to_string());
                    Task::none()
                }
            }
            Message::DeleteAssignment(assignment_id) => {
                let conn = Connection::open("db_platform").unwrap();
                match db::delete_assignment(&conn, assignment_id) {
                    Ok(_) => {
                        if let Some(current_lesson) = &self.current_lesson_for_assignments {
                            match db::get_assignments_for_lesson(&conn, current_lesson.id) {
                                Ok(assignments) => self.lesson_assignments = assignments,
                                Err(e) => self.assignment_error_message = Some(format!("Не удалось перезагрузить задания: {}", e)),
                            }
                        }
                        self.assignment_error_message = None;
                        Task::none()
                    }
                    Err(e) => {
                        self.assignment_error_message = Some(format!("Ошибка удаления задания: {}", e));
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
                if assignment.assignment_type == AssignmentType::Lecture.to_string() ||
                    assignment.assignment_type == AssignmentType::Practice.to_string() {
                    // Для TextEditor: создаем новое содержимое из описания
                    self.editing_assignment_description_content = text_editor::Content::with_text(&assignment.description);
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
                let conn = Connection::open("db_platform").unwrap();
                if let Some(selected_assignment) = &self.selected_assignment_for_detail {
                    if self.editing_assignment_title.is_empty() {
                        self.assignment_edit_error_message = Some("Название задания не может быть пустым.".to_string());
                        return Task::none() // <-- Правильный возврат!
                    }

                    let description_to_save = if selected_assignment.assignment_type == AssignmentType::Lecture.to_string() ||
                        selected_assignment.assignment_type == AssignmentType::Practice.to_string() {
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
                                        async move { // <-- Асинхронный блок - это Future, передаваемый в Task::perform
                                            // Этот фьючер spawn_blocking сам по себе возвращает Result<Result<Vec<Assignment>, String>, JoinError>
                                            let blocking_result = task::spawn_blocking(move || {
                                                let conn_task = Connection::open("db_platform")
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
                                        Message::AssignmentsLoaded // <-- Теперь это сообщение корректно ожидает Result<Vec<Assignment>, String>
                                    );
                                }
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.assignment_edit_error_message = Some(format!("Ошибка сохранения задания: {}", e));
                            Task::none()
                        }
                    }
                } else {
                    eprintln!("Попытка сохранить детали задания, но задание не выбрано.");
                    self.assignment_edit_error_message = Some("Ошибка: Не выбрано задание для сохранения.".to_string());
                    Task::none()
                }
            }
            Message::AssignmentsLoaded(result) => {
                match result {
                    Ok(assignments) => {
                        self.lesson_assignments = assignments;
                        self.assignment_edit_error_message = None; // Убедитесь, что нет ошибок от предыдущих операций
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки заданий: {}", e);
                        // self.assignment_edit_error_message = Some(e); // Можно отобразить ошибку пользователю
                    }
                }
                Task::none()
            }
            Message::LoadTeacherGroups => {
                if let Some(user) = &self.current_user { // Используйте current_user для получения email и типа
                    if user.user_type == "teacher" {
                        // ИЗМЕНЕНИЕ ЗДЕСЬ: Возвращаем Task::perform напрямую
                        return Task::perform(load_teacher_groups(user.email.clone()), Message::TeacherGroupsLoaded);
                    }
                }
                // Если условие не выполнено или user не найден, возвращаем Task::none()
                Task::none()
            }
            Message::TeacherGroupsLoaded(result) => {
                match result {
                    Ok(groups) => {
                        self.teacher_groups = groups;
                        // Опционально выберите первую группу или группу по умолчанию
                        if let Some(first_group) = self.teacher_groups.first().cloned() {
                            // ИЗМЕНЕНИЕ ЗДЕСЬ: Возвращаем результат self.update(), который сам является Task
                            return self.update(Message::SelectGroupForClasses(first_group));
                        }
                    }
                    Err(e) => {
                        eprintln!("Error loading teacher groups: {}", e);
                        // Обработка ошибки, возможно, показ сообщения
                        self.teacher_error_message = Some(format!("Ошибка загрузки групп: {}", e)); // Добавим сообщение об ошибке
                    }
                }
                // Если не было других Task (например, при ошибке или если нет групп), возвращаем Task::none()
                Task::none()
            }
            Message::LoadGroupProvenLessons => {
                if let Some(group) = &self.selected_group_for_classes {
                    let group_id = group.id;
                    return Task::perform(load_group_proven_lessons(group_id), Message::GroupProvenLessonsLoaded);
                }
                Task::none()
            }
            Message::NoOp => {Task::none()}
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
                    Screen::CoursesList => courses_screen(self),
                    Screen::UserList => user_list_screen(self),
                    Screen::GroupList => groups_screen(self),
                    Screen::Classes => classes_screen(self),
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
        self.user_birthday.clear();
        self.user_password_repeat.clear();
        self.register_error = None;
        self.registration_success = false;
    }

    // Функция для сохранения всех настроек
    pub fn save_config(app: &App) -> std::io::Result<()> {
        let config = Config {
            theme_name: theme_to_str(&app.theme).to_string(),
        };
        let json = serde_json::to_string_pretty(&config)?;
        fs::write(CONFIG_FILE, json)?;
        Ok(())
    }

    // Функция для загрузки всех настроек
    pub fn load_config() -> Config {
        let contents = fs::read_to_string(CONFIG_FILE).ok();
        contents.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| {
            // Возвращаем Config со значениями по умолчанию, если файл не найден или невалиден
            Config {
                theme_name: theme_to_str(&Theme::Dark).to_string(), // Тема по умолчанию
            }
        })
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

async fn load_teacher_groups(teacher_email: String) -> Result<Vec<Group>, String> {
    let conn = Connection::open("db_platform")
        .map_err(|e| format!("Failed to open database connection: {}", e))?;

    // Здесь мы используем ok_or_else, чтобы преобразовать Option в Result
    let teacher_id = db::get_user_id_by_email(&conn, &teacher_email)
        .ok_or_else(|| format!("Teacher with email '{}' not found.", teacher_email))?;

    db::get_groups_for_teacher(&conn, teacher_id)
        .map_err(|e| format!("Failed to load groups for teacher {}: {}", teacher_id, e))
}

async fn load_group_proven_lessons(group_id: i32) -> Result<Vec<ProvenLesson>, String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::get_proven_lessons_for_group(&conn, group_id).map_err(|e| e.to_string())
}

async fn load_teacher_assignments_for_proven_lesson(proven_lesson_id: i32) -> Result<Vec<Assignment>, String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::get_assignments_for_proven_lesson(&conn, proven_lesson_id).map_err(|e| e.to_string())
}

async fn update_assignment(assignment: Assignment) -> Result<(), String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::update_assignment(&conn, &assignment).map_err(|e| e.to_string())
}

async fn load_all_assignments() -> Result<Vec<Assignment>, String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::get_all_assignments(&conn).map_err(|e| e.to_string())
}

async fn add_existing_assignment_to_proven_lesson(proven_lesson_id: i32, assignment_id: i32) -> Result<(), String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::add_assignment_to_proven_lesson(&conn, proven_lesson_id, assignment_id).map_err(|e| e.to_string())
}

async fn delete_proven_lesson_assignment(proven_lesson_id: i32, assignment_id: i32) -> Result<(), String> {
    let conn = Connection::open("db_platform").map_err(|e| e.to_string())?;
    db::delete_assignment_from_proven_lesson(&conn, proven_lesson_id, assignment_id).map_err(|e| e.to_string())
}

