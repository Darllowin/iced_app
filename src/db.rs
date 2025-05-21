use rusqlite::{params, Connection, OptionalExtension, Result};
use tokio::task;
use crate::app::{Assignment, Course, Group, Lesson, LessonWithAssignments, PastSession, ProvenLesson, UserInfo};
use crate::db;

pub enum LoginError {
    UserNotFound,
    WrongPassword,
    DatabaseError(rusqlite::Error),
}


// Обновлено для возврата Option<Vec<u8>> для данных аватара
pub async fn authenticate_and_get_user_data(
    email_input: String,
    hashed_password: String,
) -> std::result::Result<UserInfo, String> {
    tokio::task::spawn_blocking(move || {
        let conn = Connection::open("db_platform")
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
                    "student" => db::db_get_group_name_for_student(&conn, id).unwrap_or_else(|e| {
                        eprintln!("Ошибка получения группы для студента {}: {}", id, e);
                        None
                    }),
                    "teacher" => db::db_get_group_name_for_teacher(&conn, id).unwrap_or_else(|e| {
                        eprintln!("Ошибка получения группы для учителя {}: {}", id, e);
                        None
                    }),
                    _ => None,
                };

                child_count = if user_type == "parent" {
                    db::db_get_child_count_for_parent(&conn, id).ok()
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
                    group: group_name, // Используем group_name для поля `group` в UserInfo
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
        "INSERT INTO Users (Name, Type, Birthday, Email, password, AvatarData) VALUES (?1, 'student', ?2, ?3, ?4, NULL)", // Изначально устанавливаем AvatarData в NULL
        params![full_name, birthday, email, password_hash],
    )?;
    Ok(())
}

// Обновлено для приема avatar_data как &[u8]
pub fn update_user_avatar(conn: &Connection, email: &str, avatar_data: &[u8]) -> Result<()> {
    conn.execute(
        "UPDATE Users SET AvatarData = ?1 WHERE Email = ?2", // Предполагается, что столбец называется AvatarData
        params![avatar_data, email],
    )?;
    Ok(())
}

pub fn get_courses(conn: &Connection) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare("
        SELECT
            C.ID,
            C.title,
            C.description,
            C.instructor AS instructor_id,   -- <--- Получаем ID преподавателя из Course.instructor
            U.Name AS instructor_name,       -- <--- Получаем имя преподавателя из Users.Name
            C.level,
            COUNT(L.ID) AS LessonCount
        FROM Course C
        LEFT JOIN Users U ON C.instructor = U.ID
        LEFT JOIN Lessons L ON C.ID = L.course_id
        GROUP BY C.ID, C.title, C.description, C.instructor, U.Name, C.level
        ORDER BY C.title
    ")?;

    let course_iter = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get("ID")?,
            title: row.get("title")?,
            description: row.get("description")?,
            instructor_id: row.get("instructor_id")?,   // <--- Заполняем новый ID
            instructor_name: row.get("instructor_name")?, // <--- Заполняем новое имя
            level: row.get("level").ok(),
            lesson_count: row.get("LessonCount")?,
        })
    })?;

    Ok(course_iter.collect::<Result<Vec<_>>>()?)
}

