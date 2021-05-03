use anyhow::Result;
use reqwest::Client;

const POD_NAME: &str = "fedra-ml";
const CONTAINER_NAME: &str = "fedraml";

pub type Sample = (f32, f32, f32);

#[derive(Debug, Clone)]
pub struct State {
    pub rainfall_td: Option<f32>,
    pub flow_ytd: Option<f32>,
}

impl State {
    pub fn as_pair(&self) -> Option<(f32, f32)> {
        self.rainfall_td.and_then(|rainfall_td| {
            self.flow_ytd
                .and_then(|flow_ytd| Some((rainfall_td, flow_ytd)))
        })
    }
}

pub async fn predict(sample: Sample) -> Result<f32> {
    let input_f32: Vec<f32> = vec![sample.0, sample.1, sample.2];
    let mut input_str: Vec<String> = input_f32
        .into_iter()
        .map(|x| crate::float::f32_to_u32(x).to_string())
        .collect();
    let mut command = vec![String::from("run")];
    command.append(&mut input_str);
    let query = command
        .iter()
        .map(|value| format!("command={}", value))
        .collect::<Vec<String>>();
    let query = query.join("&");

    log::info!("query {}", query);

    let url = format!(
        "https://192.168.1.126:3000/exec/default/{}/{}?{}",
        POD_NAME, CONTAINER_NAME, query,
    );
    let tls = native_tls::TlsConnector::builder()
        .use_sni(false)
        .danger_accept_invalid_certs(true)
        .build()?;
    let client = Client::builder().use_preconfigured_tls(tls).build()?;

    let result = client.post(&url).send().await?;
    let result = result.text().await?;

    let result = result.parse::<u32>()?;
    let result = f32::from_bits(result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_f32_prediction() {
        let sample: Sample = (0.31375358, 0.13323782, 0.1658887);
        let client = kube::Client::try_default().await.unwrap();
        let pod_api: Api<Pod> = Api::namespaced(client, "default");

        let prediction = predict(&pod_api, sample).await.unwrap();

        assert_eq!(prediction, 0.22571826f32);
    }
}
