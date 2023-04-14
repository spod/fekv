// https://www.pingcap.com/blog/implement-raft-in-rust/
// https://github.com/etcd-io/etcd/blob/main/contrib/raftexample/raft.go
// https://github.com/etcd-io/etcd/blob/main/server/storage/wal/doc.go
// https://github.com/hashicorp/raft-mdb
//
//
// Raft Store needs to track:
//   - Initial RaftState (HardState, ConfState)
//   - entries in the raft log
//   - snapshot which is used to send to other nodes
//
// How to store this in LMDB?
//  - One DB for raft log entries - key is term/index, value is log entry
//  - One DB for Config - key is config item, value is state for config
//  See hashicorp/raft-mdb for this in go

//
// Lots TODO below so for now silence warnings:
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fs::create_dir_all;
use std::path::Path;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use heed::types::{ByteSlice, OwnedType, SerdeBincode, SerdeJson, Str};
use heed::{Database, Env, EnvOpenOptions};

use raft::prelude::*;
use raft::{Error, StorageError};
use serde::{Deserialize, Serialize};

const DB_ENTRIES: &str = "entries";

const DB_ENV: &str = "raft.mdb";
const DB_PATH: &str = "./data";
const DB_STORE_SIZE: usize = 1_073_741_824;

// Versions of raft::Entry/EntryType which implement Serialize & Deserialize
#[derive(Clone, Debug, Serialize, Deserialize)]
struct EntryRef {
    pub entry_type: EntryTypeRef,
    pub term: u64,
    pub index: u64,
    pub data: ::bytes::Bytes,
    pub context: ::bytes::Bytes,
    pub sync_log: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum EntryTypeRef {
    EntryNormal = 0,
    EntryConfChange = 1,
    EntryConfChangeV2 = 2,
}

impl EntryTypeRef {
    pub fn to_entry_type(&self) -> EntryType {
        match self {
            crate::raftstore::EntryTypeRef::EntryNormal => raft::eraftpb::EntryType::EntryNormal,
            crate::raftstore::EntryTypeRef::EntryConfChange => {
                raft::eraftpb::EntryType::EntryConfChange
            }
            crate::raftstore::EntryTypeRef::EntryConfChangeV2 => {
                raft::eraftpb::EntryType::EntryConfChangeV2
            }
        }
    }

    pub fn from_entry_type(&self, et: EntryType) -> EntryTypeRef {
        match et {
            raft::eraftpb::EntryType::EntryNormal => crate::raftstore::EntryTypeRef::EntryNormal,
            raft::eraftpb::EntryType::EntryConfChange => {
                crate::raftstore::EntryTypeRef::EntryConfChange
            }
            raft::eraftpb::EntryType::EntryConfChangeV2 => {
                crate::raftstore::EntryTypeRef::EntryConfChangeV2
            }
        }
    }
}

impl EntryRef {
    pub fn to_entry(&self) -> Entry {
        let mut ent = Entry::new();
        ent.entry_type = self.entry_type.to_entry_type();
        ent.term = self.term;
        ent.index = self.index;
        ent.data = self.data.to_owned();
        ent.context = self.context.to_owned();
        ent.sync_log = self.sync_log;
        ent
    }

    pub fn from_entry(&self, e: Entry) -> EntryRef {
        let tr: EntryTypeRef = EntryTypeRef::EntryNormal;
        EntryRef {
            entry_type: tr.from_entry_type(e.entry_type),
            term: e.term,
            index: e.index,
            data: e.data.to_owned(),
            context: e.context.to_owned(),
            sync_log: e.sync_log,
        }
    }

    pub fn new() -> EntryRef {
        EntryRef {
            entry_type: EntryTypeRef::EntryNormal,
            term: 0,
            index: 0,
            data: ::bytes::Bytes::new(),
            context: ::bytes::Bytes::new(),
            sync_log: false,
        }
    }
}

pub struct RaftDB {
    env: Env,
    entries: Database<OwnedType<u64>, SerdeJson<EntryRef>>, // SerdeBincode
    raft_state: RaftState,
    snapshot_metadata: SnapshotMetadata,
}

impl RaftDB {
    pub fn new() -> RaftDB {
        let db_path = Path::join(Path::new(&DB_PATH), &DB_ENV);
        _ = create_dir_all(&db_path);
        let env = EnvOpenOptions::new()
            .map_size(DB_STORE_SIZE) // 10MB
            .max_dbs(3)
            .open(db_path)
            .unwrap();
        let entries = env.create_database(Some(&DB_ENTRIES)).unwrap();
        let raft_state = RaftState::new(HardState::new(), ConfState::new());
        // TODO write initial raft_state to conf database, for now we use raft::storage::RaftState;
        RaftDB {
            env: env,
            entries: entries,
            raft_state: raft_state,
            snapshot_metadata: SnapshotMetadata::new(),
        }
    }

