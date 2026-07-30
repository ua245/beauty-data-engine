mod handlers;
mod sample_data;

use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .merge(handlers::product_routes())
        .merge(handlers::ingredient_routes())
        .merge(handlers::cert_routes())
        .merge(handlers::meta_routes())
}
