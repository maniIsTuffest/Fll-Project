use rusqlite::{params, Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;

const DEFAULT_ADMIN_PATH: &str = "../../../../tempData/defaultAdmin.jsonc";

// Add Clone derive to easily return a copy of the user
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub password: String, // Note: Storing plain passwords is insecure; use a hashing library like 'argon2'
    pub rank: i16,
    pub email: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct UserResponse {
    pub username: String,
    pub rank: i16,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            username: user.username,
            rank: user.rank,
            email: user.email,
        }
    }
}

// Updated function signature to use idiomatic Rust Result for error handling
pub fn search_user(username: &str, password: &str) -> RusqliteResult<Option<User>> {
    // Connect to the database
    let db = Connection::open("login.db")?;

    // Query the database for the user
    // We select all columns required for the User struct
    let mut stmt = db.prepare(
        "SELECT username, password, rank, email FROM users WHERE username = ? AND password = ?",
    )?;

    let user_result = stmt.query_row(params![username, password], |row| {
        Ok(User {
            username: row.get(0)?,
            password: row.get(1)?,
            rank: row.get(2)?,
            email: row.get(3)?,
        })
    });

    // Match on the result to handle 'no rows found' explicitly
    match user_result {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// Updated function signature to return a proper error type (Box<dyn std::error::Error> is common for main functions)
pub fn init_db() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the data at runtime
    let default_admin_json = fs::read_to_string(DEFAULT_ADMIN_PATH)?;

    // 2. Deserialize the JSON data
    // Use `serde_json::from_str` for the string data
    let default_admin: User = serde_json::from_str(&default_admin_json)?;

    // 3. Connect to the database
    let db = Connection::open("login.db")?;

    // 4. Create the table
    db.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE, -- Added UNIQUE constraint for usernames
            password TEXT NOT NULL,
            rank INTEGER NOT NULL,
            email TEXT NOT NULL
        )",
        [],
    )?;

    // 5. Insert the default admin user into the database only if they don't exist
    // Using `params!` macro is cleaner than building arrays manually
    let insert_result = db.execute(
        "INSERT INTO users (username, password, rank, email) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(username) DO NOTHING",
        params![
            default_admin.username,
            default_admin.password,
            default_admin.rank,
            default_admin.email,
        ],
    )?;

    if insert_result == 1 {
        println!("Default admin user inserted successfully.");
    } else {
        println!("Default admin user already exists, skipping insertion.");
    }

    Ok(())
}
