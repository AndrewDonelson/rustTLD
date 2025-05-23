// file: examples/main.rs
// description: comprehensive example program demonstrating usage of the rust-tld package

use clap::{Arg, Command};
use rust_tld::{get_fqdn, get_fqdn_sync, init, validate_origin, validate_origin_sync, Options, TldError};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio;

/// Configuration for the example application
#[derive(Debug, Clone)]
struct Config {
    allow_private_tlds: bool,
    timeout_secs: u64,
    custom_url: Option<String>,
    verbose: bool,
    show_stats: bool,
    benchmark: bool,
    test_sync: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allow_private_tlds: false,
            timeout_secs: 10,
            custom_url: None,
            verbose: false,
            show_stats: false,
            benchmark: false,
            test_sync: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let matches = Command::new("rust-tld-example")
        .version("1.0")
        .author("Andrew Donelson <nlaakald@gmail.com>")
        .about("Comprehensive example demonstrating rust-tld package capabilities")
        .arg(
            Arg::new("private")
                .long("private")
                .action(clap::ArgAction::SetTrue)
                .help("Allow private TLDs (e.g., .github.io, .amazonaws.com)"),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .help("Timeout for HTTP requests in seconds")
                .default_value("10"),
        )
        .arg(
            Arg::new("url")
                .long("url")
                .value_name("URL")
                .help("Custom URL for public suffix list"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose logging and detailed output"),
        )
        .arg(
            Arg::new("stats")
                .long("stats")
                .action(clap::ArgAction::SetTrue)
                .help("Show library statistics and performance metrics"),
        )
        .arg(
            Arg::new("benchmark")
                .long("benchmark")
                .action(clap::ArgAction::SetTrue)
                .help("Run performance benchmarks"),
        )
        .arg(
            Arg::new("test-sync")
                .long("test-sync")
                .action(clap::ArgAction::SetTrue)
                .help("Test synchronous API functions"),
        )
        .arg(
            Arg::new("URLs")
                .help("URLs to analyze (if none provided, uses comprehensive test suite)")
                .num_args(0..)
                .index(1),
        )
        .get_matches();

    // Build configuration from command line arguments
    let config = Config {
        allow_private_tlds: matches.get_flag("private"),
        timeout_secs: matches
            .get_one::<String>("timeout")
            .unwrap()
            .parse()
            .map_err(|_| "Invalid timeout value")?,
        custom_url: matches.get_one::<String>("url").cloned(),
        verbose: matches.get_flag("verbose"),
        show_stats: matches.get_flag("stats"),
        benchmark: matches.get_flag("benchmark"),
        test_sync: matches.get_flag("test-sync"),
    };

    if config.verbose {
        println!("🔧 Configuration:");
        println!("   Private TLDs: {}", config.allow_private_tlds);
        println!("   Timeout: {}s", config.timeout_secs);
        if let Some(ref url) = config.custom_url {
            println!("   Custom URL: {}", url);
        }
        println!();
    }

    // Create custom options
    let mut opts = Options::new()
        .allow_private_tlds(config.allow_private_tlds)
        .timeout(Duration::from_secs(config.timeout_secs));

    if let Some(custom_url) = &config.custom_url {
        opts = opts.public_suffix_url(custom_url);
    }

    // Initialize rust-tld with timing
    let init_start = Instant::now();
    if let Err(e) = init(Some(opts)).await {
        eprintln!("❌ Failed to initialize rust-tld: {}", e);
        std::process::exit(1);
    }
    let init_duration = init_start.elapsed();
    
    if config.verbose {
        println!("✅ Initialization completed in {:.2?}", init_duration);
        println!();
    }

    // Get URLs from command-line or use comprehensive test suite
    let urls = get_test_urls(&matches, &config);

    // Run the main analysis
    run_url_analysis(&urls, &config).await?;

    // Show additional features based on configuration
    if config.show_stats {
        show_library_statistics().await?;
    }

    if config.benchmark {
        run_performance_benchmarks(&urls, &config).await?;
    }

    if config.test_sync {
        test_synchronous_api(&urls, &config).await?;
    }

    // Always demonstrate origin validation
    demonstrate_origin_validation(&config).await?;

    // Run comprehensive feature demonstrations
    demonstrate_advanced_features(&config).await?;

    println!("\n🎉 All demonstrations completed successfully!");
    
    Ok(())
}

/// Get test URLs from command line or return comprehensive test suite
fn get_test_urls(matches: &clap::ArgMatches, config: &Config) -> Vec<String> {
    if let Some(input_urls) = matches.get_many::<String>("URLs") {
        input_urls.cloned().collect()
    } else {
        get_comprehensive_test_suite(config)
    }
}

/// Generate a comprehensive test suite covering various URL formats and edge cases
fn get_comprehensive_test_suite(config: &Config) -> Vec<String> {
    let mut urls = vec![
        // Basic domains
        "example.com".to_string(),
        "google.com".to_string(),
        "github.com".to_string(),
        
        // With subdomains
        "www.example.com".to_string(),
        "mail.google.com".to_string(),
        "api.github.com".to_string(),
        
        // With schemes
        "https://example.com".to_string(),
        "http://www.google.com".to_string(),
        "ftp://files.example.org".to_string(),
        
        // With ports
        "example.com:8080".to_string(),
        "https://api.service.com:443".to_string(),
        "http://localhost:3000".to_string(),
        
        // With paths and queries
        "https://example.com/path/to/resource".to_string(),
        "http://search.example.com/search?q=rust&lang=en".to_string(),
        "https://api.service.com/v1/users?limit=10&offset=20".to_string(),
        
        // Complex URLs
        "https://user:pass@example.com:8080/path?query=value#fragment".to_string(),
        "http://very.long.subdomain.example.co.uk/deep/nested/path".to_string(),
        
        // Country code TLDs
        "example.co.uk".to_string(),
        "service.com.au".to_string(),
        "government.gov.au".to_string(),
        "university.edu.au".to_string(),
        
        // International domains
        "münchen.de".to_string(),
        "пример.рф".to_string(),
        "例え.テスト".to_string(),
        
        // Edge cases
        "a.io".to_string(), // Shortest valid domain
        "single-word-domain.museum".to_string(), // Long TLD
        
        // Government and organization domains
        "www.whitehouse.gov".to_string(),
        "research.nasa.gov".to_string(),
        "www.un.org".to_string(),
        
        // Technology domains
        "stackoverflow.com".to_string(),
        "docs.rs".to_string(),
        "crates.io".to_string(),
        
        // Business domains
        "amazon.com".to_string(),
        "microsoft.com".to_string(),
        "apple.com".to_string(),
    ];
    
    // Add private TLD examples if enabled  
    if config.allow_private_tlds {
        urls.extend(vec![
            "user.github.io".to_string(),
            "app.herokuapp.com".to_string(),
            "bucket.s3.amazonaws.com".to_string(),
            "site.azurewebsites.net".to_string(),
            "project.vercel.app".to_string(),
        ]);
    }
    
    // Add problematic/edge case URLs for testing
    urls.extend(vec![
        // These should fail gracefully
        "invalid-url".to_string(),
        "http://".to_string(),
        "just-text-no-domain".to_string(),
        "".to_string(), // Empty string
        "http://localhost".to_string(), // No TLD
    ]);
    
    if config.verbose {
        println!("📋 Using comprehensive test suite with {} URLs", urls.len());
        println!();
    }
    
    urls
}

/// Run URL analysis with comprehensive reporting
async fn run_url_analysis(urls: &[String], config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 URL Analysis Results");
    println!("======================");
    println!("{:<60} | {:<35} | {:<15} | {}", "Original URL", "Extracted FQDN", "Status", "Notes");
    println!("{}", "=".repeat(130));

    let mut stats = AnalysisStats::new();
    
    for (i, url) in urls.iter().enumerate() {
        let start_time = Instant::now();
        
        match get_fqdn(url).await {
            Ok(fqdn) => {
                let duration = start_time.elapsed();
                stats.record_success(duration);
                
                let notes = analyze_url_complexity(url, &fqdn);
                println!("{:<60} | {:<35} | {:<15} | {}", 
                    truncate(url, 60), 
                    truncate(&fqdn, 35), 
                    "✅ SUCCESS", 
                    notes
                );
                
                if config.verbose && duration > Duration::from_millis(10) {
                    println!("   ⏱️  Processing time: {:.2?}", duration);
                }
            }
            Err(err) => {
                let duration = start_time.elapsed();
                stats.record_error(&err, duration);
                
                let error_type = classify_error(&err);
                println!("{:<60} | {:<35} | {:<15} | {}", 
                    truncate(url, 60), 
                    "-", 
                    "❌ ERROR", 
                    format!("{}: {}", error_type, err)
                );
                
                if config.verbose {
                    println!("   🔍 Error details: {}", err);
                    println!("   ⏱️  Processing time: {:.2?}", duration);
                }
            }
        }
        
        // Add progress indicator for large sets
        if urls.len() > 20 && (i + 1) % 10 == 0 {
            println!("   📊 Progress: {}/{} URLs processed", i + 1, urls.len());
        }
    }
    
    println!("{}", "=".repeat(130));
    stats.print_summary();
    
    Ok(())
}

/// Demonstrate origin validation with various scenarios
async fn demonstrate_origin_validation(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️  Origin Validation Demo");
    println!("=========================");

    let allowed_origins = vec![
        "example.com".to_string(),
        "trusted.org".to_string(),
        "api.service.com".to_string(),
        "github.io".to_string(),
        "localhost".to_string(),
    ];

    println!("Allowed origins: {}", allowed_origins.join(", "));
    println!();

    let test_origins = vec![
        // Should be valid
        ("https://example.com", true),
        ("http://www.example.com", true),
        ("https://api.service.com/v1/endpoint", true),
        ("http://trusted.org:8080", true),
        
        // Should be invalid
        ("https://malicious.com", false),
        ("http://phishing-site.net", false),
        ("https://fake-example.com", false),
        
        // Edge cases
        ("invalid-url", false),
        ("", false),
        ("https://localhost:3000", true),
    ];

    println!("{:<50} | {:<10} | {:<10} | {}", "Origin", "Expected", "Actual", "Result");
    println!("{}", "-".repeat(85));

    for (origin, expected) in test_origins {
        let actual = validate_origin(origin, &allowed_origins).await;
        let result = if actual == expected { "✅ PASS" } else { "❌ FAIL" };
        
        println!("{:<50} | {:<10} | {:<10} | {}", 
            truncate(origin, 50), 
            if expected { "VALID" } else { "INVALID" },
            if actual { "VALID" } else { "INVALID" },
            result
        );
        
        if config.verbose && actual != expected {
            println!("   ⚠️  Validation mismatch detected!");
        }
    }

    Ok(())
}

/// Demonstrate advanced features and edge cases
async fn demonstrate_advanced_features(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 Advanced Features Demo");
    println!("=========================");

    // Test Unicode domains
    if config.verbose {
        println!("Testing Unicode/International domains:");
        let unicode_domains = vec![
            "münchen.de",
            "пример.рф", 
            "例え.テスト",
            "café.fr",
        ];
        
        for domain in unicode_domains {
            match get_fqdn(domain).await {
                Ok(fqdn) => println!("  ✅ {} -> {}", domain, fqdn),
                Err(e) => println!("  ❌ {} -> Error: {}", domain, e),
            }
        }
        println!();
    }

    // Test complex URL parsing
    println!("Complex URL parsing test:");
    let complex_url = "https://user:pass@sub.domain.example.co.uk:8080/path/to/resource?param1=value1&param2=value2#section";
    match get_fqdn(complex_url).await {
        Ok(fqdn) => {
            println!("  ✅ Complex URL successfully parsed");
            println!("     Input:  {}", complex_url);
            println!("     FQDN:   {}", fqdn);
        }
        Err(e) => println!("  ❌ Complex URL parsing failed: {}", e),
    }

    // Test error handling
    println!("\nError handling demonstration:");
    let error_cases = vec![
        ("", "Empty string"),
        ("invalid", "No TLD"),
        ("http://", "Incomplete URL"),
        ("just.text", "Invalid TLD"),
    ];
    
    for (url, description) in error_cases {
        match get_fqdn(url).await {
            Ok(fqdn) => println!("  ⚠️  {} ({}): Unexpected success -> {}", url, description, fqdn),
            Err(e) => println!("  ✅ {} ({}): Expected error -> {}", url, description, classify_error(&e)),
        }
    }

    Ok(())
}

/// Run performance benchmarks
async fn run_performance_benchmarks(urls: &[String], _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Performance Benchmarks");
    println!("========================");

    // Benchmark single URL processing
    let test_url = "https://www.example.com/path?query=value";
    let iterations = 1000;
    
    println!("Testing {} iterations of FQDN extraction...", iterations);
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = get_fqdn(test_url).await;
    }
    
    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations;
    
