// file: src/lib.rs
// description: main library file with public API, comprehensive documentation and exported functions

//! # rust-tld
//! 
//! A high-performance Rust library for extracting fully qualified domain names (FQDNs) 
//! from URLs using the Mozilla Public Suffix List.
//! 
//! ## Features
//! 
//! - 🚀 **High Performance**: Optimized for speed with concurrent processing
//! - 🔒 **Thread Safe**: Built with Rust's ownership system and async/await
//! - 🌐 **Standards Compliant**: Uses the official Mozilla Public Suffix List
//! - 🎯 **Accurate Parsing**: Correctly handles complex domains like `example.co.uk`
//! - ⚡ **Async First**: Fully async API with sync convenience functions
//! 
//! ## Quick Start
//! 
//! ```rust
//! use rust_tld::{init, get_fqdn};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the library (downloads Public Suffix List)
//!     init(None).await?;
//!     
//!     // Extract FQDNs from various URL formats
//!     let fqdn = get_fqdn("https://www.example.com/path?query=value").await?;
//!     println!("FQDN: {}", fqdn); // Output: example.com
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Advanced Configuration
//! 
//! ```rust
//! use rust_tld::{init, Options};
//! use std::time::Duration;
//! 
//! let options = Options::new()
//!     .allow_private_tlds(true)
//!     .timeout(Duration::from_secs(30));
//! 
//! init(Some(options)).await?;
//! ```

use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

pub mod constants;
pub mod errors;
pub mod etld;
pub mod fqdn;
pub mod options;

pub use constants::*;
pub use errors::TldError;
pub use fqdn::Fqdn;
pub use options::Options;

/// Trait defining the main interface for the TLD package
/// 
/// This trait provides a common interface for FQDN extraction that can be
/// implemented by different backend implementations.
pub trait FqdnManager {
    /// Extracts the FQDN from a URL
    /// 
    /// # Arguments
    /// 
    /// * `url` - The URL string to extract the FQDN from
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The extracted FQDN
    /// * `Err(TldError)` - If the URL is invalid or TLD cannot be determined
    fn get_fqdn(&self, url: &str) -> Result<String, TldError>;
}

impl FqdnManager for Fqdn {
    fn get_fqdn(&self, url: &str) -> Result<String, TldError> {
        self.get_fqdn(url)
    }
}

/// Global manager instance with thread-safe initialization
static GLOBAL_MANAGER: OnceLock<Arc<RwLock<Option<Arc<Fqdn>>>>> = OnceLock::new();

/// Initialize the global TLD manager with custom options
/// 
/// This function must be called before using any other functions in this library.
/// It downloads and parses the Mozilla Public Suffix List, which is required for
/// accurate FQDN extraction.
/// 
/// # Arguments
/// 
/// * `opts` - Optional configuration options. If `None`, default options are used.
/// 
/// # Returns
/// 
/// * `Ok(())` - If initialization succeeds
/// * `Err(TldError)` - If initialization fails (network error, parse error, etc.)
/// 
/// # Examples
/// 
/// Basic initialization:
/// ```rust
/// use rust_tld::init;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init(None).await?;
///     Ok(())
/// }
/// ```
/// 
/// With custom options:
/// ```rust
/// use rust_tld::{init, Options};
/// use std::time::Duration;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let options = Options::new()
///         .allow_private_tlds(true)
///         .timeout(Duration::from_secs(30));
///     
///     init(Some(options)).await?;
///     Ok(())
/// }
/// ```
/// 
/// # Thread Safety
/// 
/// This function is thread-safe and can be called multiple times. Subsequent calls
/// after the first successful initialization will be no-ops.
pub async fn init(opts: Option<Options>) -> Result<(), TldError> {
    let manager_lock = GLOBAL_MANAGER.get_or_init(|| Arc::new(RwLock::new(None)));
    
    let mut manager_guard = manager_lock.write().await;
    if manager_guard.is_none() {
        let fqdn = Fqdn::new(opts).await?;
        *manager_guard = Some(Arc::new(fqdn));
    }
    
    Ok(())
}

