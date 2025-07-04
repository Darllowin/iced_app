use std::collections::HashMap;
use std::io::Cursor;
use chrono::NaiveDate;
use image::imageops::FilterType;
use image::ImageReader;
use rusqlite::{params, Connection, OptionalExtension, Result, Error, ffi, Transaction, params_from_iter};
use serde::de::StdError;
use tokio::task;
use crate::app::state::{Assignment, Certificate, Course, Group, GroupForReport, GroupStatus, LessonWithAssignments, PastSession, Payment, StudentAttendanceStatus, UserInfo, PATH_TO_DB};


pub async fn authenticate_and_get_user_data(
    email_input: String,
    hashed_password: String,
) -> std::result::Result<UserInfo, String> {
    task::spawn_blocking(move || {
        let conn = Connection::open(PATH_TO_DB)
            .map_err(|e| format!("Не удалось открыть базу данных: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT ID, Email, Name, AvatarData, Birthday, Type, password FROM Users WHERE Email = ?1")
            .map_err(|e| format!("Ошибка подготовки запроса: {}", e))?;

        let mut rows = stmt
            .query(params![email_input])
            .map_err(|e| format!("Ошибка выполнения запроса: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| format!("Ошибка получения строки: {}", e))? {
            // Все эти переменные теперь объявлены здесь, во внешней области видимости `if let Some(row) = ...`
            let id: i32 = row.get(0).map_err(|e| format!("Ошибка получения ID: {}", e))?;
            let email_from_db: String = row.get(1).map_err(|e| format!("Ошибка получения email: {}", e))?;
            let name: String = row.get(2).map_err(|e| format!("Ошибка получения имени: {}", e))?;
            let avatar_data: Option<Vec<u8>> = row.get(3).map_err(|e| format!("Ошибка получения аватара: {}", e))?;
            let birthday: String = row.get(4).map_err(|e| format!("Ошибка получения дня рождения: {}", e))?;
            let user_type: String = row.get(5).map_err(|e| format!("Ошибка получения типа пользователя: {}", e))?;
            let stored_hash: String = row.get(6).map_err(|e| format!("Ошибка получения хэша пароля: {}", e))?;

            // **ОБЪЯВЛЯЕМ group и child_count здесь, чтобы они были видны в конце блока**
            let group_name: Option<String>; // Используем group_name, как было в UserInfo.group
            let child_count: Option<i32>;

            if stored_hash == hashed_password {
                // ПРИСВАИВАЕМ ЗНАЧЕНИЯ group_name и child_count
                group_name = match user_type.as_str() {
                    "student" => db_get_group_name_for_student(&conn, id).unwrap_or_else(|e| {
                        eprintln!("Ошибка получения группы для студента {}: {}", id, e);
                        None
                    }),
                    "teacher" => db_get_group_name_for_teacher(&conn, id).unwrap_or_else(|e| {
                        eprintln!("Ошибка получения группы для учителя {}: {}", id, e);
                        None
                    }),
                    _ => None,
                };

                child_count = if user_type == "parent" {
                    db_get_child_count_for_parent(&conn, id).ok()
                } else {
                    None
                };

                // Теперь возвращаем UserInfo. Все переменные, включая group_name и child_count,
                // определены и доступны в этой области видимости.
                Ok(UserInfo {
                    id,
                    name,
                    email: email_from_db,
                    avatar_data,
                    birthday,
                    user_type,
                    group_id: group_name, // Используем group_name для поля `group` в UserInfo
                    child_count,
                })
            } else {
                // Если пароль неверен, никаких group_name или child_count не будет
                Err("Неверный пароль. Попробуйте снова.".to_string())
            }
        } else {
            // Если пользователь не найден
            Err("Пользователь с таким email не найден.".to_string())
        }
    })
        .await
        .map_err(|e| format!("Ошибка блокирующей задачи при аутентификации: {}", e))?
}

// Функция для получения имени группы пользователя (если он студент)
pub fn db_get_group_name_for_student(conn: &Connection, user_id: i32) -> Result<Option<String>, String> {
    // Внимание: если студент может быть в нескольких группах, этот запрос вернет только одну.
    // Если нужно все группы, тип возвращаемого значения должен быть Vec<String>.
    conn.query_row(
        "SELECT T2.name FROM GroupStudent AS T1 INNER JOIN 'Group' AS T2 ON T1.group_id = T2.id WHERE T1.student_id = ?1",
        params![user_id],
        |row| row.get::<_, String>(0),
    )
        .optional() // Преобразует Err(QueryReturnedNoRows) в Ok(None), остальные Ok(Some(value))
        .map_err(|e| format!("Ошибка получения имени группы для студента: {}", e))
}

// Функция для получения имени группы учителя (группы, которую он ведет)
pub fn db_get_group_name_for_teacher(conn: &Connection, user_id: i32) -> Result<Option<String>, String> {
    // Если учитель может вести несколько групп, этот запрос вернет только одну.
    conn.query_row(
        "SELECT name FROM 'Group' WHERE teacher_id = ?1 LIMIT 1", // LIMIT 1, чтобы получить только одну, если их несколько
        params![user_id],
        |row| row.get::<_, String>(0),
    )
        .optional()
        .map_err(|e| format!("Ошибка получения имени группы для учителя: {}", e))
}


// Функция для получения количества детей у родителя
pub fn db_get_child_count_for_parent(conn: &Connection, parent_id: i32) -> Result<i32, String> {
    conn.query_row(
        "SELECT COUNT(student_id) FROM ParentStudent WHERE parent_id = ?1",
        params![parent_id],
        |row| row.get::<_, i32>(0), // COUNT всегда возвращает i32 (0, если нет совпадений)
    )
        .map_err(|e| format!("Ошибка получения количества детей: {}", e))
}

// При регистрации нового пользователя AvatarData может быть NULL по умолчанию
pub fn register_user(conn: &Connection, full_name: &str, birthday: &str, email: &str, password_hash: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Users (Name, Type, Birthday, Email, password, AvatarData) VALUES (?1, 'unconfirmed', ?2, ?3, ?4, NULL)",
        params![full_name, birthday, email, password_hash],
    )?;
    Ok(())
}

// Обновлено для приема avatar_data как &[u8]
pub fn update_user_avatar(conn: &Connection, email: &str, raw_image_data: &[u8]) -> Result<()> {
    // Вспомогательная функция для преобразования произвольных ошибок в rusqlite::Error::SqliteFailure
    fn map_image_error_to_sqlite_failure(e: impl StdError + 'static) -> Error {
        Error::SqliteFailure(
            ffi::Error {
                // ИСПРАВЛЕНО: Используем вариант перечисления ErrorCode::SQLITE_ERROR
                code: ffi::ErrorCode::OperationAborted,
                // extended_code является i32, так что 1 здесь подходит
                extended_code: 1,
            },
            Some(format!("Ошибка обработки изображения: {}", e)),
        )
    }

    // 1. Декодируем исходное изображение
    let img = ImageReader::new(Cursor::new(raw_image_data))
        .with_guessed_format()
        // *** ИСПРАВЛЕНО: Используем новую вспомогательную функцию ***
        .map_err(map_image_error_to_sqlite_failure)?
        .decode()
        // *** ИСПРАВЛЕНО: Используем новую вспомогательную функцию ***
        .map_err(map_image_error_to_sqlite_failure)?;

    // 2. Изменяем размер изображения до нужного (например, 220x220)
    let target_size = 220;
    let resized_img = img.resize(target_size, target_size, FilterType::Lanczos3);

    // 3. Кодируем изображение обратно в сжатый формат (например, PNG)
    let mut compressed_data = Vec::new();
    resized_img.write_to(&mut Cursor::new(&mut compressed_data), image::ImageFormat::Png)
        // *** ИСПРАВЛЕНО: Используем новую вспомогательную функцию ***
        .map_err(map_image_error_to_sqlite_failure)?;

    // 4. Сохраняем сжатые/измененные данные в БД
    conn.execute(
        "UPDATE Users SET AvatarData = ? WHERE Email = ?",
        (&compressed_data, email),
    )?;
    Ok(())
}

pub fn get_courses(conn: &Connection) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare("
        SELECT
            C.ID,
            C.title,
            C.description,
            C.level,
            COUNT(L.ID) AS LessonCount,
            C.total_seats,
            C.seats,
            C.price
        FROM Course C
        LEFT JOIN Lessons L ON C.ID = L.course_id
        GROUP BY C.ID, C.title, C.description, C.level -- Убран U.Name
        ORDER BY C.title
    ")?;

    let course_iter = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get("ID")?,
            title: row.get("title")?,
            description: row.get("description")?,
            level: row.get("level").ok(),
            lesson_count: row.get("LessonCount")?,
            total_seats: row.get("total_seats")?,
            seats: row.get("seats")?,
            price: row.get("price")?,
        })
    })?;

    Ok(course_iter.collect::<Result<Vec<_>>>()?)
}

