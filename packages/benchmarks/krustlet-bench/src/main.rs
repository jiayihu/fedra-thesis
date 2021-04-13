use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::future::join_all;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, PostParams, WatchEvent};
use kube::{Api, Client};
use serde_json::json;
use tokio::runtime::Builder;

const ITERATIONS: i32 = 12;
const CONCURR: i32 = 4;

fn main() {
    let runtime = Arc::new(
        Builder::new_multi_thread()
            .worker_threads(12)
            .enable_all()
            .build()
            .unwrap(),
    );

    runtime.block_on(async {
        let kubeclient = Client::try_default().await.unwrap();
        let pods: Api<Pod> = Api::namespaced(kubeclient, "default");

        for i in 0..ITERATIONS {
            let mut request_handles = Vec::new();

            let api = pods.clone();

            for j in 0..CONCURR {
                let pods = api.clone();
                let id = ((i * CONCURR) + j).clone();

                let handle = runtime.spawn(async move {
                    let name = format!("datetime-{}", id);

                    create_pod(&name, pods.clone()).await.unwrap();
                    wait_for_execution(&name, pods.clone()).await.unwrap();
                });

                request_handles.push(handle);
            }

            join_all(request_handles).await;
        }
    });
}

async fn create_pod(name: &String, api: Api<Pod>) -> Result<()> {
    let params = PostParams::default();
    let manifest = create_container_manifest(name)?;
    let _pod = api.create(&params, &manifest).await?;

    Ok(())
}

async fn wait_for_execution(name: &String, api: Api<Pod>) -> Result<()> {
    let lp = ListParams::default()
        .fields(&format!("metadata.name={}", name))
        .timeout(10);
    let mut stream = api.watch(&lp, "0").await?.boxed();

    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(_o) => {
                println!("Added pod {}", name);
            }
            WatchEvent::Modified(o) => {
                let status = o.status.as_ref().expect("status exists on pod");
                // let phase = status.phase.clone().unwrap_or_default();
                // print!("{} at phase {}", name, phase);

                let running_state = status
                    .container_statuses
                    .as_ref()
                    .and_then(|container_statuses| container_statuses.get(0))
                    .and_then(|container_status| container_status.state.as_ref())
                    .and_then(|state| state.running.as_ref());

                if let Some(running) = running_state {
                    let scheduled_time = status.start_time.as_ref().unwrap();
                    let start_time = running.started_at.as_ref().unwrap();
                    let boot_time = start_time.0.timestamp() - scheduled_time.0.timestamp();
                    println!("Container {} started with boot time {}", name, boot_time);
                }

                // Just run to completion, the loop exits on completion without a 'Successful' phase
            }
            WatchEvent::Deleted(_o) => {
                println!("Deleted pod {}", name);
            }
            WatchEvent::Error(e) => println!("Error {}", e),
            _ => {}
        }
    }

    Ok(())
}

fn create_wasi_manifest(name: &String) -> Result<Pod> {
    serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": name
        },
        "spec": {
            "containers": [
                {
                    "name": name,
                    "image": "eu.gcr.io/fedra-301320/hello-wasi:latest",
                    "imagePullPolicy": "Always",
                    "env": [
                        {
                            "name": "NAME",
                            "value": name
                        }
                    ]
                }
            ],
            "nodeSelector": {
                "kubernetes.io/arch": "wasm32-wasi"
            },
            "tolerations": [
                {
                    "key": "kubernetes.io/arch",
                    "operator": "Equal",
                    "value": "wasm32-wasi",
                    "effect": "NoExecute",
                },
                {
                    "key": "kubernetes.io/arch",
                    "operator": "Equal",
                    "value": "wasm32-wasi",
                    "effect": "NoSchedule"
                }
            ]
        }
    }))
    .map_err(|e| anyhow!("Invalid Pod JSON: {}", e))
}

fn create_container_manifest(name: &String) -> Result<Pod> {
    serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": name
        },
        "spec": {
            "containers": [
                {
                    "name": name,
                    "image": "eu.gcr.io/fedra-301320/datetime:arm64v8",
                    "imagePullPolicy": "IfNotPresent",
                    "env": [
                        {
                            "name": "NAME",
                            "value": name
                        }
                    ]
                }
            ]
        }
    }))
    .map_err(|e| anyhow!("Invalid Pod JSON: {}", e))
}
