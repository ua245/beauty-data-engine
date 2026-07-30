use crate::models::{CosingIngredient, HalalClassification};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

const COSING_API_BASE: &str = "https://cosingchecker.com/api/v1";

#[derive(Error, Debug)]
pub enum CosingError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Ingredient not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Deserialize)]
struct CosingListResponse {
    results: Vec<CosingEntry>,
}

#[derive(Debug, Deserialize)]
struct CosingEntry {
    name: String,
    slug: Option<String>,
    cas_number: Option<String>,
    ec_number: Option<String>,
    functions: Option<Vec<String>>,
}

pub struct CosingClient {
    http: Client,
}

impl Default for CosingClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CosingClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .user_agent("HalalBeautyEngine/0.1")
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub async fn lookup(&self, inci_name: &str) -> Result<CosingIngredient, CosingError> {
        let url = format!("{COSING_API_BASE}/ingredients/?q={}", urlencoding(inci_name));
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(CosingError::NotFound(inci_name.to_string()));
        }

        let data: CosingListResponse = response.json().await?;
        let entry = data
            .results
            .into_iter()
            .next()
            .ok_or_else(|| CosingError::NotFound(inci_name.to_string()))?;

        Ok(CosingIngredient {
            inci_name: entry.name,
            slug: entry.slug,
            cas_number: entry.cas_number,
            ec_number: entry.ec_number,
            functions: entry.functions.unwrap_or_default(),
            restriction: None,
            halal_classification: HalalClassification::Halal,
        })
    }

    pub async fn resolve_inci_list(&self, raw: &str) -> Vec<CosingIngredient> {
        let names: Vec<&str> = raw
            .split([',', ';'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut resolved = Vec::new();
        for name in names {
            match self.lookup(name).await {
                Ok(ingredient) => resolved.push(ingredient),
                Err(_) => resolved.push(CosingIngredient {
                    inci_name: name.to_string(),
                    slug: None,
                    cas_number: None,
                    ec_number: None,
                    functions: vec![],
                    restriction: None,
                    halal_classification: HalalClassification::Halal,
                }),
            }
        }
        resolved
    }
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
        .replace('&', "%26")
        .replace('#', "%23")
}