pub fn add_course(conn: &Connection, title: &str, description: &str, level: &String, seats: i32, price: f64, total_seats: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO Course (title, description, level, seats, price, total_seats) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            title,
            description,
            level,
            seats,
            price,
            total_seats
        ],
    )?;
    Ok(())
}

pub fn delete_course(conn: &Connection, course_id: i32) -> Result<()> {
    let mut stmt = conn.prepare("DELETE FROM Course WHERE id = ?")?;
    stmt.execute([course_id])?;
    Ok(())
}

pub fn get_all_users(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("
        SELECT
            ID,          
            Name,        
            Email,       
            Birthday,    
            Type,        
            AvatarData  
        FROM Users
        ORDER BY Name
    ")?;

    let users_result: Result<Vec<UserInfo>> = stmt.query_map(params![], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?, // Используем "Type" из БД
            avatar_data: row.get("AvatarData")?,
            group_id: None,         // Устанавливаем в None, так как колонки нет
            child_count: None,   // Устанавливаем в None, так как колонки нет
        })
    })?.collect();

    match users_result {
        Ok(users) => {
            println!("DEBUG (db::get_all_users): Загружено {} пользователей.", users.len());
            if users.is_empty() {
                println!("DEBUG (db::get_all_users): Список пользователей пуст. Проверьте данные в таблице Users.");
            }
            Ok(users)
        },
        Err(e) => {
            eprintln!("ERROR (db::get_all_users): Ошибка при загрузке пользователей: {}", e);
            Err(e)
        }
    }
}
pub fn update_course(conn: &Connection, course: &Course) -> Result<()> {
    conn.execute(
        "UPDATE Course SET title = ?1, description = ?2, level = ?3, total_seats = ?4, seats = ?5, price = ?6 WHERE ID = ?7",
        params![
            course.title,
            course.description, 
            course.level,       
            course.total_seats, 
            course.seats,      
            course.price,      
            course.id
        ],
    )?;
    Ok(())
}
pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    println!("DEBUG DB: Попытка загрузить ВСЕ группы.");
    let mut stmt = conn.prepare("
        SELECT
            G.id,
            G.name,
            G.course_id,
            G.teacher_id,
            C.title AS course_name,
            U.Name AS teacher_name,
            G.student_count,
            G.status
        FROM \"Group\" G
        LEFT JOIN \"Course\" C ON G.course_id = C.ID
        LEFT JOIN Users U ON G.teacher_id = U.ID
        ORDER BY G.name
    ")?;

    let groups_iter = stmt.query_map(params![], |row| {
        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            course_id: row.get("course_id")?,
            teacher_id: row.get("teacher_id")?,
            course_name: row.get("course_name")?,
            teacher_name: row.get("teacher_name")?,
            student_count: row.get("student_count")?,
            status: row.get("status")?,
        })
    })?;

    let groups: Vec<Group> = groups_iter.collect::<Result<Vec<_>, Error>>()?;
    println!("DEBUG DB: Загружено ВСЕХ групп: {} шт.", groups.len());
    Ok(groups)
}
pub fn get_teacher_groups_with_details(conn: &Connection, teacher_id: i32) -> Result<Vec<Group>> {
    println!("DEBUG DB: Попытка загрузить группы для teacher_id: {}", teacher_id);
    let mut stmt = conn.prepare("
        SELECT
            G.id,
            G.name,
            G.course_id,
            G.teacher_id,
            C.title AS course_name,
            U.Name AS teacher_name,
            G.student_count,
            G.status
        FROM \"Group\" G
        JOIN \"Course\" C ON G.course_id = C.ID
        JOIN Users U ON G.teacher_id = U.ID
        WHERE G.teacher_id = ?1
        ORDER BY G.name
    ")?;

    let groups_iter = stmt.query_map(params![teacher_id], |row| {
        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            course_id: row.get("course_id")?,
            teacher_id: row.get("teacher_id")?,
            course_name: row.get("course_name")?,
            teacher_name: row.get("teacher_name")?,
            student_count: row.get("student_count")?,
            status: row.get("status")?,
        })
    })?;

    let groups: Vec<Group> = groups_iter.collect::<Result<Vec<_>, Error>>()?;
    println!("DEBUG DB: Загружено групп: {} шт.", groups.len());
    Ok(groups)
}
pub fn get_lessons_for_course_and_group(conn: &Connection, course_id: i32, group_id: i32) -> Result<Vec<LessonWithAssignments>> {
    println!("DEBUG DB: Загрузка уроков для курса {} и группы {}", course_id, group_id);

    // 1. Загружаем основные данные уроков
    let mut lessons_stmt = conn.prepare("
        SELECT
            L.ID,
            L.course_id,
            L.number,
            L.title
        FROM Lessons L
        WHERE L.course_id = ?1
        AND L.ID NOT IN (
            SELECT PS.lesson_id
            FROM PastSessions PS
            WHERE PS.group_id = ?2
        )
        ORDER BY L.number
    ")?;

    let lessons_iter = lessons_stmt.query_map(params![course_id, group_id], |row| {
        Ok(LessonWithAssignments {
            id: row.get("ID")?,
            course_id: row.get("course_id")?,
            number: row.get("number")?,
            title: row.get("title")?,
            assignments: Vec::new(), // Пока оставляем пустым, заполним позже
        })
    })?;

    let mut lessons_map: HashMap<i32, LessonWithAssignments> = lessons_iter
        .map(|res| res.map(|lesson| (lesson.id, lesson)))
        .collect::<Result<HashMap<i32, LessonWithAssignments>, Error>>()?;

    println!("DEBUG DB: Загружено уроков: {}", lessons_map.len());

    // 2. Загружаем все задания, которые относятся к этим урокам
    // Мы можем загрузить все задания, а затем отфильтровать их по lesson_id
    // Или сделать JOIN с Lessons, чтобы получить только нужные
    let mut assignments_stmt = conn.prepare("
        SELECT
            A.ID,
            A.lesson_id,
            A.title,
            A.description,
            A.type AS assignment_type -- Убедитесь, что имя колонки 'type'
        FROM Assignment A
        WHERE A.lesson_id IN (SELECT ID FROM Lessons WHERE course_id = ?1)
        ORDER BY A.lesson_id, A.ID
    ")?;

    let assignments_iter = assignments_stmt.query_map(params![course_id], |row| {
        Ok(Assignment {
            id: row.get("ID")?,
            lesson_id: row.get("lesson_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            assignment_type: row.get("assignment_type")?,
        })
    })?;

    println!("DEBUG DB: Загрузка заданий...");
    let assignments: Vec<Assignment> = assignments_iter.collect::<Result<Vec<_>, Error>>()?;
    println!("DEBUG DB: Загружено заданий: {} шт.", assignments.len());

    // 3. Распределяем задания по соответствующим урокам
    for assignment in assignments {
        if let Some(lesson) = lessons_map.get_mut(&assignment.lesson_id) {
            lesson.assignments.push(assignment);
        }
    }

    // 4. Преобразуем HashMap обратно в Vec<LessonWithAssignments>, отсортировав по номеру урока
    let mut final_lessons: Vec<LessonWithAssignments> = lessons_map.into_values().collect();
    final_lessons.sort_by_key(|l| l.number);

    println!("DEBUG DB: Уроки с заданиями готовы.");
    Ok(final_lessons)
}
pub fn get_lessons_for_course(conn: &Connection, course_id_val: i32) -> Result<Vec<LessonWithAssignments>> {
    let mut stmt = conn.prepare(
        "SELECT ID, course_id, number, title FROM Lessons WHERE course_id = ?1 ORDER BY number"
    )?;
    let lessons_iter = stmt.query_map(params![course_id_val], |row| {
        Ok(LessonWithAssignments {
            id: row.get(0)?,
            course_id: row.get(1)?,
            number: row.get(2)?,
            title: row.get(3)?,
            assignments: Vec::new(), // Изначально пустой Vec, будет заполнен позже
        })
    })?;
    lessons_iter.collect()
}

// Добавить новое занятие
pub fn add_lesson(conn: &Connection, course_id: i32, number: Option<i32>, title: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Lessons (course_id, number, title) VALUES (?1, ?2, ?3)",
        params![course_id, number, title],
    )?;
    Ok(())
}

// Удалить занятие по ID
pub fn delete_lesson(conn: &Connection, lesson_id: i32) -> Result<()> {
    conn.execute("DELETE FROM Lessons WHERE ID = ?1", params![lesson_id])?;
    Ok(())
}
pub fn get_all_users_for_list(conn: &Connection, user_type_filter: Option<&str>) -> Result<Vec<UserInfo>> {
    let mut query = "
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData,
            -- Groups for students
            CASE WHEN U.Type = 'student' THEN GROUP_CONCAT(GS_student.name, ', ') ELSE NULL END AS StudentGroups,
            -- Groups for teachers (newly added)
            CASE WHEN U.Type = 'teacher' THEN GROUP_CONCAT(G_teacher.name, ', ') ELSE NULL END AS TeacherGroups,
            COUNT(PS.student_id) AS ChildCount
        FROM Users U
        LEFT JOIN GroupStudent GSS ON U.ID = GSS.student_id -- For students' groups
        LEFT JOIN \"Group\" GS_student ON GSS.group_id = GS_student.id -- For students' group names
        LEFT JOIN \"Group\" G_teacher ON U.ID = G_teacher.teacher_id -- For teachers' groups (new join)
        LEFT JOIN ParentStudent PS ON U.ID = PS.parent_id
    ".to_string();

    let mut owned_params: Vec<String> = Vec::new();
    let mut params_refs: Vec<&dyn rusqlite::ToSql> = Vec::new();

    if let Some(filter_type) = user_type_filter {
        query.push_str(" WHERE U.Type = ?1");
        owned_params.push(filter_type.to_string());
        params_refs.push(&owned_params[0]);
    }

    query.push_str("
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        ORDER BY U.Name
    ");

    let mut stmt = conn.prepare(&query)?;

    let user_iter = stmt.query_map(params_refs.as_slice(), |row| {
        let user_type: String = row.get("Type")?;

        // Determine the 'group' field based on user_type
        let group_info = if user_type == "student" {
            row.get("StudentGroups").ok()
        } else if user_type == "teacher" { // <--- Get TeacherGroups here
            row.get("TeacherGroups").ok()
        } else {
            None
        };

        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: user_type.clone(),
            avatar_data: row.get("AvatarData").ok(),
            group_id: group_info, // <--- Assign the determined group_info
            child_count: if user_type == "parent" { Some(row.get("ChildCount")?) } else { None },
        })
    })?;
    user_iter.collect()
}
pub fn update_user(
    conn: &Connection,
    original_email: &str,
    new_name: &str,
    new_email: &str,
    birthday: &str,
    user_type: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE Users SET Name = ?, Email = ?, Birthday = ?, Type = ? WHERE Email = ?",
        (new_name, new_email, birthday, user_type, original_email),
    )?;
    Ok(())
}

