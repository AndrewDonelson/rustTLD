// file: src/etld.rs
// description: manages effective top-level domains (eTLDs) with production-ready error handling

use std::sync::RwLock;

/// ETLD manages all eTLDs in lists with thread-safety
///
/// This structure provides thread-safe access to a list of effective top-level domains
/// organized by the number of dots they contain. It supports concurrent read access
/// and synchronized write operations.
#[derive(Debug)]
pub struct Etld {
    /// List of eTLD strings
    list: RwLock<Vec<String>>,
    /// Number of dots in this eTLD level
    pub dots: usize,
}

impl Etld {
    /// Creates a new empty ETLD with the specified number of dots
    ///
    /// # Arguments
    ///
    /// * `dots` - The number of dots that eTLDs in this list should contain
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(1); // For TLDs like "co.uk", "com.au"
    /// assert_eq!(etld.dots, 1);
    /// ```
    pub const fn new(dots: usize) -> Self {
        Self {
            list: RwLock::new(Vec::new()),
            dots,
        }
    }

    /// Returns the current count of eTLDs in the list
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the lock. In practice, this should be extremely rare.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// assert_eq!(etld.count(), 0);
    /// ```
    pub fn count(&self) -> usize {
        self.list.read().unwrap().len()
    }

    /// Appends a new eTLD to the list if it doesn't already exist
    ///
    /// # Arguments
    ///
    /// * `s` - The eTLD string to add
    /// * `sort_list` - Whether to sort the list after adding (expensive operation)
    ///
    /// # Returns
    ///
    /// * `true` if the item was added (didn't exist before)
    /// * `false` if the item already existed and wasn't added
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the write lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// assert!(etld.add("com".to_string(), false));
    /// assert!(!etld.add("com".to_string(), false)); // Duplicate
    /// ```
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

    /// Sorts the list of strings in alphabetical order
    ///
    /// This is required for efficient binary search operations. Should be called
    /// after all additions are complete.
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the write lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("org".to_string(), false);
    /// etld.add("com".to_string(), false);
    /// etld.sort();
    /// // List is now sorted: ["com", "org"]
    /// ```
    pub fn sort(&self) {
        let mut list = self.list.write().unwrap();
        list.sort();
    }

    /// Searches for an eTLD in the list using binary search
    ///
    /// # Arguments
    ///
    /// * `search_str` - The eTLD string to search for
    ///
    /// # Returns
    ///
    /// A tuple of (found_string, exists_bool):
    /// * If found: (matching_etld, true)
    /// * If not found: (empty_string, false)
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the read lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("com".to_string(), false);
    /// etld.sort();
    ///
    /// let (found, exists) = etld.search("com");
    /// assert!(exists);
    /// assert_eq!(found, "com");
    /// ```
    ///
    /// # Performance
    ///
    /// This function uses binary search with O(log n) complexity, but requires
    /// the list to be sorted first using the `sort()` method.
    pub fn search(&self, search_str: &str) -> (String, bool) {
        let list = self.list.read().unwrap();

        if list.is_empty() {
            return (String::new(), false);
        }

        // Use map_or_else for more idiomatic Rust
        list.binary_search(&search_str.to_string())
            .map_or_else(|_| (String::new(), false), |idx| (list[idx].clone(), true))
    }

    /// Returns a clone of the internal list for read-only access
    ///
    /// # Returns
    ///
    /// A cloned vector containing all eTLD strings in the current order
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the read lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("com".to_string(), false);
    /// let list = etld.get_list();
    /// assert_eq!(list, vec!["com"]);
    /// ```
    ///
    /// # Performance Note
    ///
    /// This method clones the entire internal vector, which may be expensive
    /// for large lists. Use sparingly or consider alternatives for performance-critical code.
    pub fn get_list(&self) -> Vec<String> {
        self.list.read().unwrap().clone()
    }

    /// Checks if the list is empty
    ///
    /// # Returns
    ///
    /// * `true` if the list contains no eTLD entries
    /// * `false` if the list contains one or more eTLD entries
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the read lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// assert!(etld.is_empty());
    ///
    /// etld.add("com".to_string(), false);
    /// assert!(!etld.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.list.read().unwrap().is_empty()
    }

    /// Clears all eTLDs from the list
    ///
    /// This removes all entries and resets the list to an empty state.
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the write lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("com".to_string(), false);
    /// assert!(!etld.is_empty());
    ///
    /// etld.clear();
    /// assert!(etld.is_empty());
    /// ```
    pub fn clear(&self) {
        let mut list = self.list.write().unwrap();
        list.clear();
    }

    /// Returns an iterator over the eTLD entries (for advanced use cases)
    ///
    /// # Returns
    ///
    /// A vector iterator over cloned eTLD strings
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the read lock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("com".to_string(), false);
    /// etld.add("org".to_string(), false);
    ///
    /// for tld in etld.iter() {
    ///     println!("TLD: {}", tld);
    /// }
    /// ```
    pub fn iter(&self) -> std::vec::IntoIter<String> {
        self.get_list().into_iter()
    }

    /// Returns the capacity of the internal vector
    ///
    /// # Returns
    ///
    /// The current capacity of the internal vector
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the read lock.
    pub fn capacity(&self) -> usize {
        self.list.read().unwrap().capacity()
    }

    /// Reserves capacity for at least `additional` more elements
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional elements to reserve space for
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned due to a panic in another thread
    /// while holding the write lock, or if the new capacity overflows.
    pub fn reserve(&self, additional: usize) {
        let mut list = self.list.write().unwrap();
        list.reserve(additional);
    }
}

