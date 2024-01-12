use camino::Utf8PathBuf;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{io, collections::{hash_map::HashMap, hash_set::HashSet}};

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

/// Get the path to the database.
fn get_db_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".ftag.db")
}

/// Update or create a database entry for a path.
/// 
/// * `path` - Path to save in the database row, used to search for existing entry.
/// * `serialized` - JSON representation of the tags to save
/// 
/// # Failure
/// 
/// Returns `Err` if there is no database in the current directory or if database queries or statements fail.
fn update_row_into_db(path: &Utf8PathBuf, serialized: String) -> Result<(), FtagError> {
    if !get_db_path().exists() {
        return Err(FtagError::NoDatabaseError);
    }
    
    let conn = Connection::open(get_db_path())?;
    
    // Query the database for that path
    let query = query_db_for_path(path);
    
    // Depending on whether a row exists, insert or update
    match query {
        Err(_) => {
            // Err means there was no such row, so we insert
            let mut stmt = conn.prepare("INSERT INTO tags(path, tags) VALUES (?, ?)")?;
            stmt.execute(params![path.to_string(), serialized])?;
        },
        Ok((id, _)) => {
            // Ok means there was a row, so we update it
            let mut stmt = conn.prepare("UPDATE tags SET tags= ? WHERE id = ?")?;
            stmt.execute(params![serialized, id])?;
        },
    }

    Ok(())
}

/// Query the database for a given path, returning the id and tags on a success.
/// 
/// * `path` - Path to query for
/// 
/// # Failure
/// 
/// Returns Err if there is no database in the current directory or if database query fails.
fn query_db_for_path(path: &Utf8PathBuf) -> Result<(u32, String), FtagError> {
    if !get_db_path().exists() {
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

/// Go through every row in the database, removing entries for paths that no longer exist
/// 
/// # Failure
/// 
/// Returns `Err` if database does not exist or there are errors when interacting with the database.
fn prune_db() -> Result<(), FtagError> {
    if !get_db_path().exists() {
        return Err(FtagError::NoDatabaseError);
    }

    // Create an empty vector of paths to remove
    let mut to_remove: Vec<String> = vec![];

    // Go through the database and add all paths that no longer exist to to_remove
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT path FROM tags;")?;
    let result = stmt.query_map( params![],
        |row| {
            let name: String = row.get(0)?;   
            let path = Utf8PathBuf::from(name.clone());

            if !path.exists() {
                to_remove.push(name);
            }
            Ok(())
        },
    )?;
    result.for_each(|_| ());

    // For all the rows that should be removed, remove them
    let mut stmt = conn.prepare("DELETE FROM tags WHERE path = ?")?;
    for name in to_remove {
        stmt.execute(params![name])?;
    }

    Ok(())
}

/// Initialize the database if it does not already exist, returning whether it was created.
/// 
/// # Failure
/// 
/// Returns `Err` if a database already exists in the current directory
pub fn init_db() -> Result<(), FtagError> {
    // Refuse to init if the database already exists
    if get_db_path().exists() {
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

/// Return the tags belonging to a certain path, or the empty set if there are none.
/// 
/// * `path` - Path to the file to check
/// 
/// # Failure
/// 
/// Returns `Err` if `path` does not exist, there is no database, or errors occur when deserializing JSON or querying the database.
pub fn get_file_tags(path: &Utf8PathBuf) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }

    let query: Result<(u32, String), FtagError> = query_db_for_path(path);
    match query {
        Ok((_, json)) => {
            let tags: Taglist = serde_json::from_str(&json)?;
            Ok(tags.tags)
        },
        Err(_) => {
            Ok(HashSet::new())
        },
    }
}

/// Return the set of all tags used in the current database.
/// 
/// # Failure
/// 
/// Returns `Err` if there is no database or errors occur when deserializing JSON or querying the database.
pub fn get_global_tags() -> Result<HashMap<String, u32>, FtagError> {
    if !get_db_path().exists() {
        return Err(FtagError::NoDatabaseError);
    }

    // Before we list the global tags, prune the db
    // This makes sure removed paths don't show up
    // TODO: But it's also probably slow. Can this be fixed or reduced?
    prune_db()?;

    // Create a HashSet that will hold the tags
    let mut tag_counts: HashMap<String, u32> = HashMap::new();

    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT tags FROM tags;")?;
    let result = stmt.query_map( params![],
        |row| {
            let tags: String = row.get(0)?;            
            // TODO: Can I avoid unwrapping?
            let deserialized: Taglist = serde_json::from_str(&tags).unwrap();
            for tag in deserialized.tags {

                if tag_counts.contains_key(&tag) {
                    // Get the current count
                    let count = tag_counts.get(&tag).unwrap() + 1;

                    // Remove and re-add the key-value pair
                    tag_counts.remove(&tag);
                    tag_counts.insert(tag, count);
                } else {
                    // Add a count of 1
                    tag_counts.insert(tag, 1);
                }
            }
            Ok(())
        },
    )?;

    // Do nothing for each item in the iterator, to get them to evaluate
    result.for_each(|_| ());

    // Once we have evaluated the iterator items, we can return the counts
    Ok(tag_counts)
}

/// Add tags to a file's record in the database, returning the set of tags now assigned to that file.
/// 
/// * `path` - Path to the file to add tags to
/// * `add_tags` - Vector containing tags to add. Duplicate tags will be ignored.
/// 
/// # Failure
/// 
/// Returns `Err` if `path` does not exist, there is no database in the current directory, or errors occur when serializing and deserializing data or interacting with the database.
pub fn add_tags(path: &Utf8PathBuf, add_tags: Vec<String>) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }
    
    let query = query_db_for_path(path);
    
    // Create an empty list of tags
    let mut newtags = Taglist { tags: HashSet::new() };

    // Deserialize any existing tags and add them into the existing tags
    if let Ok((_, json)) = &query {
        let deserialized: Taglist = serde_json::from_str(&json)?;
        for tag in deserialized.tags {
            newtags.tags.insert(tag);
        }
    }

    // Insert any unique tags to be added
    for tag in add_tags {
        newtags.tags.insert(tag);
    }

    // Update that row in the database
    update_row_into_db(path, serde_json::to_string(&newtags)?)?;    
    Ok(newtags.tags)
}

