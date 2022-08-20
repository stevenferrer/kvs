use std::collections::HashMap;

#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: String, v: String) {
        self.map.insert(k, v);
    }

    pub fn get(&mut self, k: String) -> Option<String> {
        self.map.get(&k).map(|v| v.to_string())
    }

    pub fn remove(&mut self, k: String) {
        self.map.remove(&k);
    }
}