impl Clone for Etld {
    /// Creates a deep clone of the Etld instance
    ///
    /// # Panics
    ///
    /// Panics if the internal RwLock is poisoned due to a panic in another thread
    /// while holding the read lock.
    fn clone(&self) -> Self {
        let list = self.list.read().unwrap().clone();
        Self {
            list: RwLock::new(list),
            dots: self.dots,
        }
    }
}

impl Default for Etld {
    /// Creates a new Etld with 0 dots (for single-level TLDs like .com, .org)
    fn default() -> Self {
        Self::new(0)
    }
}

impl IntoIterator for &Etld {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    /// Allows for-in loops over `&Etld` references
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_tld::etld::Etld;
    ///
    /// let etld = Etld::new(0);
    /// etld.add("com".to_string(), false);
    /// etld.add("org".to_string(), false);
    ///
    /// for tld in &etld {
    ///     println!("TLD: {}", tld);
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    fn test_const_new() {
        // Test that new() is indeed const
        const ETLD: Etld = Etld::new(1);
        assert_eq!(ETLD.dots, 1);
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

        // Sort before searching (required for binary search)
        etld.sort();

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

    #[test]
    fn test_add_with_sort() {
        let etld = Etld::new(0);
        etld.add("org".to_string(), true); // Sort after each add
        etld.add("com".to_string(), true);
        etld.add("net".to_string(), true);

        let list = etld.get_list();
        assert_eq!(list, vec!["com", "net", "org"]);
    }

    #[test]
    fn test_clear() {
        let etld = Etld::new(0);
        etld.add("com".to_string(), false);
        etld.add("org".to_string(), false);
        assert_eq!(etld.count(), 2);

        etld.clear();
        assert_eq!(etld.count(), 0);
        assert!(etld.is_empty());
    }

    #[test]
    fn test_clone() {
        let etld = Etld::new(1);
        etld.add("com".to_string(), false);
        etld.add("org".to_string(), false);

        let cloned = etld.clone();
        assert_eq!(cloned.dots, etld.dots);
        assert_eq!(cloned.count(), etld.count());
        assert_eq!(cloned.get_list(), etld.get_list());
    }

    #[test]
    fn test_default() {
        let etld = Etld::default();
        assert_eq!(etld.dots, 0);
        assert!(etld.is_empty());
    }

    #[test]
    fn test_iterator() {
        let etld = Etld::new(0);
        etld.add("com".to_string(), false);
        etld.add("org".to_string(), false);
        etld.sort();

        let mut collected: Vec<String> = etld.iter().collect();
        collected.sort(); // Sort for consistent comparison
        assert_eq!(collected, vec!["com", "org"]);
    }

    #[test]
    fn test_capacity_and_reserve() {
        let etld = Etld::new(0);
        let initial_capacity = etld.capacity();

        etld.reserve(100);
        assert!(etld.capacity() >= initial_capacity + 100);
    }

    #[test]
    fn test_search_empty_list() {
        let etld = Etld::new(0);
        let (found, exists) = etld.search("com");
        assert!(!exists);
        assert_eq!(found, "");
    }

    #[test]
    fn test_binary_search_performance() {
        let etld = Etld::new(0);

        // Add many items
        for i in 0..1000 {
            etld.add(format!("domain{}.com", i), false);
        }
        etld.sort();

        // Search should be fast even with many items
        let (found, exists) = etld.search("domain500.com");
        assert!(exists);
        assert_eq!(found, "domain500.com");
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let etld = Arc::new(Etld::new(0));
        let mut handles = vec![];

        // Spawn multiple threads adding different items
        for i in 0..10 {
            let etld_clone = Arc::clone(&etld);
            let handle = thread::spawn(move || {
                etld_clone.add(format!("domain{}.com", i), false);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(etld.count(), 10);
    }

    #[test]
    fn test_thread_safety_read_write() {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let etld = Arc::new(Etld::new(0));
        etld.add("com".to_string(), false);
        etld.add("org".to_string(), false);
        etld.sort();

        let mut handles = vec![];

        // Spawn reader threads
        for _ in 0..5 {
            let etld_clone = Arc::clone(&etld);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let (_, exists) = etld_clone.search("com");
                    assert!(exists);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Spawn a writer thread
        let etld_clone = Arc::clone(&etld);
        let writer_handle = thread::spawn(move || {
            for i in 0..10 {
                etld_clone.add(format!("new{}.com", i), false);
                thread::sleep(Duration::from_millis(5));
            }
        });
        handles.push(writer_handle);

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        assert!(etld.count() >= 12); // At least original 2 + 10 new ones
    }
}
