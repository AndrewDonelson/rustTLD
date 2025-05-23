// file: src/errors.rs
// description: defines error types for the package with production-ready error handling

use std::error::Error;
use std::fmt;

/// Custom error type for TLD-related operations
/// 
/// This enum represents all possible errors that can occur during TLD operations,
/// from URL parsing to public suffix list handling.
#[derive(Debug, Clone, PartialEq)]
pub enum TldError {
    /// Invalid URL provided
    /// 
    /// This error occurs when the provided URL string cannot be parsed
    /// or is malformed in some way.
    InvalidUrl,
    
    /// TLD not found in the public suffix list
    /// 
    /// This error occurs when the domain's TLD is not recognized
    /// according to the Mozilla Public Suffix List.
    InvalidTld,
    
    /// Failed to download public suffix file
    /// 
    /// This error occurs when network operations fail, including
    /// connection timeouts, HTTP errors, or DNS resolution failures.
    PublicSuffixDownload(String),
    
    /// Failed to parse public suffix file
    /// 
    /// This error occurs when the downloaded or loaded public suffix
    /// file cannot be parsed due to format issues or corruption.
    PublicSuffixParse(String),
    
    /// File is not the public suffix file
    /// 
    /// This error occurs when the loaded file doesn't contain the
    /// expected Mozilla Public Suffix List format or markers.
    PublicSuffixFormat(String),
}

impl fmt::Display for TldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TldError::InvalidUrl => write!(f, "invalid URL"),
            TldError::InvalidTld => write!(f, "invalid TLD"),
            TldError::PublicSuffixDownload(msg) => write!(f, "failed to download public suffix file: {msg}"),
            TldError::PublicSuffixParse(msg) => write!(f, "failed to parse public suffix file: {msg}"),
            TldError::PublicSuffixFormat(msg) => write!(f, "file is not the public suffix file: {msg}"),
        }
    }
}

impl Error for TldError {}

/// Wraps an error with additional context
/// 
/// This function takes a generic error and contextual message, then returns
/// an appropriate `TldError` variant based on the context.
/// 
/// # Arguments
/// 
/// * `err` - The source error to wrap
/// * `msg` - Contextual message describing what operation failed
/// 
/// # Returns
/// 
/// An appropriate `TldError` variant with the error message and context
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::errors::{wrap_error, TldError};
/// 
/// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
/// let wrapped = wrap_error(Box::new(io_error), "failed to download suffix list");
/// 
/// match wrapped {
///     TldError::PublicSuffixDownload(msg) => {
///         assert!(msg.contains("download"));
///     }
///     _ => panic!("Unexpected error type"),
/// }
/// ```
pub fn wrap_error(err: Box<dyn Error>, msg: &str) -> TldError {
    match err.downcast_ref::<TldError>() {
        Some(tld_err) => tld_err.clone(),
        None => match msg {
            m if m.contains("download") => TldError::PublicSuffixDownload(format!("{msg}: {err}")),
            m if m.contains("parse") => TldError::PublicSuffixParse(format!("{msg}: {err}")),
            m if m.contains("format") => TldError::PublicSuffixFormat(format!("{msg}: {err}")),
            _ => TldError::PublicSuffixDownload(format!("{msg}: {err}")),
        }
    }
}

/// Creates a `TldError::InvalidUrl` with optional context
/// 
/// This is a convenience function for creating invalid URL errors
/// with optional additional context.
/// 
/// # Arguments
/// 
/// * `context` - Optional context string to include in the error
/// 
/// # Returns
/// 
/// A `TldError::InvalidUrl` variant
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::errors::invalid_url_error;
/// 
/// let error = invalid_url_error(Some("URL too short"));
/// // Error will include the context in internal logging
/// ```
pub fn invalid_url_error(context: Option<&str>) -> TldError {
    // For now, InvalidUrl doesn't include context, but we could extend this
    // in the future if needed for better error reporting
    let _ = context; // Acknowledge parameter to avoid warnings
    TldError::InvalidUrl
}

/// Creates a `TldError::InvalidTld` with optional context
/// 
/// This is a convenience function for creating invalid TLD errors
/// with optional additional context.
/// 
/// # Arguments
/// 
/// * `context` - Optional context string to include in the error
/// 
/// # Returns
/// 
/// A `TldError::InvalidTld` variant
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::errors::invalid_tld_error;
/// 
/// let error = invalid_tld_error(Some("TLD not in public suffix list"));
/// // Error context can be used for debugging
/// ```
pub fn invalid_tld_error(context: Option<&str>) -> TldError {
    // For now, InvalidTld doesn't include context, but we could extend this
    // in the future if needed for better error reporting
    let _ = context; // Acknowledge parameter to avoid warnings
    TldError::InvalidTld
}

