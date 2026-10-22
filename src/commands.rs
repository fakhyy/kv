use crate::cli::{Cli, Command};
use crate::database::{self, EncryptionMode};
use crate::error::{KvError, Result};
use crate::{paths, storage};

pub fn run(cli: Cli) -> Result<()> {
    let path = paths::resolve_database_path(cli.db)?;

    match cli.command {
        Command::Init { encrypt } => {
            let mode = if encrypt {
                EncryptionMode::Encrypted
            } else {
                EncryptionMode::Plaintext
            };
            paths::ensure_parent_directory(&path)?;
            database::initialize_database(&path, mode)?;
            println!("Initialized {} database: {}", mode_label(mode), path.display());
        }
        Command::Set { key, value } => {
            let conn = database::open_database(&path)?;
            storage::set(&conn, &key, value.as_bytes())?;
        }
        Command::Get { key } => {
            let conn = database::open_database(&path)?;
            let bytes = storage::get(&conn, &key)?;
            let value = String::from_utf8(bytes).map_err(|_| KvError::InvalidUtf8)?;
            println!("{value}");
        }
        Command::List => {
            let conn = database::open_database(&path)?;
            for key in storage::list(&conn)? {
                println!("{key}");
            }
        }
        Command::Delete { key } => {
            let conn = database::open_database(&path)?;
            if !storage::delete(&conn, &key)? {
                return Err(KvError::KeyNotFound(key));
            }
        }
        Command::Exists { key } => {
            let conn = database::open_database(&path)?;
            println!("{}", storage::exists(&conn, &key)?);
        }
        Command::Info => {
            let conn = database::open_database(&path)?;
            let count: i64 =
                conn.query_row("SELECT count(*) FROM kv", [], |row| row.get(0))?;
            println!("Path: {}", path.display());
            println!("Keys: {count}");
            println!("Schema version: 1");
            println!("Encryption: {}", if is_plaintext_file(&path)? { "off" } else { "on" });
        }
    }

    Ok(())
}

fn mode_label(mode: EncryptionMode) -> &'static str {
    match mode {
        EncryptionMode::Plaintext => "plaintext",
        EncryptionMode::Encrypted => "encrypted",
    }
}

fn is_plaintext_file(path: &std::path::Path) -> Result<bool> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 16];
    let n = file.read(&mut header)?;
    Ok(n == 16 && &header == b"SQLite format 3\0")
}
