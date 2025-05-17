use rusqlite::{params, Connection, Result};
use crate::app::Course;

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