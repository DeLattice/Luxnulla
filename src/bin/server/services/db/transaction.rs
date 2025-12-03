use rusqlite::{Connection, Transaction};
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ServiceError {
    Repository(rusqlite::Error),
    Storage(io::Error),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Repository(e) => write!(f, "Database error: {}", e),
            ServiceError::Storage(e) => write!(f, "Filesystem error: {}", e),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<rusqlite::Error> for ServiceError {
    fn from(err: rusqlite::Error) -> Self {
        ServiceError::Repository(err)
    }
}

impl From<io::Error> for ServiceError {
    fn from(err: io::Error) -> Self {
        ServiceError::Storage(err)
    }
}

pub fn run_transaction<T, F>(conn: &mut Connection, operation: F) -> Result<T, ServiceError>
where
    F: FnOnce(&Transaction) -> Result<T, ServiceError>,
{
    let tx = conn.transaction()?;
    match operation(&tx) {
        Ok(result) => {
            tx.commit()?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback()?;
            Err(e)
        }
    }
}

pub fn run_db_transaction<T, F>(conn: &mut Connection, operation: F) -> Result<T, ServiceError>
where
    F: FnOnce(&Transaction) -> Result<T, rusqlite::Error>,
{
    let tx = conn.transaction()?;
    match operation(&tx) {
        Ok(result) => {
            tx.commit()?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback()?;
            Err(e.into())
        }
    }
}
