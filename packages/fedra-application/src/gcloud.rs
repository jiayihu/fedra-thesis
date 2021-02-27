use std::collections::HashMap;

use anyhow::{anyhow, Result};
use google_cloud::authorize::ApplicationCredentials;
use google_cloud::datastore::{self, Client as GClient, Key};

pub async fn get_client() -> Result<GClient> {
    let creds = std::env::var("ROOT_SERVICE_ACCOUNT_JSON")?;
    // let creds = std::env::var("GCP_CREDENTIALS")?;
    // let creds = base64::decode(creds.as_str())?;
    // let creds = String::from_utf8(creds)?;

    let creds = serde_json::from_str::<ApplicationCredentials>(creds.as_str())?;
    let client = datastore::Client::from_credentials("fedra-301320", creds).await?;

    Ok(client)
}

pub async fn save_state(client: &mut GClient, sample: crate::krustlet::Sample) -> Result<Key> {
    let key = datastore::Key::new("sensors");
    let properties = {
        let mut values = HashMap::new();
        values.insert(String::from("rainfall_ytd"), sample.0);
        values.insert(String::from("rainfall_td"), sample.1);
        values.insert(String::from("flow_yd"), sample.2);

        values
    };

    let id = client
        .put((key.clone(), properties))
        .await?
        .ok_or_else(|| anyhow!("No id for the saved data"))?;

    Ok(id)
}
