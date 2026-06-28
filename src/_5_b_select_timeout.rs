
// Topic: A real world use of select: timeout pattern w/ select macro

// - The select! macro runs all branches concurrently on the same task. 
// - Waits on multiple concurrent branches (e.g. async fn) in same task (or 
//   multiple tasks when f1 and f2 are spawned tasks) returning when the 
//   first branch completes, cancelling the remaining branches.
// - e.g. if f2 completes first, f1() will be dropped 
//
// - more on file src/_5_a_select_join_try_join.rs and select.jpg



use tokio::time::{sleep, Duration};

async fn fetch_data() -> String {
    // Simulate slow DB/API call
    sleep(Duration::from_secs(3)).await;
    "data".to_string()
}

#[tokio::main]
async fn main() {
    tokio::select! {
        result = fetch_data() => {
            println!("Got: {}", result);
        }
        _ = sleep(Duration::from_secs(2)) => {
            println!("Timeout! Request took too long (> 2 secs).");
        }
    }
}

