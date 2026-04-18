
// A real world use of select: timeout pattern

// The select! macro runs all branches concurrently on the same task. 
// Because all branches of the select! macro are executed on the same task, 
// they will never run simultaneously. The select! macro multiplexes 
// asynchronous operations on a single task.
// https://tokio.rs/tokio/tutorial/select
// more on select.jpg


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