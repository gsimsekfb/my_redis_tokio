
//// Topic:
//// - Fundamentals of select, join and try_join macros


use rand::RngExt;

fn random_num() -> u64 { rand::rng().random_range(1..=10) }

async fn sleep_ms(millis: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(millis)).await;
}

async fn f1() -> u64 {
    let ms = random_num()*100;
    println!("11  f1 started w/ ms: {ms}");
    sleep_ms(ms).await; 
    ms
}

async fn f2() -> u64 {
    let ms = random_num()*100;
    println!("22  f2 started w/ ms: {ms}");
    sleep_ms(ms).await; 
    ms
}

#[tokio::main]
async fn main() {
    let mut cntr = 0;
    loop {
        cntr += 1;
        println!("====== loop {cntr}");

        //// 1. select!
        // Waits on multiple concurrent branches (e.g. async fn) in same task (or 
        // multiple tasks when f1 and f2 are spawned tasks) returning when the 
        // first branch completes, cancelling the remaining branches.
        // - e.g. if f2 completes first, f1() will be dropped 

        // a. Default use case: concurrent branches
        // f1 and f2 run concurrently in the same task (no parallelism)
        // e.g. so, we do NOT need to clone an Arc, things work synchronously 
        // in the same task
        //
        tokio::select! {
            ms = f1() => { println!("11  f1() completed in {ms} ms") }
            ms = f2() => { println!("22  f2() completed in {ms} ms") }
        };

        // b. Second use case: parallel execution in different tasks
        // so, we DO need to clone the Arc

        // Spawn each async function into its own task.
        // This returns a JoinHandle, which is itself a future.
        let f1_handle = tokio::spawn(f1());
        let f2_handle = tokio::spawn(f2());
    
        // Now we select! on the JoinHandles, not the functions directly.
        tokio::select! {
            res = f1_handle => {
                // res is a Result<u64, JoinError>
                match res {
                    Ok(ms) => println!("11  f1() task completed in {ms} ms"),
                    Err(e) => eprintln!("11  f1() task failed: {e}"),
                }
            }
            res = f2_handle => {
                match res {
                    Ok(ms) => println!("22  f2() task completed in {ms} ms"),
                    Err(e) => eprintln!("22  f2() task failed: {e}"),
                }
            }
        };


        // 2. join! (and try_join!), similar to select!, read select! above first
        // Waits on multiple concurrent branches (e.g. async fn) in same task (or 
        // multiple tasks when f1 and f2 are spawned tasks) 
        // returning when all branches complete.
        // try_join: 
        // same but it waits until returning when all branches complete 
        // with Ok(_) or on the first Err(_).
        //
        let (_res_f1, _res_f2) = tokio::join!(
            f1(), f2()
        );
        // do something with the values        

        println!("== loop {cntr} ends\n");
    }
}



/* 
stdout:

====== loop 1
22  f2 started w/ ms: 200
11  f1 started w/ ms: 700
22  f2() completed in 200 ms    // f2 won the race to complete first
                                // !! f1() dropped
== loop 1 ends

====== loop 2
22  f2 started w/ ms: 500
11  f1 started w/ ms: 100
11  f1() completed in 100 ms
== loop 2 ends

====== loop 3
11  f1 started w/ ms: 800
22  f2 started w/ ms: 1000
11  f1() completed in 800 ms
== loop 3 ends

====== loop 4
22  f2 started w/ ms: 700
11  f1 started w/ ms: 300
11  f1() completed in 300 ms
== loop 4 ends
*/


/* 
Step-by-Step Execution:

1. Initialization:

- The loop starts a new iteration.
- tokio::select! evaluates all futures (f1() and f2()) and begins polling them concurrently.

2. Polling:

- Both f1() and f2() are started simultaneously (they are "spawned" as async tasks under the hood).
- The select! macro waits for whichever future completes first.

3. Completion:

a) If f1() completes first:
    The x = f1() branch is taken.
    The result is bound to x.
    println!("Operation 1 completed first with: {}", x) executes.
    The other future (f2()) is dropped (its work is discarded).

b) If f2() completes first:
    The y = f2() branch is taken.
    The result is bound to y.
    println!("Operation 2 completed first with: {}", y) executes.
    The other future (f1()) is dropped.

4. Next Iteration:

The loop restarts, and select! runs again, re-evaluating f1() and f2() from scratch (since they were dropped after the previous iteration).
*/