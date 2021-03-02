use actix_web::{web, HttpResponse, Responder};
use brokerapi::service::{
    CreatedServiceIstance, ProvisionParams, ServiceDeleteParams, ServiceInstanceRequestBody,
};

use crate::state::State;

pub async fn get_catalog(state: web::Data<State>) -> impl Responder {
    let provider = state.provider.lock().unwrap();
    HttpResponse::Ok().json(provider.get_catalog())
}

pub async fn put_service_instance(
    web::Path(instance_id): web::Path<String>,
    state: web::Data<State>,
    web::Query(_params): web::Query<ProvisionParams>,
    web::Json(body): web::Json<ServiceInstanceRequestBody>,
) -> impl Responder {
    // log::info!("params {:?}, body:\n{:#?}", params, body);

    let mut provider = state.provider.lock().unwrap();
    let ServiceInstanceRequestBody {
        service_id,
        plan_id,
        ..
    } = body;
    let response = provider
        .provision_service(&service_id, &plan_id, &instance_id)
        .await;

    match response {
        Ok(()) => HttpResponse::Created().json(CreatedServiceIstance::default()),
        Err(e) => {
            log::error!("Error provisioning the service instance {}", e);

            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_service_instance(
    web::Path(instance_id): web::Path<String>,
    state: web::Data<State>,
    web::Query(_params): web::Query<ServiceDeleteParams>,
) -> impl Responder {
    // log::info!("params {:?}", params);

    let mut provider = state.provider.lock().unwrap();

    match provider.deprovision_service(&instance_id).await {
        Ok(()) => HttpResponse::Ok().json({}),
        Err(e) => {
            log::error!("Error deprovisioning the service instance {}", e);

            // Return as if the instances is always gone in case of error, otherwise it cannot be deleted
            HttpResponse::Gone().finish()
        }
    }
}
