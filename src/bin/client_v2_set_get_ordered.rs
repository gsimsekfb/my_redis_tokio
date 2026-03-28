use bytes::Bytes;
use mini_redis::{client, Result};
use tokio::sync::{mpsc, oneshot};

// Summary:
// - Setter task sends Set cmd to Manager task
// - Getter task sends Get cmd to Manager task
// - Manager task sends each cmd to db server and sends the 
//   cmd response back to the Get/Set tasks
// - See also more info in code.


//// How get-set ordering achieved: 
// manager task spawned
// set task spawned
// manager and set tasks started "unordered" and running concurrently
//   - manager task:
//      - await connect to db server
//      - await msgs/cmds from set and get channels
//      - await send cmd to db server
//      - send resp. to set/get task, signal for cmd execution by db server 
//   - set task:
//      - await send cmd to manager task
//      - await response from manager task !! to make sure set is complete
// block: main waits for set task to complete 
    // !! key step
// get task spawned
// get task started running
// block: main waits for get task to complete
// block: main waits for manager task to complete



#[derive(Debug)]
enum Command {
    Set { key: String, val: Bytes, responder: oneshot::Sender<Result<()>> },
    Get { key: String, responder: oneshot::Sender<Option<Bytes>>  }
        // responder: 
        // channel to get the manager task send the get/set cmd 
        // result (e.g. val: 42) back to the Get/Set tasks
}

#[tokio::main]
async fn main() -> Result<()> {

    let (tx, mut rx) = mpsc::channel(32);
        // producers: tx_get, tx_set
        // consumer : manager

    //// 1. Receives cmds from Get/Set tasks, sends to db server
    let manager = tokio::spawn(async move {
        println!("-- manager task: started");
        use Command::*;
        // Open a connection to the mini-redis address.
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // recv() returns None when the channel closed (tx_get, tx_set dropped) 
        // and there are no remaining messages in the channel's buffer.
        while let Some(cmd) = rx.recv().await {
            println!("-- manager task: got cmd = {cmd:?}\n");
            match cmd {
                Set { key, val, responder } => {
                    let res= client.set(&key, val.into()).await;
                    // !! signal that set cmd processed by db server
                    let _ = responder.send(res); // send resp. to Set task
                },
                Get { key, responder } => {
                    let value = client.get(&key).await.unwrap();
                    let _ = responder.send(value); // send resp. to Get task
                }
            }
        }
    });
    
    //// x. Setter task
    let tx_set = tx.clone();
    let _set = tokio::spawn(async move {
        println!("-- set task: started");
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { 
            key: "x".to_string(), val: "42".into(), responder: resp_tx
        };
        println!("-- set task: sending cmd {cmd:?} ... \n");
        tx_set.send(cmd).await.unwrap(); // send cmd to manager task
        // !! Wait manager response to be sure that set cmd executed by db server 
        let res = resp_rx.await.unwrap(); // await response from manager task
        println!("-- set task: result: {res:?}\n");
    });

    _set.await.unwrap(); // block wait for set task to complete

    //// x. Getter task
    let tx_get = tx;
    let _get = tokio::spawn(async move {
        println!("-- get task: started");
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "x".to_string(), responder: resp_tx };
        println!("-- get task: sending cmd {cmd:?} ... \n");
        tx_get.send(cmd).await.unwrap();  // send cmd to manager task
        let res = resp_rx.await.unwrap(); // await response from manager task
        println!("-- get task: key: \"x\", val: {res:?}");
    });

    _get.await.unwrap();

    //// x.
    manager.await.unwrap();
        // Note:
        // how program ends:
        // _set completes → tx_set (which is moved into task) dropped
        // _get completes → tx_get (which is moved into task) dropped
        // Once both are dropped, rx.recv() returns None, the while exits, 
        // and the manager task ends.

    Ok(())
}