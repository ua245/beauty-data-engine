use super::registry::CertificationRegistry;
use super::{CertSearchQuery, CertSearchResult};
use crate::models::Certification;

pub fn verify_product(registry: &CertificationRegistry, query: &CertSearchQuery) -> CertSearchResult {
    let bodies = query.bodies.clone();
    let mut certifications = registry.search_all_bodies(&query.brand, &query.product_name);

    if let Some(filter_bodies) = bodies {
        certifications.retain(|c| filter_bodies.contains(&c.body));
    }

    certifications.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let best_match = certifications.first().cloned();
    let is_certified = best_match.is_some();

    CertSearchResult {
        certifications,
        best_match,
        is_certified,
    }
}

pub async fn verify_via_halalcheck_api(
    _brand: &str,
    _product_name: &str,
    _api_key: &str,
) -> Result<Vec<Certification>, String> {
    // Production: call HalalCheck API (RapidAPI) for BPJPH/LPPOM MUI certs
    // GET https://halalcheck-api.p.rapidapi.com/v1/certificates/search
    Err("HalalCheck API integration pending — configure RAPIDAPI_KEY".into())
}