    pub fn new_with_db_path(db_path: &std::path::Path) -> RaftDB {
        let db_path = Path::join(db_path, &DB_ENV);
        _ = create_dir_all(&db_path);
        let env = EnvOpenOptions::new()
            .map_size(DB_STORE_SIZE) // 10MB
            .max_dbs(3)
            .open(db_path)
            .unwrap();
        let entries = env.create_database(Some(&DB_ENTRIES)).unwrap();
        let raft_state = RaftState::new(HardState::new(), ConfState::new());
        // TODO write initial raft_state to conf database, for now we use raft::storage::RaftState;
        RaftDB {
            env: env,
            entries: entries,
            raft_state: raft_state,
            snapshot_metadata: SnapshotMetadata::new(),
        }
    }

    fn first_index(&self) -> u64 {
        let rtxn = self.env.read_txn().unwrap();
        let r = self.entries.first(&rtxn);
        if r.is_err() {
            return self.snapshot_metadata.index + 1;
        }
        match r.unwrap() {
            Some(e) => {
                assert_eq!(e.0, e.1.index);
                e.0
            }
            None => self.snapshot_metadata.index + 1,
        }
    }

    fn last_index(&self) -> u64 {
        let rtxn = self.env.read_txn().unwrap();
        let r = self.entries.last(&rtxn);
        if r.is_err() {
            return self.snapshot_metadata.index + 1;
        }
        match r.unwrap() {
            Some(e) => {
                assert_eq!(e.0, e.1.index);
                e.0
            }
            None => self.snapshot_metadata.index + 1,
        }
    }

    fn get_entry(&self, idx: u64) -> Result<Entry, heed::Error> {
        let rtxn = self.env.read_txn().unwrap();
        let res = self.entries.get(&rtxn, &idx);
        match res {
            Ok(e) => match e {
                Some(e) => return Ok(e.to_entry()),
                None => return Err(heed::Error::DatabaseClosing),
            },
            Err(err) => return Err(err),
        }
    }

    fn set_entry(&self, idx: u64, e: Entry) {
        let er = EntryRef::new().from_entry(e);
        let mut wtxn = self.env.write_txn().unwrap();
        let r = self.entries.put(&mut wtxn, &idx, &er);
        _ = wtxn.commit();
        // TODO error handling and appropriate returns
    }

    pub fn append(&mut self, ents: &[Entry]) -> Result<(), heed::Error> {
        if ents.is_empty() {
            return Ok(());
        }
        if self.first_index() > ents[0].index {
            panic!(
                "overwrite compacted raft logs, compacted: {}, append: {}",
                self.first_index() - 1,
                ents[0].index,
            );
        }
        if self.last_index() + 1 < ents[0].index {
            panic!(
                "raft logs should be continuous, last index: {}, new appended: {}",
                self.last_index(),
                ents[0].index,
            );
        }

        // Append all entries from `ents`.
        let mut wtxn = self.env.write_txn().unwrap();
        for (_, e) in ents.into_iter().enumerate() {
            let er = EntryRef::new().from_entry(e.clone());
            let r = self.entries.put(&mut wtxn, &e.index, &er);
        }
        _ = wtxn.commit();
        Ok(())
    }

    pub fn compact(&mut self, compact_index: u64) -> Result<(), heed::Error> {
        // remove any log entries up to compact_index
        if compact_index <= self.first_index() {
            // Don't need to treat this case as an error.
            return Ok(());
        }

        if compact_index > self.last_index() + 1 {
            panic!(
                "compact not received raft logs: {}, last index: {}",
                compact_index,
                self.last_index()
            );
        }

        if let Ok(entry) = self.get_entry(self.first_index()) {
            let mut wtxn = self.env.write_txn().unwrap();
            for k in self.first_index()..compact_index {
                let _r = self.entries.delete(&mut wtxn, &k);
            }
            _ = wtxn.commit();
        }
        // TODO error handling and appropriate returns
        Ok(())
    }

