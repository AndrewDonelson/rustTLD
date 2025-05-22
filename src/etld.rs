// file: src/etld.rs
// description: manages effective top-level domains (eTLDs)

use std::sync::RwLock;

/// ETLD manages all eTLDs in lists with thread-safety
#[derive(Debug)]
pub struct Etld {
    /// List of eTLD strings
    list: RwLock<Vec<String>>,
    /// Number of dots in this eTLD level
    pub dots: usize,
}

impl Etld {
    /// Creates a new empty ETLD with the specified number of dots
    pub fn new(dots: usize) -> Self {
        Self {
            list: RwLock::new(Vec::new()),
            dots,
        }
    }

    /// Returns the current count of eTLDs in the list
    pub fn count(&self) -> usize {
        self.list.read().unwrap().len()
    }

    /// Appends a new eTLD to the list if it doesn't already exist
    /// Returns true if the item was added, false if it already existed
    pub fn add(&self, s: String, sort_list: bool) -> bool {
        let mut list = self.list.write().unwrap();
        
        // Check for duplicates
        if list.contains(&s) {
            return false;
        }

        let old_count = list.len();
        list.push(s);

        if sort_list {
            list.sort();
        }

        list.len() > old_count
    }

    /// Sorts the list of strings
    pub fn sort(&self) {
        let mut list = self.list.write().unwrap();
        list.sort();
    }

    /// Searches for an eTLD in the list
    /// Returns the found eTLD and true if found, empty string and false if not found
    pub fn search(&self, str: &str) -> (String, bool) {
        let list = self.list.read().unwrap();
        
        if list.is_empty() {
            return (String::new(), false);
        }

        match list.binary_search(&str.to_string()) {
            Ok(idx) => (list[idx].clone(), true),
            Err(_) => (String::new(), false),
        }
    }

    /// Returns a clone of the internal list (for read-only access)
    pub fn get_list(&self) -> Vec<String> {
        self.list.read().unwrap().clone()
    }

    /// Checks if the list is empty
    pub fn is_empty(&self) -> bool {
        self.list.read().unwrap().is_empty()
    }

    /// Clears all eTLDs from the list
    pub fn clear(&self) {
        let mut list = self.list.write().unwrap();
        list.clear();
    }
}

impl Clone for Etld {
    fn clone(&self) -> Self {
        let list = self.list.read().unwrap().clone();
        Self {
            list: RwLock::new(list),
            dots: self.dots,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_etld() {
        let etld = Etld::new(2);
        assert_eq!(etld.dots, 2);
        assert_eq!(etld.count(), 0);
        assert!(etld.is_empty());
    }

    #[test]
    fn test_add_and_search() {
        let etld = Etld::new(1);
        
        // Add new item
        assert!(etld.add("com".to_string(), false));
        assert_eq!(etld.count(), 1);
        
        // Try to add duplicate
        assert!(!etld.add("com".to_string(), false));
        assert_eq!(etld.count(), 1);
        
        // Search for existing item
        let (found, exists) = etld.search("com");
        assert!(exists);
        assert_eq!(found, "com");
        
        // Search for non-existing item
        let (not_found, not_exists) = etld.search("org");
        assert!(!not_exists);
        assert_eq!(not_found, "");
    }

    #[test]
    fn test_sort() {
        let etld = Etld::new(1);
        etld.add("org".to_string(), false);
        etld.add("com".to_string(), false);
        etld.add("net".to_string(), false);
        
        etld.sort();
        let list = etld.get_list();
        assert_eq!(list, vec!["com", "net", "org"]);
    }
}