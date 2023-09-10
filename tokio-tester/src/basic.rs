use std::future::IntoFuture;
use std::io::BufRead;

use tokio::io;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub fn init_tokio_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    // Spawn the root task
    println!("root job");
    rt.block_on(async {
        println!("async job");
        //do some async work here
        Ok(())
    })
}

#[tokio::main]
async fn init_tokio_macro() -> Result<(), Box<dyn std::error::Error>> {
    println!("async job");
    Ok(())
}

fn tokio_spawn() {
    let rt = Runtime::new().unwrap();
    // Spawn a future onto the runtime
    rt.spawn(async {
        println!("now running on a worker thread");
    });
}

fn tokio_spawn_blocking() {
    let rt = Runtime::new().unwrap();
    // Spawn a blocking function onto the runtime
    rt.spawn_blocking(|| {
        println!("now running on a worker thread");
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::*;

    #[test]
    fn test_spawn() {
        let Ok(rt) = Runtime::new() else {
            panic!("failed to create runtime");
        };
        rt.spawn_blocking(|| {
            tokio::time::sleep(Duration::from_secs(1));
            println!("-> 2");
        });
        println!("-> 1");
        //print 1 -> 2
    }
    #[test]
    fn test_spawn_with_move() {
        let Ok(rt) = Runtime::new() else {
            panic!("failed to create runtime");
        };
        let v = vec![1, 2, 3];
        rt.spawn_blocking(move || { // move v into the closure
            println!("v = {:?}", v);
        });
        // println!("v = {:?}", v); // error: use of moved value: `v`
    }

    #[test]
    fn test_join_handle() {
        let Ok(rt) = Runtime::new() else {
            panic!("failed to create runtime");
        };

        rt.block_on(async {
            let join_handle: JoinHandle<Result<i32, io::Error>> = tokio::spawn(async {
                Ok(5 + 3)
            });
            let result = join_handle.await.unwrap().unwrap();
            assert_eq!(result, 8);
        });
    }
}