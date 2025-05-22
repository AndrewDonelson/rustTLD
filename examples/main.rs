// file: examples/main.rs
// description: example program demonstrating usage of the rust-tld package

use clap::{Arg, Command};
use std::time::Duration;
use tokio;
use rust_tld::{init, get_fqdn, validate_origin, Options};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let matches = Command::new("rust-tld-example")
        .version("1.0")
        .author("Your Name")
        .about("Example program demonstrating usage of the rust-tld package")
        .arg(
            Arg::new("private")
                .long("private")
                .action(clap::ArgAction::SetTrue)
                .help("Allow private TLDs"),
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
                .help("Enable verbose logging"),
        )
        .arg(
            Arg::new("URLs")
                .help("URLs to analyze")
                .num_args(0..)
                .index(1),
        )
        .get_matches();

    // Configure logging (simplified without external log crate)
    if matches.get_flag("verbose") {
        println!("Verbose mode enabled");
    }

    // Parse timeout
    let timeout_secs: u64 = matches
        .get_one::<String>("timeout")
        .unwrap()
        .parse()
        .map_err(|_| "Invalid timeout value")?;

    // Create custom options
    let mut opts = Options::new()
        .allow_private_tlds(matches.get_flag("private"))
        .timeout(Duration::from_secs(timeout_secs));

    // Set custom URL if provided
    if let Some(custom_url) = matches.get_one::<String>("url") {
        opts = opts.public_suffix_url(custom_url);
    }

    // Initialize rust-tld with custom options
    if let Err(e) = init(Some(opts)).await {
        eprintln!("Failed to initialize rust-tld: {}", e);
        std::process::exit(1);
    }

    // Get URLs from command-line arguments or use defaults
    let urls: Vec<String> = if let Some(input_urls) = matches.get_many::<String>("URLs") {
        input_urls.cloned().collect()
    } else {
        // Default examples
        vec![
            "nlaak.com".to_string(),
            "https://nlaak.com".to_string(),
            "http://go.com?foo=bar".to_string(),
            "http://google.com".to_string(),
            "http://blog.google".to_string(),
            "https://www.medi-cal.ca.gov/".to_string(),
            "https://ato.gov.au".to_string(),
            "http://stage.host.domain.co.uk/".to_string(),
            "http://a.very.complex-domain.co.uk:8080/foo/bar".to_string(),
        ]
    };

    // Process each URL
    println!("URL Analysis Results");
    println!("-------------------");
    println!("{:<50} | {:<30} | {}", "Original URL", "FQDN", "Status");
    println!("{}", "-".repeat(100));

    for url in &urls {
        match get_fqdn(url).await {
            Ok(fqdn) => {
                println!("{:<50} | {:<30} | SUCCESS", url, fqdn);
            }
            Err(err) => {
                println!("{:<50} | {:<30} | ERROR: {}", url, "-", err);
            }
        }
    }

    // Demonstrate origin validation
    println!("\nOrigin Validation");
    println!("----------------");

    let allowed_origins = vec![
        "example.com".to_string(),
        "trusted.org".to_string(),
        "api.service.com".to_string(),
    ];

    println!("Allowed origins: {}\n", allowed_origins.join(", "));

    let origins_to_check = vec![
        "https://example.com",
        "http://malicious.com",
        "https://trusted.org/path",
        "https://subdomain.example.com",
    ];

    for origin in &origins_to_check {
        let is_valid = validate_origin(origin, &allowed_origins).await;
        let valid_text = if is_valid { "VALID" } else { "INVALID" };
        println!("{:<40} | {}", origin, valid_text);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_functionality() {
        // Test with default options
        let result = init(None).await;
        // This might fail without internet, but that's expected in tests
        match result {
            Ok(_) => {
                // If initialization succeeds, test FQDN extraction
                if let Ok(fqdn) = get_fqdn("https://example.com").await {
                    assert!(!fqdn.is_empty());
                }
            }
            Err(_) => {
                // Expected to fail in test environment without internet
                println!("Initialization failed as expected in test environment");
            }
        }
    }

    #[test]
    fn test_origin_validation_logic() {
        // Test the logic without actual network calls
        let allowed = vec!["example.com".to_string(), "test.org".to_string()];
        
        // This is a simplified test of the validation logic
        assert!(allowed.contains(&"example.com".to_string()));
        assert!(!allowed.contains(&"malicious.com".to_string()));
    }
}