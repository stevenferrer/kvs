use std::collections::HashMap;

#[derive(Default)]
/// An in-memory key-value store
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// Constructs a KvStore
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// Sets a given key with value
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Retrieves the value for a given key
    pub fn get(&mut self, k: String) -> Option<String> {
        self.map.get(&k).map(|v| v.to_string())
    }

    /// Removes the entry for a given key
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
