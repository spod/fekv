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

use heed::types::{ByteSlice, OwnedType, SerdeBincode, Str};
use heed::{Database, Env, EnvOpenOptions};
use raft::prelude::*;
use raft::{Error, StorageError};
use serde::{Deserialize, Serialize};

const DB_ENTRIES: &str = "entries";

const DB_ENV: &str = "raft.mdb";
const DB_PATH: &str = "./data";
const DB_STORE_SIZE: usize = 1_073_741_824;

// Serde compatible clones of raft::Entry/EntryType
// TODO need to write some helpers to map to/from Entry as can't figure
// out how to make Serde remote derive work with both raft::Entry special
// protobuf fields (unknown_fields, cached_size)
// and heed::Database<...> declaration - where does #[serde(with = "EntryRef")] go?

#[derive(Clone, Serialize, Deserialize)]
struct EntryRef {
    pub entry_type: EntryTypeRef,
    pub term: u64,
    pub index: u64,
    pub data: ::bytes::Bytes,
    pub context: ::bytes::Bytes,
    pub sync_log: bool,
}

#[derive(Clone, Serialize, Deserialize)]
enum EntryTypeRef {
    EntryNormal = 0,
    EntryConfChange = 1,
    EntryConfChangeV2 = 2,
}

impl EntryTypeRef {
    pub fn to_entry_type(&self) -> EntryType {
        match self {
            crate::raftstore::EntryTypeRef::EntryNormal => raft::eraftpb::EntryType::EntryNormal,
            crate::raftstore::EntryTypeRef::EntryConfChange => raft::eraftpb::EntryType::EntryConfChange,
            crate::raftstore::EntryTypeRef::EntryConfChangeV2 => raft::eraftpb::EntryType::EntryConfChangeV2,
        }
    }
}

impl EntryRef {
    pub fn to_entry(&self) -> Entry {
        Entry {
            entry_type: self.entry_type.to_entry_type(),
            term: self.term,
            index: self.index,
            data: self.data.to_owned(),
            context: self.context.to_owned(),
            sync_log: self.sync_log,
            unknown_fields: todo!(),
            cached_size: todo!(),
        }
    }
}

pub struct RaftDB {
    env: Env,
    entries: Database<OwnedType<u64>, SerdeBincode<EntryRef>>,
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

    fn first_index(&self) -> u64 {
        let rtxn = self.env.read_txn().unwrap();
        let r = self.entries.first(&rtxn);
        if r.is_err() {
            return self.snapshot_metadata.index + 1;
        }
        match r.unwrap() {
            Some(e) => e.0,
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
            Some(e) => e.0,
            None => self.snapshot_metadata.index + 1,
        }
    }
    fn get_entry(&self, idx: u64) -> Entry {
        let rtxn = self.env.read_txn().unwrap();
        let res = self.entries.get(&rtxn, &idx);
        let er = res.unwrap().unwrap();
        er.to_entry()
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
        todo!()
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
        Ok(core.get_entry(idx - offset).term)
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
