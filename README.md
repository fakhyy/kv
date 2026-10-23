# kv

A local key-value CLI backed by SQLite, with optional full-database encryption through SQLCipher.

## Build

```bash
cargo build --release
```

The SQLCipher feature builds native dependencies, so the first build may take some time.

## Installation

Install directly from source code (cargo required).
```bash
cargo install --git https://github.com/fakhyy/kv.git
```


## Usage

```bash
# Create a plaintext database
kv init

# Store and retrieve values
kv set username fakhar
kv get username
kv list
kv exists username
kv delete username

# Create an encrypted database
kv --db ./secrets.db init --encrypt

# Encrypted database access prompts for the password every time
kv --db ./secrets.db set token abc123
kv --db ./secrets.db get token
```

The default database path is the platform's local application data directory under `kv/data.db`. On Linux this is typically `~/.local/share/kv/data.db`.

## Security notes

- Passwords are entered interactively and are not command-line arguments.
- The encrypted database uses SQLCipher. Ordinary SQLite cannot read its encrypted pages.
- A password prompt is required on every command that accesses an encrypted database.
- Do not assume deleting a key securely erases all physical copies from storage.
- Backups, filesystem snapshots, and exported files require their own protection.
- This project is a starting implementation. Test SQLCipher behavior and error handling on your target platforms before storing important secrets.

## Development

```bash
cargo fmt
cargo test
```
