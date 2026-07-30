//! Polite retailer page parsing. The sample dashboard uses bundled data; this module is the
//! starting point for live ingestion.

use anyhow::{Context, Result};
use halal_core::inci::{is_probable_declaration, parse_ingredient_list};
use halal_core::product::{AcquisitionMethod, Product, ProductImage, Retailer};
use halal_core::taxonomy::taxonomy;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::warn;

const USER_AGENT: &str = "HalalBeautyEngine/0.1 (+https://github.com/ua245/beauty-data-engine; research)";

/// Parsed fields from a retailer product page.
#[derive(Debug, Default)]
pub struct PageExtract {
    pub title: Option<String>,
    pub brand: Option<String>,
    pub image_url: Option<String>,
    pub price_text: Option<String>,
    pub ingredients_raw: Option<String>,
    pub breadcrumbs: Vec<String>,
    pub json_ld_product: Option<serde_json::Value>,
}

pub async fn fetch_page(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status} for {url}");
    }
    Ok(body)
}

/// Parse a product page HTML blob.
pub fn parse_product_page(html: &str, url: &str) -> PageExtract {
    let document = Html::parse_document(html);
    let mut out = PageExtract::default();

    if let Ok(sel) = Selector::parse("script[type='application/ld+json']") {
        for node in document.select(&sel) {
            let text = node.text().collect::<String>();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if v.get("@type")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t.contains("Product"))
                {
                    out.json_ld_product = Some(v);
                }
            }
        }
    }

    if let Some(ld) = &out.json_ld_product {
        out.title = ld
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        out.brand = ld
            .pointer("/brand/name")
            .or_else(|| ld.get("brand"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        out.image_url = ld
            .get("image")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    v.as_array()?.first()?.as_str().map(str::to_string)
                }
            });
        out.price_text = ld.pointer("/offers/price").and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.as_f64().map(|f| f.to_string()))
        });
    }

    // Beauty Bay pattern: ### Ingredients
    if let Ok(h) = Selector::parse("h3, h2, h4, dt, summary, button, [class*='ingredient']") {
        for el in document.select(&h) {
            let heading = el.text().collect::<String>().to_lowercase();
            if heading.contains("ingredient") {
                if let Some(sib) = el.next_sibling() {
                    let text: String = sib.text().collect();
                    if is_probable_declaration(&text) {
                        out.ingredients_raw = Some(text.trim().to_string());
                        break;
                    }
                }
            }
        }
    }

    // Space NK / accordion pattern
    if out.ingredients_raw.is_none() {
        if let Ok(sel) = Selector::parse("#product-ingredients, [id*='ingredient'], [data-testid*='ingredient']") {
            for el in document.select(&sel) {
                let text = el.text().collect::<String>();
                if is_probable_declaration(&text) {
                    out.ingredients_raw = Some(text.trim().to_string());
                    break;
                }
            }
        }
    }

    // Fallback: any block that looks like an INCI list
    if out.ingredients_raw.is_none() {
        if let Ok(p) = Selector::parse("p, li, div") {
            for el in document.select(&p) {
                let text = el.text().collect::<String>();
                if text.len() > 40 && is_probable_declaration(&text) {
                    out.ingredients_raw = Some(text.trim().to_string());
                    break;
                }
            }
        }
    }

    if let Ok(bc) = Selector::parse("nav[aria-label*='breadcrumb'] a, .breadcrumb a, [class*='breadcrumb'] a") {
        out.breadcrumbs = document
            .select(&bc)
            .map(|a| a.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if out.title.is_none() {
        if let Ok(h1) = Selector::parse("h1") {
            out.title = document
                .select(&h1)
                .next()
                .map(|h| h.text().collect::<String>().trim().to_string());
        }
    }

    if out.ingredients_raw.is_none() {
        warn!("no ingredient block found on {url}");
    } else if let Some(raw) = &out.ingredients_raw {
        let parsed = parse_ingredient_list(raw);
        if parsed.is_empty() {
            warn!("ingredient block on {url} did not parse");
        }
    }

    out
}

/// Map a page extract onto a partial product record.
pub fn product_from_extract(
    retailer: Retailer,
    sku: &str,
    url: &str,
    extract: &PageExtract,
) -> Product {
    let title = extract.title.clone().unwrap_or_else(|| "Unknown product".into());
    let brand = extract
        .brand
        .clone()
        .unwrap_or_else(|| "Unknown".into());
    let tx = taxonomy();
    let assignment = tx.classify(&extract.breadcrumbs, &title, &brand);

    Product {
        id: format!("{}:{sku}", retailer.id),
        name: title,
        brand,
        manufacturer: None,
        retailer,
        retailer_sku: sku.to_string(),
        url: url.to_string(),
        gtin: None,
        shade: None,
        size: None,
        price: None,
        image: extract.image_url.as_ref().map(|u| ProductImage {
            url: u.clone(),
            alt: "Product image".into(),
            rights: halal_core::product::ImageRights::RetailerHotlink,
            attribution: None,
        }),
        taxonomy: assignment,
        retailer_breadcrumbs: extract.breadcrumbs.clone(),
        ingredients_raw: extract.ingredients_raw.clone(),
        claims: vec![],
        certificate_claims: vec![],
        attestations: vec![],
        acquisition: AcquisitionMethod::PageParse,
        captured_at: chrono::Utc::now(),
    }
}
