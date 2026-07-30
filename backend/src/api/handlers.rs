use crate::api::sample_data::sample_products;
use crate::certification::{verify_product, CertSearchQuery, CertificationRegistry};
use crate::cosing::CosingClient;
use crate::halal::{classify_ingredient, HalalStandard};
use crate::models::{IngredientCheckResponse, Product, ProductFilters, ProductListResponse};
use crate::taxonomy;
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

struct AppState {
    products: Vec<Product>,
    cert_registry: CertificationRegistry,
    cosing: CosingClient,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static STATE: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    STATE.get_or_init(|| {
        Arc::new(RwLock::new(AppState {
            products: sample_products(),
            cert_registry: CertificationRegistry::new(),
            cosing: CosingClient::new(),
        }))
    })
}

pub fn product_routes() -> Router {
    Router::new()
        .route("/api/v1/products", get(list_products))
        .route("/api/v1/products/{id}", get(get_product))
}

pub fn ingredient_routes() -> Router {
    Router::new().route("/api/v1/ingredients/check", get(check_ingredient))
}

pub fn cert_routes() -> Router {
    Router::new()
        .route("/api/v1/certifications/search", get(search_certifications))
        .route("/api/v1/certifications/bodies", get(list_cert_bodies))
}

pub fn meta_routes() -> Router {
    Router::new()
        .route("/api/v1/taxonomy", get(get_taxonomy))
        .route("/api/v1/health", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "version": "0.1.0" }))
}

async fn get_taxonomy() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "taxonomy": taxonomy::boots_taxonomy(),
        "sub_types": taxonomy::all_sub_types(),
    }))
}

async fn list_products(Query(filters): Query<ProductFilters>) -> Json<ProductListResponse> {
    let state = state().read().await;
    let page = filters.page.unwrap_or(1);
    let per_page = filters.per_page.unwrap_or(20);

    let mut products: Vec<Product> = state.products.clone();

    if let Some(ref query) = filters.query {
        let q = query.to_lowercase();
        products.retain(|p| {
            p.name.to_lowercase().contains(&q) || p.brand.to_lowercase().contains(&q)
        });
    }

    if !filters.product_type.is_empty() {
        products.retain(|p| filters.product_type.contains(&p.product_type));
    }

    if !filters.sub_type.is_empty() {
        products.retain(|p| filters.sub_type.contains(&p.sub_type));
    }

    if !filters.retailer.is_empty() {
        products.retain(|p| filters.retailer.contains(&p.retailer));
    }

    if !filters.halal_status.is_empty() {
        products.retain(|p| filters.halal_status.contains(&p.halal_status));
    }

    if filters.certified_only == Some(true) {
        products.retain(|p| !p.certifications.is_empty());
    }

    if !filters.cert_body.is_empty() {
        products.retain(|p| p.certifications.iter().any(|c| filters.cert_body.contains(&c.body)));
    }

    if let Some(min) = filters.price_min {
        products.retain(|p| p.price_gbp.unwrap_or(0.0) >= min);
    }

    if let Some(max) = filters.price_max {
        products.retain(|p| p.price_gbp.unwrap_or(f64::MAX) <= max);
    }

    let total = products.len() as u64;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(products.len());
    let page_products = if start < products.len() {
        products[start..end].to_vec()
    } else {
        vec![]
    };

    Json(ProductListResponse {
        products: page_products,
        total,
        page,
        per_page,
    })
}

async fn get_product(Path(id): Path<uuid::Uuid>) -> Result<Json<Product>, axum::http::StatusCode> {
    let state = state().read().await;
    state
        .products
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

async fn check_ingredient(
    Query(params): Query<IngredientQuery>,
) -> Json<IngredientCheckResponse> {
    let inci = params.q.unwrap_or_default();
    let result = classify_ingredient(&inci, HalalStandard::Strict);

    let state = state().read().await;
    let cosing = state.cosing.lookup(&inci).await.ok();

    Json(IngredientCheckResponse {
        inci_name: inci,
        cosing,
        classification: result.classification,
        reason: result.reason,
    })
}

async fn search_certifications(
    Query(params): Query<CertQuery>,
) -> Json<serde_json::Value> {
    let state = state().read().await;
    let query = CertSearchQuery {
        brand: params.brand.unwrap_or_default(),
        product_name: params.product.unwrap_or_default(),
        bodies: None,
    };
    let result = verify_product(&state.cert_registry, &query);
    Json(serde_json::to_value(result).unwrap())
}

async fn list_cert_bodies() -> Json<serde_json::Value> {
    Json(serde_json::json!([
        { "id": "IFANCA", "name": "Islamic Food and Nutrition Council of America", "region": "US/Global" },
        { "id": "AFIC", "name": "Australian Federation of Islamic Councils", "region": "Australia" },
        { "id": "LPPOM_MUI", "name": "LPPOM Majelis Ulama Indonesia", "region": "Indonesia" },
        { "id": "HCE", "name": "Halal Certification Europe", "region": "EU" }
    ]))
}

#[derive(serde::Deserialize)]
struct IngredientQuery {
    q: Option<String>,
}

#[derive(serde::Deserialize)]
struct CertQuery {
    brand: Option<String>,
    product: Option<String>,
}
