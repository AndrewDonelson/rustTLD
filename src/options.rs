// file: src/options.rs
// description: defines options for the FQDN manager

use std::time::Duration;
use reqwest::Client;
use crate::constants::PUBLIC_SUFFIX_FILE_URL;

/// Options for the FQDN Manager
#[derive(Debug, Clone)]
pub struct Options {
    /// Determines whether private TLDs are allowed
    pub allow_private_tlds: bool,
    
    /// Timeout for HTTP requests
    pub timeout: Duration,
    
    /// Custom HTTP client for requests
    pub custom_http_client: Option<Client>,
    
    /// URL to download the public suffix list from
    pub public_suffix_url: String,
    
    /// Local file path containing the public suffix list
    pub public_suffix_file: Option<String>,
}

impl Options {
    /// Creates a new Options instance with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets whether private TLDs are allowed
    pub fn allow_private_tlds(mut self, allow: bool) -> Self {
        self.allow_private_tlds = allow;
        self
    }
    
    /// Sets the timeout for HTTP requests
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Sets a custom HTTP client
    pub fn custom_http_client(mut self, client: Client) -> Self {
        self.custom_http_client = Some(client);
        self
    }
    
    /// Sets the public suffix URL
    pub fn public_suffix_url<S: Into<String>>(mut self, url: S) -> Self {
        self.public_suffix_url = url.into();
        self
    }
    
    /// Sets the local public suffix file path
    pub fn public_suffix_file<S: Into<String>>(mut self, file: S) -> Self {
        self.public_suffix_file = Some(file.into());
        self
    }
}

impl Default for Options {
    /// Returns default options
    fn default() -> Self {
        Self {
            allow_private_tlds: false,
            timeout: Duration::from_secs(10),
            custom_http_client: None,
            public_suffix_url: PUBLIC_SUFFIX_FILE_URL.to_string(),
            public_suffix_file: None,
        }
    }
}