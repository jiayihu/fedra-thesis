use std::sync::{Arc, Mutex};
use std::time::Instant;

use futures::future::join_all;
use reqwest::Client;
use tokio::runtime::Builder;

const ITERATIONS: i32 = 15;
const CONCURR: i32 = 64;

fn main() {
    let runtime = Arc::new(
        Builder::new_multi_thread()
            .worker_threads(12)
            .enable_all()
            .build()
            .unwrap(),
    );

    runtime.block_on(async {
        let tls = native_tls::TlsConnector::builder()
            .use_sni(false)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let client = Client::builder()
            .use_preconfigured_tls(tls)
            .build()
            .unwrap();

        let response_times = Arc::new(Mutex::new(Vec::new()));

        for _i in 0..ITERATIONS {
            let mut request_handles = Vec::new();

            for _j in 0..CONCURR {
                let client = client.clone();
                let times = response_times.clone();

                let handle = runtime.spawn(async move {
                    let instant = Instant::now();
                    predict(&client, (0.1_f32, 0.1_f32, 0.1_f32)).await;
                    let elapsed = instant.elapsed().as_millis();
                    times.lock().unwrap().push(elapsed);

                    println!("Prediction done");
                });

                request_handles.push(handle);
            }

            join_all(request_handles).await;
        }

        println!("Response times {:?}", response_times.lock().unwrap());
    });
}

const POD_NAME: &str = "fedra-ml";
const CONTAINER_NAME: &str = "fedraml";

pub type Sample = (f32, f32, f32);

pub async fn predict(client: &Client, sample: Sample) {
    let input_f32: Vec<f32> = vec![sample.0, sample.1, sample.2];
    let mut input_str: Vec<String> = input_f32
        .into_iter()
        .map(|x| f32_to_u32(x).to_string())
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
        "https://192.168.1.98:3000/exec/default/{}/{}?{}",
        POD_NAME, CONTAINER_NAME, query,
    );

    let _result = client.post(&url).send().await.unwrap();
}

pub fn f32_to_u32(x: f32) -> u32 {
    unsafe { std::mem::transmute::<f32, u32>(x) }
}