pub fn add_course(conn: &Connection, title: &str, description: &str, instructor_id: Option<i32>, level: Option<&str>) -> Result<()> {
    conn.execute(
        // Используем instructor_id напрямую
        "INSERT INTO Course (title, description, instructor, level) VALUES (?1, ?2, ?3, ?4)",
        params![
            title,
            description,
            instructor_id, // <--- Теперь передаем Option<i32> напрямую
            level
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
            group: None,         // Устанавливаем в None, так как колонки нет
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
        "UPDATE Course SET title = ?1, description = ?2, instructor = ?3, level = ?4 WHERE ID = ?5",
        params![
            course.title,
            course.description,
            course.instructor_id, // <--- ИСПОЛЬЗУЕМ instructor_id НАПРЯМУЮ
            course.level.as_deref(), // Level, если у вас Option<String>, то as_deref() подходит
            course.id
        ],
    )?;
    Ok(())
}
pub fn get_lessons_for_course_and_group(conn: &Connection, course_id: i32, group_id: i32) -> Result<Vec<LessonWithAssignments>> {
    let mut stmt = conn.prepare("
        SELECT
            L.ID,
            L.course_id,
            L.number,
            L.title
        FROM Lessons L
        LEFT JOIN PastSessions PS ON L.ID = PS.lesson_id AND PS.group_id = ?2
        WHERE L.course_id = ?1 AND PS.ID IS NULL -- PS.ID IS NULL означает, что занятие не найдено в PastSessions для этой группы
        ORDER BY L.number
    ")?;

    let lesson_iter = stmt.query_map(params![course_id, group_id], |row| {
        Ok(LessonWithAssignments {
            id: row.get("ID")?,
            course_id: row.get("course_id")?,
            number: row.get("number")?,
            title: row.get("title")?,
            assignments: vec![], // Задания будут загружены отдельно
        })
    })?;

    let mut lessons_with_assignments: Vec<LessonWithAssignments> = lesson_iter.collect::<Result<Vec<_>>>()?;

    // Загружаем задания для каждого урока
    for lesson in &mut lessons_with_assignments {
        let assignments = get_assignments_for_lesson(conn, lesson.id)?;
        lesson.assignments = assignments;
    }

    Ok(lessons_with_assignments)
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
            group: group_info, // <--- Assign the determined group_info
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
pub fn get_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            g.id,
            g.name,
            g.course_id,        -- ID курса из таблицы Group
            c.title AS course_title, -- Название курса из таблицы Course
            g.teacher_id,       -- ID преподавателя из таблицы Group
            u.name AS teacher_name,  -- Имя преподавателя из таблицы Users
            COUNT(gs.student_id) AS student_count
        FROM \"Group\" g
        LEFT JOIN Course c ON g.course_id = c.id
        LEFT JOIN Users u ON g.teacher_id = u.id
        LEFT JOIN GroupStudent gs ON gs.group_id = g.id
        GROUP BY g.id, g.name, g.course_id, c.title, g.teacher_id, u.name -- Группируем по всем неагрегированным полям
        ORDER BY g.name
        "
    )?;

    let groups = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                course_id: row.get(2)?,
                course_name: row.get(3)?,  // <--- Теперь получаем название курса
                teacher_id: row.get(4)?,
                teacher_name: row.get(5)?, // <--- Теперь получаем имя преподавателя
                student_count: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    // Добавьте отладочный вывод, чтобы убедиться, что группы загружаются корректно
    println!("DEBUG: Количество загруженных групп в get_groups: {}", groups.len());
    for group in &groups {
        println!("DEBUG: Загруженная группа (с именами): {:?}", group); // Измените вывод
    }

    Ok(groups)
}



pub fn insert_group(conn: &Connection, name: &str, course_id: i32, teacher_id: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO `Group` (name, course_id, teacher_id, student_count) VALUES (?1, ?2, ?3, ?4)",
        params![name, course_id, teacher_id, 0],
    )?;
    Ok(())
}

pub fn update_group(conn: &Connection, id: i32, name: &str, course_id: i32, teacher_id: i32) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE \"Group\" SET name = ?, course_id = ?, teacher_id = ? WHERE id = ?",
        params![name, course_id, teacher_id, id],
    )?;
    Ok(())
}
// Функция для получения названия курса по его ID

