use std::{ffi::OsString, process, io::ErrorKind};

use camino::Utf8PathBuf;

use rusqlite::{params, Connection};

/// Get the path to the database
fn get_db_path() -> Utf8PathBuf {
    Utf8PathBuf::from("ftag.db")
}

/// Initialize the database if it does not already exist, returning whether it was created.
pub fn init_db() -> Result<(), ()> {
    let conn = Connection::open(get_db_path());
    if let Err(_) = conn {
        return Err(());
    }

    Ok(())
}

pub fn get_file_tags(path: Utf8PathBuf) -> Result<Vec<String>, ErrorKind> {
    if !path.exists() {
        return Err(ErrorKind::NotFound);
    }
    //todo!("get_file_tags");

    let tags = vec!("aaa".to_string(), "bbb".to_string());
    Ok(tags)
}

pub fn get_global_tags() {
    todo!("get_global_tags");
}

pub fn add_tags(path: Utf8PathBuf, _tags: Vec<OsString>) {
    todo!("add_tags");
}

pub fn remove_tags(path: Utf8PathBuf, _tags: Vec<OsString>) {
    todo!("remove_tags");
}