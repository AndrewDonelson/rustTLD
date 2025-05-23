// file: src/constants.rs
// description: defines constants for the package

/// Maximum number of groups in a domain
pub const ETLD_GROUP_MAX: usize = 5;

/// URL to download the public suffix list from
pub const PUBLIC_SUFFIX_FILE_URL: &str = "https://publicsuffix.org/list/public_suffix_list.dat";

/// Minimum size of the public suffix list file in bytes
pub const MIN_DATA_SIZE: usize = 32768;
