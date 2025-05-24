use std::collections::HashMap;
use std::io::Cursor;
use image::imageops::FilterType;
use image::ImageReader;
use rusqlite::{params, Connection, OptionalExtension, Result, Error, ffi};
use serde::de::StdError;
use tokio::task;
use crate::app::state::{Assignment, Course, Group, LessonWithAssignments, PastSession, Payment, UserInfo, PATH_TO_DB};


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

pub fn add_course(conn: &Connection, title: &str, description: &str, level: Option<&str>, seats: i32, price: f64, total_seats: i32) -> Result<()> {
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
            ID,          -- Совпадает
            Name,        -- Исправлен регистр
            Email,       -- Исправлен регистр
            Birthday,    -- Исправлен регистр
            Type,        -- Исправлено имя колонки (было user_type)
            AvatarData   -- Исправлен регистр
            -- `group` и child_count отсутствуют в схеме Users, поэтому убраны из запроса
        FROM Users
        ORDER BY Name   -- Исправлен регистр для ORDER BY
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
        // Используем course.instructor_id напрямую, без подзапроса по имени
        "UPDATE Course SET title = ?1, description = ?2, level = ?3, total_seats = ?4, seats = ?5, price =?6 WHERE ID = ?7",
        params![
            course.title,
            course.description,
            course.level.as_deref(),
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
            G.student_count
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
            G.student_count -- ИСПРАВЛЕНО: Теперь берем из столбца G.student_count
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
            G.student_count -- ИСПРАВЛЕНО: Теперь берем из столбца G.student_count
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
pub fn insert_group(conn: &Connection, name: &str, course_id: i32, teacher_id: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO `Group` (name, course_id, teacher_id, student_count) VALUES (?1, ?2, ?3, ?4)",
        params![name, course_id, teacher_id, 0],
    )?;
    Ok(())
}

pub fn update_group(conn: &Connection, id: i32, name: &str, course_id: i32, teacher_id: i32) -> Result<()> {
    conn.execute(
        "UPDATE \"Group\" SET name = ?, course_id = ?, teacher_id = ? WHERE id = ?",
        params![name, course_id, teacher_id, id],
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
            COUNT(GS.student_id) AS student_count
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
        })
    })?.collect::<Result<Vec<_>>>()?;

    println!("DEBUG: Количество загруженных групп для учителя (ID: {}) в get_groups_for_teacher: {}", teacher_id, groups.len());
    for group in &groups {
        println!("DEBUG: Загруженная группа для учителя (с именами): {:?}", group);
    }

    Ok(groups)
}
pub fn get_assignments_for_proven_lesson(conn: &Connection, proven_lesson_id: i32) -> Result<Vec<Assignment>> {
    let mut stmt = conn.prepare("
        SELECT
            A.id,
            A.lesson_id, -- Это lesson_id из таблицы Assignment, не ProvenLesson
            A.title,
            A.description,
            A.type
        FROM Assignment A
        JOIN ProvenLessonAssignment PLA ON A.id = PLA.assignment_id
        WHERE PLA.proven_lesson_id = ?1
        ORDER BY A.title
    ")?;
    let assignment_iter = stmt.query_map(params![proven_lesson_id], |row| {
        Ok(Assignment {
            id: row.get(0)?,
            lesson_id: row.get(1)?, // Это ID базового урока, к которому привязано задание
            title: row.get(2)?,
            description: row.get(3)?,
            assignment_type: row.get(4)?,
        })
    })?;
    assignment_iter.collect()
}
// Добавить существующее задание к запланированному уроку
pub fn add_assignment_to_proven_lesson(conn: &Connection, proven_lesson_id: i32, assignment_id: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO ProvenLessonAssignment (proven_lesson_id, assignment_id) VALUES (?1, ?2)",
        params![proven_lesson_id, assignment_id],
    )?;
    Ok(())
}

// Удалить задание из запланированного урока
pub fn delete_assignment_from_proven_lesson(conn: &Connection, proven_lesson_id: i32, assignment_id: i32) -> Result<()> {
    conn.execute(
        "DELETE FROM ProvenLessonAssignment WHERE proven_lesson_id = ?1 AND assignment_id = ?2",
        params![proven_lesson_id, assignment_id],
    )?;
    Ok(())
}
pub fn get_past_sessions_for_group(conn: &Connection, group_id: i32) -> Result<Vec<PastSession>> {
    let mut stmt = conn.prepare("
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

    let sessions_iter = stmt.query_map(params![group_id], |row| {
        Ok(PastSession {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            date: row.get("date")?,
            lesson_id: row.get("lesson_id")?,
            lesson_title: row.get("lesson_title")?,
            lesson_number: row.get("lesson_number")?, // .get("number")? если прямо L.number
        })
    })?;

    let sessions: Vec<PastSession> = sessions_iter.collect::<Result<Vec<_>, Error>>()?;
    Ok(sessions)
}
pub fn add_past_session(conn: &Connection, group_id: i32, lesson_id: i32) -> Result<()> {
    println!("DB: Attempting to add PastSession for group_id: {}, lesson_id: {}", group_id, lesson_id);
    let result = conn.execute(
        "INSERT INTO PastSessions (group_id, date, lesson_id) VALUES (?1, date('now'), ?2)",
        params![group_id, lesson_id],
    );
    match result {
        Ok(rows_affected) => {
            println!("DB: PastSession added successfully. Rows affected: {}", rows_affected);
            Ok(())
        }
        Err(e) => {
            println!("DB: Error adding PastSession: {}", e);
            Err(e)
        }
    }
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
            U.Name AS teacher_name -- Включаем имя учителя
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