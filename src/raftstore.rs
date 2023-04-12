
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


use std::sync::{Arc, RwLock};

use heed::types::{ByteSlice, OwnedType, Str};
use heed::{Database, Env, EnvOpenOptions};

use raft::{
    self,
    eraftpb::{self, ConfState, Entry, HardState, Snapshot},
    Error as RaftError, GetEntriesContext, RaftState, Ready, Storage, StorageError,
};

const DB_PATH: &str = "./data";
const DB_NAME: &str = "raft.mdb";
const DB_STORE_SIZE: usize = 1_073_741_824;

struct RaftDB {
    env: Env,
    // TODO - likely need to use heed Serde functionality, ByteSlice is a placeholder for now
    conf: Database<Str, ByteSlice>,         
    logs: Database<OwnedType<i64>, ByteSlice>
}

impl RaftDB {
    pub fn new() -> RaftDB {
        todo!()
    }

}

pub struct RaftDiskStorage {
    raftdb: Arc<RwLock<RaftDB>>
}


impl RaftDiskStorage {
    pub fn new() -> RaftDiskStorage {
        RaftDiskStorage { 
            raftdb: Arc::new(RwLock::new(RaftDB::new()))
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
        todo!()
    }
}

impl Storage for RaftDiskStorage {
    fn initial_state(&self) -> raft::Result<raft::RaftState> {
        todo!()
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
        todo!()
    }

    fn first_index(&self) -> raft::Result<u64> {
        todo!()
    }

    fn last_index(&self) -> raft::Result<u64> {
        todo!()
    }

    fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<raft::prelude::Snapshot> {
        todo!()
    }
}
