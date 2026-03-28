use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


// Main task:
// - create db: Arc<Mutex<HashMap<String, Bytes>>>, own it
// - loop 
//   . await for a new network connection 
//   . once new connection arrived, create socket, spawn worker task and 
//     share a copy of ptr to db with it and move socket to into it too.
// Worker task:
// - create a connection from socket
// - await frames from connection
// - convert frame to cmd and execute cmd
// - send result to client

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("-- main task: Listening ...");

    let db: Arc<Mutex<HashMap<String, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // Wait for new connection
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone(); // clones the Arc ptr, not the HashMap

        println!("-- main task: Accepted new connection");
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    println!("-- worker task: started, socket: {socket:?}");

    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    // incoming req frame:
    //   e.g. 
    //      frame: Array([Bulk(b"set"), Bulk(b"x"), Bulk(b"42")])
    //      cmd: set, cmd.key: x, cmd.value: 42
    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame);
        println!("-- worker task: fn process(): incoming cmd: {:?}", cmd);
        let response = match cmd.unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("worker task: unimplemented cmd {cmd:?}"),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
    println!("-- worker task: fn process() ends..\n");
}