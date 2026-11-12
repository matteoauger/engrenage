//! Cryptographic primitives for the auth foundation.
//!
//! Two deliberately different primitives live here:
//! - **Password pair** (`hash_password`/`verify_password`): argon2id via the
//!   `argon2` crate, PHC string format with embedded salt and parameters.
//!   Verification parses the *stored* hash (salt included) — never re-hash
//!   and string-compare, the salt would differ every time.
//! - **Token hashing** (`hash_token`): plain SHA-256. Session tokens are
//!   256-bit uniform-random, so no slow KDF is needed — and `hash_token`
//!   runs on every authenticated request, so it must be fast.
//!
//! Keep these functions pure (no DB, no axum): unit tests belong here.

use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::RngExt;
use sha2::{Digest, Sha256};

const SESSION_TOKEN_BYTES: usize = 32;

/// Hash a plaintext password with argon2id.
///
/// Returns a PHC-format string (algorithm, parameters, salt, hash) suitable
/// for storage in `accounts.password_hash` and for `verify_password`.
pub fn hash_password(plain: &str) -> Result<String> {
    if plain.is_empty() {
        return Err(anyhow!("cannot hash an empty password"));
    }
    let hashed = Argon2::default()
        .hash_password(plain.as_bytes())
        .map_err(|e| anyhow!("password hashing failed: {e}"))?;
    Ok(hashed.to_string())
}

/// Verify a plaintext password against a stored PHC hash.
///
/// Returns `Ok(false)` for a wrong password, `Ok(true)` for a match.
/// Malformed/unverifiable hashes are an `Err`, not `Ok(false)`.
pub fn verify_password(plain: &str, phc_hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(phc_hash).map_err(|e| anyhow!("invalid stored hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

/// Generate a fresh opaque session token (returned to the client once, in a
/// cookie). 32 random bytes, hex-encoded.
pub fn generate_session_token() -> Result<String> {
    let mut bytes = [0u8; SESSION_TOKEN_BYTES];
    rand::rng().fill(&mut bytes);
    Ok(hex_encode(&bytes))
}

/// Hash a session token for storage/lookup (`sessions.token_hash`).
pub fn hash_token(token: &str) -> Result<String> {
    Ok(hex_encode(&Sha256::digest(token.as_bytes())))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_is_phc_and_verifies() {
        let hash = hash_password("correct-horse").unwrap();
        assert!(hash.starts_with("$argon2id$"), "PHC format expected");
        assert!(verify_password("correct-horse", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn password_hash_uses_random_salt() {
        // Same input twice must produce different PHC strings (salted),
        // while both verify against the same password.
        let a = hash_password("same-input").unwrap();
        let b = hash_password("same-input").unwrap();
        assert_ne!(a, b);
        assert!(verify_password("same-input", &a).unwrap());
        assert!(verify_password("same-input", &b).unwrap());
    }

    #[test]
    fn malformed_hash_is_error_not_false() {
        assert!(verify_password("x", "not-a-phc-hash").is_err());
    }

    #[test]
    fn empty_password_is_rejected() {
        assert!(hash_password("").is_err());
    }

    #[test]
    fn session_tokens_are_hex_and_unique() {
        let a = generate_session_token().unwrap();
        let b = generate_session_token().unwrap();
        assert_eq!(a.len(), SESSION_TOKEN_BYTES * 2, "hex-encoded 32 bytes");
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b, "tokens must never repeat");
    }

    #[test]
    fn token_hash_is_sha256_hex() {
        let token = generate_session_token().unwrap();
        let hash = hash_token(&token).unwrap();
        assert_eq!(hash.len(), 64, "sha256 hex digest");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        // Same token → same hash (lookup key), different token → different.
        assert_eq!(hash_token(&token).unwrap(), hash);
        assert_ne!(hash_token("other").unwrap(), hash);
    }
}
