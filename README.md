# raftexample-rs

Key Value datastore with a REST API with raft based consensus in rust.

## Approach

Building something comparable [etcd/contrib/raftexample](https://github.com/etcd-io/etcd/tree/main/contrib/raftexample) to in rust using the [tikv/raft-rs](https://github.com/tikv/raft-rs) raft library, with either [redb](https://github.com/cberner/redb) or [sled](https://github.com/spacejam/sled) for storage and the [hyper](https://hyper.rs/) web framework.

Implementation will roughly follow [tinykv course outline](https://github.com/talent-plan/tinykv).

# references
* https://www.pingcap.com/blog/implement-raft-in-rust/
* https://github.com/etcd-io/etcd/blob/main/contrib/raftexample/README.md
* https://pdos.csail.mit.edu/6.824/labs/lab-kvraft.html
* https://github.com/talent-plan/tinykv
* https://hyper.rs/
