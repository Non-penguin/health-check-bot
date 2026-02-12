use futures::future::join_all;
use reqwest::{Client, Error};
use serde::Deserialize;
use std::fs;
use tokio;

#[derive(Deserialize)]
struct Config {
    urls: Vec<String>,
    timeout: u64,
}

async fn check_health(client: &Client, url: &String) -> (String, Result<(), Error>) {
    let response = client.get(url).send().await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                println!("{} is up and running!", url);
                (url.clone(), Ok(()))
            } else {
                println!("Failed to reach {}: {}", url, response.status());
                (url.clone(), Ok(()))
            }
        }
        Err(e) => (url.clone(), Err(e)),
    }
}

#[tokio::main]
async fn main() {
    let config_content = match fs::read_to_string("config.toml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading config.toml: {}", e);
            return;
        }
    };

    let config: Config = match toml::from_str(&config_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config.toml: {}", e);
            return;
        }
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout))
        .build()
        .unwrap();

    let checks = config.urls.iter().map(|url| check_health(&client, url));

    let results = join_all(checks).await;

    for (url, result) in results {
        if let Err(e) = result {
            println!("Failed to send request to {}: {}", url, e);
        }
    }
}