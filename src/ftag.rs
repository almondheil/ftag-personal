use camino::Utf8PathBuf;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, io::ErrorKind};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Taglist {
    tags: Vec<String>,
}

/// Errors that can occur when using ftag functions
#[derive(Debug)]
pub enum FtagError {
    IoError(ErrorKind),
    DatabaseError(rusqlite::Error),
}
impl From<rusqlite::Error> for FtagError {
    // allows us to use ? with rusqlite operations (very helpful)
    fn from(err: rusqlite::Error) -> Self {
        FtagError::DatabaseError(err)
    }
}

/// Get the path to the database
fn get_db_path() -> Utf8PathBuf {
    Utf8PathBuf::from("ftag.db")
}

/// Determine whether the database exists in the current directory
fn db_exists() -> bool {
    get_db_path().exists()
}

/// Initialize the database if it does not already exist, returning whether it was created.
pub fn init_db() -> Result<(), FtagError> {
    // Refuse to init if the database already exists
    if db_exists() {
        return Err(FtagError::IoError(ErrorKind::AlreadyExists));
    }

    // Create a database and a table within it
    let conn = Connection::open(get_db_path())?;
    conn.execute(
        "CREATE TABLE tags (
            id      INTEGER AUTOINCREMENT PRIMARY KEY,
            path    TEXT NOT NULL,
            tags    TEXT
        )",
        (),
    )?;

    Ok(())
}

pub fn get_file_tags(path: &Utf8PathBuf) -> Result<Vec<String>, FtagError> {
    // TODO: Can we have different errors for "No database" and "File not found"?
    if !db_exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    // TODO: when there are no tags, we should return an
    let tags = vec!["fake".to_string(), "file".to_string(), "tags".to_string()];
    Ok(tags)
}

pub fn get_global_tags() -> Result<Vec<String>, FtagError> {
    if !db_exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    let tags = vec!["fake".to_string(), "global".to_string(), "tags".to_string()];
    Ok(tags)
}

pub fn add_tags(path: &Utf8PathBuf, tags: Vec<OsString>) -> Result<(), FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    // Check if there is already an entry for this path (existing tags)
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT * FROM tags WHERE path = :path;")?;
    let mut rows = stmt.query(())?;
    let row = rows.next();
    // Construct the tags
    Ok(())
}

pub fn remove_tags(path: &Utf8PathBuf, tags: Vec<OsString>) -> Result<(), FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    Ok(())
}
