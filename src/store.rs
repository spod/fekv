// TODO in memory backed Store implementation using std::collections::HashMap;
//
// Store should be a trait with methods:
//   get(key) -> string
//   set(key, value)
//
//   and later I guess delete(...) and range(...)
//

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};

pub trait Storage<'a> {
    fn get(&self, key: &str) -> Result<&[u8]>;
    fn set(&mut self, key: &'a str, buf: &'a [u8]);
}

#[derive(Debug)]
pub struct MemStore<'a> {
    store: HashMap<&'a str, &'a [u8]>,
}

impl<'a> MemStore<'a> {
    pub fn new() -> MemStore<'a> {
        MemStore {
            store: HashMap::new(),
        }
    }
}

impl<'a> Storage<'a> for MemStore<'a> {
    fn get(&self, key: &str) -> Result<&[u8]> {
        let err = Error::new(ErrorKind::Other, "missing key");
        self.store.get(key).ok_or(err).copied()
    }
    fn set(&mut self, key: &'a str, buf: &'a [u8]) {
        self.store.insert(key, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memstore() {
        let mut ms = MemStore::new();

        ms.set("foo", b"bar");
        ms.set("bar", b"baz");
        assert_eq!(ms.get("foo").unwrap(), b"bar");
        assert_eq!(ms.get("bar").unwrap(), b"baz");

        let e = ms.get("missing");
        assert!(e.is_err())
    }
}
