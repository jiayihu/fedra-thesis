use actix_web::{web, HttpResponse, Responder};
use brokerapi::service::{CreatedServiceIstance, ProvisionParams, ServiceInstanceRequestBody};

use crate::error::Result;
use crate::pod::PodKey;
use crate::provider::Provision;
use crate::state::State;

pub async fn get_catalog(state: web::Data<State>) -> impl Responder {
    let provider = state.provider.lock().unwrap();
    HttpResponse::Ok().json(provider.get_catalog())
}

pub async fn put_service_instance(
    web::Path(instance_id): web::Path<String>,
    state: web::Data<State>,
    web::Query(params): web::Query<ProvisionParams>,
    web::Json(body): web::Json<ServiceInstanceRequestBody>,
) -> impl Responder {
    log::info!("params {:?}, body:\n{:#?}", params, body);

    let mut provider = state.provider.lock().unwrap();
    let ServiceInstanceRequestBody {
        service_id,
        plan_id,
        ..
    } = body;
    let response: Result<PodKey> = provider
        .provision_service(service_id.clone(), plan_id.clone())
        .await;

    match response {
        Ok(pod_key) => {
            let provision = Provision {
                instance_id,
                service_id: service_id.clone(),
                plan_id: plan_id.clone(),
                pod_key,
            };
            provider.register_provision(provision);

            HttpResponse::Created().json(CreatedServiceIstance::default())
        }
        Err(e) => {
            log::error!("Error provisioning the service instance {}", e);

            HttpResponse::InternalServerError().json({})
        }
    }
}
