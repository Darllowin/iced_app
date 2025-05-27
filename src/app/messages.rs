use std::path::PathBuf;
use iced_aw::date_picker::Date;
use crate::app::state::{Assignment, AssignmentType, Certificate, Course, CoursePickListItem, Group, GroupPickListItem, GroupStatus, LessonWithAssignments, Level, PastSession, Payment, ReportType, StudentAttendance, StudentPickListItem, TextInputOrEditorInput, UserInfo};

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
    GoToPayment,
    GoToCertificates,
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
    AvatarChosen(Result<Vec<u8>, String>),
    //
    NewCourseLevelChanged(Level),
    ToggleAddCourseModal(bool),
    NewCourseTitleChanged(String),
    NewCourseDescriptionChanged(String),
    NewCourseTotalSeatsChanged(String),
    NewCourseSeatsChanged(String),
    NewCoursePriceChanged(String),

    SubmitNewCourse,
    DeleteCourse(i32),
    // Редактирование курса
    StartEditingCourse(Course),
    EditCourseTitleChanged(String),
    EditCourseDescriptionChanged(String),
    EditCourseLevelChanged(Level),
    EditCourseTotalSeatsChanged(String),
    EditCourseSeatsChanged(String),
    EditCoursePriceChanged(String),
    SubmitEditedCourse,
    CancelEditingCourse,
    // Редактирование пользователя
    StartEditingUser(UserInfo),
    CancelEditingUser,
    SubmitEditedUser,
    DeleteUser(String),
    UserDeleted(Result<String, String>),
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
    NewGroupCourseChanged(Option<Course>),
    NewGroupTeacherChanged(Option<UserInfo>),
    NewGroupStatusChanged(GroupStatus),

    EditGroupNameChanged(String),
    EditGroupCourseChanged(Option<Course>),
    EditGroupTeacherChanged(Option<UserInfo>),
    EditGroupStatusChanged(GroupStatus),

    SubmitNewGroup,
    SubmitEditedGroup,
    StartEditingGroup(Group),
    CancelEditingGroup,
    DeleteGroup(i32),
    GroupFilterChanged(String),

    OpenManageStudentsModal(i32),

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
    LoadTeacherGroups(i32),
    TeacherGroupsLoaded(Result<Vec<Group>, String>), // Result для обработки ошибок
    SelectGroupForClasses(Group),

    AssignmentsLoaded(Result<Vec<Assignment>, String>),

    // Cообщение для загрузки уроков с заданиями
    GroupLessonsWithAssignmentsLoaded(Result<Vec<LessonWithAssignments>, String>),
    // Сообщение для загрузки проведенных занятий (если будете их отображать)
    PastSessionsLoaded(Result<Vec<PastSession>, String>),
    //ConductLesson(i32, i32),

    CourseLessonsLoaded(Result<Vec<LessonWithAssignments>, String>),

    LoadAllCourses,
    AllCoursesLoaded(Result<Vec<Course>, String>), // Course должен быть импортирован

    ConductLessonResult(Result<Vec<PastSession>, String>), // Результат добавления и загрузки PastSessions

    OpenGroupLessonsModal(i32, i32), // group_id, course_id
    GroupLessonsModalLoaded(Result<(Vec<LessonWithAssignments>, Vec<PastSession>), String>), // (доступные уроки, пройденные уроки)
    CloseGroupLessonsModal,

    LoadAllGroups, // <-- НОВОЕ СООБЩЕНИЕ: Загрузить все группы
    StudentsInGroupLoaded(Result<Vec<UserInfo>, String>),
    StudentsWithoutGroupLoaded(Result<Vec<UserInfo>, String>),

    CoursesForPicklistLoaded(Result<Vec<Course>, String>),
    UsersForPicklistLoaded(Result<Vec<UserInfo>, String>),
    RemoveStudentFromGroup(i32, i32),
    ErrorOccurred(String),
    LoadStudentGroupInfo, // Для загрузки группы студента
    StudentGroupInfoLoaded(Result<Option<Group>, String>),
    ShowGroupStudents(i32), // Показать модальное окно с студентами группы (передаем group_id)
    //LoadGroupStudents(i32), // Сообщение для асинхронной загрузки студентов
    GroupStudentsLoaded(Result<(i32, Vec<UserInfo>), String>), // i32 - group_id, Vec<StudentInfo> - студенты
    CloseGroupStudentsModal, // Закрыть модальное окно

    AddStudentToGroup(i32, i32),
    SelectedStudentToAddChanged(UserInfo),
    StudentsAndGroupsReloaded(i32, i32), // (group_id, teacher_id)
    AllGroupsLoaded(Result<Vec<Group>, String>),
    // Payment
    PaymentsFetched(Result<Vec<Payment>, String>),
    ToggleAddPaymentModal,
    NewPaymentFormStudentSelected(StudentPickListItem), 
    NewPaymentFormCourseSelected(CoursePickListItem),   
    NewPaymentFormGroupSelected(GroupPickListItem),
    NewPaymentFormTypeChanged(String),
    AddPaymentConfirmed,
    PaymentAdded(Result<(), String>),
    PaymentsUpdated(Vec<Payment>),
    DeletePayment(i32),
    GroupsFetched(Result<Vec<Group>, String>),
    // Сообщения для получения данных в модальном окне
    StudentsWithoutGroupFetched(Result<Vec<UserInfo>, String>),
    CoursesWithSeatsFetched(Result<Vec<Course>, String>),
    GroupsForCourseFetched(Result<Vec<Group>, String>),
    NoOp,
    //
    ConductLessonClicked(i32, i32), // Старое ConductLesson, переименовано для ясности
    OpenConductLessonModal(i32, i32), // Новое: для вызова модального окна
    ToggleStudentAttendance(i32), // Для переключения чекбокса в модальном окне
    SaveAttendance, // Для сохранения посещаемости и проведенного занятия
    StudentsForAttendanceLoaded(Result<Vec<StudentAttendance>, String>), // Callback для загрузки студентов
    AttendanceSavedResult(Result<Vec<PastSession>, String>), // Callback после сохранения посещаемости

    CourseCompletionChecked(Result<(), String>), // Результат проверки завершения курса
    //CertificatesLoaded(Result<Vec<Certificate>, String>),
    // Изменено: теперь StudentsWithCertificatesLoaded принимает Vec<UserInfo>
    StudentsWithCertificatesLoaded(Result<Vec<UserInfo>, String>),

    // Изменено: OpenStudentCertificatesModal теперь принимает UserInfo
    OpenStudentCertificatesModal(UserInfo),
    StudentCertificatesLoaded(Result<Vec<Certificate>, String>),
    CloseStudentCertificatesModal,
    // Сообщение для генерации сертификата
    GenerateCertificatePdf(Certificate, UserInfo),
    CertificatePdfGenerated(Result<PathBuf, String>), // Результат генерации PDF: путь к файлу или ошибка
    //
    ToggleReportModal,
    GeneratePaymentReport,
    ChooseStartDate,
    ChooseEndDate,
    SubmitStartDate(Date),
    SubmitEndDate(Date),
    CancelDatePicker,
    ReportTypeSelected(Option<ReportType>),
    ReportGenerated(Result<PathBuf, String>),
    //
    ToggleCertificateReportModal,
    ChooseCertificateReportStartDate,
    ChooseCertificateReportEndDate,
    SubmitCertificateReportStartDate(Date), // или какой у тебя тип даты
    SubmitCertificateReportEndDate(Date),
    GenerateCertificateReport,
    CertificateReportGenerated(Result<PathBuf, String>),
    //
    ToggleGroupReportModal,
    GroupReportGenerated(Result<PathBuf, String>),
    GenerateGroupReport,
}