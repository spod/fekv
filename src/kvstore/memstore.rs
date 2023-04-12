use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::vec::Vec;

use super::KVStorage;

#[derive(Debug)]
pub struct MemKVStore {
    store: HashMap<String, Vec<u8>>,
}

impl MemKVStore {
    #[allow(dead_code)]
    pub fn new() -> MemKVStore {
        MemKVStore {
            store: HashMap::new(),
        }
    }
}

impl KVStorage for MemKVStore {
    fn get(&self, key: String) -> Result<Vec<u8>> {
        let err = Error::new(ErrorKind::Other, "missing key");
        self.store.get(&key).ok_or(err).cloned()
    }

    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool> {
        self.store.insert(key, buf);
        Ok(true)
    }

    fn delete(&mut self, key: String) -> Result<bool> {
        let res = self.store.remove(&key);
        match res {
            Some(_res) => return Ok(true),
            None => return Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memkvstore() {
        let mut ms = MemKVStore::new();

        // set & get
        ms.set(String::from("foo"), b"bar".to_vec()).unwrap();
        ms.set(String::from("bar"), b"baz".to_vec()).unwrap();
        assert_eq!(ms.get(String::from("foo")).unwrap(), b"bar");
        assert_eq!(ms.get(String::from("bar")).unwrap(), b"baz");

        // get non existant key
        let e = ms.get(String::from("missing"));
        assert!(e.is_err());

        // delete
        ms.set(String::from("delete_me"), b"junk".to_vec()).unwrap();
        assert_eq!(ms.get(String::from("delete_me")).unwrap(), b"junk");
        // can delete once
        let res = ms.delete(String::from("delete_me"));
        assert_eq!(res.unwrap(), true);
        // second get should throw an error
        let e = ms.get(String::from("delete_me"));
        assert!(e.is_err());
        // second delete should return false as key removed
        let res = ms.delete(String::from("delete_me"));
        assert_eq!(res.unwrap(), false);
    }
}