pub fn delete_user(conn: &Connection, email: &str) -> Result<()> {
    conn.execute("DELETE FROM Users WHERE Email = ?", (email,))?;
    Ok(())
}
pub fn is_email_taken_except(conn: &Connection, email: &str, exclude_email: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM Users WHERE Email = ?1 AND Email != ?2")?;
    let count: i64 = stmt.query_row((email, exclude_email), |row| row.get(0))?;
    Ok(count > 0)
}
pub fn is_email_taken(conn: &Connection, email: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM Users WHERE Email = ?1")?;
    let count: i64 = stmt.query_row([email], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn get_student_group_by_user_id(conn: &Connection, user_id: i32) -> Result<Option<Group>> {
    println!("DEBUG DB: Попытка загрузить группу для студента user_id: {}", user_id);
    let mut stmt = conn.prepare("
        SELECT
            G.id,
            G.name,
            G.course_id,
            G.teacher_id,
            C.title AS course_name,
            U.Name AS teacher_name,
            G.student_count, 
            G.status
        FROM \"Group\" G
        JOIN GroupStudent GS ON G.id = GS.group_id
        JOIN Users U ON G.teacher_id = U.ID
        JOIN Course C ON G.course_id = C.ID
        WHERE GS.student_id = ?1
    ")?;

    let group_opt = stmt.query_row(params![user_id], |row| {
        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            course_id: row.get("course_id")?,
            teacher_id: row.get("teacher_id")?,
            course_name: row.get("course_name")?,
            teacher_name: row.get("teacher_name")?,
            student_count: row.get("student_count")?,
            status: row.get("status")?,
        })
    }).optional()?;

    if group_opt.is_some() {
        println!("DEBUG DB: Группа студента загружена.");
    } else {
        println!("DEBUG DB: Студент не найден в группе.");
    }
    Ok(group_opt)
}
pub fn get_students_without_group(conn: &Connection) -> Result<Vec<UserInfo>> {
    println!("DEBUG DB: Загрузка студентов без группы...");
    let mut stmt = conn.prepare("
        SELECT
            U.ID,
            U.Name,
            U.Email,
            U.Birthday,
            U.Type,
            U.AvatarData
        FROM Users U
        WHERE U.Type = 'student'
        AND U.ID NOT IN (SELECT student_id FROM GroupStudent) -- ИСПРАВЛЕНО ЗДЕСЬ: student_id
        ORDER BY U.Name
    ")?;

    let students_iter = stmt.query_map(params![], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?,
            avatar_data: row.get("AvatarData")?,
            group_id: None,
            child_count: None,
        })
    })?;

    let students: Vec<UserInfo> = students_iter.collect::<Result<Vec<_>, Error>>()?;
    println!("DEBUG DB: Загружено студентов без группы: {} шт.", students.len());
    Ok(students)
}
pub fn insert_group(conn: &Connection, name: &str, course_id: i32, teacher_id: i32, status: GroupStatus) -> Result<()> {
    conn.execute(
        "INSERT INTO `Group` (name, course_id, teacher_id, student_count, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, course_id, teacher_id, 0, status],
    )?;
    Ok(())
}

