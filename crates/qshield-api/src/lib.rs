//! Q-SHIELD REST API Engine (Axum + Tokio).

pub mod routes;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use routes::*;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/images", post(register_image_handler))
        .route("/api/v1/transfers", post(create_transfer_handler))
        .route("/api/v1/transfers/{id}", get(get_transfer_handler))
        .route("/api/v1/transfers/{id}/fragments", get(get_transfer_fragments_handler))
        .route("/api/v1/transfers/{id}/verification", get(get_transfer_verification_handler))
        .route("/api/v1/simulation/run", post(run_simulation_handler))
        .route("/api/v1/simulation/results", get(get_simulation_results_handler))
        .route("/api/v1/contacts", get(get_contacts_handler))
        .route("/api/v1/experiments", get(get_experiments_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
