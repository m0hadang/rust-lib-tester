use tokio::runtime::Runtime;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn init_tokio_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let rt  = Runtime::new()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tokio_1() {
        // init_tokio_1().wait().unwrap();
    }
}
