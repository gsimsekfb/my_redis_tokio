![Clippy](https://github.com/gsimsekfb/my_redis_tokio/actions/workflows/clippy.yml/badge.svg)

This is an educational library with working self-contained examples with lots of comments and explanations.

**Main topic:** Tokio    
**Sub topic:** In memory Redis server impl.    

==========================

### 1. Chapters

**Shared State**   
> Mutex:  
  > > [2_a_mutex_high_contention.rs](src/_2_a_mutex_high_contention.rs)  
  > > [2_b_mutex_sharding.rs](src/_2_b_mutex_sharding.rs)   

> Channels:    
  > > [server.rs](src/bin/server.rs)  
  > > [client_v1_set_get_unordered.rs](src/bin/client_v1_set_get_unordered.rs)    
  > > [client_v2_set_get_ordered.rs](src/bin/client_v2_set_get_ordered.rs)    

**Semaphores**  

> [2_d_semaphore.rs](src/_2_d_semaphore.rs)  
> [2_e_semaphore_vs_channel_capacity.rs](src/_2_e_semaphore_vs_channel_capacity.rs)  

**Other Concepts**  
[3_io_chapter.rs.todo](src/_3_io_chapter.rs.todo)  
[4_framing.rs.todo](src/_4_framing.rs.todo)  
[5_a_select_join_try_join.rs](src/_5_a_select_join_try_join.rs)  
[5_b_select_timeout.rs](src/_5_b_select_timeout.rs)  
[6_streams.rs](src/_6_streams.rs)


### 2. Usage

A. Channels - In-memory DB server / client (actor architecture) example:  

  > > [server.rs](src/bin/server.rs)  
  > > [client_v1_set_get_unordered.rs](src/bin/client_v1_set_get_unordered.rs)    
  > > [client_v2_set_get_ordered.rs](src/bin/client_v2_set_get_ordered.rs)    


**server**: serves in-memory db as `Arc<Mutex<HashMap<String, Bytes>>>`  
**client**: sends Set and Get cmds (e.g. Set "x": 42)  

```
cargo r --bin server
cr --bin client_v1_set_get_unordered
cr --bin client_v2_set_get_ordered
``` 

B. [2_a_mutex_high_contention.rs](src/_2_a_mutex_high_contention.rs)
- Shows the problem: Contention on a synchronous mutex becomes a problem, sync mutex blocks the whole thread. e.g. 100 "not fast" tasks compete for the same Mutex guarded shared data.
- Shows 3 solutions for this problem.

See more info in the source file.

```
cr --bin 2_a
```

C. [2_b_mutex_sharding.rs](src/_2_b_mutex_sharding.rs)

Instead of having one mutex: ->   Arc<Mutex<HashMap<_,_>>>,   
we have vec of mutexes:     -> Arc<Vec<Mutex<HashMap<_,_>>> 

See more info in the source file.

```
cr --bin 2_b (server, this code)
cr --bin client_v2 
```



### Notes
- `server.rs` has production level error handling
  
### Todo
- debug stack trace for runtime error when connecting to offline redis server 


