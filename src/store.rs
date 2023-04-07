// Storage backends
//
// Store - Trait backends have to implement
// MemStore - Memory backed Store implementation using std::collections::HashMap;
//
// For now Storage Trait is get/set only
//  - need to add delete(...), range(...) and possibly more, see:
//    - see https://github.com/talent-plan/tinykv/blob/course/proto/proto/kvrpcpb.proto#L12-L64
//  - may want some admin queries
//
// Will add file backends using:
//   - Flat File (if necessary)
//   - redb/sled
//

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::vec::Vec;

pub trait Storage {
    fn get(&self, key: String) -> Result<Vec<u8>>;
    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool>;
    fn delete(&mut self, key: String) -> Result<bool>;
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

    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool> {
        self.store.insert(key, buf);
        Ok(true)
    }

    fn delete(&mut self, key: String) -> Result<bool> {
        let res = self.store.remove(&key);
        match res {
            Some(_res) => return Ok(true),
            None => {
                let err = Error::new(ErrorKind::Other, "no key");
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memstore() {
        let mut ms = MemStore::new();

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
        // second delete should also throw an error
        let e = ms.delete(String::from("delete_me"));
        assert!(e.is_err());
    }
}
