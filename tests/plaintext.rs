use std::process::Command;
use tempfile::tempdir;

fn kv() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kv"))
}

#[test]
fn plaintext_crud_round_trip() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("test.db");

    assert!(kv().args(["--db", db.to_str().unwrap(), "init"]).status().unwrap().success());
    assert!(kv().args(["--db", db.to_str().unwrap(), "set", "language", "rust"]).status().unwrap().success());

    let output = kv().args(["--db", db.to_str().unwrap(), "get", "language"]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "rust");

    let output = kv().args(["--db", db.to_str().unwrap(), "exists", "language"]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "true");

    assert!(kv().args(["--db", db.to_str().unwrap(), "delete", "language"]).status().unwrap().success());

    let output = kv().args(["--db", db.to_str().unwrap(), "get", "language"]).output().unwrap();
    assert!(!output.status.success());
}
