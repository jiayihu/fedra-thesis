use anyhow::{anyhow, Result};

pub async fn get_rainfall_ytd(client: &reqwest::Client) -> Result<f32> {
    #[cfg(feature = "dev")]
    let url = format!("http://localhost:8084/records/ytd");
    #[cfg(not(feature = "dev"))]
    let url = format!("http://fedra-db-service/records/ytd");

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("GET error: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow!("GET HTTP error: {}", e))?;
    let rainfall_bytes = resp
        .bytes()
        .await
        .map_err(|e| anyhow!("Body bytes error: {}", e))?;
    let rainfall = crate::float::bytes_to_f32(rainfall_bytes.to_vec())
        .map_err(|e| anyhow!("Bytes to f32 error: {}", e))?;

    Ok(rainfall)
}

pub async fn save_rainfall_ytd(client: &reqwest::Client, rainfall: f32) -> Result<()> {
    #[cfg(feature = "dev")]
    let url = format!("http://localhost:8084/records/ytd");
    #[cfg(not(feature = "dev"))]
    let url = format!("http://fedra-db-service/records/ytd");

    let body = crate::float::f32_to_bytes(rainfall)?;

    client.post(&url).body(body).send().await?;

    Ok(())
}
