//
// Disk backed KVStorage implementation using heed
//

use std::fs::create_dir_all;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::vec::Vec;

use heed::types::{ByteSlice, Str};
use heed::{Database, Env, EnvOpenOptions};

use super::KVStorage;

const DB_PATH: &str = "./data";
const DB_NAME: &str = "fekv.mdb";
const DB_STORE_SIZE: usize = 1_073_741_824;

pub struct DiskKVStore {
    env: Env,
    db: Database<Str, ByteSlice>,
}

impl DiskKVStore {
    pub fn new() -> DiskKVStore {
        let db_path = Path::join(Path::new(&DB_PATH), &DB_NAME);
        _ = create_dir_all(&db_path);
        let env = EnvOpenOptions::new()
            .map_size(DB_STORE_SIZE) // 10MB
            .max_dbs(3)
            .open(db_path)
            .unwrap();
        let db = env.create_database(Some(&DB_NAME)).unwrap();
        DiskKVStore { env: env, db: db }
    }
}

impl KVStorage for DiskKVStore {
    fn get(&self, key: String) -> Result<Vec<u8>> {
        let rtxn = self.env.read_txn().unwrap();
        let r = self.db.get(&rtxn, &key);
        match r {
            Ok(ro) => match ro {
                Some(ro) => Ok(ro.to_owned()),
                None => {
                    let err = Error::new(ErrorKind::Other, "no key");
                    return Err(err);
                }
            },
            Err(err) => {
                let err = Error::new(ErrorKind::Other, err.to_string());
                return Err(err);
            }
        }
    }

    fn set(&mut self, key: String, buf: Vec<u8>) -> Result<bool> {
        let mut wtxn = self.env.write_txn().unwrap();
        let r = self.db.put(&mut wtxn, &key, &buf);
        if r.is_err() {
            let err = r.unwrap_err();
            return Err(Error::new(ErrorKind::Other, err.to_string()));
        }
        let r = wtxn.commit();
        match r {
            Ok(_r) => return Ok(true),
            Err(err) => {
                let err = Error::new(ErrorKind::Other, err.to_string());
                return Err(err);
            }
        }
    }

    fn delete(&mut self, key: String) -> Result<bool> {
        let mut wtxn = self.env.write_txn().unwrap();
        let r = self.db.delete(&mut wtxn, &key);
        if r.is_err() {
            let err = r.unwrap_err();
            return Err(Error::new(ErrorKind::Other, err.to_string()));
        }
        let deleted = r.unwrap();
        let r = wtxn.commit();
        if r.is_err() {
            let err = r.unwrap_err();
            return Err(Error::new(ErrorKind::Other, err.to_string()));
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_heed() {
        let env_path = Path::new("target").join("heed-tst.mdb");
        let _ = fs::remove_dir_all(&env_path);

        fs::create_dir_all(&env_path).unwrap();
        let env = EnvOpenOptions::new()
            .map_size(10 * 1024 * 1024) // 10MB
            .max_dbs(3)
            .open(env_path)
            .unwrap();

        let mut wtxn = env.write_txn().unwrap();

        let test: Database<Str, Str> = env
            .create_database_with_txn(Some("text"), &mut wtxn)
            .unwrap();

        let _r = test.put(&mut wtxn, "I am here", "to test things");
        let _r = wtxn.commit();
    }

    #[test]
    fn test_diskkvstore() {
        // TODO - extract this test out to one that is shared across all store types
        //
        // TODO - alternate constructor that lets us create a temp db and/or some way to pass
        // a config context through from main server or something which is ignored by MemStore
        //
        // WARNING - for now this test will touch the "prod" db on disk
        let mut ms = DiskKVStore::new();

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
