
//// Example: Contention on a synchronous mutex becomes a problem
////          problem: Sync mutex blocks the whole thread 
////
//// Solution: 
//// If contention on a synchronous mutex becomes a problem, 
//// the best fix is *rarely* to switch to the Tokio mutex. Instead,
//// options to consider are to:
////
// - Dedicated task manage state and use message passing (see todo.rs)
// - Shard the mutex (see mutex_sharding.rs)
// - Restructure the code to avoid the mutex
//
// https://tokio.rs/tokio/tutorial/shared-state

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env().filter_level(log::LevelFilter::Info)
        .init();
    use log::info;
    info!("Application started");

    // A shared synchronous Mutex (The bottleneck)
    let shared_data = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // We spawn 100 tasks onto the Tokio runtime
    for i in 0..100 {
        let data = shared_data.clone();
        
        handles.push(tokio::spawn(async move {
            info!("Task {i} started");
            // PROBLEM STARTS HERE
            // If Task 1 holds this lock, and Task 2 tries to grab it...
            let mut lock = data.lock().unwrap();
            
            info!("Task {i} calculating expensive...");
            // Simulate some CPU work inside the lock (holding it for a while)
            // This mimics high contention processing
            let result = do_expensive_calculation(*lock); 
            *lock = result;
            
            info!("Task {i} finished.");
        }));
    }

    for h in handles {
        let _ = h.await;
    }
}

fn do_expensive_calculation(val: i32) -> i32 {
    // Simulate blocking CPU work (e.g., 50ms)
    thread::sleep(Duration::from_millis(10000)); 
    val + 1
}
// problem: calculation can NOT be done async
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