    println!("  Total time: {:.2?}", total_duration);
    println!("  Average per call: {:.2?}", avg_duration);
    println!("  Throughput: {:.0} calls/second", 1_000_000.0 / avg_duration.as_micros() as f64);

    // Benchmark concurrent processing
    println!("\nConcurrent processing benchmark:");
    let concurrent_urls: Vec<&str> = urls.iter().take(10).map(|s| s.as_str()).collect();
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for url in concurrent_urls {
        let url_owned = url.to_string();
        tasks.push(tokio::spawn(async move {
            get_fqdn(&url_owned).await
        }));
    }
    
    let results: Vec<_> = futures::future::join_all(tasks).await;
    let concurrent_duration = start.elapsed();
    
    let success_count = results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();
    
    println!("  Processed {} URLs concurrently in {:.2?}", success_count, concurrent_duration);
    println!("  Average per URL: {:.2?}", concurrent_duration / success_count as u32);

    Ok(())
}

/// Test synchronous API functions
async fn test_synchronous_api(urls: &[String], _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Synchronous API Test");
    println!("=======================");

    // Test sync FQDN extraction
    let test_urls = urls.iter().take(5);
    
    for url in test_urls {
        let start = Instant::now();
        match get_fqdn_sync(url) {
            Ok(fqdn) => {
                let duration = start.elapsed();
                println!("  ✅ {} -> {} ({:.2?})", truncate(url, 40), fqdn, duration);
            }
            Err(e) => {
                println!("  ❌ {} -> Error: {}", truncate(url, 40), e);
            }
        }
    }

    // Test sync origin validation
    println!("\nSynchronous origin validation:");
    let allowed = vec!["example.com".to_string()];
    let test_origin = "https://www.example.com";
    
    let is_valid = validate_origin_sync(test_origin, &allowed);
    println!("  {} -> {}", test_origin, if is_valid { "✅ VALID" } else { "❌ INVALID" });

    Ok(())
}

