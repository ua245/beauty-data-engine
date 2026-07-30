mod registry;
mod verifier;

pub use registry::CertificationRegistry;
pub use verifier::verify_product;

use crate::models::{CertBody, Certification, CertStatus};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertSearchQuery {
    pub brand: String,
    pub product_name: String,
    pub bodies: Option<Vec<CertBody>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertSearchResult {
    pub certifications: Vec<Certification>,
    pub best_match: Option<Certification>,
    pub is_certified: bool,
}

pub fn sample_certifications() -> Vec<Certification> {
    vec![
        Certification {
            body: CertBody::Ifanca,
            cert_number: "IFANCA-2024-12345".into(),
            brand: "Inika Organic".into(),
            product_names: vec![
                "Certified Organic Lipstick".into(),
                "Loose Mineral Foundation".into(),
            ],
            issue_date: Some(Utc::now() - Duration::days(365)),
            expiry_date: Some(Utc::now() + Duration::days(365)),
            status: CertStatus::Active,
            inspection_body: None,
            confidence: 0.95,
        },
        Certification {
            body: CertBody::LppomMui,
            cert_number: "ID12110049849780326".into(),
            brand: "Wardah".into(),
            product_names: vec![
                "Lightening Liquid Foundation".into(),
                "Perfect Bright Moisturizer".into(),
            ],
            issue_date: Some(Utc::now() - Duration::days(180)),
            expiry_date: Some(Utc::now() + Duration::days(545)),
            status: CertStatus::Active,
            inspection_body: Some("LPPOM MUI".into()),
            confidence: 0.98,
        },
        Certification {
            body: CertBody::Hce,
            cert_number: "HCE-COS-2023-789".into(),
            brand: "PHB Ethical Beauty".into(),
            product_names: vec![
                "Mineral Foundation".into(),
                "Lipstick".into(),
                "Natural Perfume".into(),
            ],
            issue_date: Some(Utc::now() - Duration::days(400)),
            expiry_date: Some(Utc::now() + Duration::days(330)),
            status: CertStatus::Active,
            inspection_body: Some("Halal Certification Europe".into()),
            confidence: 0.92,
        },
        Certification {
            body: CertBody::Afic,
            cert_number: "AFIC-AU-2024-5678".into(),
            brand: "MCoBeauty".into(),
            product_names: vec!["XtendLash Mascara".into(), "Lip Gloss".into()],
            issue_date: Some(Utc::now() - Duration::days(200)),
            expiry_date: Some(Utc::now() + Duration::days(530)),
            status: CertStatus::Active,
            inspection_body: Some("AFIC".into()),
            confidence: 0.88,
        },
    ]
}
