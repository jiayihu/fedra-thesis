mod akri;
mod fedra_db;
mod float;
mod gcloud;
mod krustlet;

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use krustlet::State;
use kube::{Api, Client};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let kubeclient = Client::try_default()
        .await
        .map_err(|e| anyhow!("Cannot create the k8s client from env config: {}", e))?;
    let httpclient = reqwest::Client::new();

    let instances = akri::get_instances(kubeclient.clone()).await?;
    let rainfall_svc = akri::find_resource_service(&instances, &String::from("rainfall"))
        .await
        .ok_or_else(|| anyhow!("No resource rainfall found in akrii"))?;
    let flow_svc = akri::find_resource_service(&instances, &String::from("flow"))
        .await
        .ok_or_else(|| anyhow!("No resource flow found in akrii"))?;

    let (rainfall_tx, mut rainfall_rx) = unbounded_channel();
    let (flow_tx, mut flow_rx) = unbounded_channel();
    let (state_tx, mut state_rx) = unbounded_channel();

    let rainfall_handle =
        akri::subscribe_resource(&rainfall_svc.0, &rainfall_svc.1, rainfall_tx).await?;
    let flow_handle = akri::subscribe_resource(&flow_svc.0, &flow_svc.1, flow_tx).await?;

    let state = Arc::new(Mutex::new(State {
        rainfall_td: None,
        flow_ytd: None,
    }));

    let r_state = state.clone();
    let f_state = state.clone();

    let r_state_tx = state_tx.clone();
    let f_state_tx = state_tx.clone();

    std::thread::spawn(move || {
        spawn_async_single_thread(async {
            loop {
                match rainfall_rx.recv().await {
                    Some(data) => {
                        // log::info!("Received rainfall data {}", data);

                        let value = data.parse::<f32>().unwrap();
                        let state = {
                            let mut state = r_state.lock().unwrap();
                            state.rainfall_td = Some(value);

                            // Release the Mutex lock ASAP
                            state.clone()
                        };

                        r_state_tx.send(state).unwrap();
                    }
                    None => {
                        log::error!("rainfall channel closed");
                        break;
                    }
                }
            }
        })
    });

    std::thread::spawn(move || {
        spawn_async_single_thread(async {
            loop {
                match flow_rx.recv().await {
                    Some(data) => {
                        // log::info!("Received flow data {}", data);

                        let value = data.parse::<f32>().unwrap();
                        let state = {
                            let mut state = f_state.lock().unwrap();
                            state.flow_ytd = Some(value);

                            // Release the Mutex lock ASAP
                            state.clone()
                        };

                        f_state_tx.send(state).unwrap();
                    }
                    None => {
                        log::error!("flow channel closed");
                        break;
                    }
                }
            }
        })
    });

    std::thread::spawn(move || {
        spawn_async_single_thread(async move {
            let pod_api: Api<Pod> = Api::namespaced(kubeclient.clone(), "default");
            let gclient = gcloud::get_client().await.unwrap();

            loop {
                let pod_api_iter = pod_api.clone();
                let mut gclient_iter = gclient.clone();
                let httpclient_iter = httpclient.clone();
                let state = state_rx.recv().await.unwrap();

                match state.as_pair() {
                    Some(sample_td) => {
                        // Run tasks concurrently

                        match fedra_db::get_rainfall_ytd(&httpclient_iter).await {
                            Ok(rainfall_ytd) => {
                                log::info!("---------------------------------------");
                                log::info!("Got rainfall_ytd {}", rainfall_ytd);

                                let sample = (rainfall_ytd, sample_td.0, sample_td.1);
                                let save_rainfall_handle = tokio::spawn(async move {
                                    save_rainfall_to_local_db(&httpclient_iter, sample_td.0).await;
                                });

                                let prediction_handle = tokio::task::spawn(async move {
                                    match krustlet::predict(&pod_api_iter, sample).await {
                                        Ok(predicted_flow) => {
                                            log::info!("The predicted flow is {}", predicted_flow)
                                        }
                                        Err(e) => log::error!(
                                            "Prediction using WASM module failed: {}",
                                            e
                                        ),
                                    }
                                });

                                let upload_handle = tokio::task::spawn(async move {
                                    gcloud::save_state(&mut gclient_iter, sample).await.unwrap();
                                    log::info!("Saved the state in the cloud");
                                });

                                save_rainfall_handle.await.unwrap();
                                prediction_handle.await.unwrap();
                                upload_handle.await.unwrap();
                            }
                            Err(e) => {
                                log::error!("Error while requesting rainfall_ytd: {}", e);

                                let save_rainfall_handle = tokio::spawn(async move {
                                    save_rainfall_to_local_db(&httpclient_iter, sample_td.0).await;
                                });

                                save_rainfall_handle.await.unwrap();
                            }
                        }
                    }
                    None => {
                        continue;
                    }
                }
            }
        })
    });

    rainfall_handle.await??;
    flow_handle.await??;

    Ok(())
}

async fn save_rainfall_to_local_db(client: &reqwest::Client, rainfall: f32) {
    match fedra_db::save_rainfall_ytd(client, rainfall).await {
        Ok(()) => {
            log::info!("Saved rainfall {} in local db", rainfall)
        }
        Err(e) => log::error!("Error saving the rainfall in local db: {}", e),
    }
}

fn spawn_async_single_thread<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tokio_rt.block_on(future)
}
