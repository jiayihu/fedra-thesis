use crate::pod::PodKey;
use anyhow::{anyhow, Result};
use brokerapi::binding::Endpoint;
use brokerapi::catalog::{Catalog, Service, ServicePlan};
use k8s_openapi::api::core::v1::{Pod, Service as KubeService};
use kube::api::{DeleteParams, Meta, PostParams};
use kube::{Api, Client};
use serde_json::json;
use std::collections::HashMap;

const FEDRA_DB: &str = "fedra-db";
const FREE_PLAN: &str = "free";

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
    kube_services: Api<KubeService>,

    plans: HashMap<String, ServicePlan>,
    catalog: Catalog,
    // A map of provisioned services by instance id
    provisions: HashMap<String, Provision>,
    // A map of binded k8s services by binding id
    bindings: HashMap<String, Endpoint>,
}

impl FedraProvider {
    pub fn new(client: Client, namespace: String) -> Self {
        let mut provider = FedraProvider {
            pods: Api::namespaced(client.clone(), &namespace),
            kube_services: Api::namespaced(client.clone(), &namespace),

            plans: HashMap::new(),
            catalog: Catalog::default(),
            provisions: HashMap::new(),
            bindings: HashMap::new(),
        };

        provider.add_plan(Self::create_free_plan());
        let hello_wasi = provider.create_service(
            String::from(FEDRA_DB),
            String::from("Local key-value store in cluster"),
            FREE_PLAN,
        );
        provider.add_service(hello_wasi);

        provider
    }

    pub async fn provision_service(
        &mut self,
        service_id: &String,
        plan_id: &String,
        instance_id: &String,
    ) -> Result<()> {
        log::info!(
            "Provisioning service {} with plan_id {} and instance_id {}",
            service_id,
            plan_id,
            instance_id
        );

        match service_id.as_str() {
            FEDRA_DB => {
                let pod_key = self.provision_fedra_db(instance_id, plan_id).await?;
                let provision = Provision {
                    instance_id: instance_id.clone(),
                    service_id: service_id.clone(),
                    plan_id: plan_id.clone(),
                    pod_key,
                };

                self.provisions
                    .insert(provision.instance_id.clone(), provision);

                Ok(())
            }
            _ => Err(anyhow!("Unknown service_id {} to provision", service_id)),
        }
    }

    pub async fn deprovision_service(&mut self, instance_id: &String) -> Result<()> {
        log::info!("Deprovisioning service with instance_id {}", instance_id);

        let provision = self
            .provisions
            .get(instance_id)
            .ok_or_else(|| anyhow!("Instance id {} not found", instance_id))?;

        let params = DeleteParams::default();
        self.pods
            .delete(provision.pod_key.name().as_str(), &params)
            .await?;

        self.provisions.remove(instance_id);

        Ok(())
    }

    pub async fn create_service_binding(
        &mut self,
        service_id: &String,
        instance_id: &String,
        binding_id: &String,
    ) -> Result<Endpoint> {
        log::info!(
            "Creating binding for service {} with instance_id {} and binding_id {}",
            service_id,
            instance_id,
            binding_id
        );

        match service_id.as_str() {
            FEDRA_DB => {
                let endpoint = self.bind_fedra_db_instance(instance_id).await?;
                self.bindings.insert(binding_id.clone(), endpoint.clone());

                Ok(endpoint)
            }
            _ => Err(anyhow!(
                "Unknown service_id {} to provide a binding for",
                service_id
            )),
        }
    }

    pub async fn delete_service_binding(&mut self, binding_id: &String) -> Result<()> {
        log::info!("Deleting the binding with id {}", binding_id);

        let endpoint = self
            .bindings
            .get(binding_id)
            .ok_or_else(|| anyhow!("Binding id {} not found", binding_id))?;

        let params = DeleteParams::default();
        self.kube_services
            .delete(endpoint.host.as_str(), &params)
            .await?;

        self.bindings.remove(binding_id);

        Ok(())
    }

    pub fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    async fn provision_fedra_db(&self, instance_id: &String, _plan_id: &String) -> Result<PodKey> {
        let hash = Self::create_hash(&FEDRA_DB.to_string(), instance_id);
        let hash = format!("{:x}", hash);
        let manifest: Pod = serde_json::from_value(json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": format!("{}-{}", FEDRA_DB, hash),
                "labels": {
                    "app": format!("{}-app-{}", FEDRA_DB, hash)
                }
            },
            "spec": {
                "containers": [
                    {
                        "name": "fedra-db",
                        "image": "eu.gcr.io/fedra-301320/fedra-db:arm64v8",
                        "imagePullPolicy": "Always"
                    }
                ]
            }
        }))
        .map_err(|e| anyhow!("Invalid Pod JSON: {}", e))?;
        let params = PostParams::default();

        let pod = self.pods.create(&params, &manifest).await?;
        let name = Meta::name(&pod);
        log::info!("Created pod {:#?}", name);

        Ok(PodKey::from(pod))
    }

    async fn bind_fedra_db_instance(&self, instance_id: &String) -> Result<Endpoint> {
        let hash = Self::create_hash(&FEDRA_DB.to_string(), instance_id);
        let hash = format!("{:x}", hash);
        let manifest: KubeService = serde_json::from_value(json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": format!("{}-service-{}", FEDRA_DB, hash)
            },
            "spec": {
                "selector": {
                    "app": format!("{}-app-{}", FEDRA_DB, hash)
                },
                "ports": [
                    {
                        "protocol": "TCP",
                        "port": 80,
                        "targetPort": 8084
                    }
                ]
            }
        }))
        .map_err(|e| anyhow!("Invalid Service JSON: {}", e))?;

        let params = PostParams::default();

        let service = self.kube_services.create(&params, &manifest).await?;
        let name = Meta::name(&service);
        log::info!("Created service {:#?}", name);

        let endpoint = Endpoint {
            host: name,
            ports: vec![80.to_string()],
            protocol: None,
        };

        Ok(endpoint)
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
        free_plan.id = FREE_PLAN.to_string();
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

    fn create_hash(name: &String, id: &String) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut state = DefaultHasher::new();
        format!("{}-{}", name, id).hash(&mut state);

        state.finish()
    }
}
