use chrono::{DateTime, Utc};
use halal_core::product::{Product, Retailer};
use halal_core::scoring::{HalalAssessment, RuleProfile};
use halal_core::taxonomy::Taxonomy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogProduct {
    pub product: Product,
    pub assessment: HalalAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub profile: String,
    pub products: Vec<CatalogProduct>,
    pub retailers: Vec<Retailer>,
    pub taxonomy: Taxonomy,
    pub profiles: Vec<RuleProfile>,
}