// Функция для получения имени пользователя по его ID
pub fn get_user_name_by_id(conn: &Connection, user_id: i32) -> Result<Option<String>> {
    conn.query_row(
        "SELECT name FROM Users WHERE ID = ?1",
        params![user_id],
        |row| row.get(0),
    ).optional() // Используем .optional(), чтобы возвращать Option<String>
}
pub fn delete_group(conn: &Connection, id: i32) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM \"Group\" WHERE id = ?", params![id])?;
    Ok(())
}
pub fn get_students_for_group(conn: &Connection, group_id: i32) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData,
            GROUP_CONCAT(G_all.name, ', ') AS StudentGroups
        FROM Users U
        JOIN GroupStudent GS ON U.ID = GS.student_id
        JOIN \"Group\" G_filter ON GS.group_id = G_filter.id
        LEFT JOIN GroupStudent GS_all ON U.ID = GS_all.student_id
        LEFT JOIN \"Group\" G_all ON GS_all.group_id = G_all.id
        WHERE G_filter.id = ?1
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData
        ORDER BY U.Name
    ")?;

    let users = stmt.query_map([group_id], |row| {
        Ok(UserInfo {
            id: row.get("ID")?,
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: row.get("Type")?, // Должен быть 'student'
            avatar_data: row.get("AvatarData").ok(),
            group: row.get("StudentGroups").ok(),
            child_count: None, // <-- Добавляем инициализацию поля количества детей
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(users)
}
pub fn get_all_student_names(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT Name FROM Users WHERE Type = 'student'")?;
    let names = stmt.query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(names)
}
pub fn add_student_to_group(conn: &Connection, group_id: i32, student_name: &str) -> Result<()> {
    let student_id: i32 = conn.query_row(
        "SELECT ID FROM Users WHERE Name = ? AND Type = 'student'",
        [student_name],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO GroupStudent (group_id, student_id) VALUES (?, ?)",
        [group_id, student_id],
    )?;

    Ok(())
}
pub fn remove_student_from_group(conn: &Connection, group_id: i32, student_name: &str) -> Result<()> {
    let student_id: i32 = conn.query_row(
        "SELECT ID FROM Users WHERE Name = ? AND Type = 'student'",
        [student_name],
        |row| row.get(0),
    )?;

    conn.execute(
        "DELETE FROM GroupStudent WHERE group_id = ? AND student_id = ?",
        [group_id, student_id],
    )?;

    Ok(())
}
pub fn get_students_without_group(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "
        SELECT Name
        FROM Users
        WHERE Type = 'student'
          AND ID NOT IN (
              SELECT student_id FROM GroupStudent
          )
        "
    )?;

    let students = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    Ok(students)
}
pub fn get_user_group(conn: &Connection, user_id: i32) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT g.name FROM \"Group\" g
         JOIN GroupStudent gs ON g.id = gs.group_id
         WHERE gs.student_id = ?1 LIMIT 1"
    )?;

    let mut rows = stmt.query([user_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}
pub fn get_teacher_group_name(conn: &Connection, teacher_id: i32) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM \"Group\" WHERE teacher_id = ?1 LIMIT 1"
    )?;

    let mut rows = stmt.query(params![teacher_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}
pub fn get_user_id_by_email(conn: &Connection, email: &str) -> Option<i32> {
    let mut stmt = conn.prepare("SELECT ID FROM Users WHERE Email = ?1").ok()?;
    let mut rows = stmt.query([email]).ok()?;
    rows.next().ok().flatten().map(|row| row.get(0).ok()).flatten()
}
pub fn get_children_for_parent(conn: &Connection, parent_email: &str) -> Result<Vec<UserInfo>, rusqlite::Error> {
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
            group: row.get("StudentGroups").ok(),
            child_count: None, // <-- Добавляем инициализацию поля количества детей
        })
    })?;

    children_iter.collect()
}

pub fn delete_child_for_parent(conn: &Connection, parent_email: &str, child_email: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM ParentStudent
         WHERE parent_id = (SELECT ID FROM Users WHERE Email = ?1)
         AND student_id = (SELECT ID FROM Users WHERE Email = ?2)",
        [parent_email, child_email],
    )?;
    Ok(())
}
pub fn get_unassigned_children(conn: &Connection) -> Result<Vec<UserInfo>, rusqlite::Error> {
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
                group: row.get("StudentGroups").ok(),
                child_count: None, // <-- Добавляем инициализацию поля количества детей
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(children)
}

