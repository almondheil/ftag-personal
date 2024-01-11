use camino::Utf8PathBuf;
use rusqlite::{Connection, params};
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
    JsonError(serde_json::Error),
}
impl From<rusqlite::Error> for FtagError {
    // allows us to use ? with rusqlite operations (very helpful)
    fn from(err: rusqlite::Error) -> Self {
        FtagError::DatabaseError(err)
    }
}
impl From<serde_json::Error> for FtagError {
    // allows us to use ? with rusqlite operations (very helpful)
    fn from(err: serde_json::Error) -> Self {
        FtagError::JsonError(err)
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
            id      INTEGER PRIMARY KEY,
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
    
    // Create an empty list of tags
    let mut taglist = Taglist { tags: vec![] };

    // Prepare a query for the correct row of the database
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT tags FROM tags WHERE path = ?")?;
    
    // Query for a row with matching path
    let json = stmt.query_row(params![path.to_string()], |row| {
        let json: String = row.get(0)?;
        Ok(json)
    });

    // If an entry exists, append its tags to the taglist
    if let Ok(json) = json {
        let mut deserialized: Taglist = serde_json::from_str(&json).unwrap();
        taglist.tags.append(&mut deserialized.tags);
    }

    // Push each of the provided tags into the vector too
    for tag in tags {
        // TODO: Is there a better way to do this?
        let tag: String = tag.to_string_lossy().to_string();
        taglist.tags.push(tag);
    }

    // Serialize the vector as json
    let serialized = serde_json::to_string(&taglist)?;

    // Insert or replace the correct row with the updated tags
    let mut stmt = conn.prepare("INSERT OR REPLACE INTO tags(path, tags) VALUES (?, ?);")?;
    stmt.execute(params![path.to_string(), serialized])?;

    Ok(())
}

pub fn remove_tags(path: &Utf8PathBuf, tags: Vec<OsString>) -> Result<(), FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    Ok(())
}