pub fn update_group(conn: &Connection, id: i32, name: &str, course_id: i32, teacher_id: i32, status: GroupStatus) -> Result<()> {
    conn.execute(
        "UPDATE \"Group\" SET name = ?, course_id = ?, teacher_id = ?, status = ? WHERE id = ?",
        params![name, course_id, teacher_id, status, id],
    )?;
    Ok(())
}

pub fn delete_group(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM \"Group\" WHERE id = ?", params![id])?;
    Ok(())
}

pub fn add_student_to_group(conn: &mut Connection, student_id: i32, group_id: i32) -> Result<()> {
    println!("DEBUG DB: Добавление студента ID: {} в группу ID: {}", student_id, group_id);

    let tx = conn.transaction()?;

    // 1. Добавляем студента в GroupStudent
    tx.execute(
        "INSERT INTO GroupStudent (student_id, group_id) VALUES (?1, ?2)",
        params![student_id, group_id],
    )?;

    // 2. Пересчитываем количество студентов в группе
    let new_student_count: i32 = tx.query_row(
        "SELECT COUNT(*) FROM GroupStudent WHERE group_id = ?1",
        params![group_id],
        |row| row.get(0),
    )?;

    // 3. Обновляем student_count в таблице "Group"
    // ИСПРАВЛЕНО: Добавлены кавычки вокруг "Group"
    tx.execute(
        "UPDATE \"Group\" SET student_count = ?1 WHERE id = ?2",
        params![new_student_count, group_id],
    )?;

    tx.commit()?;

    println!("DEBUG DB: student_count для группы ID {} обновлен до {}", group_id, new_student_count);
    Ok(())
}

pub fn remove_student_from_group(conn: &mut Connection, student_id: i32, group_id: i32) -> Result<()> {
    println!("DEBUG DB: Удаление студента ID: {} из группы ID: {}", student_id, group_id);

    // Начало транзакции для атомарности операций
    let tx = conn.transaction()?;

    // 1. Удаляем студента из GroupStudent
    tx.execute(
        "DELETE FROM GroupStudent WHERE student_id = ?1 AND group_id = ?2",
        params![student_id, group_id],
    )?;

    // 2. Пересчитываем количество студентов в группе
    let new_student_count: i32 = tx.query_row(
        "SELECT COUNT(*) FROM GroupStudent WHERE group_id = ?1",
        params![group_id],
        |row| row.get(0),
    )?;
    // 3. Обновляем student_count в таблице "Group"
    // ИСПРАВЛЕНО: Добавлены кавычки вокруг "Group"
    tx.execute(
        "UPDATE \"Group\" SET student_count = ?1 WHERE id = ?2",
        params![new_student_count, group_id],
    )?;

    // Коммитим транзакцию
    tx.commit()?;

    println!("DEBUG DB: student_count для группы ID {} обновлен до {}", group_id, new_student_count);
    Ok(())
}


pub fn get_students_in_group(conn: &Connection, group_id: i32) -> Result<Vec<UserInfo>> {
    println!("DEBUG DB: Загрузка студентов для group_id: {}", group_id);
    let mut stmt = conn.prepare("
        SELECT
            U.ID,
            U.Name,
            U.Email,
            U.Birthday,
            U.Type,
            U.AvatarData
        FROM Users U
        JOIN GroupStudent GS ON U.ID = GS.student_id 
        WHERE GS.group_id = ?1
        ORDER BY U.Name
    ")?;

    let students_iter = stmt.query_map(params![group_id], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?,
            avatar_data: row.get("AvatarData")?,
            group_id: None,
            child_count: None,
        })
    })?;

    let students: Vec<UserInfo> = students_iter.collect::<Result<Vec<_>, Error>>()?;
    println!("DEBUG DB: Загружено студентов в группе: {} шт.", students.len());
    Ok(students)
}
pub fn get_user_id_by_email(conn: &Connection, email: &str) -> Option<i32> {
    let mut stmt = conn.prepare("SELECT ID FROM Users WHERE Email = ?1").ok()?;
    let mut rows = stmt.query([email]).ok()?;
    rows.next().ok().flatten().map(|row| row.get(0).ok()).flatten()
}
pub fn get_children_for_parent(conn: &Connection, parent_email: &str) -> Result<Vec<UserInfo>, Error> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData,
            GROUP_CONCAT(G.name, ', ') AS StudentGroups
        FROM Users U
        JOIN ParentStudent PS ON U.ID = PS.student_id
        JOIN Users P ON PS.parent_id = P.ID
        LEFT JOIN GroupStudent GS ON U.ID = GS.student_id
        LEFT JOIN \"Group\" G ON GS.group_id = G.id
        WHERE P.email = ?1
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        ORDER BY U.Name
    ")?;

    let children_iter = stmt.query_map([parent_email], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?, // Должен быть 'student'
            avatar_data: row.get("AvatarData").ok(),
            group_id: row.get("StudentGroups").ok(),
            child_count: None, // <-- Добавляем инициализацию поля количества детей
        })
    })?;

    children_iter.collect()
}

