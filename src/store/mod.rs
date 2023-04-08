use std::io::Result;
use std::vec::Vec;

pub trait Storage {
    fn get(&self, key: String) -> Result<Vec<u8>>;
    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool>;
    fn delete(&mut self, key: String) -> Result<bool>;
}

pub mod diskstore;
pub mod memstore;

// pub enum StoreKind {
//     MEMORY,
//     DISK,
// }

// pub fn GetStore(kind: StoreKind) -> Box<dyn Storage> {
//     match kind {
//         StoreKind::MEMORY => Box::new(memstore::MemStore::new()),
//         StoreKind::DISK => Box::new(diskstore::DiskStore::new()),
//     }
// }
