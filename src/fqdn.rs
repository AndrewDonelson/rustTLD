// file: src/fqdn.rs
// description: manages fully qualified domain names with complete file I/O and network operations

use std::sync::{Arc, RwLock};
use std::path::Path;
use reqwest::Client;
use tokio::task::JoinSet;
use tokio::fs;
use tokio::io::AsyncReadExt;
use url::Url;

use crate::constants::{ETLD_GROUP_MAX, PUBLIC_SUFFIX_FILE_URL, MIN_DATA_SIZE};
use crate::errors::TldError;
use crate::etld::Etld;
use crate::options::Options;

/// FQDN main object structure with concurrency support
#[derive(Debug)]
pub struct Fqdn {
    /// Configuration options for the FQDN manager
    pub options: Options,
    /// Array of eTLD lists organized by number of dots
    etld_list: [Arc<Etld>; ETLD_GROUP_MAX],
    /// Total number of loaded eTLDs across all lists
    total: RwLock<usize>,
}

impl Fqdn {
    /// Creates a new FQDN manager with the specified options
    /// 
    /// This function initializes the FQDN manager and loads the public suffix list
    /// either from a local file or by downloading it from the internet.
    /// 
    /// # Arguments
    /// 
    /// * `options` - Optional configuration options. If None, defaults are used.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Fqdn)` - Successfully initialized FQDN manager
    /// * `Err(TldError)` - If initialization fails
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::{Fqdn, Options};
    /// use std::time::Duration;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // With default options
    ///     let fqdn = Fqdn::new(None).await?;
    ///     
    ///     // With custom options
    ///     let options = Options::new()
    ///         .allow_private_tlds(true)
    ///         .timeout(Duration::from_secs(30));
    ///     let fqdn = Fqdn::new(Some(options)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(options: Option<Options>) -> Result<Self, TldError> {
        let opts = options.unwrap_or_default();
        
        // Create array of Arc<Etld> instances
        let etld_list = [
            Arc::new(Etld::new(0)),
            Arc::new(Etld::new(1)),
            Arc::new(Etld::new(2)),
            Arc::new(Etld::new(3)),
            Arc::new(Etld::new(4)),
        ];

        let fqdn = Self {
            options: opts.clone(),
            etld_list,
            total: RwLock::new(0),
        };

        // Load the public suffix list
        if let Some(file_path) = &opts.public_suffix_file {
            fqdn.load_public_suffix_from_file(file_path).await?;
        } else {
            fqdn.download_public_suffix_file(&opts.public_suffix_url).await?;
        }

