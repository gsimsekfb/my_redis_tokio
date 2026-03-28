
// Usage: todo
//
// cr --bin server
//
// cr --bin hello-tokio-my-redis
//
//
// todo
// - repeat and organize all files, add usage notes
// - debug stack trace for runtime error when connecting to offline redis server 


// 1. Simple db server / client example
//    server: serves in-memory db as Arc<Mutex<HashMap<String, Bytes>>>
//    client: sends Set and Get cmds (e.g. Set "x": 42)
// 
// cargo r --bin server
// cr --bin client_v1_set_get_unordered
// cr --bin client_v2_set_get_ordered

fn main() {
    
}
