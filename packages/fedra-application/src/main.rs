use std::collections::HashMap;

use anyhow::{anyhow, Result};
use kube::api::{ListParams, Meta, ObjectList};
use kube::{Api, Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let client = Client::try_default()
        .await
        .map_err(|e| anyhow!("Cannot create the k8s client from env config: {}", e))?;

    let temperature_svc = get_resource_service(client.clone(), &String::from("rainfall")).await?;

    match temperature_svc {
        Some(svc) => {
            log::info!("Svc {} resource {}", svc.0, svc.1);
            let url = format!("http://{}:80{}", svc.0, svc.1);
            log::info!("Requesting the resource to url {}", url);
            let resp = reqwest::get(&url).await?;
            let content = resp.text().await?;

            log::info!("Service replied with content {}", content);
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
