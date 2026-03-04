/// `cipher-diary` – Error handling module
///
/// This file defines the main error type used throughout the project.
/// The errors are designed to be user‑friendly and to surface detailed diagnostics
/// for debugging while keeping the public API clean.
///
/// Author: Myroslav Mokhammad Abdeljawwad

use std::fmt::{self, Display};
use std::io;
use std::num::ParseIntError;

use thiserror::Error;

/// The central error type for `cipher-diary`.
///
/// It covers all possible failures from file I/O, encryption,
/// configuration parsing, and summary generation.  Each variant
/// contains context that is useful when logging or printing to the
/// user.
///
/// All variants are serializable via `thiserror`'s derive macro.
#[derive(Debug, Error)]
pub enum CipherDiaryError {
    /// Failure while reading a file (journal, config, template).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Configuration parsing failed (JSON/YAML/TOML/etc.).
    #[error("Configuration error: {0}")]
    Config(#[source] serde_json::Error),

    /// Encryption / decryption algorithm encountered an error.
    #[error("Encryption error: {0}")]
    Encrypt(String),

    /// Summary generation failed due to internal logic or invalid input.
    #[error("Summary generation error: {0}")]
    Summary(String),

    /// The user supplied a command line argument that could not be parsed.
    #[error("Argument parsing error: {0}")]
    ArgParse(#[source] clap::Error),

    /// An unexpected format was encountered in the journal file.
    #[error("Journal format error: {0}")]
    JournalFormat(String),

    /// A value could not be converted to an integer.
    #[error("Parsing integer failed: {0}")]
    ParseInt(#[from] ParseIntError),

    /// The template engine reported an error.
    #[error("Template rendering error: {0}")]
    Template(String),
}

impl From<serde_json::Error> for CipherDiaryError {
    fn from(err: serde_json::Error) -> Self {
        CipherDiaryError::Config(err)
    }
}

impl From<clap::Error> for CipherDiaryError {
    fn from(err: clap::Error) -> Self {
        CipherDiaryError::ArgParse(err)
    }
}

impl From<std::io::Error> for CipherDiaryError {
    fn from(err: std::io::Error) -> Self {
        CipherDiaryError::Io(err)
    }
}

/// Convenience type alias for `Result<T, CipherDiaryError>`.
pub type Result<T> = std::result::Result<T, CipherDiaryError>;

/// Utility functions that convert common errors into the unified error type.
///
/// These helpers are useful in tests and small helper modules where
/// you want to avoid a full `?` chain.

pub fn wrap_io_error<E: Into<io::Error>>(err: E) -> CipherDiaryError {
    err.into()
}

pub fn wrap_parse_int_error(err: ParseIntError) -> CipherDiaryError {
    CipherDiaryError::ParseInt(err)
}