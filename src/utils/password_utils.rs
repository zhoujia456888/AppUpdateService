use crate::model::error::AppError;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub const SECRET_KEY: &str = "YOUR SECRET_KEY";

/// 🔒 Hash a plaintext password
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err("Failed to hash password".into()),
    }
}

/// ✅ Verify a plaintext password against a hashed one
pub fn verify_password(password: &str, hash: &str) -> bool {
    verify_password_result(password, hash).unwrap_or(false)
}

pub fn verify_password_result(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("密码哈希格式无效: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
