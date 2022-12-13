use std::path;
use std::result;

#[derive(Default)]
/// An in-memory key-value store
pub struct KvStore {}

#[derive(Debug)]
pub struct Error;

/// KVS result
pub type Result<T> = result::Result<T, Error>;

impl KvStore {
    /// Open kv-store file
    pub fn open(_: impl Into<path::PathBuf>) -> Result<KvStore> {
        panic!("unimplemented")
    }

    /// Sets a given key with value
    pub fn set(&mut self, _: String, _: String) -> Result<()> {
        panic!("unimplemented")
    }

    /// Retrieves the value for a given key
    pub fn get(&mut self, _: String) -> Result<Option<String>> {
        panic!("unimplemented")
    }

    /// Removes the entry for a given key
    pub fn remove(&mut self, _: String) -> Result<()> {
        panic!("unimplemented")
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        todo!()
    }
}
