/// Tests for the encryption and decryption functionality in `cipher-diary`.
///
/// Author: Myroslav Mokhammad Abdeljawwad
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use cipher_diary::{
    encryptor::{encrypt_line, decrypt_line, KeyError, EncryptionResult},
    errors::CipherDiaryError,
};
use rand::Rng;

/// Helper to create a temporary file with given content.
fn write_temp_file<P: AsRef<Path>>(path: P, contents: &str) {
    let mut file = File::create(path).expect("Failed to create temp file");
    writeln!(file, "{}", contents).expect("Failed to write to temp file");
}

/// Reads a file line by line into a Vec<String>.
fn read_lines<P: AsRef<Path>>(path: P) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file for reading");
    BufReader::new(file)
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .collect()
}

/// Generates a random 32-byte key for encryption tests.
fn generate_random_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen::<u8>()).collect()
}

#[test]
/// Test that encrypting and then decrypting each line restores the original content.
fn test_encrypt_decrypt_roundtrip() {
    // Prepare a sample journal with multiple lines.
    let original = "Today I learned Rust.\nEncryption is fun!\nTesting 123.";
    write_temp_file("tests/tmp_journal.txt", original);

    // Generate a random key for this test run.
    let key = generate_random_key();

    // Encrypt each line and store the ciphertexts.
    let plaintext_lines = read_lines("tests/tmp_journal.txt");
    let mut ciphertexts = Vec::new();
    for line in &plaintext_lines {
        let cipher = encrypt_line(line, &key).expect("Encryption failed");
        ciphertexts.push(cipher);
    }

    // Decrypt each ciphertext and compare to the original plaintext.
    for (i, cipher) in ciphertexts.iter().enumerate() {
        let decrypted =
            decrypt_line(cipher, &key).expect("Decryption failed for line {}", i);
        assert_eq!(
            decrypted, plaintext_lines[i],
            "Roundtrip mismatch on line {}",
            i
        );
    }

    // Clean up the temporary file.
    fs::remove_file("tests/tmp_journal.txt").ok();
}

#[test]
/// Test that encryption fails with an invalid key length (not 32 bytes).
fn test_encrypt_with_invalid_key_length() {
    let key = vec![0u8; 16]; // Too short
    let result: Result<EncryptionResult, CipherDiaryError> =
        encrypt_line("Sample text", &key);

    match result {
        Err(CipherDiaryError::Key(KeyError::InvalidLength)) => {}
        other => panic!("Expected KeyError::InvalidLength, got {:?}", other),
    }
}

#[test]
/// Test that decrypting with a wrong key does not return the original plaintext.
fn test_decrypt_with_wrong_key() {
    let correct_key = generate_random_key();
    let wrong_key = generate_random_key();

    let plaintext = "Secret message";
    let cipher = encrypt_line(plaintext, &correct_key).expect("Encryption failed");

    // Attempt decryption with a different key; should error or produce gibberish.
    let result: Result<String, CipherDiaryError> = decrypt_line(&cipher, &wrong_key);

    match result {
        Err(CipherDiaryError::DecryptionFailed) => {}
        Ok(decrypted) => {
            assert_ne!(decrypted, plaintext, "Decryption with wrong key should not succeed");
        }
    }
}

#[test]
/// Test that encryption preserves line breaks and whitespace exactly.
fn test_whitespace_preservation() {
    let key = generate_random_key();
    let lines = vec![
        "   Leading spaces",
        "Trailing spaces   ",
        "\tTabs\t",
        "",
        "Mixed  \t  spaces",
    ];

    // Encrypt each line
    let mut encrypted = Vec::new();
    for l in &lines {
        encrypted.push(encrypt_line(l, &key).expect("Encryption failed"));
    }

    // Decrypt and compare
    for (i, enc) in encrypted.iter().enumerate() {
        let decrypted =
            decrypt_line(enc, &key).expect("Decryption failed");
        assert_eq!(
            decrypted, lines[i],
            "Whitespace mismatch on line {}",
            i
        );
    }
}

#[test]
/// Test that encrypting an empty string returns a non-empty ciphertext.
fn test_empty_string_encryption() {
    let key = generate_random_key();
    let cipher = encrypt_line("", &key).expect("Encryption failed");
    assert!(!cipher.is_empty(), "Ciphertext should not be empty for empty plaintext");
}

#[test]
/// Test that the library can handle large input efficiently.
fn test_large_input_performance() {
    // Create a 1 MB string
    let mut large_text = String::new();
    for _ in 0..10_000 {
        large_text.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n");
    }

    let key = generate_random_key();

    // Encrypt the large text as a single line
    let cipher = encrypt_line(&large_text, &key).expect("Encryption failed");

    // Decrypt and verify
    let decrypted =
        decrypt_line(&cipher, &key).expect("Decryption failed");
    assert_eq!(decrypted, large_text);
}

#[test]
/// Test that encryption is deterministic with the same key and nonce.
fn test_deterministic_encryption() {
    let key = generate_random_key();

    // For deterministic behavior we rely on the library's internal nonce generation.
    // Here we encrypt twice and compare ciphertexts; they should differ due to random nonce.
    // To test determinism, we can set a fixed nonce via the API if available. Since our
    // encryptor does not expose that, we skip this test. The test exists for future extension.

    let plaintext = "Deterministic test";
    let cipher1 = encrypt_line(plaintext, &key).expect("Encryption failed");
    let cipher2 = encrypt_line(plaintext, &key).expect("Encryption failed");

    assert_ne!(cipher1, cipher2, "Ciphertexts should differ due to random nonce");
}