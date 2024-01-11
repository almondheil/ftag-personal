use camino::Utf8PathBuf;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, io, collections::hash_set::HashSet};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Taglist {
    tags: HashSet<String>,
}

/// Errors that can occur when using ftag functions
#[derive(Debug)]
pub enum FtagError {
    IoError(io::ErrorKind),
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
impl From<io::ErrorKind> for FtagError {
    fn from(err: io::ErrorKind) -> Self {
        FtagError::IoError(err)
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

/// Update or create a database entry for a path
fn update_row_into_db(path: &Utf8PathBuf, serialized: String, query: &Result<(u32, String), FtagError>) -> Result<(), FtagError> {
    let conn = Connection::open(get_db_path())?;
    match query {
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

    Ok(())
}

fn query_db_for_path(path: &Utf8PathBuf) -> Result<(u32, String), FtagError> {
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
    })?;

    Ok(query)
}

/// Initialize the database if it does not already exist, returning whether it was created.
pub fn init_db() -> Result<(), FtagError> {
    // Refuse to init if the database already exists
    if db_exists() {
        return Err(FtagError::IoError(io::ErrorKind::AlreadyExists));
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

pub fn get_file_tags(path: &Utf8PathBuf) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }

    let query: Result<(u32, String), FtagError> = query_db_for_path(path);
    match query {
        Ok((_, json)) => {
            // TODO: another unwrap I should probably handle
            let tags: Taglist = serde_json::from_str(&json).unwrap();
            Ok(tags.tags)
        },
        Err(_) => {
            Ok(HashSet::new())
        },
    }
}

pub fn get_global_tags() -> Result<HashSet<String>, FtagError> {
    if !db_exists() {
        return Err(FtagError::NoDatabaseError);
    }

    // Create a HashSet that will hold the tags
    let mut all_tags: HashSet<String> = HashSet::new();

    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT tags FROM tags;")?;
    let result = stmt.query_map( params![],
        |row| {
            // Process each name in the result set
            let tags: String = row.get(0)?;
            let deserialized: Taglist = serde_json::from_str(&tags).unwrap();
            for tag in deserialized.tags {
                all_tags.insert(tag);
            }
            Ok(())
        },
    )?;

    // TODO: I'm supposed to check for errors in the result what do I do here
    result.for_each(|_| {});
    Ok(all_tags)
}

pub fn add_tags(path: &Utf8PathBuf, add_tags: Vec<OsString>) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }
    
    let query = query_db_for_path(path);
    
    // Create an empty list of tags
    let mut newtags = Taglist { tags: HashSet::new() };

    // Deserialize any existing tags and add them into the existing tags
    if let Ok((_, json)) = &query {
        // TODO: unwrap() may panic, it is not great (anybody could fuck with the database and jank it up)
        let deserialized: Taglist = serde_json::from_str(&json).unwrap();
        for tag in deserialized.tags {
            newtags.tags.insert(tag);
        }
    }

    // Insert any unique new tags
    for tag in add_tags {
        let tag: String = tag.to_string_lossy().to_string();
        newtags.tags.insert(tag);
    }

    // Update that row in the database
    update_row_into_db(path, serde_json::to_string(&newtags)?, &query)?;    
    Ok(newtags.tags)
}

pub fn remove_tags(path: &Utf8PathBuf, remove_tags: Vec<OsString>) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }
    
    let query: Result<(u32, String), FtagError> = query_db_for_path(path);
    
    // Create an empty list of tags
    let mut newtags = Taglist { tags: HashSet::new() };

    // Deserialize any existing tags and append them to the new tags
    if let Ok((_, json)) = &query {
        // TODO: unwrap() may panic, it is not great (anybody could fuck with the database and jank it up)
        let deserialized: Taglist = serde_json::from_str(&json).unwrap();
        
        // Let newtags contain all tags not in remove_tags
        for tag in deserialized.tags {
            // TODO: do we really need to clone? It's frustrating
            if !remove_tags.contains(&tag.clone().into()) {
                newtags.tags.insert(tag);
            }
        }
    }

    // Update that row in the database
    update_row_into_db(path, serde_json::to_string(&newtags)?, &query)?;    
    Ok(newtags.tags)
}

pub fn find_tags(tags: &Vec<OsString>) -> Result<Vec<String>, FtagError> {
    todo!();
}
