use rusqlite::{Connection, Result as SqliteResult, Transaction};

pub struct TransactionManager;

impl TransactionManager {
    pub fn execute_with_result<F, T>(conn: &mut Connection, operation: F) -> SqliteResult<T>
    where
        F: FnOnce(&Transaction) -> SqliteResult<T>,
    {
        let tx = conn.transaction()?;
        let result = operation(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}