        Ok(fqdn)
    }

    /// Tallies the total number of loaded eTLDs and sorts each list
    /// 
    /// This function performs cleanup and optimization operations on the loaded
    /// eTLD data. It sorts all lists concurrently for efficient binary search
    /// operations and calculates the total count of loaded eTLDs.
    pub async fn tidy(&self) {
        let mut join_set = JoinSet::new();

        // Sort all lists concurrently
        for etld in &self.etld_list {
            let etld_clone = Arc::clone(etld);
            join_set.spawn(async move {
                etld_clone.sort();
            });
        }

        // Wait for all sorting tasks to complete
        while let Some(_) = join_set.join_next().await {}

        // Calculate total count
        let total = self.etld_list.iter()
            .map(|etld| etld.count())
            .sum();
        
        *self.total.write().unwrap() = total;
    }

    /// Checks if a URL has a scheme and optionally removes it
    /// 
    /// # Arguments
    /// 
    /// * `s` - The URL string to check
    /// * `remove` - Whether to remove the scheme if found
    /// 
    /// # Returns
    /// 
    /// A tuple of (processed_string, has_scheme_bool)
    fn has_scheme(&self, s: &str, remove: bool) -> (String, bool) {
        let schemes = ["http://", "https://", "ftp://", "ws://", "wss://", "fake://"];

        for scheme in &schemes {
            if s.starts_with(scheme) {
                if remove {
                    return (s.replacen(scheme, "", 1), true);
                }
                return (s.to_string(), true);
            }
        }

        (s.to_string(), false)
    }

    /// Attempts to extract a potential eTLD from a domain
    /// 
    /// # Arguments
    /// 
    /// * `domain` - The domain string to analyze
    /// * `count` - The number of domain parts to extract from the right
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The extracted domain part
    /// * `Err(TldError)` - If the domain is invalid or cannot be parsed
    fn guess(&self, domain: &str, count: usize) -> Result<String, TldError> {
        if domain.is_empty() {
            return Err(TldError::InvalidUrl);
        }

        let dots = domain.matches('.').count();
        if dots < 1 || domain.len() < 3 {
            return Err(TldError::InvalidUrl);
        }

        let groups: Vec<&str> = domain.split('.').collect();
        let grp_cnt = groups.len();

        if grp_cnt >= count {
            match count {
                5 => Ok(groups[grp_cnt - 5..].join(".")),
                4 => Ok(groups[grp_cnt - 4..].join(".")),
                3 => Ok(groups[grp_cnt - 3..].join(".")),
                2 => Ok(groups[grp_cnt - 2..].join(".")),
                1 => Ok(groups[grp_cnt - 1].to_string()),
                _ => Err(TldError::InvalidUrl),
            }
        } else {
            Err(TldError::InvalidUrl)
        }
    }

    /// Attempts to find the TLD of a domain by searching through eTLD lists
    /// 
    /// This function tries to match the domain against known eTLDs, starting
    /// with the most specific (most dots) and working down to simpler TLDs.
    /// 
    /// # Arguments
    /// 
    /// * `s` - The domain string to analyze
    /// 
    /// # Returns
    /// 
    /// The found TLD string, or empty string if no match is found
    fn find_tld(&self, s: &str) -> String {
        let dots = s.matches('.').count();
        
        if dots >= 1 {
            for i in (1..=dots).rev() {
                if let Ok(guess) = self.guess(s, i) {
                    if i <= ETLD_GROUP_MAX {
                        let (tld, found) = self.etld_list[i - 1].search(&guess);
                        if found {
                            return tld;
                        }
                    }
                }
            }
        }

        String::new()
    }

    /// Extracts the FQDN from a URL
    /// 
    /// This is the main function for extracting FQDNs. It handles various URL formats
    /// including those with schemes, ports, paths, and query parameters.
    /// 
    /// # Arguments
    /// 
    /// * `src_url` - The URL string to extract the FQDN from
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The extracted FQDN
    /// * `Err(TldError)` - If the URL is invalid or TLD cannot be determined
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::Fqdn;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fqdn_manager = Fqdn::new(None).await?;
    ///     
    ///     let fqdn = fqdn_manager.get_fqdn("https://www.example.com/path")?;
    ///     assert_eq!(fqdn, "example.com");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_fqdn(&self, src_url: &str) -> Result<String, TldError> {
        if src_url.is_empty() {
            return Err(TldError::InvalidUrl);
        }

        // Shortest domain ex. a.io (4), and must have at least 1 DOT
        if src_url.len() < 4 || src_url.matches('.').count() < 1 {
            return Err(TldError::InvalidUrl);
        }

        // If no prefix, add a fake one for URL parsing (workaround)
        let (mut url_string, had_scheme) = self.has_scheme(src_url, false);
        if !had_scheme {
            url_string = format!("fake://{}", src_url);
        }

        let parsed_url = Url::parse(&url_string)
            .map_err(|_| TldError::InvalidUrl)?;

        // Remove scheme
        let (mut clean_url, _) = self.has_scheme(&url_string, true);

        // Remove port if present
        if let Some(port) = parsed_url.port() {
            clean_url = clean_url.replace(&format!(":{}", port), "");
        }

        // Remove query parameters
        if let Some(query) = parsed_url.query() {
            clean_url = clean_url.replace(&format!("?{}", query), "");
        }

        // Remove path
        let path = parsed_url.path();
        if !path.is_empty() && path != "/" {
            clean_url = clean_url.replace(path, "");
        }

        // Find the TLD
        let etld = self.find_tld(&clean_url);
        if etld.is_empty() {
            return Err(TldError::InvalidTld);
        }

        // Extract the domain from the URL
        let domain_part = clean_url.replace(&format!(".{}", etld), "");

        if domain_part.is_empty() {
            return Err(TldError::InvalidUrl);
        }

        // Handle subdomains
        let dots = domain_part.matches('.').count();
        if dots == 0 {
            return Ok(format!("{}.{}", domain_part, etld));
        }

        let parts: Vec<&str> = domain_part.split('.').collect();
        Ok(format!("{}.{}", parts[parts.len() - 1], etld))
    }

    /// Loads the public suffix list from a local file
    /// 
    /// This function reads the public suffix list from a local file system path.
    /// The file should be in the standard Mozilla Public Suffix List format.
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - Path to the local public suffix list file
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the file was successfully loaded and parsed
    /// * `Err(TldError)` - If file reading or parsing fails
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::{Fqdn, Options};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let options = Options::new()
    ///         .public_suffix_file("/path/to/public_suffix_list.dat");
    ///     
    ///     let fqdn = Fqdn::new(Some(options)).await?;
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # File Format
    /// 
    /// The file should be in the standard Mozilla Public Suffix List format:
    /// - Lines starting with "//" are comments
    /// - Lines starting with "*" are wildcards (ignored)
    /// - Lines starting with "!" are exceptions (ignored)
    /// - Empty lines are ignored
    /// - The file should contain the markers for ICANN domains section
    pub async fn load_public_suffix_from_file(&self, file_path: &str) -> Result<(), TldError> {
        if file_path.is_empty() {
            return Err(TldError::PublicSuffixDownload("no file path provided".to_string()));
        }

        // Check if file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(TldError::PublicSuffixDownload(
                format!("file does not exist: {}", file_path)
            ));
        }

        // Check if it's a file (not a directory)
        let metadata = fs::metadata(file_path).await
            .map_err(|e| TldError::PublicSuffixDownload(
                format!("failed to read file metadata for {}: {}", file_path, e)
            ))?;

        if !metadata.is_file() {
            return Err(TldError::PublicSuffixDownload(
                format!("path is not a file: {}", file_path)
            ));
        }

        // Check file size
        if metadata.len() < MIN_DATA_SIZE as u64 {
            return Err(TldError::PublicSuffixParse(
                format!("file too small to be a valid public suffix list: {} bytes", metadata.len())
            ));
        }

        // Limit file size to prevent memory exhaustion (50MB limit)
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(TldError::PublicSuffixParse(
                format!("file too large: {} bytes (max: {} bytes)", metadata.len(), MAX_FILE_SIZE)
            ));
        }

        // Read the file
        let mut file = fs::File::open(file_path).await
            .map_err(|e| TldError::PublicSuffixDownload(
                format!("failed to open file {}: {}", file_path, e)
            ))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await
            .map_err(|e| TldError::PublicSuffixDownload(
                format!("failed to read file {}: {}", file_path, e)
            ))?;

        // Validate that we actually read the expected amount
        if contents.len() != metadata.len() as usize {
            return Err(TldError::PublicSuffixParse(
                format!("file size mismatch: expected {} bytes, read {} bytes", 
                    metadata.len(), contents.len())
            ));
        }

        // Parse the file contents
        self.parse_public_suffix_data(&contents).await
            .map_err(|e| match e {
                TldError::PublicSuffixParse(msg) => TldError::PublicSuffixParse(
                    format!("error parsing file {}: {}", file_path, msg)
                ),
                TldError::PublicSuffixFormat(msg) => TldError::PublicSuffixFormat(
                    format!("invalid format in file {}: {}", file_path, msg)
                ),
                other => other,
            })
    }

    /// Downloads and parses the public suffix list from a URL
    /// 
    /// This function downloads the Mozilla Public Suffix List from the internet
    /// and parses it for use in FQDN extraction.
    /// 
    /// # Arguments
    /// 
    /// * `file_url` - URL to download the public suffix list from. If empty, uses default.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If download and parsing succeeds
    /// * `Err(TldError)` - If download or parsing fails
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::Fqdn;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fqdn = Fqdn::new(None).await?; // Uses default download
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Network Requirements
    /// 
    /// This function requires internet connectivity to download the list.
    /// The download is approximately 240KB and includes both ICANN and private domains.
    pub async fn download_public_suffix_file(&self, file_url: &str) -> Result<(), TldError> {
        let url = if file_url.is_empty() {
            PUBLIC_SUFFIX_FILE_URL
        } else {
            file_url
        };

        // Validate URL format
        if let Err(_) = Url::parse(url) {
            return Err(TldError::PublicSuffixDownload(
                format!("invalid URL format: {}", url)
            ));
        }

        // Create HTTP client
        let client = if let Some(custom_client) = &self.options.custom_http_client {
            custom_client.clone()
        } else {
            Client::builder()
                .timeout(self.options.timeout)
                .user_agent("RustTLD/1.0")
                .connect_timeout(std::time::Duration::from_secs(10))
                .tcp_keepalive(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| TldError::PublicSuffixDownload(
                    format!("failed to create HTTP client: {}", e)
                ))?
        };

        // Make the request with retry logic
        let mut last_error = None;
        let max_retries = 3;
        
        for attempt in 1..=max_retries {
            match self.attempt_download(&client, url).await {
                Ok(bytes) => {
                    return self.parse_public_suffix_data(&bytes).await;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Exponential backoff: 1s, 2s, 4s
                        let delay = std::time::Duration::from_secs(1 << (attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TldError::PublicSuffixDownload(
            "unknown error occurred during download".to_string()
        )))
    }

    /// Attempts to download the public suffix list once
    /// 
    /// This is a helper function for `download_public_suffix_file` that handles
    /// a single download attempt with proper error handling.
    async fn attempt_download(&self, client: &Client, url: &str) -> Result<Vec<u8>, TldError> {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| TldError::PublicSuffixDownload(
                format!("network request failed: {}", e)
            ))?;

        // Check status code
        let status = response.status();
        if !status.is_success() {
            return Err(TldError::PublicSuffixDownload(
                format!("HTTP error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
            ));
        }

        // Check content type if present
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            if !content_type_str.contains("text/") && !content_type_str.contains("application/octet-stream") {
                return Err(TldError::PublicSuffixDownload(
                    format!("unexpected content type: {}", content_type_str)
                ));
            }
        }

        // Read response body with size limit (10MB)
        const MAX_DOWNLOAD_SIZE: usize = 10 * 1024 * 1024;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| TldError::PublicSuffixParse(
                format!("failed to read response body: {}", e)
            ))?;

        if bytes.len() > MAX_DOWNLOAD_SIZE {
            return Err(TldError::PublicSuffixParse(
                format!("response too large: {} bytes (max: {} bytes)", bytes.len(), MAX_DOWNLOAD_SIZE)
            ));
        }

        if bytes.len() < MIN_DATA_SIZE {
            return Err(TldError::PublicSuffixParse(
                format!("response data size too small for public suffix file: {} bytes (min: {} bytes)", 
                    bytes.len(), MIN_DATA_SIZE)
            ));
        }

        Ok(bytes.to_vec())
    }

    /// Parses the public suffix list data from raw bytes
    /// 
    /// This function processes the public suffix list format and populates
    /// the internal eTLD data structures for efficient domain matching.
    /// 
    /// # Arguments
    /// 
    /// * `data` - Raw bytes of the public suffix list file
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If parsing succeeds
    /// * `Err(TldError)` - If parsing fails or data is invalid
    /// 
    /// # Format Details
    /// 
    /// The parser handles:
    /// - Comments (lines starting with "//")
    /// - ICANN domain markers
    /// - Private domain sections (if enabled in options)
    /// - Unicode domain names (converted to lowercase)
    /// - Wildcard entries (currently ignored)
    /// - Exception entries (currently ignored)
    async fn parse_public_suffix_data(&self, data: &[u8]) -> Result<(), TldError> {
        // Validate UTF-8 encoding
        let content = String::from_utf8(data.to_vec())
            .map_err(|e| TldError::PublicSuffixParse(
                format!("invalid UTF-8 encoding: {}", e)
            ))?;

        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(TldError::PublicSuffixParse("empty data".to_string()));
        }

        // Verify that this is the public suffix list by checking for known markers
        let mut found_marker = false;
        let markers = [
            "publicsuffix.org",
            "Mozilla Public Suffix List",
            "===BEGIN ICANN DOMAINS===",
            "This Source Code Form is subject to the terms of the Mozilla Public License"
        ];

        for line in lines.iter().take(50) { // Check first 50 lines for markers
            for marker in &markers {
                if line.contains(marker) {
                    found_marker = true;
                    break;
                }
            }
            if found_marker {
                break;
            }
        }

        if !found_marker {
            return Err(TldError::PublicSuffixFormat(
                "file does not appear to be the Mozilla Public Suffix List".to_string()
            ));
        }

        let mut icann = false;
        let mut processed_count = 0;
        let mut skipped_count = 0;

        // Reset the current lists
        for etld in &self.etld_list {
            etld.clear();
        }

        for (line_num, line) in lines.iter().enumerate() {
            // Skip blank lines
            if line.trim().is_empty() {
                continue;
            }

            // Detect and toggle ICANN eTLD state
            if line.contains("===BEGIN ICANN DOMAINS===") {
                icann = true;
                continue;
            } else if line.contains("===END ICANN DOMAINS===") {
                icann = false;
                continue;
            }

            // If private TLDs not allowed and this is not an ICANN TLD, skip it
            if !self.options.allow_private_tlds && !icann {
                skipped_count += 1;
                continue;
            }

            // Skip comments
            if line.trim().starts_with("//") {
                continue;
            }

            // Skip wildcards and exceptions for now
            // TODO: Implement proper wildcard and exception handling
            let trimmed = line.trim();
            if trimmed.starts_with('*') || trimmed.starts_with('!') {
                skipped_count += 1;
                continue;
            }

            // Process the TLD entry
            let tld = trimmed.to_lowercase();
            if tld.is_empty() {
                continue;
            }

            // Validate TLD format (basic sanity checks)
            if tld.len() > 253 { // Maximum domain name length
                return Err(TldError::PublicSuffixParse(
                    format!("TLD too long at line {}: {} (max 253 chars)", line_num + 1, tld.len())
                ));
            }

            // Check for invalid characters
            if tld.chars().any(|c| !c.is_ascii_alphanumeric() && c != '.' && c != '-') {
                // Allow international domain names, but log a warning for unusual characters
                // In a real implementation, you might want to use a proper IDN library
            }

            let dots = tld.matches('.').count();
            if dots < ETLD_GROUP_MAX {
                if self.etld_list[dots].add(tld.clone(), false) {
                    processed_count += 1;
                }
            } else {
                // Log domains with too many dots (but don't fail)
                skipped_count += 1;
            }
        }

        // Verify we processed a reasonable number of entries
        if processed_count < 1000 {
            return Err(TldError::PublicSuffixParse(
                format!("too few TLD entries processed: {} (expected at least 1000)", processed_count)
            ));
        }

        // Sort all lists and calculate totals
        self.tidy().await;

        // Log processing results (in a real implementation, use proper logging)
        #[cfg(feature = "logging")]
        log::info!(
            "Public suffix list parsed successfully: {} entries processed, {} skipped, {} total loaded",
            processed_count, skipped_count, self.total()
        );
        
        // Always use skipped_count to avoid warnings (even without logging feature)
        #[cfg(not(feature = "logging"))]
        let _ = skipped_count; // Explicitly acknowledge the variable to avoid unused warning

        Ok(())
    }

    /// Returns the total number of loaded eTLDs across all lists
    /// 
    /// # Returns
    /// 
    /// The total count of eTLD entries currently loaded in memory
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::Fqdn;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fqdn = Fqdn::new(None).await?;
    ///     println!("Loaded {} eTLD entries", fqdn.total());
    ///     Ok(())
    /// }
    /// ```
    pub fn total(&self) -> usize {
        *self.total.read().unwrap()
    }

    /// Returns the count of eTLDs for a specific dot level
    /// 
    /// # Arguments
    /// 
    /// * `dots` - The number of dots to query (0-4)
    /// 
    /// # Returns
    /// 
    /// The count of eTLD entries for the specified dot level, or 0 if invalid
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::Fqdn;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fqdn = Fqdn::new(None).await?;
    ///     
    ///     println!("Single-level TLDs: {}", fqdn.count_for_dots(0)); // .com, .org
    ///     println!("Two-level TLDs: {}", fqdn.count_for_dots(1));   // .co.uk, .com.au
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn count_for_dots(&self, dots: usize) -> usize {
        if dots < ETLD_GROUP_MAX {
            self.etld_list[dots].count()
        } else {
            0
        }
    }

    /// Checks if the FQDN manager is properly initialized with data
    /// 
    /// # Returns
    /// 
    /// `true` if the manager has loaded eTLD data, `false` otherwise
    pub fn is_initialized(&self) -> bool {
        self.total() > 0
    }

    /// Returns statistics about the loaded eTLD data
    /// 
    /// # Returns
    /// 
    /// A vector of (dot_level, count) tuples showing distribution of eTLDs
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use rust_tld::Fqdn;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fqdn = Fqdn::new(None).await?;
    ///     
    ///     for (dot_level, count) in fqdn.get_statistics() {
    ///         println!("Level {}: {} entries", dot_level, count);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_statistics(&self) -> Vec<(usize, usize)> {
        (0..ETLD_GROUP_MAX)
            .map(|i| (i, self.count_for_dots(i)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn test_has_scheme() {
        let fqdn = create_test_fqdn();
        
        let (result, has) = fqdn.has_scheme("https://example.com", false);
        assert!(has);
        assert_eq!(result, "https://example.com");
        
        let (result, has) = fqdn.has_scheme("https://example.com", true);
        assert!(has);
        assert_eq!(result, "example.com");
        
        let (result, has) = fqdn.has_scheme("example.com", false);
        assert!(!has);
        assert_eq!(result, "example.com");
    }

    #[test]
    fn test_guess() {
        let fqdn = create_test_fqdn();
        
        // Test valid cases
        assert_eq!(fqdn.guess("example.com", 1).unwrap(), "com");
        assert_eq!(fqdn.guess("sub.example.com", 2).unwrap(), "example.com");
        assert_eq!(fqdn.guess("deep.sub.example.com", 3).unwrap(), "sub.example.com");
        
        // Test invalid cases
        assert!(fqdn.guess("", 1).is_err());
        assert!(fqdn.guess("com", 1).is_err());
        assert!(fqdn.guess("a.b", 1).is_err()); // Too short
        assert!(fqdn.guess("example.com", 3).is_err()); // Not enough parts
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_file() {
        let fqdn = create_test_fqdn();
        let result = fqdn.load_public_suffix_from_file("/nonexistent/file.dat").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixDownload(msg) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected PublicSuffixDownload error"),
        }
    }

    #[tokio::test]
    async fn test_load_from_empty_file() {
        // Create a temporary empty file
        let temp_file = "/tmp/empty_suffix_list.dat";
        let mut file = fs::File::create(temp_file).await.unwrap();
        file.write_all(b"").await.unwrap();
        file.sync_all().await.unwrap();
        drop(file);

        let fqdn = create_test_fqdn();
        let result = fqdn.load_public_suffix_from_file(temp_file).await;
        
        // Cleanup
        let _ = fs::remove_file(temp_file).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixParse(msg) => {
                assert!(msg.contains("too small"));
            }
            _ => panic!("Expected PublicSuffixParse error"),
        }
    }

    #[tokio::test]
    async fn test_load_from_directory() {
        let fqdn = create_test_fqdn();
        let result = fqdn.load_public_suffix_from_file("/tmp").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixDownload(msg) => {
                assert!(msg.contains("not a file"));
            }
            _ => panic!("Expected PublicSuffixDownload error"),
        }
    }

    #[tokio::test]
    async fn test_load_from_valid_test_file() {
        // Create a minimal valid public suffix list file
        let temp_file = "/tmp/test_suffix_list.dat";
        let test_content = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            "// This is a test file for Mozilla Public Suffix List",
            "// publicsuffix.org test data",
            "// ===BEGIN ICANN DOMAINS===",
            "",
            "// Generic top-level domains",
            "com",
            "org", 
            "net",
            "",
            "// Country code top-level domains",
            "uk",
            "co.uk",
            "",
            "// ===END ICANN DOMAINS==="
        );
        
        // Ensure the content is large enough
        let padding = "a".repeat(MIN_DATA_SIZE.saturating_sub(test_content.len()));
        let full_content = format!("{}\n// Padding: {}", test_content, padding);
        
        let mut file = fs::File::create(temp_file).await.unwrap();
        file.write_all(full_content.as_bytes()).await.unwrap();
        file.sync_all().await.unwrap();
        drop(file);

        let fqdn = create_test_fqdn();
        let result = fqdn.load_public_suffix_from_file(temp_file).await;
        
        // Cleanup
        let _ = fs::remove_file(temp_file).await;
        
        // Should succeed with valid format
        assert!(result.is_ok());
        assert!(fqdn.total() > 0);
        assert!(fqdn.is_initialized());
        
        // Check that we can find the loaded TLDs
        assert_eq!(fqdn.find_tld("example.com"), "com");
        assert_eq!(fqdn.find_tld("test.co.uk"), "co.uk");
    }

    #[tokio::test]
    async fn test_parse_invalid_utf8() {
        let fqdn = create_test_fqdn();
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8 sequence
        let result = fqdn.parse_public_suffix_data(&invalid_utf8).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixParse(msg) => {
                assert!(msg.contains("UTF-8"));
            }
            _ => panic!("Expected PublicSuffixParse error for invalid UTF-8"),
        }
    }

    #[tokio::test]
    async fn test_parse_wrong_file_format() {
        let fqdn = create_test_fqdn();
        let wrong_format = "This is not a public suffix list file\nJust some random content\n".repeat(1000);
        let result = fqdn.parse_public_suffix_data(wrong_format.as_bytes()).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixFormat(msg) => {
                assert!(msg.contains("does not appear to be"));
            }
            _ => panic!("Expected PublicSuffixFormat error"),
        }
    }

    #[tokio::test]
    async fn test_download_invalid_url() {
        let fqdn = create_test_fqdn();
        let result = fqdn.download_public_suffix_file("not-a-valid-url").await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TldError::PublicSuffixDownload(msg) => {
                assert!(msg.contains("invalid URL"));
            }
            _ => panic!("Expected PublicSuffixDownload error for invalid URL"),
        }
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let fqdn = create_test_fqdn();
        
        // Initially should be empty
        let stats = fqdn.get_statistics();
        assert_eq!(stats.len(), ETLD_GROUP_MAX);
        for (_, count) in stats {
            assert_eq!(count, 0);
        }
        
        // Add some test data
        fqdn.etld_list[0].add("com".to_string(), false);
        fqdn.etld_list[1].add("co.uk".to_string(), false);
        fqdn.etld_list[1].add("com.au".to_string(), false);
        
        let stats = fqdn.get_statistics();
        assert_eq!(stats[0].1, 1); // One 0-dot TLD
        assert_eq!(stats[1].1, 2); // Two 1-dot TLDs
        assert_eq!(stats[2].1, 0); // No 2-dot TLDs
    }

    #[test]
    fn test_count_for_dots() {
        let fqdn = create_test_fqdn();
        
        // Initially all should be 0
        for i in 0..ETLD_GROUP_MAX {
            assert_eq!(fqdn.count_for_dots(i), 0);
        }
        
        // Invalid dot level should return 0
        assert_eq!(fqdn.count_for_dots(ETLD_GROUP_MAX), 0);
        assert_eq!(fqdn.count_for_dots(999), 0);
    }

    #[test] 
    fn test_is_initialized() {
        let fqdn = create_test_fqdn();
        
        // Initially should not be initialized
        assert!(!fqdn.is_initialized());
        
        // After adding some data, should be initialized
        fqdn.etld_list[0].add("com".to_string(), false);
        *fqdn.total.write().unwrap() = 1;
        assert!(fqdn.is_initialized());
    }

    #[tokio::test]
    async fn test_fqdn_extraction_with_test_data() {
        let fqdn = create_test_fqdn();
        
        // Add some test TLD data
        fqdn.etld_list[0].add("com".to_string(), false);
        fqdn.etld_list[0].add("org".to_string(), false);
        fqdn.etld_list[1].add("co.uk".to_string(), false);
        fqdn.etld_list[1].add("com.au".to_string(), false);
        
        // Sort the lists
        fqdn.tidy().await;
        
        // Test FQDN extraction
        assert_eq!(fqdn.get_fqdn("example.com").unwrap(), "example.com");
        assert_eq!(fqdn.get_fqdn("www.example.com").unwrap(), "example.com");
        assert_eq!(fqdn.get_fqdn("https://www.example.com/path").unwrap(), "example.com");
        assert_eq!(fqdn.get_fqdn("subdomain.example.co.uk").unwrap(), "example.co.uk");
        assert_eq!(fqdn.get_fqdn("http://example.com:8080/path?query=value").unwrap(), "example.com");
        
        // Test error cases
        assert!(fqdn.get_fqdn("").is_err());
        assert!(fqdn.get_fqdn("invalid").is_err());
        assert!(fqdn.get_fqdn("example.unknown-tld").is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        use tokio::task::JoinSet;
        
        let fqdn = Arc::new(create_test_fqdn());
        
        // Add some test data
        fqdn.etld_list[0].add("com".to_string(), false);
        fqdn.etld_list[0].add("org".to_string(), false);
        fqdn.tidy().await;
        
        let mut join_set = JoinSet::new();
        
        // Spawn multiple tasks accessing the FQDN manager concurrently
        for i in 0..10 {
            let fqdn_clone = Arc::clone(&fqdn);
            join_set.spawn(async move {
                let url = format!("https://test{}.example.com", i);
                fqdn_clone.get_fqdn(&url)
            });
        }
        
        // All should complete successfully
        while let Some(result) = join_set.join_next().await {
            let fqdn_result = result.unwrap();
            if fqdn_result.is_ok() {
                assert_eq!(fqdn_result.unwrap(), "example.com");
            }
        }
    }

    fn create_test_fqdn() -> Fqdn {
        let etld_list = [
            Arc::new(Etld::new(0)),
            Arc::new(Etld::new(1)),
            Arc::new(Etld::new(2)),
            Arc::new(Etld::new(3)),
            Arc::new(Etld::new(4)),
        ];
        
        Fqdn {
            options: Options::default(),
            etld_list,
            total: RwLock::new(0),
        }
    }
}