//
// Non replicated Key Value Store
//
// Contains two implementations:
//   kvstore::diskstore::DiskKVStore - backed by a lmdb db using the heed crate
//   kvstore::memstore::MemKVStore - backed by a std::vec::Vec
//

use std::io::Result;
use std::vec::Vec;

pub trait KVStorage {
    fn get(&self, key: String) -> Result<Vec<u8>>;
    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool>;
    fn delete(&mut self, key: String) -> Result<bool>;
}

pub mod diskstore;
pub mod memstore;

// pub enum KVStoreKind {
//     MEMORY,
//     DISK,
// }

// pub fn GetKVStore(kind: KVStoreKind) -> Box<dyn KVStorage> {
//     match kind {
//         KVStoreKind::MEMORY => Box::new(memstore::MemKVStore::new()),
//         KVStoreKind::DISK => Box::new(diskstore::DiskKVStore::new()),
//     }
// }
