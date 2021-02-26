mod akri;
mod krustlet;

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use krustlet::State;
use kube::{Api, Client};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::try_default()
        .await
        .map_err(|e| anyhow!("Cannot create the k8s client from env config: {}", e))?;
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), "default");

    // connect_krustlet(client.clone()).await?;

    let instances = akri::get_instances(client.clone()).await?;
    let rainfall_svc = akri::find_resource_service(&instances, &String::from("rainfall"))
        .await
        .unwrap();
    let flow_svc = akri::find_resource_service(&instances, &String::from("flow"))
        .await
        .unwrap();

    let (rainfall_tx, mut rainfall_rx) = channel(1);
    let (flow_tx, mut flow_rx) = channel(1);

    let rainfall_handle =
        akri::subscribe_resource(&rainfall_svc.0, &rainfall_svc.1, rainfall_tx).await?;
    let flow_handle = akri::subscribe_resource(&flow_svc.0, &flow_svc.1, flow_tx).await?;

    let state = Arc::new(Mutex::new(State {
        rainfall_ytd: None,
        rainfall_td: None,
        flow_td: None,
    }));

    let f_pod_api = pod_api.clone();

    let r_state = state.clone();
    let f_state = state.clone();

    tokio::task::spawn(async move {
        loop {
            match rainfall_rx.recv().await {
                Some(data) => {
                    log::info!("Received rainfall data {}", data);

                    let value = data.parse::<f32>().unwrap();
                    let state = {
                        let mut state = r_state.lock().unwrap();
                        state.rainfall_ytd = state.rainfall_td;
                        state.rainfall_td = Some(value);

                        // Release the Mutex lock ASAP
                        state.clone()
                    };

                    match krustlet::predict(&pod_api, &state).await {
                        Ok(predicted_flow) => {
                            log::info!("The predicted flow is {}", predicted_flow);
                        }
                        Err(e) => {
                            log::error!("Prediction using WASM module failed: {}", e);
                        }
                    }
                }
                None => {
                    log::error!("rainfall channel closed");
                    break;
                }
            }
        }
    });

    tokio::task::spawn(async move {
        loop {
            match flow_rx.recv().await {
                Some(data) => {
                    log::info!("Received flow data {}", data);

                    let value = data.parse::<f32>().unwrap();
                    let state = {
                        let mut state = f_state.lock().unwrap();
                        state.flow_td = Some(value);

                        // Release the Mutex lock ASAP
                        state.clone()
                    };

                    match krustlet::predict(&f_pod_api, &state).await {
                        Ok(predicted_flow) => {
                            log::info!("The predicted flow is {}", predicted_flow);
                        }
                        Err(e) => {
                            log::error!("Prediction using WASM module failed: {}", e);
                        }
                    }
                }
                None => {
                    log::error!("flow channel closed");
                    break;
                }
            }
        }
    });

    rainfall_handle.await??;
    flow_handle.await??;

    Ok(())
}
