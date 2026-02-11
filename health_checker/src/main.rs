use reqwest::Error;
use tokio;

async fn check_health(url: &str) -> Result<(), Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let responce = client.get(url).send().await?;

    if responce.status().is_success() {
        println!("{} is up and running!", url);
    } else {
        println!("Failed to reach {}: {}", url, responce.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let urls = vec![
        "http://192.168.50.60:3000",
        "http://192.168.50.53:9090",
        "http://192.168.50.53:3000",
    ];

    for url in urls {
        match check_health(url).await {
            Ok(_) => (),
            Err(e) => println!("Failed to send request to {} : {}", url, e),
        }
    }
    
    Ok(())
}