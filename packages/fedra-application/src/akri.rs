use anyhow::{anyhow, Result};
use kube::api::{ListParams, Meta, ObjectList};
use kube::{Api, Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use websocket::client::ClientBuilder;
use websocket::OwnedMessage;

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

pub async fn get_instances(client: Client) -> Result<ObjectList<AkriInstance>> {
    let akrii: Api<AkriInstance> = Api::namespaced(client, "default");
    let lp = ListParams::default();
    let instances: ObjectList<AkriInstance> = akrii.list(&lp).await?;

    Ok(instances)
}

pub async fn find_resource_service(
    instances: &ObjectList<AkriInstance>,
    resource_name: &String,
) -> Option<(String, String)> {
    for instance in instances {
        let resource = instance.spec.metadata.get(resource_name);
        if let Some(resource) = resource {
            let mut name = instance.name();
            name.push_str("-svc");

            return Some((name, resource.to_owned()));
        }
    }

    None
}

pub async fn subscribe_resource(
    _service: &String,
    resource: &String,
    tx: Sender<String>,
) -> Result<JoinHandle<Result<()>>> {
    // let url = format!("ws://{}:80{}", service, resource);
    let url = format!("ws://192.168.1.126:8083{}", resource);

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
                OwnedMessage::Text(content) => match tx.send(content).await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error sending to the resource channel {}", e);
                    }
                },
                m => {
                    log::info!("Received unknown content from resource: {:?}", m);
                }
            }
        }

        Ok(())
    });

    Ok(handle)
}

async fn _request_resource(service: &String, resource: &String) -> Result<()> {
    let url = format!("http://{}:80{}", service, resource);

    log::info!("Requesting the resource to url {}", url);
    let resp = reqwest::get(&url).await?;
    let content = resp.text().await?;

    log::info!("Service replied with content {}", content);

    Ok(())
}
