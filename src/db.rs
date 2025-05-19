use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::app::{Assignment, Course, Group, Lesson, UserInfo};

pub enum LoginError {
    UserNotFound,
    WrongPassword,
    DatabaseError(rusqlite::Error),
}


// Обновлено для возврата Option<Vec<u8>> для данных аватара
pub fn check_user_credentials(
    conn: &Connection,
    email: &str,
    hashed_password: &str,
) -> Result<(String, Option<Vec<u8>>, String, String), LoginError> { // Тип возвращаемого значения изменен для аватара
    let mut stmt = conn
        .prepare("SELECT password, Name, AvatarData, Birthday, Type FROM Users WHERE Email = ?1") // Предполагается, что столбец называется AvatarData и хранит BLOB
        .map_err(LoginError::DatabaseError)?;

    let mut rows = stmt
        .query(params![email])
        .map_err(LoginError::DatabaseError)?;

    if let Some(row) = rows.next().map_err(LoginError::DatabaseError)? {
        let stored_hash: String = row.get(0).map_err(LoginError::DatabaseError)?;
        let name: String = row.get(1).map_err(LoginError::DatabaseError)?;
        let avatar_data: Option<Vec<u8>> = row.get(2).map_err(LoginError::DatabaseError)?; // Получаем данные аватара как байты
        let birthday: String = row.get(3).map_err(LoginError::DatabaseError)?;
        let type_user: String = row.get(4).map_err(LoginError::DatabaseError)?;

        if stored_hash == hashed_password {
            Ok((name, avatar_data, birthday, type_user))
        } else {
            Err(LoginError::WrongPassword)
        }
    } else {
        Err(LoginError::UserNotFound)
    }
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
            C.ID, C.title, C.description, U.name AS instructor, C.level,
            COUNT(L.ID) AS LessonCount -- <-- Подсчитываем занятия
        FROM Course C
        LEFT JOIN Users U ON C.instructor = U.ID -- Для имени преподавателя
        LEFT JOIN Lessons L ON C.ID = L.course_id -- <-- Добавляем соединение с таблицей занятий
        GROUP BY C.ID, C.title, C.description, U.name, C.level -- Группируем по полям курса и преподавателя
        ORDER BY C.title -- Опционально: сортировка курсов
    ")?;

    let course_iter = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get("ID")?,
            title: row.get("title")?,
            description: row.get("description")?,
            instructor: row.get("instructor").ok(), // <--- ИСПРАВЛЕНО
            level: row.get("level").ok(),
            lesson_count: row.get("LessonCount")?,
        })
    })?;

    Ok(course_iter.collect::<Result<Vec<_>>>()?) // Используем collect::<Result<Vec<_>>>()
}

pub fn add_course(conn: &Connection, title: &str, description: &str, instructor: Option<&str>, level: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO Course (title, description, instructor, level) VALUES (?1, ?2,
            (SELECT ID FROM Users WHERE name = ?3), ?4)",
        params![title, description, instructor, level],
    )?;
    Ok(())
}

pub fn delete_course(conn: &Connection, course_id: i32) -> Result<()> {
    let mut stmt = conn.prepare("DELETE FROM Course WHERE id = ?")?;
    stmt.execute([course_id])?;
    Ok(())
}

