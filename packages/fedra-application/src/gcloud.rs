use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use google_cloud::authorize::ApplicationCredentials;
use google_cloud::datastore::{self, Client as GClient, Key};

pub async fn get_client() -> Result<GClient> {
    #[cfg(feature = "dev")]
    let creds = std::env::var("ROOT_SERVICE_ACCOUNT_JSON")?;
    #[cfg(not(feature = "dev"))]
    let creds = {
        let creds = std::env::var("GCP_CREDENTIALS")?;
        let creds = base64::decode(creds.as_str())?;

        String::from_utf8(creds)?
    };

    let creds = serde_json::from_str::<ApplicationCredentials>(creds.as_str())?;
    let client = datastore::Client::from_credentials("fedra-301320", creds).await?;

    Ok(client)
}

pub async fn save_state(client: &mut GClient, sample: crate::krustlet::Sample) -> Result<Key> {
    let key = datastore::Key::new("sensors");
    let properties = {
        let mut values = HashMap::new();
        values.insert(String::from("rainfall_ytd"), sample.0.to_string());
        values.insert(String::from("rainfall_td"), sample.1.to_string());
        values.insert(String::from("flow_yd"), sample.2.to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        values.insert(String::from("timestamp"), timestamp.to_string());

        values
    };

    let id = client
        .put((key.clone(), properties))
        .await?
        .ok_or_else(|| anyhow!("No id for the saved data"))?;

    Ok(id)
}
