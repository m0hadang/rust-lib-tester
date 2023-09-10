#![allow(unused)]

use std::ops::AddAssign;
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod basic;
mod channel;

#[cfg(test)]
mod tests {
    use super::*;

    async fn tokio_lock() {
        let value: Arc<RwLock<i32>> = Arc::new(RwLock::new(1));
        let value_copy1: Arc<RwLock<i32>> = value.clone();
        let value_copy2: Arc<RwLock<i32>> = value.clone();
        tokio::spawn(async move {
            let read_lock: RwLockReadGuard<i32> = value_copy1.read().await;
            assert!(
                (*read_lock == 1) || (*read_lock == 2)
            );
        });
        tokio::spawn(async move {
            let mut write_lock: RwLockWriteGuard<i32> = value.write().await;
            write_lock.add_assign(1);
        });

        tokio::time::sleep(Duration::from_secs(1)).await;

        let read_lock: RwLockReadGuard<i32> = value_copy2.read().await;
        assert_eq!(*read_lock, 2);
    }


    #[test]
    fn test_tokio() {
        let Ok(rt) = Runtime::new() else {
            panic!("failed to create runtime");
        };

        // Execute the future, blocking the current thread until completion
        rt.block_on(async {
            let handle = tokio_lock();
            // parallel
            tokio::join!(handle);
        });
    }
}
