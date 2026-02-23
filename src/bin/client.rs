use bytes::Bytes;
use mini_redis::{client, Result};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Set { key: String, val: Bytes },
    Get { key: String, responder: oneshot::Sender<Option<Bytes>>  }
        // responder: 
        // Provided by the requester (get) task, used by the manager task
        // to send the command response back to the requester.
}

#[tokio::main]
async fn main() -> Result<()> {

    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        use Command::*;
        // Open a connection to the mini-redis address.
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            // println!("-- got cmd = {cmd:?}");
            match cmd {
                Set { key, val } => {
                    let res = client.set(&key, val.into()).await.unwrap();
                    dbg!(res);
                    // let _ = responder.send(value);
                },
                Get { key, responder } => {
                    let value = client.get(&key).await.unwrap();
                    let _ = responder.send(value);
                }
            }
        }
    });

    let tx_get = tx.clone();
    let _get = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "hello".to_string(), responder: resp_tx };
        tx_get.send(cmd).await.unwrap();
        // Await response
        let res = resp_rx.await.unwrap();
        println!("Getter: got = {res:?} for key: \"hello\"");
    });

    let tx_set = tx;
    let _set = tokio::spawn(async move {
        // let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { 
            key: "hello".to_string(), val: "aa".into() 
        };
        tx_set.send(cmd).await.unwrap();
            // todo: optional 
            // Await response
            // let res = resp_rx.await;
            // println!("Setter: got = {res:?}");
    });

    _set.await.unwrap();
    _get.await.unwrap();
    manager.await.unwrap();

    Ok(())
}