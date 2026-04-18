![Clippy](https://github.com/gsimsekfb/my_redis_tokio/actions/workflows/clippy.yml/badge.svg)

### Todo
- debug stack trace for runtime error when connecting to offline redis server 

### Notes
- server.rs has production level error handling



### Usage

1. In-memory db server / client example
   server: serves in-memory db as Arc<Mutex<HashMap<String, Bytes>>>
   client: sends Set and Get cmds (e.g. Set "x": 42)

```
cargo r --bin server
cr --bin client_v1_set_get_unordered
cr --bin client_v2_set_get_ordered
``` 