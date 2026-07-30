use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Retailer {
    Boots,
    Superdrug,
    LookFantastic,
    CultBeauty,
    SpaceNk,
}

impl FromStr for Retailer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "boots" => Ok(Self::Boots),
            "superdrug" => Ok(Self::Superdrug),
            "look_fantastic" => Ok(Self::LookFantastic),
            "cult_beauty" => Ok(Self::CultBeauty),
            "space_nk" => Ok(Self::SpaceNk),
            other => Err(format!("unknown retailer: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Makeup,
    Skincare,
    Fragrance,
    Haircare,
    BathBody,
    MensGrooming,
    Nails,
    ToolsAccessories,
}

impl FromStr for ProductType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "makeup" => Ok(Self::Makeup),
            "skincare" => Ok(Self::Skincare),
            "fragrance" => Ok(Self::Fragrance),
            "haircare" => Ok(Self::Haircare),
            "bath_body" => Ok(Self::BathBody),
            "mens_grooming" => Ok(Self::MensGrooming),
            "nails" => Ok(Self::Nails),
            "tools_accessories" => Ok(Self::ToolsAccessories),
            other => Err(format!("unknown product type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HalalStatus {
    Halal,
    LikelyHalal,
    Mashbooh,
    LikelyHaram,
    Haram,
}

impl FromStr for HalalStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "halal" => Ok(Self::Halal),
            "likely_halal" => Ok(Self::LikelyHalal),
            "mashbooh" => Ok(Self::Mashbooh),
            "likely_haram" => Ok(Self::LikelyHaram),
            "haram" => Ok(Self::Haram),
            other => Err(format!("unknown halal status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CertBody {
    Ifanca,
    Afic,
    LppomMui,
    Hce,
}

impl FromStr for CertBody {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "IFANCA" => Ok(Self::Ifanca),
            "AFIC" => Ok(Self::Afic),
            "LPPOM_MUI" => Ok(Self::LppomMui),
            "HCE" => Ok(Self::Hce),
            other => Err(format!("unknown cert body: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosingIngredient {
    pub inci_name: String,
    pub slug: Option<String>,
    pub cas_number: Option<String>,
    pub ec_number: Option<String>,
    pub functions: Vec<String>,
    pub restriction: Option<String>,
    pub halal_classification: HalalClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HalalClassification {
    Halal,
    Mashbooh,
    Haram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalalFlag {
    pub ingredient: String,
    pub classification: HalalClassification,
    pub reason: String,
    pub penalty: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub body: CertBody,
    pub cert_number: String,
    pub brand: String,
    pub product_names: Vec<String>,
    pub issue_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub status: CertStatus,
    pub inspection_body: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub retailer: Retailer,
    pub retailer_sku: String,
    pub name: String,
    pub brand: String,
    pub image_url: String,
    pub category_path: String,
    pub product_type: ProductType,
    pub sub_type: String,
    pub ingredients_raw: Option<String>,
    pub ingredients: Vec<CosingIngredient>,
    pub halal_score: u8,
    pub halal_status: HalalStatus,
    pub halal_flags: Vec<HalalFlag>,
    pub certifications: Vec<Certification>,
    pub price_gbp: Option<f64>,
    pub url: String,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFilters {
    pub query: Option<String>,
    #[serde(default, deserialize_with = "deserialize_comma_vec")]
    pub product_type: Vec<ProductType>,
    #[serde(default, deserialize_with = "deserialize_comma_vec_str")]
    pub sub_type: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_comma_vec")]
    pub retailer: Vec<Retailer>,
    #[serde(default, deserialize_with = "deserialize_comma_vec_str")]
    pub brand: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_comma_vec")]
    pub halal_status: Vec<HalalStatus>,
    pub certified_only: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_comma_vec")]
    pub cert_body: Vec<CertBody>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

fn deserialize_comma_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(vec![]),
        Some(s) if s.is_empty() => Ok(vec![]),
        Some(s) => s
            .split(',')
            .map(|part| T::from_str(part.trim()).map_err(serde::de::Error::custom))
            .collect(),
    }
}

fn deserialize_comma_vec_str<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(match s {
        None => vec![],
        Some(s) if s.is_empty() => vec![],
        Some(s) => s.split(',').map(|p| p.trim().to_string()).collect(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub products: Vec<Product>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientCheckResponse {
    pub inci_name: String,
    pub cosing: Option<CosingIngredient>,
    pub classification: HalalClassification,
    pub reason: String,
}
