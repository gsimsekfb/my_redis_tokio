
//// Note:
// Taking care and picking good bounds is a big part of writing reliable 
// Tokio applications.
// Src: https://tokio.rs/tokio/tutorial/channels


//// Semaphore vs Channel Capacity
// Semaphore limits the number of active workers, the Channel Capacity 
// limits the size of the waiting line.
//
// In Tokio, this concept is called Backpressure.


use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use std::sync::Arc;

// #[tokio::main]
async fn main__() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    
    // 1. Limit concurrency to 100 simultaneous connections
    // We wrap it in an Arc to share it across the loop
    let semaphore = Arc::new(Semaphore::new(100));

    loop {
        // 2. Acquire a permit *before* accepting (or immediately after)
        // acquire_owned() returns a permit that can be moved into the spawned task
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        // 3. Accept the connection
        let (socket, _) = listener.accept().await.unwrap();

        // 4. Spawn the task, moving the permit inside
        tokio::spawn(async move {
            // The permit is held here. It is automatically dropped (and returned to the pool)
            // when this block ends (even if it panics).
            let _permit = permit; 
            
            process(socket).await;
        });
    }
}

async fn process(socket: tokio::net::TcpStream) {
    // Handle the connection...
    println!("Processing connection");
}


// ====


// use tokio::net::TcpListener;
use tokio::sync::mpsc;

// #[tokio::main]
async fn main_() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    // 1. Create a channel with a fixed capacity (Bound = 100)
    // "tx" is the sender (Producer), "rx" is the receiver (Consumer)
    let (tx, mut rx) = mpsc::channel(100);

    // 2. Spawn a background "Manager" task (The Consumer)
    // This task pulls items off the queue and processes them.
    tokio::spawn(async move {
        while let Some(connection) = rx.recv().await {
            // Simulate processing the connection
            process_(connection).await;
        }
    });

    // 3. The Main Loop (The Producer)
    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // CLONE the sender for the new connection
        let tx_clone = tx.clone();

        // 4. Send the work to the queue
        tokio::spawn(async move {
            // CRITICAL MOMENT:
            // If the channel has 100 items in it, this line will WAIT here.
            // It will not return. This creates "backpressure."
            if tx_clone.send(socket).await.is_err() {
                println!("Receiver dropped, system shutting down");
            }
        });
    }
}

async fn process_(_socket: tokio::net::TcpStream) {
    // Heavy work happens here
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Processed connection");
}
