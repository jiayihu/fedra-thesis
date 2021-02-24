use std::collections::HashMap;

use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, Meta, ObjectList};
use kube::{Api, Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use websocket::client::ClientBuilder;
use websocket::OwnedMessage;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::try_default()
        .await
        .map_err(|e| anyhow!("Cannot create the k8s client from env config: {}", e))?;

    connect_krustlet(client.clone()).await?;

    // connect_akri(client.clone());

    Ok(())
}

const POD_NAME: &str = "fedra-ml";
const CONTAINER_NAME: &str = "fedraml";

async fn connect_krustlet(client: Client) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client, "default");
    let mut ap = kube::api::AttachParams::default();
    ap.container = Some(CONTAINER_NAME.to_string());

    let input_f32: Vec<f32> = vec![0.31375358, 0.13323782, 0.1658887];
    let mut input_str: Vec<String> = input_f32
        .into_iter()
        .map(|x| f32_to_u32(x).to_string())
        .collect();
    let mut command = vec![String::from("run")];
    command.append(&mut input_str);

    let mut process = pods.exec(POD_NAME, command, &ap).await?;
    let mut output = process.stdout().unwrap();
    let mut buffer = [0; 10];
    let n = output.read(&mut buffer[..]).await?;
    let result = String::from_utf8_lossy(&buffer[..n]);
    let result = result.parse::<u32>()?;
    let result = f32::from_bits(result);

    log::info!("Result of exec {}", result);

    Ok(())
}

fn f32_to_u32(x: f32) -> u32 {
    unsafe { std::mem::transmute::<f32, u32>(x) }
}

async fn connect_akri(client: Client) -> Result<()> {
    let resource_svc = get_resource_service(client, &String::from("rainfall")).await?;

    match resource_svc {
        Some(svc) => {
            log::info!("Svc {} resource {}", svc.0, svc.1);
            let handle = subscribe_resource(&svc.0, &svc.1).await?;
            handle.await??;
        }
        None => {
            log::info!("No service found for the resource")
        }
    }

    Ok(())
}

#[derive(CustomResource, Deserialize, Serialize, JsonSchema, Clone, Debug)]
#[kube(
    struct = "AkriInstance",
    apiextensions = "v1",
    group = "akri.sh",
    version = "v0",
    kind = "Instance",
    namespaced,
    shortname = "akrii"
)]
#[serde(rename_all = "camelCase")]
pub struct AkriInstanceSpec {
    configuration_name: String,
    metadata: HashMap<String, String>,
    shared: bool,
    nodes: Vec<String>,
    device_usage: HashMap<String, String>,
    rbac: String,
}

async fn get_resource_service(
    client: Client,
    resource_name: &String,
) -> Result<Option<(String, String)>> {
    let akrii: Api<AkriInstance> = Api::namespaced(client, "default");
    let lp = ListParams::default();
    let instances: ObjectList<AkriInstance> = akrii.list(&lp).await?;

    for instance in instances {
        let resource = instance.spec.metadata.get(resource_name);
        if let Some(resource) = resource {
            let mut name = instance.name();
            name.push_str("-svc");

            return Ok(Some((name, resource.to_owned())));
        }
    }

    Ok(None)
}

async fn request_resource(service: &String, resource: &String) -> Result<()> {
    let url = format!("http://{}:80{}", service, resource);

    log::info!("Requesting the resource to url {}", url);
    let resp = reqwest::get(&url).await?;
    let content = resp.text().await?;

    log::info!("Service replied with content {}", content);

    Ok(())
}

async fn subscribe_resource(service: &String, resource: &String) -> Result<JoinHandle<Result<()>>> {
    let url = format!("ws://{}:80{}", service, resource);

    log::info!("Subscribing to the resource at url {}", url);

    let client = ClientBuilder::new(url.as_str())?.connect_insecure()?;

    let (mut receiver, mut sender) = client.split().unwrap();

    let handle = tokio::spawn(async move {
        for message in receiver.incoming_messages() {
            let message = message.map_err(|e| anyhow!("Received error in websocket: {}", e))?;

            match message {
                OwnedMessage::Close(reason) => {
                    log::info!("Websocket connection closed with reason: {:?}", reason);

                    sender.send_message(&OwnedMessage::Close(None))?;
                }
                OwnedMessage::Binary(content) => {
                    let content = String::from_utf8_lossy(&content[..]);
                    log::info!("Received binary content from resource {}", content);
                }
                OwnedMessage::Text(content) => {
                    log::info!("Received text content from resource {}", content);
                }
                m => {
                    log::info!("Received unknown content from resource: {:?}", m);
                }
            }
        }

        Ok(())
    });

    Ok(handle)
}