    // this should only be used in test setup etc
    fn clear(&self) {
        let mut wtxn = self.env.write_txn().unwrap();
        let r = self.entries.clear(&mut wtxn);
        _ = wtxn.commit();
        // TODO error handling and appropriate returns
    }
}

pub struct RaftDiskStorage {
    raftdb: Arc<RwLock<RaftDB>>, // Do we need to use tokio::sync::RwLock?
}

impl RaftDiskStorage {
    pub fn new() -> RaftDiskStorage {
        RaftDiskStorage {
            raftdb: Arc::new(RwLock::new(RaftDB::new())),
        }
    }

    pub fn new_with_db_path(db_path: &std::path::Path) -> RaftDiskStorage {
        RaftDiskStorage {
            raftdb: Arc::new(RwLock::new(RaftDB::new_with_db_path(db_path))),
        }
    }

    pub fn new_with_conf_state<T>(conf_state: T) -> RaftDiskStorage
    where
        ConfState: From<T>,
    {
        let store = RaftDiskStorage::new();
        store.initialize_with_conf_state(conf_state);
        store
    }

    pub fn initialize_with_conf_state<T>(&self, conf_state: T)
    where
        ConfState: From<T>,
    {
        assert!(!self.initial_state().unwrap().initialized());
        let mut core = self.wl();
        core.raft_state.conf_state = ConfState::from(conf_state);
    }

    pub fn rl(&self) -> RwLockReadGuard<'_, RaftDB> {
        self.raftdb.read().unwrap()
    }

    pub fn wl(&self) -> RwLockWriteGuard<'_, RaftDB> {
        self.raftdb.write().unwrap()
    }
}

// From https://github.com/tikv/raft-rs/blob/master/src/util.rs with modifications
pub fn limit_size(entries: &mut Vec<Entry>, max: Option<u64>) {
    if entries.len() <= 1 {
        return;
    }
    let max = match max {
        None | Some(raft::NO_LIMIT) => return,
        Some(max) => max,
    };

    let mut size = 0;
    let limit = entries
        .iter()
        .take_while(|&e| {
            if size == 0 {
                size += u64::from(compute_size(e));
                return true;
            }
            size += u64::from(compute_size(e));
            size <= max
        })
        .count();
    entries.truncate(limit);
}

fn compute_size(ent: &Entry) -> u32 {
    // hack
    ent.data.len() as u32 + ent.context.len() as u32 + 4 as u32
}

impl Storage for RaftDiskStorage {
    fn initial_state(&self) -> raft::Result<raft::RaftState> {
        Ok(self.rl().raft_state.clone())
    }

    fn entries(
        &self,
        low: u64,
        high: u64,
        max_size: impl Into<Option<u64>>,
        context: raft::GetEntriesContext,
    ) -> raft::Result<Vec<raft::prelude::Entry>> {
        let max_size = max_size.into();
        let core = self.rl();
        if low < core.first_index() {
            return Err(Error::Store(StorageError::Compacted));
        }

        if high > core.last_index() + 1 {
            panic!(
                "index out of bound (last: {}, high: {})",
                core.last_index() + 1,
                high
            );
        }

        let mut ents: Vec<Entry> = std::vec::Vec::new();

        for k in low..high {
            let e = core.get_entry(k as u64);
            ents.push(core.get_entry(k as u64).unwrap());
        }
        limit_size(&mut ents, max_size);
        Ok(ents)
    }

    fn term(&self, idx: u64) -> raft::Result<u64> {
        let core = self.rl();
        if idx == core.snapshot_metadata.index {
            return Ok(core.snapshot_metadata.term);
        }

        let offset = core.first_index();
        if idx < offset {
            return Err(raft::Error::Store(StorageError::Compacted));
        }
        if idx > core.last_index() {
            return Err(Error::Store(StorageError::Unavailable));
        }
        // note we store using idx as key in backing store, rather than
        // using a vec! in memory - so no need to use (idx - offset)
        let res = core.get_entry(idx);
        match res {
            Ok(e) => Ok(e.term),
            Err(err) => Err(Error::Store(StorageError::Unavailable)),
        }
    }

