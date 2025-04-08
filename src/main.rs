use eventsource_stream::*;
use futures_util::StreamExt;
use serde_json::Value;
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
        //println!("{}", &event.unwrap().data);
        pretty_print(&event.unwrap().data).await?;
    }

    Ok(())
}

async fn pretty_print(data: &String) -> Result<(), Box<dyn Error>> {
    let v: Value = serde_json::from_str(data)?;
    println!(
        "id: {}\ntype: {}\ndisplay login: {}",
        // Note the syntax for accessing nested JSON:
        v["id"],
        v["type"],
        v["actor"]["display_login"]
    );

    Ok(())
}
