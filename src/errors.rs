// file: src/errors.rs
// description: defines error types for the package

use std::error::Error;
use std::fmt;

/// Custom error type for TLD-related operations
#[derive(Debug, Clone, PartialEq)]
pub enum TldError {
    /// Invalid URL provided
    InvalidUrl,
    /// TLD not found in the public suffix list
    InvalidTld,
    /// Failed to download public suffix file
    PublicSuffixDownload(String),
    /// Failed to parse public suffix file
    PublicSuffixParse(String),
    /// File is not the public suffix file
    PublicSuffixFormat(String),
}

impl fmt::Display for TldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TldError::InvalidUrl => write!(f, "invalid URL"),
            TldError::InvalidTld => write!(f, "invalid TLD"),
            TldError::PublicSuffixDownload(msg) => write!(f, "failed to download public suffix file: {}", msg),
            TldError::PublicSuffixParse(msg) => write!(f, "failed to parse public suffix file: {}", msg),
            TldError::PublicSuffixFormat(msg) => write!(f, "file is not the public suffix file: {}", msg),
        }
    }
}

impl Error for TldError {}

/// Wraps an error with additional context
pub fn wrap_error(err: Box<dyn Error>, msg: &str) -> TldError {
    match err.downcast_ref::<TldError>() {
        Some(tld_err) => tld_err.clone(),
        None => match msg {
            m if m.contains("download") => TldError::PublicSuffixDownload(format!("{}: {}", msg, err)),
            m if m.contains("parse") => TldError::PublicSuffixParse(format!("{}: {}", msg, err)),
            m if m.contains("format") => TldError::PublicSuffixFormat(format!("{}: {}", msg, err)),
            _ => TldError::PublicSuffixDownload(format!("{}: {}", msg, err)),
        }
    }
}