    fn first_index(&self) -> raft::Result<u64> {
        Ok(self.rl().first_index())
    }

    fn last_index(&self) -> raft::Result<u64> {
        Ok(self.rl().last_index())
    }

    fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<raft::prelude::Snapshot> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    // Running all tests may fail due to shared backing DB across threads
    //
    // TODO RaftDiskStorage needs configurable env and tests should have
    // a per test tmp backing env
    //
    // "Fix" intermittent test failures with:
    // $ cargo test -- --test-threads=1

    // where noted tests are based on tests from:
    // https://github.com/tikv/raft-rs/blob/master/src/storage.rs

    use std::panic::{self, AssertUnwindSafe};

    use super::{RaftDiskStorage, Storage};
    use raft::eraftpb::{ConfState, Entry, Snapshot};
    use raft::GetEntriesContext;
    use tempfile::tempdir;

    fn temp_store_with_entries(ents: &Vec<Entry>) -> RaftDiskStorage {
        let tmp = tempdir().unwrap();
        let storage = RaftDiskStorage::new_with_db_path(tmp.as_ref());
        storage.wl().clear();
        for (_, e) in ents.clone().drain(..).enumerate() {
            let core = storage.wl();
            core.set_entry(e.index as u64, e);
        }
        storage
    }

    fn new_entry(index: u64, term: u64) -> Entry {
        let mut e = Entry::default();
        e.term = term;
        e.index = index;
        e
    }

    #[test]
    fn test_storage_term() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
        let storage = temp_store_with_entries(&ents);

        let mut tests = vec![
            (2, Err("err")),
            (3, Ok(3)),
            (4, Ok(4)),
            (5, Ok(5)),
            (6, Err("err")),
        ];

