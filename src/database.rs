use crate::error::{KvError, Result};
use crate::password;
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::Path;

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    Plaintext,
    Encrypted,
}

/// Initializes a new database. It refuses to overwrite any existing path.
pub fn initialize_database(path: &Path, mode: EncryptionMode) -> Result<()> {
    if path.exists() {
        return Err(KvError::DatabaseExists(path.display().to_string()));
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let password = match mode {
        EncryptionMode::Plaintext => None,
        EncryptionMode::Encrypted => Some(password::prompt_new_password()?),
    };

    initialize_database_with_password(path, mode, password.as_deref())
}

/// Password-injected variant used by tests and internal callers.
/// Do not expose passwords as CLI arguments.
pub fn initialize_database_with_password(
    path: &Path,
    mode: EncryptionMode,
    password: Option<&str>,
) -> Result<()> {
    // create_new prevents accidentally replacing an existing file, including
    // a file created by another process after the initial existence check.
    let mut options = OpenOptions::new();
    options.write(true).read(true).create_new(true);
    let file = options.open(path)?;
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    let result = (|| {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;

        if mode == EncryptionMode::Encrypted {
            let secret = password.ok_or(KvError::EmptyPassword)?;
            if secret.is_empty() {
                return Err(KvError::EmptyPassword);
            }
            apply_key(&conn, secret)?;
        }

        // First write creates the database pages using the selected mode.
        create_schema(&conn)?;
        validate_schema(&conn)?;
        Ok(())
    })();

    if result.is_err() {
        // Remove only the file this call created; do not leave an apparently
        // initialized but unusable database behind.
        let _ = fs::remove_file(path);
    }

    result
}

/// Opens an existing database. Plain SQLite is recognized by its file header.
/// Any other header is treated as an encrypted candidate and requires a
/// password; it is never automatically initialized or converted.
pub fn open_database(path: &Path) -> Result<Connection> {
    if !path.exists() {
        return Err(KvError::DatabaseMissing(path.display().to_string()));
    }
    if !path.is_file() {
        return Err(KvError::NotAFile(path.display().to_string()));
    }

    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 16];
    let bytes_read = file.read(&mut header)?;
    if bytes_read < 16 {
        return Err(KvError::InvalidDatabase);
    }

    if &header == SQLITE_HEADER {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        validate_schema(&conn)?;
        return Ok(conn);
    }

    let secret = password::prompt_password()?;
    open_encrypted_database_with_password(path, &secret)
}

/// Password-injected variant for automated tests.
pub fn open_encrypted_database_with_password(
    path: &Path,
    secret: &str,
) -> Result<Connection> {
    if secret.is_empty() {
        return Err(KvError::EmptyPassword);
    }

    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    apply_key(&conn, secret)?;

    // Force SQLCipher to read a database page before trusting the connection.
    let validation = conn.query_row(
        "SELECT count(*) FROM sqlite_master",
        [],
        |row| row.get::<_, i64>(0),
    );

    if validation.is_err() {
        return Err(KvError::DatabaseUnlockFailed);
    }

    validate_schema(&conn).map_err(|_| KvError::DatabaseUnlockFailed)?;
    Ok(conn)
}

fn apply_key(conn: &Connection, secret: &str) -> Result<()> {
    // rusqlite binds the value safely; never interpolate a password into SQL.
    conn.pragma_update(None, "key", secret)?;
    Ok(())
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kv (
            key TEXT PRIMARY KEY NOT NULL,
            value BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        ",
    )?;

    conn.execute(
        "INSERT INTO metadata (key, value) VALUES ('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

pub fn validate_schema(conn: &Connection) -> Result<()> {
    let version_text: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| KvError::InvalidDatabase)?;

    let version = version_text
        .parse::<i64>()
        .map_err(|_| KvError::InvalidDatabase)?;

    if version != SCHEMA_VERSION {
        return Err(KvError::UnsupportedSchema(version));
    }

    // Confirm the expected data table exists as well.
    conn.query_row("SELECT count(*) FROM kv", [], |row| row.get::<_, i64>(0))
        .map_err(|_| KvError::InvalidDatabase)?;

    Ok(())
}
