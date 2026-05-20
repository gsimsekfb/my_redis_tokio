![Clippy](https://github.com/gsimsekfb/my_redis_tokio/actions/workflows/clippy.yml/badge.svg)

This is an educational library with working self-contained examples with lots of comments.

**Main topic:**   
Tokio    

==========================

Chapter: Shared State   
> Mutex:  
  > > [_2_a_mutex_high_contention.rs](src/_2_a_mutex_high_contention.rs)  
  > > [_2_b_mutex_sharding.rs](src/_2_b_mutex_sharding.rs)   

> Channels:    
  > > [server.rs](src/bin/server.rs)  
  > > [client_v1_set_get_unordered.rs](src/bin/client_v1_set_get_unordered.rs)    
  > > [client_v2_set_get_ordered.rs](src/bin/client_v2_set_get_ordered.rs)    

> [_2_d_semaphore.rs](src/_2_d_semaphore.rs)  
> [_2_e_semaphore_vs_channel_capacity.rs](src/_2_e_semaphore_vs_channel_capacity.rs)  
> [_2__shared_state](src/_2__shared_state)

[_3_io_chapter.rs.todo](src/_3_io_chapter.rs.todo)  
[_4_framing.rs.todo](src/_4_framing.rs.todo)  
[_5_select_timeout.rs](src/_5_select_timeout.rs)  
[_6_streams.rs](src/_6_streams.rs)


### Usage

1. In-memory db server / client example  
   server: serves in-memory db as `Arc<Mutex<HashMap<String, Bytes>>>`  
   client: sends Set and Get cmds (e.g. Set "x": 42)  

```
cargo r --bin server
cr --bin client_v1_set_get_unordered
cr --bin client_v2_set_get_ordered
``` 

### Notes
- `server.rs` has production level error handling
  
### Todo
- debug stack trace for runtime error when connecting to offline redis server 


