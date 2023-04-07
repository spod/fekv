# raftexample-rs

Key Value datastore with a REST API with raft based consensus in rust.

## Usage

Run a server: `cargo run`

Make some queries:
``` shell
$ curl --fail -X "GET" localhost:3000/kv/foo
curl: (22) The requested URL returned error: 404
$ curl --fail -X "POST" localhost:3000/kv/foo -d "bar" ; echo
OK
$ curl --fail -X "GET" localhost:3000/kv/foo ; echo
bar
$ curl --fail -X "POST" localhost:3000/kv/foo -d "baz" ; echo
OK
$ curl --fail -X "GET" localhost:3000/kv/foo ; echo
baz
$ curl --fail -X "DELETE" localhost:3000/kv/foo ; echo
OK
$ curl --fail -X "GET" localhost:3000/kv/foo
curl: (22) The requested URL returned error: 404
```

## Warning
This is a toy project and it is not intended for real world use.

It is missing many things including: Authentication/Authorization, Configuration, Logging, Metrics, Security or Code reviews, testing etc.

## TODO
* [ ] add a stat/info endpoint to show some basic info on backing store, keys etc
* [ ] add raft
* [ ] figure out max value sizes and handle errors appropriately
* [ ] add some checksumming to ensure we don't have errors / damage data
* [ ] add a logging backend and/or config of some kind so `ab -n 100000 ...` tests aren't blocked on console output

## Approach

Building something comparable [etcd/contrib/raftexample](https://github.com/etcd-io/etcd/tree/main/contrib/raftexample) to in rust using the [tikv/raft-rs](https://github.com/tikv/raft-rs) raft library, using the [heed library](https://github.com/meilisearch/heed) for [lmdb](http://www.lmdb.tech/doc/index.html) as storage and the [hyper](https://hyper.rs/) web framework.

Implementation will roughly follow [tinykv course outline](https://github.com/talent-plan/tinykv).

# references
* https://www.pingcap.com/blog/implement-raft-in-rust/
* https://github.com/etcd-io/etcd/blob/main/contrib/raftexample/README.md
* https://pdos.csail.mit.edu/6.824/labs/lab-kvraft.html
* https://github.com/talent-plan/tinykv
* https://hyper.rs/
* https://dev.to/daaitch/rust-ownership-and-borrows-fighting-the-borrow-checker-4ea3


