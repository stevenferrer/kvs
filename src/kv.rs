#[derive(Default)]
pub struct KvStore {}

impl KvStore {
  pub fn new() -> KvStore {
    KvStore {}
  }

  pub fn set(&mut self, _k: String, _v: String) {
    unimplemented!()
  }

  pub fn get(&mut self, _k: String) -> Option<String> {
    unimplemented!()
  }

  pub fn remove(&mut self, _k: String) {
    unimplemented!()
  }
}
