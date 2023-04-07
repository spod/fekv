// Storage backends
//
// Store - Trait backends have to implement
// MemStore - Memory backed Store implementation using std::collections::HashMap;
//
// For now Storage Trait is get/set only
//  - may need to add delete(...), range(...) for raft?
//  - may want some admin queries
//
// Will add file backends using:
//   - Flat File
//   - redb/sled
//

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::vec::Vec;

pub trait Storage {
    fn get(&self, key: String) -> Result<Vec<u8>>;
    fn set(&mut self, key: String, buf: Vec<u8>);
}

#[derive(Debug)]
pub struct MemStore {
    store: HashMap<String, Vec<u8>>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore {
            store: HashMap::new(),
        }
    }
}

impl Storage for MemStore {
    fn get(&self, key: String) -> Result<Vec<u8>> {
        let err = Error::new(ErrorKind::Other, "missing key");
        self.store.get(&key).ok_or(err).cloned()
    }

    fn set(&mut self, key: String, buf: Vec<u8>) {
        self.store.insert(key, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memstore() {
        let mut ms = MemStore::new();

        ms.set(String::from("foo"), b"bar".to_vec());
        ms.set(String::from("bar"), b"baz".to_vec());
        assert_eq!(ms.get(String::from("foo")).unwrap(), b"bar");
        assert_eq!(ms.get(String::from("bar")).unwrap(), b"baz");

        let e = ms.get(String::from("missing"));
        assert!(e.is_err())
    }
}
