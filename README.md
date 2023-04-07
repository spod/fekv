# raftexample-rs

Key Value datastore with a REST API with raft based consensus in rust.

## Usage

Run a server: `cargo run`

Make some queries:
``` shell
$ curl --fail localhost:3000/kv/foo
curl: (22) The requested URL returned error: 404
$ curl localhost:3000/kv/foo -XPOST -d 'bar' 
OK
$ curl --fail localhost:3000/kv/foo
bar
```

## Warning
This is a toy project and it is not intended for real world use.

It is missing many things including: Authentication/Authorization, Configuration, Logging, Metrics, Security or Code reviews, testing etc.

## TODO
* [ ] improve error handling in kv(...) route handler
* [ ] add a stat/info endpoint to show some basic info on backing store, keys etc
* [ ] pick one of redb or sled as backing store and get it working
* [ ] add raft
* [ ] figure out max value sizes and handle errors appropriately
* [ ] add some checksumming to ensure we don't have errors / damage data
* [ ] add a logging backend and/or config of some kind so `ab -n 100000 ...` tests aren't blocked on console output

## Approach

Building something comparable [etcd/contrib/raftexample](https://github.com/etcd-io/etcd/tree/main/contrib/raftexample) to in rust using the [tikv/raft-rs](https://github.com/tikv/raft-rs) raft library, with either [redb](https://github.com/cberner/redb) or [sled](https://github.com/spacejam/sled) for storage and the [hyper](https://hyper.rs/) web framework.

Implementation will roughly follow [tinykv course outline](https://github.com/talent-plan/tinykv).

# references
* https://www.pingcap.com/blog/implement-raft-in-rust/
* https://github.com/etcd-io/etcd/blob/main/contrib/raftexample/README.md
* https://pdos.csail.mit.edu/6.824/labs/lab-kvraft.html
* https://github.com/talent-plan/tinykv
* https://hyper.rs/
* https://dev.to/daaitch/rust-ownership-and-borrows-fighting-the-borrow-checker-4ea3

