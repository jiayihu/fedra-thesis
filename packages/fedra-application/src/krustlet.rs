use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use tokio::io::AsyncReadExt;

const POD_NAME: &str = "fedra-ml";
const CONTAINER_NAME: &str = "fedraml";

#[derive(Debug, Clone)]
pub struct State {
    pub rainfall_ytd: Option<f32>,
    pub rainfall_td: Option<f32>,
    pub flow_td: Option<f32>,
}

pub async fn predict(api: &Api<Pod>, state: &State) -> Result<f32> {
    let rainfall_ytd = state
        .rainfall_ytd
        .ok_or_else(|| anyhow!("No rainfall_ytd, skipping predict"))?;
    let rainfall_td = state
        .rainfall_td
        .ok_or_else(|| anyhow!("No rainfall_td, skipping predict"))?;
    let flow_td = state
        .flow_td
        .ok_or_else(|| anyhow!("No flow_td, skipping predict"))?;

    let mut ap = kube::api::AttachParams::default();
    ap.container = Some(CONTAINER_NAME.to_string());

    let input_f32: Vec<f32> = vec![rainfall_ytd, rainfall_td, flow_td];
    let mut input_str: Vec<String> = input_f32
        .into_iter()
        .map(|x| f32_to_u32(x).to_string())
        .collect();
    let mut command = vec![String::from("run")];
    command.append(&mut input_str);

    let mut process = api.exec(POD_NAME, command, &ap).await?;
    let mut output = process.stdout().unwrap();
    let mut buffer = [0; 10];
    let n = output.read(&mut buffer[..]).await?;
    let result = String::from_utf8_lossy(&buffer[..n]);
    let result = result.parse::<u32>()?;
    let result = f32::from_bits(result);

    Ok(result)
}

fn f32_to_u32(x: f32) -> u32 {
    unsafe { std::mem::transmute::<f32, u32>(x) }
}
