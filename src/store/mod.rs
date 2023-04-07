use std::io::Result;
use std::vec::Vec;

pub trait Storage {
    fn get(&self, key: String) -> Result<Vec<u8>>;
    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool>;
    fn delete(&mut self, key: String) -> Result<bool>;
}

pub mod diskstore;
pub mod memstore;
