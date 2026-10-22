use crate::error::{KvError, Result};
use rusqlite::Connection;

pub fn set(conn: &Connection, key: &str, value: &[u8]) -> Result<()> {
    conn.execute(
        "INSERT INTO kv (key, value, created_at, updated_at)
         VALUES (?1, ?2, unixepoch(), unixepoch())
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = unixepoch()",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, key: &str) -> Result<Vec<u8>> {
    conn.query_row(
        "SELECT value FROM kv WHERE key = ?1",
        [key],
        |row| row.get::<_, Vec<u8>>(0),
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => KvError::KeyNotFound(key.to_owned()),
        other => KvError::Database(other),
    })
}

pub fn list(conn: &Connection) -> Result<Vec<String>> {
    let mut statement = conn.prepare("SELECT key FROM kv ORDER BY key")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut keys = Vec::new();

    for row in rows {
        keys.push(row?);
    }

    Ok(keys)
}

pub fn delete(conn: &Connection, key: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM kv WHERE key = ?1", [key])?;
    Ok(affected > 0)
}

pub fn exists(conn: &Connection, key: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM kv WHERE key = ?1)",
        [key],
        |row| row.get(0),
    )?)
}
