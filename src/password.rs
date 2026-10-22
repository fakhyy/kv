use crate::error::{KvError, Result};

pub fn prompt_password() -> Result<String> {
    rpassword::prompt_password("Database password: ")
        .map_err(|err| KvError::PasswordPrompt(err.to_string()))
}

pub fn prompt_new_password() -> Result<String> {
    let first = rpassword::prompt_password("New database password: ")
        .map_err(|err| KvError::PasswordPrompt(err.to_string()))?;
    if first.is_empty() {
        return Err(KvError::EmptyPassword);
    }

    let second = rpassword::prompt_password("Confirm password: ")
        .map_err(|err| KvError::PasswordPrompt(err.to_string()))?;

    if first != second {
        return Err(KvError::PasswordMismatch);
    }

    Ok(first)
}
