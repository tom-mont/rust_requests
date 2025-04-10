use eventsource_stream::{Event, Eventsource};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use std::fmt;
use thiserror::Error;
use tokio::signal;

// TODO fix up warnings

// Custom error type for application
#[derive(Error, Debug)]
enum AppError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Event stream error: {0}")]
    EventStreamError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone)]
struct Config {
    api_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://github-firehose.libraries.io/events".to_string(),
        }
    }
}

// Type-safe github event model
#[derive(Debug, Deserialize, Serialize)]
struct GithubEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    actor: Actor,
}

#[derive(Debug, Deserialize, Serialize)]
struct Actor {
    display_login: String,
    // add other fields as needed
}

// Display implementation for pretty printing
impl fmt::Display for GithubEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}\ntype: {}\ndisplay loging: {}",
            self.id, self.event_type, self.actor.display_login
        )
    }
}

struct EventFetcher {
    client: reqwest::Client,
    config: Config,
}

impl EventFetcher {
    fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn fetch_events(
        &self,
    ) -> std::result::Result<
        impl futures_util::Stream<Item = std::result::Result<Event, String>>,
        AppError,
    > {
        let response = self
            .client
            .get(&self.config.api_url)
            .send()
            .await?
            .bytes_stream()
            .eventsource()
            .map(|result| result.map_err(|e| e.to_string()));

        Ok(response)
    }
}

struct EventProcessor {}

impl EventProcessor {
    fn new() -> Self {
        Self {}
    }

    async fn process_event(&self, event: Event) -> std::result::Result<GithubEvent, AppError> {
        let event_data = event.data;
        let github_event: GithubEvent =
            serde_json::from_str(&event_data).map_err(AppError::JsonError)?;
        Ok(github_event)
    }
}

struct Application {
    fetcher: EventFetcher,
    processor: EventProcessor,
    config: Config,
}

impl Application {
    fn new(config: Config) -> Self {
        Self {
            fetcher: EventFetcher::new(config.clone()),
            processor: EventProcessor::new(),
            config,
        }
    }

    async fn run(&self) -> std::result::Result<(), AppError> {
        let mut stream = self.fetcher.fetch_events().await?;

        loop {
            tokio::select! {
                Some(event_result) = stream.next() => {
                    match event_result {
                        Ok(event) => {
                            match self.processor.process_event(event).await {
                                Ok(github_event) => println!("{}", github_event),
                                Err(e) => eprintln!("Event stream error: {}", e)
                            }
                        }
                        Err(e) => {
                            eprintln!("Event stream error: {}", e)
                        }
                    }
                }
                _ = signal::ctrl_c() => {
                println!("Shutting down gracefully...");
                break;
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Setup logging (consider using a proper logging framework)
    println!("GitHub Event Monitor Starting...");

    // Create config (could be loaded from file/env)
    let config = Config::default();

    // Initialize and run application
    let app = Application::new(config);

    if let Err(e) = app.run().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