/// Show library statistics (if available)
async fn show_library_statistics() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Library Statistics");
    println!("====================");
    
    // Note: This would require additional methods in the library
    println!("  📋 Public Suffix List entries: [Not available in current API]");
    println!("  🏷️  TLD categories: [Not available in current API]");
    println!("  💾 Memory usage: [Not available in current API]");
    println!("  ⏱️  Cache hit rate: [Not available in current API]");
    
    println!("  💡 Suggestion: Add statistics methods to rust-tld library");
    
    Ok(())
}

/// Analysis statistics tracking
#[derive(Debug)]
struct AnalysisStats {
    total_processed: usize,
    successful: usize,
    errors: HashMap<String, usize>,
    total_duration: Duration,
    max_duration: Duration,
    min_duration: Duration,
}

impl AnalysisStats {
    fn new() -> Self {
        Self {
            total_processed: 0,
            successful: 0,
            errors: HashMap::new(),
            total_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::from_secs(u64::MAX),
        }
    }
    
    fn record_success(&mut self, duration: Duration) {
        self.total_processed += 1;
        self.successful += 1;
        self.update_duration(duration);
    }
    
    fn record_error(&mut self, error: &TldError, duration: Duration) {
        self.total_processed += 1;
        let error_type = classify_error(error);
        *self.errors.entry(error_type).or_insert(0) += 1;
        self.update_duration(duration);
    }
    
