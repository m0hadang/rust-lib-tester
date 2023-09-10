
#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;
    use super::*;

    async fn tokio_channel_1() {
        let (tx, mut rx)
            = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let val = String::from("hi");
            if let Err(_) = tx.send(val).await {
                return;
            }
            /*
            can't use val after send
            println!("send {}", val);
            */
        });

        if let Some(received) = rx.recv().await {
            assert_eq!(received, "hi");
        } else {
            assert!(false);
        }
    }
    async fn tokio_channel_2() {
        let (tx, mut rx) =
            tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            for i in 0..10 {
                if let Err(_) = tx.send(i).await {
                    return;
                }
            }
        });
        let mut sum = 0;
        while let Some(i) = rx.recv().await { // wait for next value
            sum += i;
        }

        assert_eq!(sum, (0..10).sum());
    }
    async fn tokio_channel_3() {
        let (tx, mut rx) =
            tokio::sync::mpsc::channel(100);
        let tx_copy = tx.clone();
        tokio::spawn(async move {
            for i in 0..10 {
                if let Err(_) = tx.send(i).await {
                    return;
                }
            }
        });
        tokio::spawn(async move {
            for i in 10..20 {
                if let Err(_) = tx_copy.send(i).await {
                    return;
                }
            }
        });

        let mut sum = 0;
        while let Some(i) = rx.recv().await { // wait for next value
            sum += i;
        }

        assert_eq!(sum, (0..20).sum());
    }

    #[test]
    fn test_channel() {
        let Ok(rt) = Runtime::new() else {
            panic!("failed to create runtime");
        };

        // Execute the future, blocking the current thread until completion
        rt.block_on(async {
            // parallel
            tokio::join!(
                tokio_channel_1(),
                tokio_channel_2(),
                tokio_channel_3(),
            );
        });
    }
}
