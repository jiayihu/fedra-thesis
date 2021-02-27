use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use tokio::io::AsyncReadExt;

const POD_NAME: &str = "fedra-ml";
const CONTAINER_NAME: &str = "fedraml";

pub type Sample = (f32, f32, f32);

#[derive(Debug, Clone)]
pub struct State {
    pub rainfall_ytd: Option<f32>,
    pub rainfall_td: Option<f32>,
    pub flow_ytd: Option<f32>,
}

impl State {
    pub fn as_sample(&self) -> Option<Sample> {
        self.rainfall_ytd.and_then(|rainfall_ytd| {
            self.rainfall_td.and_then(|rainfall_td| {
                self.flow_ytd
                    .and_then(|flow_ytd| Some((rainfall_ytd, rainfall_td, flow_ytd)))
            })
        })
    }
}

pub async fn predict(api: &Api<Pod>, sample: Sample) -> Result<f32> {
    let mut ap = kube::api::AttachParams::default();
    ap.container = Some(CONTAINER_NAME.to_string());

    let input_f32: Vec<f32> = vec![sample.0, sample.1, sample.2];
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