pub fn delete_child_for_parent(conn: &Connection, parent_email: &str, child_email: &str) -> Result<(), Error> {
    conn.execute(
        "DELETE FROM ParentStudent
         WHERE parent_id = (SELECT ID FROM Users WHERE Email = ?1)
         AND student_id = (SELECT ID FROM Users WHERE Email = ?2)",
        [parent_email, child_email],
    )?;
    Ok(())
}
pub fn get_unassigned_children(conn: &Connection) -> Result<Vec<UserInfo>, Error> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData,
            GROUP_CONCAT(G.name, ', ') AS StudentGroups
        FROM Users U
        LEFT JOIN GroupStudent GS ON U.ID = GS.student_id
        LEFT JOIN \"Group\" G ON GS.group_id = G.id
        WHERE U.Type = 'student'
          AND U.ID NOT IN (SELECT student_id FROM ParentStudent)
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        ORDER BY U.Name
    ")?;

    let children = stmt
        .query_map([], |row| {
            Ok(UserInfo {
                id: row.get("ID")?,
                name: row.get("Name")?,
                email: row.get("Email")?,
                birthday: row.get("Birthday")?,
                user_type: row.get("Type")?, // Должен быть 'student'
                avatar_data: row.get("AvatarData").ok(),
                group_id: row.get("StudentGroups").ok(),
                child_count: None, // <-- Добавляем инициализацию поля количества детей
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(children)
}

pub fn add_child_to_parent(conn: &Connection, parent_email: &str, child_email: &str) -> Result<(), Error> {
    let parent_id: i32 = conn.query_row("SELECT ID FROM Users WHERE Email = ?", [parent_email], |row| row.get(0))?;
    let child_id: i32 = conn.query_row("SELECT ID FROM Users WHERE Email = ?", [child_email], |row| row.get(0))?;
    conn.execute("INSERT INTO ParentStudent (parent_id, student_id) VALUES (?, ?)", [parent_id, child_id])?;
    Ok(())
}
pub fn get_assignments_for_lesson(conn: &Connection, lesson_id_val: i32) -> Result<Vec<Assignment>> {
    let mut stmt = conn.prepare(
        "SELECT id, lesson_id, title, description, type FROM Assignment WHERE lesson_id = ?1 ORDER BY id"
    )?;
    let assignment_iter = stmt.query_map(params![lesson_id_val], |row| {
        Ok(Assignment {
            id: row.get(0)?,
            lesson_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            assignment_type: row.get(4)?, // Колонка в БД называется "type"
        })
    })?;
    assignment_iter.collect()
}

pub fn add_assignment(conn: &Connection, lesson_id_val: i32, title_val: &str, description_val: &str, type_val: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Assignment (lesson_id, title, description, type) VALUES (?1, ?2, ?3, ?4)",
        params![lesson_id_val, title_val, description_val, type_val],
    )?;
    Ok(())
}

pub fn delete_assignment(conn: &Connection, assignment_id_val: i32) -> Result<()> {
    conn.execute("DELETE FROM Assignment WHERE id = ?1", params![assignment_id_val])?;
    Ok(())
}
pub fn update_assignment(conn: &Connection, assignment: &Assignment) -> Result<()> {
    let rows_affected = conn.execute(
        "UPDATE Assignment SET title = ?1, description = ?2 WHERE id = ?3",
        params![assignment.title, assignment.description, assignment.id],
    )?;
    if rows_affected == 0 {
        // Можно вернуть ошибку, если задание с таким ID не найдено
        eprintln!("Попытка обновить несуществующее задание с ID: {}", assignment.id);
        // return Err(rusqlite::Error::QueryReturnedNoRows); // Пример ошибки
    }
    Ok(())
}
pub fn get_groups_for_teacher(conn: &Connection, teacher_id: i32) -> Result<Vec<Group>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            G.id,
            G.name,
            G.course_id,       -- Включаем ID курса
            C.title AS course_title, -- Название курса
            G.teacher_id,      -- Включаем ID учителя
            U_teacher.name AS teacher_name, -- Имя учителя
            COUNT(GS.student_id) AS student_count,
            G.status
        FROM \"Group\" G
        LEFT JOIN Course C ON G.course_id = C.id
        LEFT JOIN Users U_teacher ON G.teacher_id = U_teacher.id
        LEFT JOIN GroupStudent GS ON G.id = GS.group_id
        WHERE G.teacher_id = ?1
        GROUP BY G.id, G.name, G.course_id, C.title, G.teacher_id, U_teacher.name
        ORDER BY G.name
        "
    )?;

    let groups = stmt.query_map(params![teacher_id], |row| {
        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            course_id: row.get("course_id")?,       // Получаем ID курса
            course_name: row.get("course_title")?, // Получаем название курса
            teacher_id: row.get("teacher_id")?,      // Получаем ID учителя
            teacher_name: row.get("teacher_name")?, // Получаем имя учителя
            student_count: row.get("student_count")?,
            status: row.get("status")?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    println!("DEBUG: Количество загруженных групп для учителя (ID: {}) в get_groups_for_teacher: {}", teacher_id, groups.len());
    for group in &groups {
        println!("DEBUG: Загруженная группа для учителя (с именами): {:?}", group);
    }

    Ok(groups)
}
pub fn get_past_sessions_for_group(conn: &Connection, group_id: i32) -> Result<Vec<PastSession>> {
    // 1. Загружаем основные данные прошедших сессий
    let mut stmt_sessions = conn.prepare("
        SELECT
            PS.id,
            PS.group_id,
            PS.date,
            PS.lesson_id,
            L.title AS lesson_title,
            L.number AS lesson_number
        FROM PastSessions PS
        JOIN Lessons L ON PS.lesson_id = L.ID
        WHERE PS.group_id = ?1
        ORDER BY PS.date DESC
    ")?;

    let sessions_iter = stmt_sessions.query_map(params![group_id], |row| {
        Ok(PastSession {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            date: row.get("date")?,
            lesson_id: row.get("lesson_id")?,
            lesson_title: row.get("lesson_title")?,
            lesson_number: row.get("lesson_number")?,
            attendance_records: Vec::new(), // Инициализируем пустым, заполним позже
        })
    })?;

    let mut past_sessions_map: HashMap<i32, PastSession> = sessions_iter
        .map(|res| res.map(|session| (session.id, session)))
        .collect::<Result<HashMap<i32, PastSession>, Error>>()?;

    // 2. Загружаем данные о посещаемости для всех этих прошедших сессий
    // Используем `IN` для фильтрации по ID сессий
    if past_sessions_map.is_empty() {
        return Ok(Vec::new()); // Если нет прошедших сессий, сразу возвращаем пустой вектор
    }

    let session_ids: Vec<i32> = past_sessions_map.keys().cloned().collect();
    let query_placeholders = session_ids.iter().map(|_| "?").collect::<Vec<&str>>().join(",");

    let query_attendance = format!("
        SELECT
            A.id,
            A.lesson_id AS past_session_id, -- lesson_id в Attendance фактически ссылается на PastSession.id
            A.student_id,
            U.Name AS student_name,
            A.present AS present_status
        FROM Attendance A
        JOIN Users U ON A.student_id = U.ID
        WHERE A.lesson_id IN ({}) -- Используем PastSession.id, т.к. в схеме Attendance.lesson_id ссылается на PastSessions.id
        ORDER BY A.lesson_id, U.Name
    ", query_placeholders);

    let mut stmt_attendance = conn.prepare(&query_attendance)?;

    // Преобразуем Vec<i32> в Vec<rusqlite::params::Value> для использования в query_map
    let attendance_iter = stmt_attendance.query_map(params_from_iter(session_ids.iter()), |row| {
        Ok((
            row.get("past_session_id")?, // ID PastSession, к которой относится эта запись
            StudentAttendanceStatus {
                student_id: row.get("student_id")?,
                student_name: row.get("student_name")?,
                present_status: row.get("present_status")?,
            }
        ))
    })?;

    // 3. Распределяем записи о посещаемости по соответствующим PastSession
    for res in attendance_iter {
        let (past_session_id, attendance_record) = res?;
        if let Some(session) = past_sessions_map.get_mut(&past_session_id) {
            session.attendance_records.push(attendance_record);
        }
    }

    // 4. Преобразуем HashMap обратно в Vec<PastSession>
    let mut final_sessions: Vec<PastSession> = past_sessions_map.into_values().collect();
    // Сохраняем исходный порядок, если это важно (например, по дате DESC)
    final_sessions.sort_by_key(|s| s.date.clone()); // или по id, или по дате, как вам нужно
    final_sessions.reverse(); // Чтобы сохранить убывающий порядок по дате, если `sort_by_key` его отменил

    Ok(final_sessions)
}
pub fn add_past_session(conn: &Connection, group_id: i32, lesson_id: i32) -> Result<i32> {
    let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string(); // Формат ГГГГ-ММ-ДД ЧЧ:ММ:СС

    conn.execute(
        "INSERT INTO PastSessions (group_id, date, lesson_id) VALUES (?1, ?2, ?3)",
        params![group_id, date_str, lesson_id],
    )?;
    Ok(conn.last_insert_rowid() as i32) // Возвращаем ID
}
pub fn get_all_payments_with_details(conn: &Connection) -> Result<Vec<Payment>> {
    let mut stmt = conn.prepare("
        SELECT
            P.id,
            P.student_id,
            P.date,
            P.amount,
            P.type,
            P.course_id,
            P.group_id,
            U.Name AS student_name,
            C.title AS course_title,
            G.name AS group_name
        FROM Payment P
        JOIN Users U ON P.student_id = U.ID
        JOIN Course C ON P.course_id = C.ID
        JOIN \"Group\" G ON P.group_id = G.id
        ORDER BY P.date DESC
    ")?;

    let payments_iter = stmt.query_map(params![], |row| {
        Ok(Payment {
            id: row.get("id")?,
            student_id: row.get("student_id")?,
            date: row.get("date")?,
            amount: row.get("amount")?,
            payment_type: row.get("type")?,
            course_id: row.get("course_id")?,
            group_id: row.get("group_id")?,
            student_name: row.get("student_name")?,
            course_title: row.get("course_title")?,
            group_name: row.get("group_name")?,
        })
    })?;

    payments_iter.collect::<Result<Vec<_>, Error>>()
}

// получить студентов, которые не состоят ни в одной группе
pub fn get_students_not_in_any_group(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        FROM Users U
        WHERE U.Type = 'student' AND U.ID NOT IN (SELECT student_id FROM GroupStudent)
        ORDER BY U.Name
    ")?;

    let users_iter = stmt.query_map(params![], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?,
            avatar_data: row.get("AvatarData").ok(),
            group_id: None,
            child_count: None,
        })
    })?;

    users_iter.collect::<Result<Vec<_>, Error>>()
}

// получить курсы со свободными местами
pub fn get_courses_with_available_seats(conn: &Connection) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare("
        SELECT
            ID,
            title,
            description,
            level,
            total_seats,
            seats, -- <--- Просто выбираем существующий столбец seats
            price
        FROM Course
        ORDER BY title
    ")?;

    let courses_iter = stmt.query_map(params![], |row| {
        Ok(Course {
            id: row.get("ID")?,
            title: row.get("title")?,
            description: row.get("description")?,
            level: row.get("level")?,
            total_seats: row.get("total_seats")?,
            seats: row.get("seats")?, // <--- Получаем значение из БД
            price: row.get("price")?,
            lesson_count: 0,
        })
    })?;
    courses_iter.collect::<Result<Vec<_>>>()
}

// получить группы по course_id
pub fn get_groups_by_course_id(conn: &Connection, course_id: i32) -> Result<Vec<Group>> {
    let mut stmt = conn.prepare("
        SELECT
            G.id,
            G.course_id,
            G.teacher_id,
            G.name,
            G.student_count,
            U.Name AS teacher_name,
            G.status
        FROM \"Group\" G
        LEFT JOIN Users U ON G.teacher_id = U.ID
        WHERE G.course_id = ?1
        ORDER BY G.name
    ")?;

    let groups_iter = stmt.query_map(params![course_id], |row| {
        Ok(Group {
            id: row.get("id")?,
            course_id: row.get("course_id")?,
            teacher_id: row.get("teacher_id")?,
            name: row.get("name")?,
            student_count: row.get("student_count")?,
            course_name: None, // Не получаем здесь название курса
            teacher_name: row.get("teacher_name").ok(),
            status: row.get("status")?,
        })
    })?;
    groups_iter.collect::<Result<Vec<_>>>()
}

// добавить платеж
pub fn add_payment(
    conn: &Connection,
    student_id: i32,
    date: &str,
    amount: f64,
    payment_type: &str,
    course_id: i32,
    group_id: i32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO Payment (student_id, date, amount, type, course_id, group_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![student_id, date, amount, payment_type, course_id, group_id],
    )?;
    Ok(())
}
pub fn delete_payment(conn: &Connection, course_id: i32) -> Result<()> {
    let mut stmt = conn.prepare("DELETE FROM Payment WHERE id = ?")?;
    stmt.execute([course_id])?;
    Ok(())
}
pub fn load_payments(conn: &Connection) -> Result<Vec<Payment>> {
    let mut stmt = conn.prepare("
        SELECT
            p.id,
            p.student_id,
            u.name AS student_name,
            p.course_id,
            c.title AS course_title,
            p.group_id,
            g.name AS group_name,
            p.date,
            p.amount,
            p.type
        FROM Payment p
        JOIN Users u ON p.student_id = u.id
        JOIN Course c ON p.course_id = c.id
        JOIN \"Group\" g ON p.group_id = g.id
    ")?;


    let payments = stmt.query_map([], |row| {
        Ok(Payment {
            id: row.get(0)?,
            student_id: row.get(1)?,
            student_name: row.get(2)?,
            course_id: row.get(3)?,
            course_title: row.get(4)?,
            group_id: row.get(5)?,
            group_name: row.get(6)?,
            date: row.get(7)?,
            amount: row.get(8)?,
            payment_type: row.get(9)?,
        })
    })?
        .filter_map(Result::ok)
        .collect();

    Ok(payments)
}
pub fn add_attendance(
    tx: &Transaction, // Используем транзакцию для атомарности
    group_id: i32,
    past_session_id: i32, // Это должен быть ID записи PastSessions
    student_id: i32,
    present_status: &str, // "Present" или "Absent"
) -> Result<()> {
    tx.execute(
        "INSERT INTO Attendance (group_id, lesson_id, student_id, present) VALUES (?1, ?2, ?3, ?4)",
        params![group_id, past_session_id, student_id, present_status],
    )?;
    Ok(())
}
/// Получает общее количество уроков для данного курса.
pub fn get_total_lessons_for_course(conn: &Connection, course_id: i32) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM Lessons WHERE course_id = ?1")?;
    let count: i32 = stmt.query_row(params![course_id], |row| row.get(0))?;
    Ok(count)
}

/// Получает количество посещенных уроков студентом в определенной группе.
/// Возвращает HashMap<student_id, count_of_attended_lessons>
pub fn get_student_attendance_counts(
    conn: &Connection,
    group_id: i32,
) -> Result<HashMap<i32, i32>> {
    let mut stmt = conn.prepare("
        SELECT
            A.student_id,
            COUNT(CASE WHEN A.present = 'Present' THEN 1 ELSE NULL END) AS attended_lessons_count
        FROM Attendance A
        WHERE A.group_id = ?1
        GROUP BY A.student_id
    ")?;

    let mut attendance_counts = HashMap::new();
    let iter = stmt.query_map(params![group_id], |row| {
        Ok((row.get("student_id")?, row.get("attended_lessons_count")?))
    })?;

    for result in iter {
        let (student_id, count) = result?;
        attendance_counts.insert(student_id, count);
    }
    Ok(attendance_counts)
}

/// Добавляет запись о сертификате.
/// Используем `&Transaction` для атомарности, если вызывается внутри транзакции.
pub fn add_certificate(
    tx: &Transaction, // <--- Важно: используем транзакцию
    student_id: i32,
    course_id: i32,
    issue_date: &str,
    grade: &str,
) -> Result<i32> {
    // Для простоты пока оставим grade_or_status, так как в схеме у вас grade TEXT NOT NULL
    tx.execute(
        "INSERT INTO Certificates (student_id, course_id, issue_date, grade) VALUES (?1, ?2, ?3, ?4)",
        params![student_id, course_id, issue_date, grade],
    )?;
    Ok(tx.last_insert_rowid() as i32)
}

/// Проверяет, завершила ли группа все занятия и выдает сертификаты.
/// Эту функцию следует вызывать после сохранения посещаемости.
pub fn check_course_completion_and_issue_certificates(
    tx: &Transaction, // Эта функция уже корректно принимает `&Transaction`
    group_id: i32,
    course_id: i32,
) -> Result<()> {
    println!("DEBUG DB: Проверка завершения курса для группы {} и курса {}", group_id, course_id);

    // УДАЛИТЕ ЭТУ СТРОКУ: let conn_ref = tx.conn(); // Больше не нужна

    // 1. Получаем общее количество уроков в курсе
    // Передаем `tx` напрямую
    let total_lessons_in_course = get_total_lessons_for_course(tx, course_id)?;
    println!("DEBUG DB: Всего уроков в курсе {}: {}", course_id, total_lessons_in_course);

    // 2. Получаем количество прошедших занятий для этой группы (только уникальные lesson_id)
    // Используем `tx` напрямую для prepare
    let mut past_sessions_count_stmt = tx.prepare("
        SELECT COUNT(DISTINCT lesson_id) FROM PastSessions WHERE group_id = ?1
    ")?;
    let completed_lessons_count: i32 = past_sessions_count_stmt.query_row(params![group_id], |row| row.get(0))?;
    println!("DEBUG DB: Проведено уникальных уроков для группы {}: {}", group_id, completed_lessons_count);


    if completed_lessons_count >= total_lessons_in_course && total_lessons_in_course > 0 {
        println!("DEBUG DB: Группа {} завершила все уроки курса {}. Выдача сертификатов.", group_id, course_id);

        // Получаем всех студентов в группе
        // Используем `tx` напрямую для prepare
        let mut students_in_group_stmt = tx.prepare("
            SELECT U.ID, U.Name FROM Users U
            JOIN GroupStudent GS ON U.ID = GS.student_id
            WHERE GS.group_id = ?1
        ")?;
        let students_iter = students_in_group_stmt.query_map(params![group_id], |row| {
            Ok(UserInfo {
                id: row.get("ID")?,
                name: row.get("Name")?,
                user_type: "".to_string(),
                email: "".to_string(),
                avatar_data: None,
                group_id: None,
                birthday: "".to_string(),
                child_count: None,
            })
        })?;
        let students_in_group: Vec<UserInfo> = students_iter.collect::<Result<Vec<_>, Error>>()?;
        println!("DEBUG DB: Студентов в группе {}: {}", group_id, students_in_group.len());


        // Получаем данные о посещаемости для всех студентов в этой группе
        // Передаем `tx` напрямую
        let student_attendance_counts = get_student_attendance_counts(tx, group_id)?;
        println!("DEBUG DB: Собраны данные посещаемости для {} студентов.", student_attendance_counts.len());


        let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let issue_date_str = now.format("%Y-%m-%d").to_string();

        for student in students_in_group {
            let attended_lessons = student_attendance_counts.get(&student.id).copied().unwrap_or(0);
            println!("DEBUG DB: Студент {}: {} из {} уроков посетил.", student.name, attended_lessons, total_lessons_in_course);

            let grade = if total_lessons_in_course > 0 {
                let percentage = (attended_lessons as f64 / total_lessons_in_course as f64) * 100.0;
                if percentage >= 100.0 {
                    "Отлично".to_string()
                } else if percentage >= 75.0 {
                    "Хорошо".to_string()
                } else {
                    "Удовлетворительно".to_string()
                }
            } else {
                "Неизвестно".to_string()
            };

            // Проверяем, есть ли уже сертификат у студента за этот курс
            // Используем `tx` напрямую для prepare
            let mut cert_exists_stmt = tx.prepare("
                SELECT COUNT(*) FROM Certificates WHERE student_id = ?1 AND course_id = ?2
            ")?;
            let cert_exists: i32 = cert_exists_stmt.query_row(params![student.id, course_id], |row| row.get(0))?;

            if cert_exists == 0 {
                // Добавляем сертификат, используя текущую транзакцию
                add_certificate(tx, student.id, course_id, &issue_date_str, &grade)?;
                println!("DEBUG DB: Сертификат выдан студенту {} за курс {} с оценкой '{}'.", student.name, course_id, grade);
            } else {
                println!("DEBUG DB: Сертификат для студента {} за курс {} уже существует.", student.name, course_id);
            }
        }
        // --- ДОБАВЛЯЕМ ЛОГИКУ ОБНОВЛЕНИЯ СТАТУСА ГРУППЫ ЗДЕСЬ ---
        println!("DEBUG DB: Обновление статуса группы {} на 'Неактивна'.", group_id);
        tx.execute(
            "UPDATE `Group` SET status = 'Неактивна' WHERE ID = ?1",
            params![group_id],
        )?;
        println!("DEBUG DB: Статус группы {} успешно обновлен на 'Неактивна'.", group_id);
        
    } else {
        println!("DEBUG DB: Группа {} еще не завершила все уроки курса {}.", group_id, course_id);
    }
    Ok(())
}
/// Получает список студентов (UserInfo), у которых есть хотя бы один сертификат,
/// с количеством их сертификатов.
pub fn get_students_with_certificates_info(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID,
            U.Name,
            U.Email,
            U.Birthday,
            U.Type, -- Соответствует user_type
            U.AvatarData,
            COUNT(C.id) AS certificate_count -- Дополнительное поле
        FROM Users U
        JOIN Certificates C ON U.ID = C.student_id
        WHERE U.Type = 'student' -- Убеждаемся, что получаем только студентов
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        ORDER BY U.Name ASC
    ")?;

    let students_iter = stmt.query_map(params![], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?, // Используем "Type" из БД
            avatar_data: row.get("AvatarData")?,
            group_id: None, // Не получаем здесь информацию о группе
            child_count: row.get("certificate_count").ok(), 
        })
    })?;

    students_iter.collect()
}

/// Получает все сертификаты для конкретного студента.
// Эта функция остается без изменений
pub fn get_certificates_for_student(conn: &Connection, student_id: i32) -> Result<Vec<Certificate>> {
    let mut stmt = conn.prepare("
        SELECT
            C.id,
            C.student_id,
            U.Name AS student_name,
            C.course_id,
            Co.title AS course_title,
            C.issue_date,
            C.grade
        FROM Certificates C
        JOIN Users U ON C.student_id = U.ID
        JOIN Course Co ON C.course_id = Co.ID
        WHERE C.student_id = ?1
        ORDER BY C.issue_date DESC, Co.title ASC
    ")?;

    let certificates_iter = stmt.query_map(params![student_id], |row| {
        Ok(Certificate {
            id: row.get("id")?,
            student_id: row.get("student_id")?,
            student_name: row.get("student_name")?,
            course_id: row.get("course_id")?,
            course_title: row.get("course_title")?,
            issue_date: row.get("issue_date")?,
            grade: row.get("grade")?,
        })
    })?;

    certificates_iter.collect()
}
pub fn get_payments_between(
    conn: &Connection,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<Payment>> {
    let query = r#"
        SELECT
            p.date,
            u.name AS student_name,
            c.title AS course_title,
            p.type AS payment_type,
            p.amount
        FROM Payment p
        JOIN Users u ON p.student_id = u.ID
        JOIN Course c ON p.course_id = c.ID
        WHERE p.date BETWEEN ?1 AND ?2
        ORDER BY p.date ASC
    "#;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(query)?;

    let payments_iter = stmt.query_map(params![start_str, end_str], |row| {
        Ok(Payment {
            id: 0,
            student_id: 0,
            date: row.get(0)?,
            student_name: row.get(1)?,
            course_title: row.get(2)?,
            payment_type: row.get(3)?,
            course_id: 0,
            amount: row.get(4)?,
            group_id: 0,
            group_name: "".to_string(),
        })
    })?;

    let mut payments = Vec::new();
    for payment in payments_iter {
        payments.push(payment?);
    }

    Ok(payments)
}
pub fn get_certificates_between(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Vec<Certificate>> {
    let mut stmt = conn.prepare(
        "SELECT
            c.id,
            c.student_id,
            u.name as student_name,
            c.course_id,
            cr.title as course_title,
            c.issue_date,
            c.grade
         FROM Certificates c
         JOIN Users u ON u.id = c.student_id
         JOIN Course cr ON cr.id = c.course_id
         WHERE c.issue_date BETWEEN ?1 AND ?2
         ORDER BY c.issue_date"
    )?;

    let from_str = from.format("%Y-%m-%d").to_string();
    let to_str = to.format("%Y-%m-%d").to_string();

    let cert_iter = stmt.query_map(params![from_str, to_str], |row| {
        Ok(Certificate {
            id: row.get(0)?,
            student_id: row.get(1)?,
            student_name: row.get(2)?,
            course_id: row.get(3)?,
            course_title: row.get(4)?,
            issue_date: row.get(5)?,
            grade: row.get(6)?,
        })
    })?;

    let mut certs = Vec::new();
    for cert in cert_iter {
        certs.push(cert?);
    }

    Ok(certs)
}
pub fn get_all_groups_for_report(conn: &Connection) -> Result<Vec<GroupForReport>, Error> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.course_id, c.title, g.teacher_id, u.Name, g.student_count, g.status
         FROM `Group` g
         LEFT JOIN Course c ON g.course_id = c.ID
         LEFT JOIN Users u ON g.teacher_id = u.ID"
    )?;

    let group_rows = stmt.query_map([], |row| {
        Ok(GroupForReport {
            id: row.get(0)?,
            name: row.get(1)?,
            course_id: row.get(2)?,
            course_name: row.get(3)?,
            teacher_id: row.get(4)?,
            teacher_name: row.get(5)?,
            student_count: row.get(6)?,
            status: row.get(7)?,
            students: Vec::new(), // Заполним позже
        })
    })?;

    let mut groups: Vec<GroupForReport> = Vec::new();
    for mut group in group_rows {
        let mut stmt = conn.prepare(
            "SELECT u.Name FROM GroupStudent gs
             JOIN Users u ON gs.student_id = u.ID
             WHERE gs.group_id = ?1"
        )?;

        let student_names = stmt
            .query_map([group.as_ref().unwrap().id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        group.as_mut().unwrap().students = student_names;
        groups.push(group?);
    }

    Ok(groups)
}

