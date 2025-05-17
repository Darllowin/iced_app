use rusqlite::{params, Connection, Result};
use crate::app::{Course, Group, UserInfo};

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
        SELECT Course.ID, Course.title, Course.description, Users.name, Course.level
        FROM Course
        LEFT JOIN Users ON Course.instructor = Users.ID
    ")?;

    let course_iter = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            instructor: row.get(3).ok(),
            level: row.get(4).ok(),
        })
    })?;

    Ok(course_iter.collect::<Result<Vec<_>, _>>()?)
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
pub fn get_all_users_for_list(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("SELECT Name, Email, Birthday, Type, AvatarData FROM Users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(UserInfo {
            name: row.get(0)?,
            email: row.get(1)?,
            birthday: row.get(2)?,
            user_type: row.get(3)?,
            avatar_data: row.get(4)?,
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
pub fn get_groups(conn: &Connection) -> rusqlite::Result<Vec<Group>> {
    let mut stmt = conn.prepare("
        SELECT \"Group\".id, \"Group\".name, Course.title, Users.Name
        FROM \"Group\"
        LEFT JOIN Course ON \"Group\".course_id = Course.ID
        LEFT JOIN Users ON \"Group\".teacher_id = Users.ID
    ")?;

    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            course: row.get(2).ok(),
            teacher: row.get(3).ok(),
        })
    })?
        .filter_map(Result::ok)
        .collect();

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
pub fn get_students_for_group(conn: &Connection, group_id: i32) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("
        SELECT Users.Name
        FROM GroupStudent
        JOIN Users ON GroupStudent.student_id = Users.ID
        WHERE GroupStudent.group_id = ?
    ")?;

    let students = stmt.query_map([group_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(students)
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