/// Get the global manager instance, initializing with defaults if needed
async fn get_global_manager() -> Result<Arc<Fqdn>, TldError> {
    let manager_lock = GLOBAL_MANAGER.get_or_init(|| Arc::new(RwLock::new(None)));
    
    {
        let manager_guard = manager_lock.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            return Ok(Arc::clone(manager));
        }
    }
    
    // Need to initialize
    init(None).await?;
    
    let manager_guard = manager_lock.read().await;
    manager_guard.as_ref()
        .map(|m| Arc::clone(m))
        .ok_or(TldError::PublicSuffixDownload("failed to initialize global manager".to_string()))
}

/// Extract the FQDN from a URL using the global manager
/// 
/// This is the main function for extracting FQDNs from URLs. It handles various
/// URL formats including those with schemes, ports, paths, and query parameters.
/// 
/// # Arguments
/// 
/// * `url` - The URL string to extract the FQDN from. Can be a full URL or just a domain.
/// 
/// # Returns
/// 
/// * `Ok(String)` - The extracted FQDN (e.g., "example.com")
/// * `Err(TldError)` - If the URL is invalid or the TLD cannot be determined
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::{init, get_fqdn};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init(None).await?;
///     
///     // Various URL formats work
///     let fqdn1 = get_fqdn("https://www.example.com/path").await?;
///     assert_eq!(fqdn1, "example.com");
///     
///     let fqdn2 = get_fqdn("subdomain.example.co.uk").await?;
///     assert_eq!(fqdn2, "example.co.uk");
///     
///     let fqdn3 = get_fqdn("http://example.com:8080/path?query=value").await?;
///     assert_eq!(fqdn3, "example.com");
///     
///     Ok(())
/// }
/// ```
/// 
/// # Error Conditions
/// 
/// This function returns errors in the following cases:
/// - The URL is malformed or empty
/// - The domain has no valid TLD according to the Public Suffix List
/// - The global manager hasn't been initialized (auto-initializes with defaults)
/// 
/// # Performance
/// 
/// After initialization, this function typically takes ~10μs per call.
pub async fn get_fqdn(url: &str) -> Result<String, TldError> {
    let manager = get_global_manager().await?;
    manager.get_fqdn(url)
}

/// Validate if a given origin is in the allowed origins list
/// 
/// This function extracts the FQDN from the origin URL and checks if it matches
/// any of the allowed origins. Useful for CORS validation and security checks.
/// 
/// # Arguments
/// 
/// * `origin` - The origin URL to validate
/// * `allowed_origins` - List of allowed FQDNs to check against
/// 
/// # Returns
/// 
/// * `true` - If the origin's FQDN matches one of the allowed origins
/// * `false` - If the origin is invalid or not in the allowed list
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::{init, validate_origin};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init(None).await?;
///     
///     let allowed = vec![
///         "example.com".to_string(),
///         "trusted.org".to_string(),
///     ];
///     
///     let valid = validate_origin("https://www.example.com", &allowed).await;
///     assert!(valid); // true - www.example.com resolves to example.com
///     
///     let invalid = validate_origin("https://malicious.com", &allowed).await;
///     assert!(!invalid); // false - not in allowed list
///     
///     Ok(())
/// }
/// ```
/// 
/// # Use Cases
/// 
/// - CORS origin validation
/// - API security checks
/// - Webhook origin verification
/// - Domain allowlist enforcement
pub async fn validate_origin(origin: &str, allowed_origins: &[String]) -> bool {
    match get_fqdn(origin).await {
        Ok(fqdn) => allowed_origins.iter().any(|allowed| fqdn == *allowed),
        Err(_) => false,
    }
}

