use crate::error::Result;
use crate::pod::PodKey;
use brokerapi::catalog::{Catalog, Service, ServicePlan};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Meta, PostParams};
use kube::{Api, Client};
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Provision {
    pub instance_id: String,
    pub service_id: String,
    pub plan_id: String,
    pub pod_key: PodKey,
}

#[derive(Clone)]
pub struct FedraProvider {
    pods: Api<Pod>,

    plans: HashMap<String, ServicePlan>,
    catalog: Catalog,
    // A map of provisioned services by instance id
    provisions: HashMap<String, Provision>,
}

impl FedraProvider {
    pub fn new(client: Client, namespace: String) -> Self {
        let mut provider = FedraProvider {
            pods: Api::namespaced(client, &namespace),

            plans: HashMap::new(),
            catalog: Catalog::default(),
            provisions: HashMap::new(),
        };

        provider.add_plan(Self::create_free_plan());
        let hello_wasi = provider.create_service(
            String::from("hello_wasi"),
            String::from("Hello world in WASI"),
            "free",
        );
        provider.add_service(hello_wasi);

        provider
    }

    pub async fn provision_service(&self, _service_id: String, _plan_id: String) -> Result<PodKey> {
        let manifest: Pod = serde_json::from_value(json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name": "hello-wasi" },
            "spec": {
                "containers": [
                    {
                        "name": "hello-wasi",
                        "image": "eu.gcr.io/fedra-thesis/hello-wasi:latest"
                    }
                ],
                "nodeSelector": { "kubernetes.io/arch": "wasm32-wasi" },
                "tolerations": [
                    {
                        "key": "kubernetes.io/arch",
                        "operator": "Equal",
                        "value": "wasm32-wasi",
                        "effect": "NoExecute"
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
        .expect("Invalid Pod JSON");
        let params = PostParams::default();

        let pod = self.pods.create(&params, &manifest).await?;
        let name = Meta::name(&pod);
        log::info!("Created pod {:#?}", name);

        Ok(PodKey::from(pod))
    }

    pub fn register_provision(&mut self, provision: Provision) {
        self.provisions
            .insert(provision.instance_id.clone(), provision);
    }

    pub fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    fn add_plan(&mut self, plan: ServicePlan) -> &Self {
        self.plans.insert(plan.id.clone(), plan);

        self
    }

    fn get_plan(&self, plan_id: &str) -> Option<&ServicePlan> {
        self.plans.get(plan_id)
    }

    fn add_service(&mut self, service: Service) -> &Self {
        self.catalog.services.push(service);

        self
    }

    fn create_free_plan() -> ServicePlan {
        let mut free_plan = ServicePlan::default();
        free_plan.id = "free".to_string();
        free_plan.name = "free".to_string();
        free_plan.description = "Free plan".to_string();
        free_plan.maximum_polling_duration = Some(60);

        free_plan
    }

    fn create_service(&self, name: String, description: String, plan: &str) -> Service {
        let plan = self.get_plan(plan).unwrap();

        let mut hello_wasi = Service::default();
        hello_wasi.id = name.clone();
        hello_wasi.name = name.clone();
        hello_wasi.description = description;
        hello_wasi.tags.push(name);
        hello_wasi.bindable = true;
        hello_wasi.plans.push(plan.clone());

        hello_wasi
    }
}
