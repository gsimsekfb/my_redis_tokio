use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// use std::time::Duration;
//     tokio::time::sleep(Duration::from_secs(1)).await;

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

fn new_sharded_db(num_shards: usize)-> ShardedDb {
    let mut vec = Vec::with_capacity(num_shards);
    (0..num_shards).for_each(|_| {
        vec.push(Mutex::new(HashMap::new()));
    });
    Arc::new(vec)
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = new_sharded_db(1000);

    println!("-- Listening ...");

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        println!("-- Accepted");
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

fn shard_idx(key: &str, num_shards: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let key_hash = hasher.finish();
    (key_hash % (num_shards as u64)) as usize
}

async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    let num_shards = db.len();

    // e.g. incoming req frame: 
    //      frame: Array([Bulk(b"set"), Bulk(b"hello"), Bulk(b"world")])
    //      cmd: set, cmd.key: hello, cmd.value: world
    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let shard_idx = shard_idx(cmd.key(), num_shards);
                let mut shard = db[shard_idx].lock().unwrap();
                // The value is stored as `Vec<u8>`
                shard.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let shard_idx = shard_idx(cmd.key(), num_shards);
                let shard = db[shard_idx].lock().unwrap();
                if let Some(value) = shard.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented cmd {cmd:?}"),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}
