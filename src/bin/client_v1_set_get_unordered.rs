use bytes::Bytes;
use mini_redis::{client, Result};
use tokio::sync::{mpsc, oneshot};

// Summary:
// - Note: Types of order for tasks: spawn, execution, wait to complete
// - Note: "Execution" order of tasks are non-deterministic in this version.
// - There are 3 tasks. Consumer: Manager, Producers: Get, Set
// - Getter task sends Get cmd to Manager task
// - Setter task sends Set cmd to Manager task
// - Manager task sends each cmd to db server and sends the 
//   cmd response back to the Get/Set tasks
// - See also more info in code.

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

    //// Note: 1,2,3 is spawn order. Execution is unordered.

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

    //// 2. Getter task
    let tx_get = tx.clone();
    let _get = tokio::spawn(async move {
        println!("-- get task: started");
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "x".to_string(), responder: resp_tx };
        println!("-- Get task: sending cmd {cmd:?} ... \n");
        tx_get.send(cmd).await.unwrap();  // send cmd to manager task
        let res = resp_rx.await.unwrap(); // await response from manager task
        println!("-- Get task: key: \"x\", val: {res:?}");
    });

    //// 3. Setter task
    let tx_set = tx;
    let _set = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        println!("-- set task: started");
        let cmd = Command::Set { 
            key: "x".to_string(), val: "42".into(), responder: resp_tx
        };
        println!("-- Set task: sending cmd {cmd:?} ... \n");
        tx_set.send(cmd).await.unwrap(); // send cmd to manager task
        // !! Wait manager response to be sure that set cmd executed by db server 
        let res = resp_rx.await.unwrap(); // await response from manager task
        println!("-- Set task: result: {res:?}");
    });

    //// 4. main's "wait task to complete" order
    //// Tasks already spawned in order 1,2,3.
    //// But spawning doesn't determine execution order — Tokio schedules them 
    //// and they run concurrently !! So in practice the "execution" order is 
    //// "non-deterministic".
    //// See and compare client_v2 for ordered execution
    //// And these are the main's blocking "wait task to complete" order:
    //// (again, not execution order)
    _set.await.unwrap(); // this line does not mean set cmd sent/processed 
                         // by db server before get cmd
    _get.await.unwrap();
    manager.await.unwrap();
        // Note: 
        // how program ends:
        // _set completes → tx_set (which is moved into task) dropped
        // _get completes → tx_get (which is moved into task) dropped
        // Once both are dropped, rx.recv() returns None, the while exits, 
        // and the manager task ends.

    Ok(())
}