/// Remove tags from a file's record in the database, returning the set of tags now assigned to that file.
/// 
/// * path - Path to the file to remove tags from
/// * remove_tags - Vector containing tags to remove. Any tags not belonging to `path` will be ignored.
/// 
/// # Failure
/// 
/// Returns `Err` if `path` does not exist, there is no database in the current directory, or errors occur when serializing and deserializing data or interacting with the database.
pub fn remove_tags(path: &Utf8PathBuf, remove_tags: Vec<String>) -> Result<HashSet<String>, FtagError> {
    if !path.exists() {
        return Err(FtagError::IoError(io::ErrorKind::NotFound));
    }
    
    let query: Result<(u32, String), FtagError> = query_db_for_path(path);
    
    // Create an empty list of tags
    let mut newtags = Taglist { tags: HashSet::new() };

    // Deserialize any existing tags and append them to the new tags
    if let Ok((_, json)) = &query {
        let deserialized: Taglist = serde_json::from_str(&json)?;
        
        // Let newtags contain all tags not in remove_tags
        for tag in deserialized.tags {
            if !remove_tags.contains(&tag) {
                newtags.tags.insert(tag);
            }
        }
    }

    // Update that row in the database
    update_row_into_db(path, serde_json::to_string(&newtags)?)?;    
    Ok(newtags.tags)
}

/// Check the entire database for files containg all of `find_tags`, returning their paths.
/// 
/// * `find_tags` - Vector of tags to filter by. Any matching files will have all of the tags in `find_tags`.
/// 
/// # Failure
/// 
/// Returns `Err` if there is no database, errors occur when deserializing data, or errors occur when querying the database.
pub fn find_tags(find_tags: &Vec<String>, exclude_tags: &Vec<String>) -> Result<Vec<String>, FtagError> {
    if !get_db_path().exists() {
        return Err(FtagError::NoDatabaseError);
    }

    // Convert find and exclude tags into HashSets, as we'll be checking containment a lot
    let find_tags: HashSet<String> = HashSet::from_iter(find_tags.iter().cloned());
    let exclude_tags: HashSet<String> = HashSet::from_iter(exclude_tags.iter().cloned());

    // Store a vector of the files containing those tags
    let mut matching_files: Vec<String> = vec![];

    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT path, tags FROM tags;")?;
    let result = stmt.query_map( params![],
        |row| {
            // Process each name in the result set
            let name: String = row.get(0)?;
            let tags: String = row.get(1)?;
            // TODO: This unwrap should be avoided
            let deserialized: Taglist = serde_json::from_str(&tags).unwrap();

            // Are all tags in find_tags contained by deserialized?
            let find_tags_contained = find_tags
                .iter()
                .all(|tag| deserialized.tags.contains(tag));

            // Are all tags in exclude_tags NOT contained by deserialized?
            let exclude_tags_not_contained = exclude_tags
                .iter()
                .all(|tag| !deserialized.tags.contains(tag));
           
            // Store the filename if it satisfies both conditions
            if find_tags_contained && exclude_tags_not_contained {
                matching_files.push(name);
            }

            Ok(())
        },
    )?;

    result.for_each(|_| ());
    Ok(matching_files)
}
