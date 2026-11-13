//! New-password validation: length bounds + common-password denylist.
//! Pure rules, no DB — unit tests live here.
//!
//! Keep `COMMON_PASSWORDS` in sync with web/src/lib/passwordPolicy.ts —
//! duplicated by design (shortlist, both sides fail closed); see
//! docs/DECISIONS.md. All entries are lowercase and at least 8 characters
//! long (shorter candidates are already rejected by the length rule).

use anyhow::{Result, anyhow};

const COMMON_PASSWORDS: &[&str] = &[
    "password",
    "password1",
    "password123",
    "passw0rd",
    "p@ssw0rd",
    "12345678",
    "123456789",
    "1234567890",
    "1234qwer",
    "qwer1234",
    "1q2w3e4r5t",
    "qwerty123",
    "abc123456",
    "admin1234",
    "adminadmin",
    "administrator1",
    "superadmin1",
    "root1234",
    "toor1234",
    "letmein1",
    "welcome1",
    "iloveyou1",
    "changeme1",
    "test1234",
    "monkey123",
    "dragon123",
    "master123",
    "sunshine1",
    "princess1",
    "football1",
    "baseball1",
    "starwars1",
    "whatever1",
    "trustno1",
    "engrenage1",
    "engrenage123",
];

/// Validate a candidate new password. The error message is user-facing
/// (mapped to `BadRequest` by callers).
///
/// Length is measured in characters (`chars().count()`), matching the
/// user-facing wording; the JS side counts UTF-16 units, which only diverges
/// for surrogate pairs.
pub fn validate_new_password(candidate: &str) -> Result<()> {
    let char_count = candidate.chars().count();
    if char_count < 8 {
        return Err(anyhow!("new password must be at least 8 characters"));
    }
    if char_count > 128 {
        return Err(anyhow!("new password must be at most 128 characters"));
    }
    if COMMON_PASSWORDS.contains(&candidate.to_lowercase().as_str()) {
        return Err(anyhow!("password is too common"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_long_and_common() {
        assert_eq!(
            validate_new_password("short").unwrap_err().to_string(),
            "new password must be at least 8 characters"
        );
        assert_eq!(
            validate_new_password(&"x".repeat(129))
                .unwrap_err()
                .to_string(),
            "new password must be at most 128 characters"
        );
        assert_eq!(
            validate_new_password("Password123")
                .unwrap_err()
                .to_string(),
            "password is too common"
        );
        assert_eq!(
            validate_new_password("ADMIN1234").unwrap_err().to_string(),
            "password is too common"
        );
    }

    #[test]
    fn accepts_decent_passwords() {
        assert!(validate_new_password("correct-horse").is_ok());
        assert!(validate_new_password("changed-password-1").is_ok());
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        // 8 chars = 16 bytes in UTF-8: must pass the 8-character minimum.
        assert!(validate_new_password(&"é".repeat(8)).is_ok());
        // 129 chars (258 bytes) must fail the maximum.
        assert!(validate_new_password(&"é".repeat(129)).is_err());
    }
}
