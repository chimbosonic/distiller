use rusqlite::{params, Connection, Transaction};
use serde_rusqlite::to_params_named;
use std::{error::Error, fs};

use crate::file::FileData;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub fn write_to_db(datas: Vec<FileData>, dbpath: String) -> Result<()> {
    let mut connection = rusqlite::Connection::open(dbpath)?;
    let transaction = connection.transaction()?;

    for data in datas {
        match add_filedata_sql(&data, &transaction) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Could not add filedata to database: {}", e);
                return Err("Could not add filedata to database".into());
            }
        };
    }

    log::info!("Commiting to Database");
    transaction.commit()?;
    close_connection(connection)?;
    Ok(())
}

pub fn setup_database(db_path: &String) -> Result<()> {
    let _ = fs::remove_file(db_path);

    let connection = Connection::open(db_path)?;

    create_table(&connection)?;

    close_connection(connection)?;

    Ok(())
}

fn close_connection(connection: Connection) -> Result<()> {
    if connection.close().is_err() {
        log::error!("Could not close database");
        return Err("Could not close database".into());
    }
    Ok(())
}

fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE comments (id TEXT, comment TEXT, filehash TEXT)",
        [],
    )?;
    connection.execute("CREATE TABLE files (id TEXT, filename TEXT)", [])?;
    Ok(())
}

fn add_filedata_sql(filedata: &FileData, transaction: &Transaction) -> Result<()> {
    log::info!("Adding {}'s results to results database", filedata.path);
    for comment in filedata.comments.as_slice() {
        transaction.execute(
            "INSERT INTO comments (id, comment, filehash) VALUES (:hash, :comment, :file_hash)",
            to_params_named(comment)?.to_slice().as_slice(),
        )?;
    }
    transaction.execute(
        "INSERT INTO files (id, filename) VALUES (?1, ?2)",
        params![filedata.hash, filedata.path],
    )?;

    Ok(())
}