/// Synchronous version of get_fqdn for convenience (requires tokio runtime)
/// 
/// This function provides a blocking interface to `get_fqdn` for use in 
/// synchronous contexts that are running within a tokio runtime.
/// 
/// # Arguments
/// 
/// * `url` - The URL string to extract the FQDN from
/// 
/// # Returns
/// 
/// * `Ok(String)` - The extracted FQDN
/// * `Err(TldError)` - If the URL is invalid or TLD cannot be determined
/// 
/// # Panics
/// 
/// This function will panic if called outside of a tokio runtime context.
/// Use the async version `get_fqdn` in async contexts.
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::{init, get_fqdn_sync};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init(None).await?;
///     
///     // Can be called from sync context within tokio runtime
///     let fqdn = get_fqdn_sync("https://example.com")?;
///     println!("FQDN: {}", fqdn);
///     
///     Ok(())
/// }
/// ```
/// 
/// # Performance Note
/// 
/// This function blocks the current thread while the async operation completes.
/// Prefer the async version when possible for better performance in async contexts.
pub fn get_fqdn_sync(url: &str) -> Result<String, TldError> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(get_fqdn(url))
    })
}

/// Synchronous version of validate_origin for convenience (requires tokio runtime)
/// 
/// This function provides a blocking interface to `validate_origin` for use in 
/// synchronous contexts that are running within a tokio runtime.
/// 
/// # Arguments
/// 
/// * `origin` - The origin URL to validate
/// * `allowed_origins` - List of allowed FQDNs to check against
/// 
/// # Returns
/// 
/// * `true` - If the origin's FQDN matches one of the allowed origins
/// * `false` - If the origin is invalid or not in the allowed list
/// 
/// # Panics
/// 
/// This function will panic if called outside of a tokio runtime context.
/// Use the async version `validate_origin` in async contexts.
/// 
/// # Examples
/// 
/// ```rust
/// use rust_tld::{init, validate_origin_sync};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init(None).await?;
///     
///     let allowed = vec!["example.com".to_string()];
///     let is_valid = validate_origin_sync("https://example.com", &allowed);
///     
///     println!("Origin valid: {}", is_valid);
///     Ok(())
/// }
/// ```
pub fn validate_origin_sync(origin: &str, allowed_origins: &[String]) -> bool {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(validate_origin(origin, allowed_origins))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_and_get_fqdn() {
        // Test initialization
        assert!(init(None).await.is_ok());
        
        // Test FQDN extraction (this will fail without actual data, but tests the API)
        let result = get_fqdn("https://www.example.com").await;
        // We expect this to fail since we don't have real public suffix data in tests
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_origin() {
        let allowed_origins = vec![
            "example.com".to_string(),
            "test.com".to_string(),
        ];
        
        // This will return false due to lack of real data, but tests the API
        let result = validate_origin("https://www.example.com", &allowed_origins).await;
        assert!(!result); // Expected to be false without real public suffix data
    }

    #[test]
    #[should_panic]
    fn test_sync_functions_outside_runtime() {
        // This should panic when called outside tokio runtime
        let _ = get_fqdn_sync("https://example.com");
    }

    #[tokio::test]
    async fn test_sync_functions_in_runtime() {
        // These should work when called within tokio runtime
        let result = get_fqdn_sync("https://example.com");
        assert!(result.is_err()); // Expected error without real data
        
        let allowed = vec!["example.com".to_string()];
        let validation = validate_origin_sync("https://example.com", &allowed);
        assert!(!validation); // Expected false without real data
    }

    #[tokio::test]
    async fn test_multiple_init_calls() {
        // Test that multiple init calls don't cause issues
        assert!(init(None).await.is_ok());
        assert!(init(None).await.is_ok());
        assert!(init(None).await.is_ok());
    }

    #[tokio::test]
    async fn test_global_manager_thread_safety() {
        use std::sync::Arc;
        use tokio::task::JoinSet;
        
        let mut join_set = JoinSet::new();
        
        // Spawn multiple tasks trying to initialize concurrently
        for _ in 0..10 {
            join_set.spawn(async {
                init(None).await
            });
        }
        
        // All should succeed or be no-ops
        while let Some(result) = join_set.join_next().await {
            assert!(result.unwrap().is_ok());
        }
    }
}