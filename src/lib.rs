#![deny(missing_docs)]
//! a simple key-value store

mod error;
mod kv;

pub use error::{Error, Result};
pub use kv::DB;
