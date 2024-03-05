use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn verify_password(hash: &str, password: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow::anyhow!("Failed to hash password: {}", err))?;
    Ok(hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "password123";
        let hashed_password = hash_password(password).unwrap();

        // Ensure the hashed password is not the same as the original password
        assert_ne!(hashed_password, password);

        // Ensure the hashed password is not empty
        assert!(!hashed_password.is_empty());

        // Ensure that different passwords are hashed differently
        let hashed_password2 = hash_password("wrongpassword").unwrap();
        assert_ne!(hashed_password, hashed_password2);
    }

    #[test]
    fn test_verify_password() {
        let password = "password123";
        let hashed_password = hash_password(password).unwrap();

        // Ensure the password is verified correctly
        assert!(verify_password(&hashed_password, password));

        // Ensure a wrong password is not verified
        assert!(!verify_password(&hashed_password, "wrongpassword"));
    }
}
