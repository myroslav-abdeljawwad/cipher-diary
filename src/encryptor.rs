use std::{
    fmt::{self, Display},
    io,
};

use aes_gcm::{
    aead::{AeadInPlace, KeyInit, OsRng, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Pbkdf2,
};
use rand::RngCore;
use sha2::Sha256;

/// Errors that can occur during encryption or decryption.
#[derive(Debug)]
pub enum EncryptorError {
    /// Key derivation failed.
    KeyDerivation(String),
    /// Encryption operation failed.
    Encryption(String),
    /// Decryption operation failed.
    Decryption(String),
    /// Base64 decoding error.
    Base64Decode(base64::DecodeError),
}

impl Display for EncryptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyDerivation(e) => write!(f, "key derivation failed: {e}"),
            Self::Encryption(e) => write!(f, "encryption failed: {e}"),
            Self::Decryption(e) => write!(f, "decryption failed: {e}"),
            Self::Base64Decode(e) => write!(f, "base64 decoding failed: {e}"),
        }
    }
}

impl std::error::Error for EncryptorError {}

/// A simple line‑level encryptor that uses AES‑256‑GCM.
/// The key is derived from a passphrase using PBKDF2 with SHA‑256.
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    /// Create a new `Encryptor` from a raw 32‑byte key.
    pub fn from_key(key: [u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES key length is always valid");
        Self { cipher }
    }

    /// Derive a key from a passphrase and a random salt.
    /// Returns the encryptor and the Base64‑encoded salt that must be stored with
    /// the ciphertext to allow decryption later.
    pub fn from_passphrase(passphrase: &str) -> Result<(Self, String), EncryptorError> {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let salt_str = SaltString::b64_encode(&salt).map_err(|e| {
            EncryptorError::KeyDerivation(format!("invalid salt: {e}"))
        })?;
        let password_hash = Pbkdf2.hash_password(passphrase.as_bytes(), &salt_str)
            .map_err(|e| EncryptorError::KeyDerivation(e.to_string()))?;

        // The hash contains the derived key.
        let key_hex = PasswordHash::new(&password_hash).unwrap().hash.unwrap();
        let key_bytes = hex::decode(key_hex).map_err(|e| {
            EncryptorError::KeyDerivation(format!("hex decode error: {e}"))
        })?;
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);

        Ok((Self::from_key(key_array), salt_str.to_string()))
    }

    /// Encrypt a single line.
    /// The output format is `BASE64(SALT|NONCE|CIPHERTEXT)`.
    pub fn encrypt_line(
        &self,
        line: &str,
        salt_b64: Option<&str>,
    ) -> Result<String, EncryptorError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Prepare plaintext as bytes.
        let mut buffer = line.as_bytes().to_vec();
        self.cipher
            .encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| EncryptorError::Encryption(e.to_string()))?;
        let tag = self.cipher.tag();

        // Concatenate: nonce || ciphertext || tag
        let mut combined = Vec::with_capacity(12 + buffer.len() + 16);
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&buffer);
        combined.extend_from_slice(tag);

        // Prepend salt if provided.
        let mut payload = Vec::new();
        if let Some(salt) = salt_b64 {
            payload.extend_from_slice(salt.as_bytes());
            payload.push(b'|');
        }
        payload.extend(combined);

        Ok(URL_SAFE_NO_PAD.encode(&payload))
    }

    /// Decrypt a single line that was produced by `encrypt_line`.
    pub fn decrypt_line(
        &self,
        encrypted_b64: &str,
    ) -> Result<String, EncryptorError> {
        let decoded = URL_SAFE_NO_PAD
            .decode(encrypted_b64)
            .map_err(EncryptorError::Base64Decode)?;

        // If a salt is present, split it out.
        let (payload, _salt) = if let Some(idx) = decoded.iter().position(|&b| b == b'|') {
            (&decoded[idx + 1..], &decoded[..idx])
        } else {
            (&decoded[..], None)
        };

        if payload.len() < 12 + 16 {
            return Err(EncryptorError::Decryption(
                "payload too short".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(&payload[0..12]);
        let tag_start = payload.len() - 16;
        let ciphertext = &mut payload[12..tag_start];
        let tag = Payload {
            msg: ciphertext,
            aad: b"",
        };

        // Reconstruct the combined buffer for decryption.
        let mut buf = Vec::new();
        buf.extend_from_slice(ciphertext);
        self.cipher
            .decrypt_in_place_detached(nonce, b"", &mut buf, tag)
            .map_err(|e| EncryptorError::Decryption(e.to_string()))?;

        String::from_utf8(buf).map_err(|e| {
            EncryptorError::Decryption(format!("UTF‑8 error: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_line() {
        let passphrase = "correct horse battery staple";
        let (enc, salt) = Encryptor::from_passphrase(passphrase).unwrap();

        let line = "Today I wrote a Rust library.";
        let encrypted = enc.encrypt_line(line, Some(&salt)).unwrap();
        assert_ne!(encrypted, line);

        // Decrypt with same key
        let decrypted = enc.decrypt_line(&encrypted).unwrap();
        assert_eq!(decrypted, line);
    }

    #[test]
    fn test_different_keys() {
        let passphrase1 = "first";
        let passphrase2 = "second";

        let (enc1, salt1) = Encryptor::from_passphrase(passphrase1).unwrap();
        let (enc2, _salt2) = Encryptor::from_passphrase(passphrase2).unwrap();

        let line = "Secret message.";
        let encrypted1 = enc1.encrypt_line(line, Some(&salt1)).unwrap();

        // Decryption with different key should fail
        assert!(enc2.decrypt_line(&encrypted1).is_err());
    }
}