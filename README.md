### Intro

A mini-redis, supporting `get`, `set (with ttl)`, `del`, `publish`, `subscribe`.


Also with AOF, master-slave, cluster, Graceful exit and transactions.

BTW, all of above can run in the same time. 

If you want to use multiple-master nodes or use master-slave and cluster at the same time, then modify the `src/config.rs` and `src/redis.toml`, defaultly you can only use one.


### How to build

```bash
cargo update
cargo build
```

### How to run

```bash
cargo run --bin server [name]
cargo run --bin client [addr] [cmd]
cargo run --bin proxy  [name]
```

or with executable file:

```bash
./server [name]
./client [addr] [cmd]
./proxy  [name]
```

### Usage

Check the `*test.sh` for more details.
