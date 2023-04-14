#! /usr/bin/env python3

import lmdb

env = lmdb.open("./data/raft.mdb", max_dbs=2)
entries = env.open_db(b'entries')

with env.begin() as txn:
    for key, value in txn.cursor(db=entries):
        print('  ', key, value)