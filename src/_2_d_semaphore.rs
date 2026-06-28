
//// Topic:
// - Semaphore main use case
// Semaphore limits the number of active workers
// Maybe better to describe — limits concurrent access or active workers to any 
// limited resource.
// e.g. 
// - cap concurrent file or socket operations to avoid exhausting OS limits.
// - cap concurrent spawn_blocking tasks to avoid oversubscribing CPU cores.
//
// also another angle:
    // - "limit the number of connections to a server" which 
    //   demonstrates the "active workers" concept.
    // - not best example: mutex guarded shared data (this) example, cuz after 
    //   passing semaphore, there is a wait for mutex which does not look like
    //   "active working".


use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;

struct AppState {
    // The Semaphore limits how many people are "in the room"
    // preventing the system from being overwhelmed.
    limit: Semaphore,

    // The Mutex protects the specific data variable
    // preventing data corruption.
    shared_data: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        limit: Semaphore::new(50), // Only 50 tasks allowed concurrently
        shared_data: Mutex::new(Vec::new()),
    });

    // Imagine spawning 10,000 tasks
    for _ in 0..10_000 {
        let state_clone = state.clone();
        tokio::spawn(async move {
            // STEP 1: The Bouncer (Flow Control) permit
            // We wait here if 50 tasks are already running.
            let _permit = state_clone.limit.acquire().await.unwrap();

            // STEP 2: The Business Logic - !! does not involve lock
            // We are now inside the "club". We do some heavy calculation 
            // that doesn't require locking yet.
            let result = do_heavy_calculation().await;

            // STEP 3: The Bathroom Stall (Data Safety)
            // Now we briefly lock just to save our result.
            let mut data = state_clone.shared_data.lock().await;
            data.push(result);
            
            // Lock is dropped here
            // Permit is dropped here
        });
    }
}

async fn do_heavy_calculation() -> String {
    "processed".to_string()
}

// BAD PATTERN: Awaiting reverse order
/* 
    let guard = mutex.lock().await; // 1. You grabbed the data lock
        // ... 
    let permit = semaphore.acquire().await; // 2. Now you wait for a permit
        // IF the semaphore is full, you are now sleeping while holding the lock.
        // No one else can process data to free up a permit. System freezes.
            // todo: is this also called deadlock ?
*/