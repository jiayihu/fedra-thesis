use std::collections::HashMap;

use crate::state::State;
use actix_web::{web, HttpResponse, Responder};
use brokerapi::binding::{Binding, BindingDeleteParams, BindingRequestBody, ProvisionParams};

pub async fn put_binding(
    web::Path((instance_id, binding_id)): web::Path<(String, String)>,
    state: web::Data<State>,
    web::Query(_params): web::Query<ProvisionParams>,
    web::Json(body): web::Json<BindingRequestBody>,
) -> impl Responder {
    // log::info!("params {:?}, body:\n{:#?}", params, body);

    let BindingRequestBody { service_id, .. } = body;
    let mut provider = state.provider.lock().unwrap();

    match provider
        .create_service_binding(&service_id, &instance_id, &binding_id)
        .await
    {
        Ok(endpoint) => {
            let mut binding = Binding::default();
            let mut credentials = HashMap::new();
            credentials.insert("host".to_string(), endpoint.host.clone());
            credentials.insert("port".to_string(), endpoint.ports[0].clone());
            binding.credentials = Some(credentials);
            binding.endpoints = Some(vec![endpoint]);

            HttpResponse::Created().json(binding)
        }
        Err(e) => {
            log::error!("Error creating the binding for the service instance: {}", e);

            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_binding(
    web::Path((_instance_id, binding_id)): web::Path<(String, String)>,
    state: web::Data<State>,
    web::Query(_params): web::Query<BindingDeleteParams>,
) -> impl Responder {
    // log::info!("params {:?}", params);

    let mut provider = state.provider.lock().unwrap();

    match provider.delete_service_binding(&binding_id).await {
        Ok(()) => HttpResponse::Ok().json({}),
        Err(e) => {
            log::error!("Error deleting the binding: {}", e);

            // Return as if the binding is always gone in case of error, otherwise it cannot be deleted
            HttpResponse::Gone().finish()
        }
    }
}