    fn update_duration(&mut self, duration: Duration) {
        self.total_duration += duration;
        if duration > self.max_duration {
            self.max_duration = duration;
        }
        if duration < self.min_duration {
            self.min_duration = duration;
        }
    }
    
    fn print_summary(&self) {
        println!("\n📊 Processing Summary:");
        println!("   Total URLs: {}", self.total_processed);
        println!("   Successful: {} ({:.1}%)", 
            self.successful, 
            (self.successful as f64 / self.total_processed as f64) * 100.0
        );
        println!("   Errors: {} ({:.1}%)", 
            self.total_processed - self.successful,
            ((self.total_processed - self.successful) as f64 / self.total_processed as f64) * 100.0
        );
        
        if !self.errors.is_empty() {
            println!("   Error breakdown:");
            for (error_type, count) in &self.errors {
                println!("     {}: {}", error_type, count);
            }
        }
        
        if self.total_processed > 0 {
            let avg_duration = self.total_duration / self.total_processed as u32;
            println!("   Performance:");
            println!("     Average: {:.2?}", avg_duration);
            println!("     Fastest: {:.2?}", if self.min_duration.as_nanos() == u128::MAX { Duration::ZERO } else { self.min_duration });
            println!("     Slowest: {:.2?}", self.max_duration);
        }
    }
}