        for (i, (idx, wterm)) in tests.drain(..).enumerate() {
            let t = storage.term(idx);
            // raft errors are crate private so just check if we got any err when we expect an error
            if wterm.is_err() && !t.is_err() {
                panic!("#{}: expect res {:?}, got {:?}", i, wterm, t);
            }
            if wterm.is_ok() {
                let tmpt = t.as_ref().ok();
                let tmpw = wterm.as_ref().ok();
                if tmpt != tmpw {
                    panic!("#{}: expect res {:?}, got {:?}", i, wterm, t);
                }
            }
        }
    }

    #[test]
    fn test_storage_entries() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![
            new_entry(3, 3),
            new_entry(4, 4),
            new_entry(5, 5),
            new_entry(6, 6),
        ];
        let storage = temp_store_with_entries(&ents);
        let max_u64 = u64::max_value();
        let mut tests = vec![
            (2, 6, max_u64, Err("err")),
            (3, 4, max_u64, Ok(vec![new_entry(3, 3)])),
            (4, 5, max_u64, Ok(vec![new_entry(4, 4)])),
            (4, 6, max_u64, Ok(vec![new_entry(4, 4), new_entry(5, 5)])),
            (
                4,
                7,
                max_u64,
                Ok(vec![new_entry(4, 4), new_entry(5, 5), new_entry(6, 6)]),
            ),
            // even if maxsize is zero, the first entry should be returned
            (4, 7, 0, Ok(vec![new_entry(4, 4)])),
            // limit to 2
            (
                4,
                7,
                // u64::from(size_of(&ents[1]) + size_of(&ents[2])),
                8 as u64,
                Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
            ),
            (
                4,
                7,
                // u64::from(size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3]) / 2),
                10 as u64,
                Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
            ),
            (
                4,
                7,
                // u64::from(size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3]) - 1),
                11 as u64,
                Ok(vec![new_entry(4, 4), new_entry(5, 5)]),
            ),
            // all
            (
                4,
                7,
                // u64::from(size_of(&ents[1]) + size_of(&ents[2]) + size_of(&ents[3])),
                12 as u64,
                Ok(vec![new_entry(4, 4), new_entry(5, 5), new_entry(6, 6)]),
            ),
        ];
        for (i, (lo, hi, maxsize, wentries)) in tests.drain(..).enumerate() {
            let e = storage.entries(lo, hi, maxsize, GetEntriesContext::empty(false));
            if e.is_err() && !wentries.is_err() {
                panic!("#{}: expect entries {:?}, got {:?}", i, wentries, e);
            }
            if wentries.is_ok() {
                let tmpe = e.as_ref().ok();
                let tmpw = wentries.as_ref().ok();
                if tmpe != tmpw {
                    panic!("#{}: expect entries {:?}, got {:?}", i, wentries, e);
                }
            }
        }
    }

    #[test]
    fn test_storage_last_index() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
        let storage = temp_store_with_entries(&ents);
        let wresult = Ok(5);
        let result = storage.last_index();
        if result != wresult {
            panic!("FAIL: want {:?}, got {:?}", wresult, result);
        }
        storage.wl().append(&[new_entry(6, 5)]).unwrap();
        let wresult = Ok(6);
        let result = storage.last_index();
        if result != wresult {
            panic!("want {:?}, got {:?}", wresult, result);
        }
    }

    #[test]
    fn test_storage_first_index() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
        let storage = temp_store_with_entries(&ents);
        assert_eq!(storage.first_index(), Ok(3));
        storage.wl().compact(4).unwrap();
        assert_eq!(storage.first_index(), Ok(4));
        storage.wl().compact(5).unwrap();
        assert_eq!(storage.first_index(), Ok(5));
    }

    #[test]
    fn test_storage_compact() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
        let storage = temp_store_with_entries(&ents);

        let mut tests = vec![(2, 3, 3, 3), (3, 3, 3, 3), (4, 4, 4, 2), (5, 5, 5, 1)];
        for (i, (idx, windex, wterm, wlen)) in tests.drain(..).enumerate() {
            storage.wl().compact(idx).unwrap();
            let index = storage.first_index().unwrap();
            if index != windex {
                panic!("#{}: want {}, index {}", i, windex, index);
            }
            let term = if let Ok(v) =
                storage.entries(index, index + 1, 1, GetEntriesContext::empty(false))
            {
                v.first().map_or(0, |e| e.term)
            } else {
                0
            };
            if term != wterm {
                panic!("#{}: want {}, term {}", i, wterm, term);
            }
            let last = storage.last_index().unwrap();
            let len = storage
                .entries(index, last + 1, 100, GetEntriesContext::empty(false))
                .unwrap()
                .len();
            if len != wlen {
                panic!("#{}: want {}, term {}", i, wlen, len);
            }
        }
    }


    #[test]
    fn test_storage_append() {
        // note this is a test from tikv/raft-rs/storage.rs with some modifications
        let ents = vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)];
        let storage = temp_store_with_entries(&ents);
        let mut tests = vec![
            (
                vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)],
                Some(vec![new_entry(3, 3), new_entry(4, 4), new_entry(5, 5)]),
            ),
            (
                vec![new_entry(3, 3), new_entry(4, 6), new_entry(5, 6)],
                Some(vec![new_entry(3, 3), new_entry(4, 6), new_entry(5, 6)]),
            ),
            (
                vec![
                    new_entry(3, 3),
                    new_entry(4, 4),
                    new_entry(5, 5),
                    new_entry(6, 5),
                ],
                Some(vec![
                    new_entry(3, 3),
                    new_entry(4, 4),
                    new_entry(5, 5),
                    new_entry(6, 5),
                ]),
            ),
            // overwrite compacted raft logs is not allowed
            (
                vec![new_entry(2, 3), new_entry(3, 3), new_entry(4, 5)],
                None,
            ),
            // truncate the existing entries and append
            (
                vec![new_entry(4, 5)],
                Some(vec![new_entry(3, 3), new_entry(4, 5)]),
            ),
            // direct append
            (
                vec![new_entry(6, 6)],
                Some(vec![
                    new_entry(3, 3),
                    new_entry(4, 4),
                    new_entry(5, 5),
                    new_entry(6, 6),
                ]),
            ),
        ];
        for (i, (entries, wentries)) in tests.drain(..).enumerate() {
            let storage = temp_store_with_entries(&ents);

            let res = panic::catch_unwind(AssertUnwindSafe(|| storage.wl().append(&entries)));
            if let Some(wentries) = wentries {
                let _ = res.unwrap();
                let e = &storage.entries(3, 3 + wentries.len() as u64, 100000, GetEntriesContext::empty(false)).unwrap();
                if *e != wentries {
                    panic!("#{}: want {:?}, entries {:?}", i, wentries, e);
                }
            } else {
                res.unwrap_err();
            }
        }
    }
}
