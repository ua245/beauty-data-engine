use super::sample_certifications;
use crate::models::{CertBody, Certification, CertStatus};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;

pub struct CertificationRegistry {
    certs: Vec<Certification>,
    brand_index: HashMap<String, Vec<usize>>,
}

impl Default for CertificationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificationRegistry {
    pub fn new() -> Self {
        let certs = sample_certifications();
        let mut brand_index: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, cert) in certs.iter().enumerate() {
            let key = cert.brand.to_lowercase();
            brand_index.entry(key).or_default().push(idx);
        }

        Self { certs, brand_index }
    }

    pub fn search_by_brand(&self, brand: &str) -> Vec<Certification> {
        let key = brand.to_lowercase();
        self.brand_index
            .get(&key)
            .map(|indices| indices.iter().map(|&i| self.certs[i].clone()).collect())
            .unwrap_or_default()
    }

    pub fn search_by_product(&self, brand: &str, product_name: &str) -> Vec<Certification> {
        let matcher = SkimMatcherV2::default();
        let brand_certs = self.search_by_brand(brand);

        brand_certs
            .into_iter()
            .filter(|cert| {
                cert.product_names.iter().any(|name| {
                    matcher
                        .fuzzy_match(&name.to_lowercase(), &product_name.to_lowercase())
                        .is_some()
                })
            })
            .collect()
    }

    pub fn search_all_bodies(&self, brand: &str, product_name: &str) -> Vec<Certification> {
        let matcher = SkimMatcherV2::default();
        let brand_lower = brand.to_lowercase();
        let product_lower = product_name.to_lowercase();

        self.certs
            .iter()
            .filter(|cert| {
                let brand_match = matcher
                    .fuzzy_match(&cert.brand.to_lowercase(), &brand_lower)
                    .is_some();

                let product_match = cert.product_names.iter().any(|name| {
                    matcher
                        .fuzzy_match(&name.to_lowercase(), &product_lower)
                        .is_some()
                });

                brand_match || product_match
            })
            .cloned()
            .collect()
    }

    pub fn filter_by_body(&self, body: CertBody) -> Vec<Certification> {
        self.certs
            .iter()
            .filter(|c| c.body == body && matches!(c.status, CertStatus::Active))
            .cloned()
            .collect()
    }

    pub fn body_label(body: &CertBody) -> &'static str {
        match body {
            CertBody::Ifanca => "IFANCA",
            CertBody::Afic => "AFIC",
            CertBody::LppomMui => "LPPOM MUI",
            CertBody::Hce => "HCE",
        }
    }

    pub fn all(&self) -> &[Certification] {
        &self.certs
    }
}
