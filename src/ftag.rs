use camino::Utf8PathBuf;
use rusqlite::{params, Connection};
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
    NoDatabaseError,
    DatabaseError(rusqlite::Error),
    JsonError(serde_json::Error),
}
impl From<rusqlite::Error> for FtagError {
    fn from(err: rusqlite::Error) -> Self {
        FtagError::DatabaseError(err)
    }
}
impl From<serde_json::Error> for FtagError {
    fn from(err: serde_json::Error) -> Self {
        FtagError::JsonError(err)
    }
}
impl ToString for FtagError {
    fn to_string(&self) -> String {
        match self {
            FtagError::IoError(err) => format!("IO Error: {}", err.to_string()),
            FtagError::NoDatabaseError => "Database error: Database not initialized".to_string(),
            FtagError::DatabaseError(err) => format!("Database Error: {}", err.to_string()),
            FtagError::JsonError(err) => format!("JSON Error: {}", err.to_string()),
        }
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
        return Err(FtagError::NoDatabaseError);
    }

    let tags = vec!["fake".to_string(), "global".to_string(), "tags".to_string()];
    Ok(tags)
}

pub fn add_tags(path: &Utf8PathBuf, tags: Vec<OsString>) -> Result<Vec<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }
    if !db_exists() {
        return Err(FtagError::NoDatabaseError);
    }

    
    // Prepare a query for the correct row of the database
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT id, tags FROM tags WHERE path = ?")?;
    
    // Query for a row with matching path
    let query = stmt.query_row(params![path.to_string()], |row| {
        let id: u32 = row.get(0)?;
        let json: String = row.get(1)?;
        Ok((id, json))
    });
    
    // Create an empty list of tags
    let mut newtags = Taglist { tags: vec![] };

    // Deserialize any existing tags and append them to the new tags
    if let Ok((_, json)) = &query {
        // TODO: unwrap() may panic, it is not great (anybody could fuck with the database and jank it up)
        let mut deserialized: Taglist = serde_json::from_str(&json).unwrap();
        newtags.tags.append(&mut deserialized.tags);
    }

    // Push unique tags onto the end of the vector
    for tag in tags {
        let tag: String = tag.to_string_lossy().to_string();

        if !newtags.tags.contains(&tag) {
            newtags.tags.push(tag);
        }
    }

    // Either insert a row into the database for this file or update the existing one
    let serialized = serde_json::to_string(&newtags)?;
    match &query {
        Err(_) => {
            // If there was not a row, insert one
            let mut stmt = conn.prepare("INSERT INTO tags(path, tags) VALUES (?, ?)")?;
            stmt.execute(params![path.to_string(), serialized])?;
        },
        Ok((id, _)) => {
            // If query is Ok (there was already a row), update it
            let mut stmt = conn.prepare("UPDATE tags SET tags= ? WHERE id = ?")?;
            stmt.execute(params![serialized, id])?;
        },
    }

    // Return the new tags to the caller
    Ok(newtags.tags)
}

pub fn remove_tags(path: &Utf8PathBuf, _tags: Vec<OsString>) -> Result<Vec<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(ErrorKind::NotFound));
    }

    todo!();
}