/// Type alias for Results that return TldError
/// 
/// This provides a convenient shorthand for functions that return
/// `Result<T, TldError>`.
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::errors::TldResult;
/// 
/// fn parse_domain(url: &str) -> TldResult<String> {
///     // Function implementation
///     Ok("example.com".to_string())
/// }
/// ```
pub type TldResult<T> = Result<T, TldError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let errors = vec![
            (TldError::InvalidUrl, "invalid URL"),
            (TldError::InvalidTld, "invalid TLD"),
            (
                TldError::PublicSuffixDownload("network error".to_string()),
                "failed to download public suffix file: network error"
            ),
            (
                TldError::PublicSuffixParse("bad format".to_string()),
                "failed to parse public suffix file: bad format"
            ),
            (
                TldError::PublicSuffixFormat("not PSL file".to_string()),
                "file is not the public suffix file: not PSL file"
            ),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(TldError::InvalidUrl, TldError::InvalidUrl);
        assert_eq!(TldError::InvalidTld, TldError::InvalidTld);
        assert_eq!(
            TldError::PublicSuffixDownload("test".to_string()),
            TldError::PublicSuffixDownload("test".to_string())
        );
        
        assert_ne!(TldError::InvalidUrl, TldError::InvalidTld);
        assert_ne!(
            TldError::PublicSuffixDownload("test1".to_string()),
            TldError::PublicSuffixDownload("test2".to_string())
        );
    }

    #[test]
    fn test_wrap_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        
        // Test download error wrapping
        let wrapped = wrap_error(Box::new(io_error), "failed to download");
        match wrapped {
            TldError::PublicSuffixDownload(msg) => {
                assert!(msg.contains("download"));
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected PublicSuffixDownload error"),
        }
        
        // Test parse error wrapping
        let parse_error = io::Error::new(io::ErrorKind::InvalidData, "bad data");
        let wrapped = wrap_error(Box::new(parse_error), "failed to parse");
        match wrapped {
            TldError::PublicSuffixParse(msg) => {
                assert!(msg.contains("parse"));
                assert!(msg.contains("bad data"));
            }
            _ => panic!("Expected PublicSuffixParse error"),
        }
        
        // Test format error wrapping
        let format_error = io::Error::new(io::ErrorKind::InvalidData, "wrong format");
        let wrapped = wrap_error(Box::new(format_error), "invalid format");
        match wrapped {
            TldError::PublicSuffixFormat(msg) => {
                assert!(msg.contains("format"));
                assert!(msg.contains("wrong format"));
            }
            _ => panic!("Expected PublicSuffixFormat error"),
        }
        
        // Test default case
        let other_error = io::Error::new(io::ErrorKind::Other, "other error");
        let wrapped = wrap_error(Box::new(other_error), "something else");
        match wrapped {
            TldError::PublicSuffixDownload(msg) => {
                assert!(msg.contains("something else"));
                assert!(msg.contains("other error"));
            }
            _ => panic!("Expected PublicSuffixDownload error as default"),
        }
    }

    #[test]
    fn test_wrap_existing_tld_error() {
        let existing_error = TldError::InvalidUrl;
        let wrapped = wrap_error(Box::new(existing_error.clone()), "additional context");
        
        // Should return the original TldError unchanged
        assert_eq!(wrapped, existing_error);
    }

    #[test]
    fn test_convenience_functions() {
        let url_error = invalid_url_error(Some("test context"));
        assert_eq!(url_error, TldError::InvalidUrl);
        
        let tld_error = invalid_tld_error(Some("test context"));
        assert_eq!(tld_error, TldError::InvalidTld);
        
        // Test without context
        let url_error = invalid_url_error(None);
        assert_eq!(url_error, TldError::InvalidUrl);
        
        let tld_error = invalid_tld_error(None);
        assert_eq!(tld_error, TldError::InvalidTld);
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TldError>();
    }

    #[test]
    fn test_error_source() {
        // Test that our error implements the Error trait properly
        let error = TldError::PublicSuffixDownload("test error".to_string());
        let error_trait: &dyn Error = &error;
        
        // Should not panic and should return our error message
        let display = format!("{error_trait}");
        assert!(display.contains("test error"));
        
        // Source should be None for our simple errors
        assert!(error_trait.source().is_none());
    }

    #[test]
    fn test_tld_result_type_alias() {
        fn test_function() -> TldResult<String> {
            Ok("success".to_string())
        }
        
        fn test_error_function() -> TldResult<String> {
            Err(TldError::InvalidUrl)
        }
        
        assert!(test_function().is_ok());
        assert!(test_error_function().is_err());
    }

    #[test]
    fn test_error_debug_format() {
        let error = TldError::PublicSuffixDownload("debug test".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("PublicSuffixDownload"));
        assert!(debug_str.contains("debug test"));
    }
}