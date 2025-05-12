use rusqlite::{params, Connection, Result};

pub fn check_user_credentials(conn: &Connection, email: &str, password_hash: &str) -> Option<String> {
    let mut stmt = conn.prepare("SELECT password, Name FROM Users WHERE Email = ?1").unwrap();
    let mut rows = stmt.query([email]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let stored_hash: String = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();

        if stored_hash == password_hash {
            return Some(name);
        }
    }
    None
}

pub fn register_user(conn: &Connection, full_name: &str, birthday: &str, email: &str, password_hash: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Users (Name, Type, Birthday, Email, password) VALUES (?1, 'student', ?2, ?3, ?4)",
        params![full_name, birthday ,email, password_hash],
    )?;
    Ok(())
}