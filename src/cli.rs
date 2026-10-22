use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "kv", version, about = "Local key-value storage")]
pub struct Cli {
    /// Database file to use (defaults to the user's application data directory)
    #[arg(long, global = true, value_name = "PATH")]
    pub db: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a database. Existing files are never overwritten.
    Init {
        /// Encrypt the entire database with SQLCipher
        #[arg(long)]
        encrypt: bool,
    },
    /// Store or update a text value
    Set {
        key: String,
        value: String,
    },
    /// Retrieve a text value
    Get {
        key: String,
    },
    /// List all keys
    List,
    /// Delete a key
    Delete {
        key: String,
    },
    /// Check whether a key exists
    Exists {
        key: String,
    },
    /// Show basic database information
    Info,
}
