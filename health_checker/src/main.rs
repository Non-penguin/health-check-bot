use anyhow::{anyhow, Context, Result};
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::fs;
use tokio;

#[derive(Deserialize)]
struct Config {
    urls: Vec<String>,
    timeout: u64,
}

async fn check_health(client: &Client, url: &str) -> Result<()> {
    let response = client.get(url).send().await.with_context(|| format!("Failed to send request to {}", url))?;

    if response.status().is_success() {
        println!("✅ {} is up and running!", url);
        Ok(())
    } else {
        let status = response.status();
        let error_message = match status {
            StatusCode::NOT_FOUND => "Not Found (404)".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error (500)".to_string(),
            _ => format!("Request failed with status: {}", status),
        };
        Err(anyhow!("{} - {}", url, error_message))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_content = fs::read_to_string("config.toml").context("Failed to read config.toml")?;

    let config: Config = toml::from_str(&config_content).context("Failed to parse config.toml")?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout))
        .build()
        .context("Failed to build HTTP client")?;

    let checks = config
        .urls
        .iter()
        .map(|url| async move { (url.clone(), check_health(&client, url).await) });

    let results = join_all(checks).await;

    let mut has_errors = false;
    for (url, result) in results {
        if let Err(e) = result {
            eprintln!("❌ Error checking {}: {:?}", url, e);
            has_errors = true;
        }
    }

    if has_errors {
        Err(anyhow!("One or more health checks failed"))
    } else {
        println!("\nAll health checks passed successfully!");
        Ok(())
    }
}