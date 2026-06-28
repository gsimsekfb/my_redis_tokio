use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


//// Topic:
// - Mutex sharding example and also a solution for 2_a_mutex_high_contention.rs
// - see 2_a_mutex_high_contention.rs for more about mutex sharding.
//
//// Example:
// - Db Server 
// - Instead of having one mutex:    Arc<Mutex<HashMap<_,_>>>, 
//   we have vec of mutexes:     Arc<Vec<Mutex<HashMap<_,_>>> 
// 
// How to try:
// server: cr --bin 2_b (server, this code)
// client: cr --bin client_v2 


// Sharded Mutex
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

/// create sharded db (Vec) with `num_shards` Mutex<HashMap<String, Bytes>>'s.
fn new_sharded_db(num_shards: usize)-> ShardedDb {
    let mut vec = Vec::with_capacity(num_shards);
    (0..num_shards).for_each(|_| {
        vec.push(Mutex::new(HashMap::new()));
    });
    Arc::new(vec)
}

/// Create a new task w/ fn `process` for every connection
#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = new_sharded_db(1000);

    println!("-- Listening ...");

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        let socket_addr = socket.peer_addr();

        println!("-- \nAccepted connection: socket:{socket_addr:?}");

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        let db_ = db.clone();
        tokio::spawn(async move {
            process(socket, db_).await;
        });
        println!("-- Spawned new task w/ fn process(socket:{socket_addr:?})");
    }
}

/// shard index: 0-999
/// key: x (val: 42), num_shards: 1000 
/// e.g. return: Hash(key) % 1000
fn shard_index(key: &str, num_shards: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let key_hash = hasher.finish(); // u64
    (key_hash % (num_shards as u64)) as usize
}

/// - Open new connection with `socket`
/// - Read incoming req. frame: 
///   frame: Array([Bulk(b"set"), Bulk(b"x"), Bulk(b"42")])
/// - Convert it into cmd:
///   cmd: set, cmd.key: x, cmd.value: 42
/// - Find shard_index (0-999) for the key ("x")
/// - Insert key,val into shard db[shard_index] which is Mutex<HashMap<_,_>>  
/// - Send the result of insert as response to the client
async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    let socket_addr = socket.peer_addr();

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    let num_shards = db.len();

    // e.g. incoming req frame: 
    //      frame: Array([Bulk(b"set"), Bulk(b"x"), Bulk(b"42")])
    //      cmd: set, cmd.key: x, cmd.value: 42
    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let shard_idx = shard_index(cmd.key(), num_shards);
                let mut shard = db[shard_idx].lock().unwrap(); // HashMap<_,_>
                // The value is stored as `Vec<u8>`
                shard.insert(cmd.key().to_string(), cmd.value().clone());
                println!("-- process(): {cmd:?}");
                Frame::Simple("OK: set".to_string())
            }
            Get(cmd) => {
                let shard_idx = shard_index(cmd.key(), num_shards);
                let shard = db[shard_idx].lock().unwrap();
                println!("-- process(): {cmd:?}");
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

        println!("-- process(): response: {response}, socket: {socket_addr:?}");

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
    println!("-- process(): complete\n");
}