pub fn add_child_to_parent(conn: &Connection, parent_email: &str, child_email: &str) -> Result<(), rusqlite::Error> {
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

pub fn get_proven_lessons_for_group(conn: &Connection, group_id: i32) -> Result<Vec<ProvenLesson>> {
    let mut stmt = conn.prepare("
        SELECT
            PL.id,
            PL.group_id,
            PL.date,
            PL.topic,
            L.ID AS lesson_id,
            L.number AS lesson_number,
            L.title AS lesson_title
        FROM ProvenLesson PL
        JOIN Lessons L ON PL.lesson_id = L.ID -- Предполагаем, что теперь есть lesson_id в таблице ProvenLesson
        WHERE PL.group_id = ?1
        ORDER BY PL.date, L.number
    ")?;

    let proven_lesson_iter = stmt.query_map(params![group_id], |row| {
        Ok(ProvenLesson {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            date: row.get("date")?,
            topic: row.get("topic")?,
            lesson_id: row.get("lesson_id")?, // Убедитесь, что это извлекается
            lesson_number: row.get("lesson_number")?,
            lesson_title: row.get("lesson_title")?,
            assignments: vec![],
        })
    })?;
    proven_lesson_iter.collect()
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

// Функция для получения всех заданий (для выбора)
pub fn get_all_assignments(conn: &Connection) -> Result<Vec<Assignment>> {
    let mut stmt = conn.prepare(
        "SELECT id, lesson_id, title, description, type FROM Assignment ORDER BY title"
    )?;
    let assignment_iter = stmt.query_map([], |row| {
        Ok(Assignment {
            id: row.get(0)?,
            lesson_id: row.get(1)?,
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

// NEW: Обновить тему ProvenLesson (если нужно)
pub fn update_proven_lesson_topic(conn: &Connection, proven_lesson_id: i32, new_topic: &str) -> Result<()> {
    conn.execute(
        "UPDATE ProvenLesson SET topic = ?1 WHERE id = ?2",
        params![new_topic, proven_lesson_id],
    )?;
    Ok(())
}
pub async fn load_proven_lessons_for_group(group_id: i32) -> std::result::Result<Vec<ProvenLesson>, String> {
    // Теперь `task::spawn_blocking` будет найден благодаря импорту `use tokio::task;`
    task::spawn_blocking(move || { // <-- Используем task::spawn_blocking
        let conn = Connection::open("db_platform")
            .map_err(|e| format!("Не удалось открыть базу данных: {}", e))?;
        get_proven_lessons_for_group(&conn, group_id)
            .map_err(|e| format!("Не удалось получить запланированные уроки: {}", e))
    })
        .await
        .map_err(|e| format!("Ошибка блокирующей задачи: {}", e))?
}

pub fn get_group_by_id(conn: &Connection, group_id_val: i32) -> Result<Option<Group>> {
    // В этой функции вам нужно получить полные данные Group, включая названия курса и учителя,
    // если вы хотите, чтобы она возвращала Group с полностью заполненными полями.
    // Если вам нужны только ID, как было изначально, то текущий запрос и поля соответствуют.
    // Я предполагаю, что вы хотите полную информацию.

    conn.query_row(
        "
        SELECT
            G.id,
            G.name,
            G.course_id,
            C.title AS course_title,
            G.teacher_id,
            U_teacher.name AS teacher_name
        FROM \"Group\" G
        LEFT JOIN Course C ON G.course_id = C.id
        LEFT JOIN Users U_teacher ON G.teacher_id = U_teacher.id
        WHERE G.id = ?1
        ",
        params![group_id_val],
        |row| Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            course_id: row.get("course_id")?,
            course_name: row.get("course_title")?,
            teacher_id: row.get("teacher_id")?,
            teacher_name: row.get("teacher_name")?,
            student_count: 0, // student_count не извлекается из этого запроса, устанавливаем в 0
        })
    ).optional() // Используем optional(), чтобы вернуть Option
}
pub fn get_course_title_by_id(conn: &Connection, course_id: i32) -> Result<Option<String>> {
    conn.query_row(
        "SELECT title FROM Course WHERE ID = ?1",
        params![course_id],
        |row| row.get(0)
    ).optional()
}
pub fn get_past_sessions_for_group(conn: &Connection, group_id_val: i32) -> Result<Vec<PastSession>> {
    let mut stmt = conn.prepare("
        SELECT PS.id, PS.group_id, PS.date, PS.lesson_id, L.number, L.title
        FROM PastSessions PS
        JOIN Lessons L ON PS.lesson_id = L.ID
        WHERE PS.group_id = ?1
        ORDER BY PS.date DESC
    ")?;
    let past_sessions_iter = stmt.query_map(params![group_id_val], |row| {
        Ok(PastSession {
            id: row.get(0)?,
            group_id: row.get(1)?,
            date: row.get(2)?,
            lesson_id: row.get(3)?,
            lesson_number: row.get(4)?,
            lesson_title: row.get(5)?,
        })
    })?;
    past_sessions_iter.collect()
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