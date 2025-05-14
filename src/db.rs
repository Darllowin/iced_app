use rusqlite::{params, Connection, Result};
use crate::app::Course;

pub enum LoginError {
    UserNotFound,
    WrongPassword,
    DatabaseError(rusqlite::Error),
}
pub fn check_user_credentials(
    conn: &Connection,
    email: &str,
    hashed_password: &str,
) -> Result<(String, Option<String>, String, String), LoginError> {
    let mut stmt = conn
        .prepare("SELECT password, Name, AvatarPath, Birthday, Type FROM Users WHERE Email = ?1")
        .map_err(LoginError::DatabaseError)?;

    let mut rows = stmt
        .query(params![email])
        .map_err(LoginError::DatabaseError)?;

    if let Some(row) = rows.next().map_err(LoginError::DatabaseError)? {
        let stored_hash: String = row.get(0).map_err(LoginError::DatabaseError)?;
        let name: String = row.get(1).map_err(LoginError::DatabaseError)?;
        let avatar_path: Option<String> = row.get(2).map_err(LoginError::DatabaseError)?; 
        let birthday: String = row.get(3).map_err(LoginError::DatabaseError)?;
        let type_user: String = row.get(4).map_err(LoginError::DatabaseError)?;

        if stored_hash == hashed_password {
            Ok((name, avatar_path, birthday, type_user))
        } else {
            Err(LoginError::WrongPassword)
        }
    } else {
        Err(LoginError::UserNotFound)
    }
}

pub fn register_user(conn: &Connection, full_name: &str, birthday: &str, email: &str, password_hash: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Users (Name, Type, Birthday, Email, password) VALUES (?1, 'student', ?2, ?3, ?4)",
        params![full_name, birthday ,email, password_hash],
    )?;
    Ok(())
}
pub fn update_user_avatar(conn: &Connection, email: &str, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE Users SET AvatarPath = ?1 WHERE Email = ?2",
        params![path, email],
    )?;
    Ok(())
}
pub fn get_courses(conn: &Connection) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare("SELECT ID, title, description FROM Course")?;
    let course_iter = stmt.query_map([], |row| {
        Ok(Course {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
        })
    })?;

    let mut courses = Vec::new();
    for course in course_iter {
        courses.push(course?);
    }
    Ok(courses)
}