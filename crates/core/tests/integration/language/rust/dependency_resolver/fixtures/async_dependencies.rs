use std::future::Future;

async fn fetch_data() -> Result<String, String> {
    Ok("data".to_string())
}

async fn process_data() -> Result<(), String> {
    let data = fetch_data().await?;
    println!("Processed: {}", data);
    Ok(())
}

fn spawn_task<F: Future<Output = ()>>(future: F) {
    // Simulate spawning async task
    std::hint::black_box(future);
}

async fn main() {
    let _ = process_data().await;
    
    spawn_task(async {
        let data = fetch_data().await.unwrap();
        println!("In spawn: {}", data);
    });
}