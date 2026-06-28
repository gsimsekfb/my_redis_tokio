
//// Problem: Contention on a synchronous mutex becomes a problem
////          problem: Sync mutex blocks the whole thread 
////
//// Example:
//// - Summary: 100 "not fast" tasks e.g. (db connection) compete for 
////   the same Mutex guarded shared data.
////   Note: even a fast task/critical section can cause contention under very
////   high concurrency (e.g. millions of ops/sec), but the real big danger is
////   caused by a slow/blocking critical section.
//// - shared data: synchronous Mutex (The bottleneck)
////   let shared_data = Arc::new(Mutex::new(0))
//// - Moment of exact problem: 1 task locks, runs slow fn, other tasks
////   wait for lock. 
//// 
//// Solution: 
//// If contention on a synchronous mutex becomes a problem, 
//// the best fix is *rarely* to switch to the Tokio mutex. Instead,
//// options to consider are to:
////
// - Dedicated task manage state and use message passing e.g. actor pattern,
    // see example in bin/.
    // With the mutex, every task itself executes the e.g. blocking DB call on 
    // a runtime worker thread; 
    // with message passing, tasks just send a cheap async message and move on,
    // while only the single dedicated task/thread actually performs the 
    // blocking work.   
// - Shard the mutex (see 2_b_mutex_sharding.rs)
    // sharding still uses multiple mutexes - just smaller ones to reduce 
    // contention
// - Restructure the code to avoid the mutex - e.g. ownership redesign
    // Each task owns its own piece of data exclusively, instead of multiple
    // tasks sharing one piece of state via a mutex.
    // The key difference: sharding still has shared, lockable state 
    // (just split into smaller pieces, so two tasks can still collide on the 
    // same shard), while ownership redesign has no shared state at all — each
    // task's data is exclusively its own, so there's never anything to lock.
// https://tokio.rs/tokio/tutorial/shared-state

//// How to try:
// cr --bin 2_a

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env().filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Application started");

    // A shared synchronous Mutex (The bottleneck)
    let shared_data = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // We spawn 100 tasks onto the Tokio runtime
    for i in 0..100 {
        let data = shared_data.clone();
        
        handles.push(tokio::spawn(async move {
            log::info!("Task {i} started");
            // PROBLEM STARTS HERE: high contention
            // Every task blocks there waiting for the previous lock-holder to
            // release it.
            let mut lock = data.lock().unwrap();
            
            log::info!("Task {i} calculating expensive...");
            // Simulate some CPU work inside the lock (holding it for a while)
            let result = slow_operation(*lock); 
            *lock = result;
            
            log::info!("Task {i} finished.");
        }));
    }

    for h in handles {
        let _ = h.await;
    }
}

// e.g. connect to a db and update value 
fn slow_operation(val: i32) -> i32 {
    // Simulate blocking CPU work (e.g., 50ms)
    thread::sleep(Duration::from_millis(10000)); 
    val + 1
}
// problem: calculation can NOT be done async
// other tasks all wait, while "only one task" is working for every 10 secs
    // 11:53:25   Application started
    // 11:53:25   Task 33 started
    // 11:53:25   Task 42 started
    // 11:53:25   Task 9 started
    // 11:53:25   Task 16 started
    // 11:53:25   Task 1 started
    // 11:53:25   Task 4 started
    // 11:53:25   Task 33 calculating expensive...
    // 11:53:25   Task 24 started
    // 11:53:25   Task 0 started
    // ...
    // ... 10 secs
    // ...
    // 11:53:35   Task 33 finished.
    // 11:53:35   Task 34 started
    // 11:53:35   Task 42 calculating expensive...
    // ...
    // ... 10 secs
    // ...
    // 11:53:45   Task 42 finished.
    // 11:53:45   Task 9 calculating expensive...
    // 11:53:45   Task 43 started
    // ...
    // ... 10 secs
    // ...
    // 11:53:55   Task 9 finished.
    // 11:53:55   Task 10 started
    // 11:53:55   Task 16 calculating expensive...