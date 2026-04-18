

// https://tokio.rs/tokio/tutorial/streams

// Streams
//
// A stream is an asynchronous series of values. 
// It is the asynchronous almost equivalent to Rust's std::iter::Iterator and 
// is represented by the Stream trait

// 1. Simple iteration
// Note: 
// Currently, Rust does not support async `for` loops. Instead, iterating 
// streams is done using a while let loop paired with StreamExt::next().

// use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(&[1, 2, 3]);

    // consumer:
    while let Some(value) = stream.next().await {
        println!("GOT = {value:?}", );
    }

    // or
    // async-stream crate provides a stream! macro that transforms the 
    // input into a stream:
    use async_stream::stream;
    use tokio::time::{sleep, Duration, Instant};

    // Use case: Throttled/timed stream - emits values at regular intervals.
    let stream = stream! {
        let mut when = Instant::now();
        for _ in 0..3 {
            let delay = when.saturating_duration_since(Instant::now());
            sleep(delay).await;
            yield ();
            when += Duration::from_millis(3000);
        }
    };

    tokio::pin!(stream); // needed for iteration - explained below

    // consumer:
    while let Some(value) = stream.next().await {
        println!("GOT = {value:?}");
    }


}


// 2. Real world example: Mini-Redis broadcast

// server: $ mini-redis-server
// client: $ cr --bin hello-tokio-my-redis

use tokio_stream::StreamExt;
use mini_redis::client;
use tokio::time::{sleep, Duration};

async fn publish() -> mini_redis::Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    println!("-- publisher: connected");

    let sleep_x_seconds = async || sleep(Duration::from_secs(1)).await;

    // Publish some data
    client.publish("numbers", "1".into()).await?;     sleep_x_seconds().await;
    client.publish("numbers", "two".into()).await?;   sleep_x_seconds().await;
    client.publish("numbers", "3".into()).await?;     sleep_x_seconds().await;
    client.publish("numbers", "four".into()).await?;  sleep_x_seconds().await;
    client.publish("numbers", "five".into()).await?;  sleep_x_seconds().await;
    client.publish("numbers", "6".into()).await?;
 
    println!("-- publisher: publish done");

    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;

    // todo
    #[allow(clippy::match_like_matches_macro)]
    let messages = subscriber.into_stream()
        // Adapters: fns that take a Stream and return another Stream
        // todo:
        // .filter(|msg| matches!(msg, Ok(msg) if msg.content.len() == 1))
        .filter(|msg| match msg { 
            Ok(msg) if msg.content.len() == 1 => true,
            _ => false
        })
        .take(2)
        .map(|msg| msg.unwrap().content);
            // -- publisher: connected
            // -- subscriber: got = b"1"
            // -- subscriber: got = b"3"
            // -- DONE
    // or w/o Adapters
    // let messages = subscriber.into_stream();
            // -- publisher: connected
            // -- subscriber: got = Ok(Message { channel: "numbers", content: b"two" })
            // -- subscriber: got = Ok(Message { channel: "numbers", content: b"3" })
            // -- subscriber: got = Ok(Message { channel: "numbers", content: b"four" })
            // -- subscriber: got = Ok(Message { channel: "numbers", content: b"five" })
            // -- publisher: publish done
            // -- subscriber: got = Ok(Message { channel: "numbers", content: b"6" })
                // Note: w/ this code, program does not end, wait in loop

    // !! note that the stream is pinned to the stack using tokio::pin!. 
    //    Calling next() on a stream requires the stream to be pinned.
    // 
    // A Rust value is "pinned" when it can no longer be moved in memory. 
    // A key property of a pinned value is that pointers can be taken to the 
    // pinned data and the caller can be confident the pointer stays valid. 
    // This feature is used by async/await to support borrowing data 
    // across .await points.
    tokio::pin!(messages);

    // consumer:
    while let Some(msg) = messages.next().await {
        println!("-- subscriber: got = {:?}", msg);
    }

    Ok(())
}

#[tokio::main]
async fn main_() -> mini_redis::Result<()> {
    let _handle_publish = tokio::spawn(async { publish().await });
    subscribe().await?;

    // let _res = _handle_publish.await.unwrap();
    // dbg!(_res);

    println!("-- main: DONE");

    Ok(())
}