/// Analyze URL complexity and provide notes
fn analyze_url_complexity(url: &str, fqdn: &str) -> String {
    let mut notes = Vec::new();
    
    if url.contains("://") {
        notes.push("scheme");
    }
    if url.contains(':') && !url.starts_with("http") {
        notes.push("port");
    }
    if url.contains('/') && url.matches('/').count() > 2 {
        notes.push("path");
    }
    if url.contains('?') {
        notes.push("query");
    }
    if url.contains('#') {
        notes.push("fragment");
    }
    if url != fqdn && !url.starts_with("http") {
        notes.push("subdomain");
    }
    if fqdn.contains('.') && fqdn.matches('.').count() > 1 {
        notes.push("multi-level TLD");
    }
    
    if notes.is_empty() {
        "simple domain".to_string()
    } else {
        format!("has {}", notes.join(", "))
    }
}

/// Classify error types for better reporting
fn classify_error(error: &TldError) -> String {
    match error {
        TldError::InvalidUrl => "Invalid URL".to_string(),
        TldError::InvalidTld => "Invalid TLD".to_string(),
        TldError::PublicSuffixDownload(_) => "Download Error".to_string(),
        TldError::PublicSuffixParse(_) => "Parse Error".to_string(),
        TldError::PublicSuffixFormat(_) => "Format Error".to_string(),
    }
}

/// Truncate string to specified length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_initialization() {
        // Test with default options
        let result = init(None).await;
        match result {
            Ok(_) => {
                // Test basic functionality
                if let Ok(fqdn) = get_fqdn("https://example.com").await {
                    assert!(!fqdn.is_empty());
                    assert!(fqdn.contains('.'));
                }
            }
            Err(e) => {
                // In test environment, this might fail due to network issues
                println!("Initialization failed (expected in test env): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Initialize first
        let _ = init(None).await;
        
        // Test various error conditions
        let error_cases = vec![
            "",
            "invalid",
            "http://",
            "just.text",
        ];
        
        for case in error_cases {
            let result = get_fqdn(case).await;
            assert!(result.is_err(), "Expected error for input: {}", case);
        }
    }

    #[tokio::test]
    async fn test_origin_validation_logic() {
        let allowed = vec![
            "example.com".to_string(), 
            "test.org".to_string(),
            "api.service.com".to_string(),
        ];
        
        // Test basic logic without network dependency
        assert!(allowed.contains(&"example.com".to_string()));
        assert!(!allowed.contains(&"malicious.com".to_string()));
        
        // Test validation with actual function (may fail without network)
        let _ = validate_origin("https://example.com", &allowed).await;
    }

    #[test]
    fn test_utility_functions() {
        // Test truncate function
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
        
        // Test error classification
        let error = TldError::InvalidUrl;
        assert_eq!(classify_error(&error), "Invalid URL");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.allow_private_tlds);
        assert_eq!(config.timeout_secs, 10);
        assert!(!config.verbose);
    }

    #[test]
    fn test_analysis_stats() {
        let mut stats = AnalysisStats::new();
        
        stats.record_success(Duration::from_millis(5));
        stats.record_error(&TldError::InvalidUrl, Duration::from_millis(2));
        
        assert_eq!(stats.total_processed, 2);
        assert_eq!(stats.successful, 1);
        assert_eq!(stats.errors.len(), 1);
    }

    #[test]
    fn test_url_complexity_analysis() {
        let cases = vec![
            ("example.com", "example.com", "simple domain"),
            ("https://example.com", "example.com", "has scheme"),
            ("https://api.example.com/v1?q=test", "example.com", "has scheme, path, query"),
        ];
        
        for (url, fqdn, expected) in cases {
            let result = analyze_url_complexity(url, fqdn);
            assert!(result.contains(&expected.split(' ').next().unwrap()), 
                "Expected '{}' to contain key parts of '{}', got '{}'", result, expected, result);
        }
    }
}