pub fn get_all_users(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM Users")?;
    let users = stmt.query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(users)
}
pub fn update_course(conn: &Connection, course: &Course) -> Result<()> {
    conn.execute(
        "UPDATE Course SET title = ?1, description = ?2, instructor = (SELECT ID FROM Users WHERE name = ?3), level = ?4 WHERE ID = ?5",
        params![
            course.title,
            course.description,
            course.instructor.as_deref(), 
            course.level.as_deref(),      
            course.id
        ],
    )?;
    Ok(())
}
pub fn get_lessons_for_course(conn: &Connection, course_id: i32) -> Result<Vec<Lesson>> {
    let mut stmt = conn.prepare("SELECT ID, course_id, number, title FROM Lessons WHERE course_id = ?1 ORDER BY number, title")?; // Сортируем по номеру и названию

    let lesson_iter = stmt.query_map([course_id], |row| {
        Ok(Lesson {
            id: row.get(0)?,
            course_id: row.get(1)?,
            number: row.get(2).ok(), // number может быть NULL
            title: row.get(3)?,
        })
    })?;

    lesson_iter.collect()
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
pub fn get_all_users_for_list(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("
        SELECT
            U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData,
            -- Получаем группы для студентов (как делали ранее)
            CASE WHEN U.Type = 'student' THEN GROUP_CONCAT(G.name, ', ') ELSE NULL END AS StudentGroups,
            -- Подсчитываем количество связанных student_id для этого пользователя (если он родитель)
            -- LEFT JOIN гарантирует, что мы получим 0 для тех, у кого нет детей
            COUNT(PS.student_id) AS ChildCount
        FROM Users U
        LEFT JOIN GroupStudent GS ON U.ID = GS.student_id -- Для групп студентов
        LEFT JOIN \"Group\" G ON GS.group_id = G.id -- Для имен групп
        LEFT JOIN ParentStudent PS ON U.ID = PS.parent_id -- <-- Добавляем соединение с таблицей связей родитель-ребенок
        GROUP BY U.ID, U.Name, U.Email, U.Birthday, U.Type, U.AvatarData -- Группируем по всем полям пользователя
        ORDER BY U.Name -- Опционально: сортировка
    ")?;

    let user_iter = stmt.query_map([], |row| {
        let user_type: String = row.get("Type")?; // Считываем тип пользователя

        Ok(UserInfo {
            name: row.get("Name")?,
            email: row.get("Email")?,
            birthday: row.get("Birthday")?,
            user_type: user_type.clone(), // Используем считанный тип
            avatar_data: row.get("AvatarData").ok(),
            // Группы только для студентов, читаем агрегированную строку
            group: if user_type == "student" { row.get("StudentGroups").ok() } else { None },
            // Количество детей только для родителей, читаем подсчитанное значение
            // COUNT(*) с LEFT JOIN вернет 0, если нет связанных строк. Мы можем считать 0 как Some(0).
            child_count: if user_type == "parent" { Some(row.get("ChildCount")?) } else { None }, // <-- Читаем подсчитанное количество детей
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
            c.title AS course_title,
            u.name AS teacher_name,
            COUNT(gs.student_id) AS student_count
        FROM \"Group\" g
        LEFT JOIN Course c ON g.course_id = c.id
        LEFT JOIN Users u ON g.teacher_id = u.id
        LEFT JOIN GroupStudent gs ON gs.group_id = g.id
        GROUP BY g.id
        ORDER BY g.name
        "
    )?;

    let groups = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                course: row.get(2)?,
                teacher: row.get(3)?,
                student_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(groups)
}

pub fn insert_group(conn: &Connection, name: &str, course_title: &str, teacher_name: &str) -> rusqlite::Result<()> {
    let course_id: i32 = conn.query_row("SELECT ID FROM Course WHERE title = ?", [course_title], |row| row.get(0))?;
    let teacher_id: i32 = conn.query_row("SELECT ID FROM Users WHERE Name = ?", [teacher_name], |row| row.get(0))?;

    conn.execute(
        "INSERT INTO \"Group\" (name, course_id, teacher_id) VALUES (?, ?, ?)",
        params![name, course_id, teacher_id],
    )?;
    Ok(())
}

pub fn update_group(conn: &Connection, id: i32, name: &str, course_title: &str, teacher_name: &str) -> rusqlite::Result<()> {
    let course_id: i32 = conn.query_row("SELECT ID FROM Course WHERE title = ?", [course_title], |row| row.get(0))?;
    let teacher_id: i32 = conn.query_row("SELECT ID FROM Users WHERE Name = ?", [teacher_name], |row| row.get(0))?;

    conn.execute(
        "UPDATE \"Group\" SET name = ?, course_id = ?, teacher_id = ? WHERE id = ?",
        params![name, course_id, teacher_id, id],
    )?;
    Ok(())
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
            assignment_type: row.get(4)?, // Имя колонки в БД "type"
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
