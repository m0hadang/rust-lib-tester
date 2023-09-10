use std::future::IntoFuture;
use std::io::BufRead;

use tokio::io;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

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
    use super::*;

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