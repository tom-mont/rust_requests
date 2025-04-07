use eventsource_stream::*;
use futures_util::StreamExt;
use std::error::Error;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut stream = reqwest::Client::new()
        .get("http://github-firehose.libraries.io/events")
        .send()
        .await?
        .bytes_stream()
        .eventsource();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => println!("received event[type={}]: {}", event.event, event.data),
            Err(e) => eprintln!("error occured: {}", e),
        }
    }

    Ok(())
}
