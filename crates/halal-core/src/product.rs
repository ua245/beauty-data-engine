//! The product record that the scraper produces and the dashboard consumes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::certification::CertificateClaim;
use crate::inci::{parse_ingredient_list, IngredientList};
use crate::lexicon::EvidenceKind;
use crate::taxonomy::TaxonomyAssignment;

/// Price in minor units, kept as an integer to avoid float drift on money.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub minor_units: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Gbp,
    Eur,
    Usd,
}

impl Money {
    pub fn gbp(minor_units: i64) -> Self {
        Self {
            minor_units,
            currency: Currency::Gbp,
        }
    }

    pub fn as_major(&self) -> f64 {
        self.minor_units as f64 / 100.0
    }
}

/// Provenance of a product image, which matters because retailer photography is licensed for
/// affiliate promotion at best and is not ours to redistribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageRights {
    /// Supplied through an affiliate feed, usable for promotion of that retailer's offer.
    AffiliateFeed,
    /// Open Beauty Facts photography, CC BY-SA, usable with attribution and share-alike.
    OpenBeautyFacts,
    /// Provided directly by the brand for this use.
    BrandSupplied,
    /// Hot-linked from a retailer page. Not redistributable; recorded so it can be excluded from
    /// any published build.
    RetailerHotlink,
    /// A generated placeholder, used for sample data.
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    pub alt: String,
    pub rights: ImageRights,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

/// A claim printed on the pack or made in the retailer's copy. These are inputs to the engine,
/// not conclusions: "vegan" excludes animal routes, which resolves a whole class of doubt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LabelClaim {
    Vegan,
    VeganCertified,
    CrueltyFree,
    AlcoholFree,
    HalalCertified,
    WaterPermeable,
    PlantBased,
    FragranceFree,
}

impl LabelClaim {
    /// Evidence this claim supplies to the ingredient engine.
    pub fn evidence(self) -> Option<EvidenceKind> {
        match self {
            LabelClaim::VeganCertified => Some(EvidenceKind::VeganCertification),
            // An unverified "vegan" claim is weaker than a certification but still excludes
            // animal inputs under UK consumer-protection rules, so it counts as a plant-origin
            // declaration rather than a certification.
            LabelClaim::Vegan | LabelClaim::PlantBased => {
                Some(EvidenceKind::PlantSourceDeclaration)
            }
            LabelClaim::HalalCertified => Some(EvidenceKind::HalalCertificate),
            LabelClaim::CrueltyFree
            | LabelClaim::AlcoholFree
            | LabelClaim::WaterPermeable
            | LabelClaim::FragranceFree => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LabelClaim::Vegan => "Vegan",
            LabelClaim::VeganCertified => "Certified vegan",
            LabelClaim::CrueltyFree => "Cruelty free",
            LabelClaim::AlcoholFree => "Alcohol free",
            LabelClaim::HalalCertified => "Halal certified",
            LabelClaim::WaterPermeable => "Water permeable",
            LabelClaim::PlantBased => "Plant based",
            LabelClaim::FragranceFree => "Fragrance free",
        }
    }
}

/// A statement from a brand or supplier about the origin of one specific ingredient.
///
/// This is the mechanism that lets a product move from "doubtful" to "verified" without a full
/// certification audit: the brand says, on the record and against a hashed document, that its
/// glycerin is palm-derived. It is weaker than a certificate and is presented as such, but it is
/// the missing middle rung between a bare ingredient list and a certified product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceAttestation {
    /// Lexicon rule this attestation answers, e.g. `glycerin`.
    pub rule_id: String,
    pub evidence: EvidenceKind,
    /// What the brand or supplier actually said.
    pub statement: String,
    pub source_url: Option<String>,
    /// SHA-256 of the supporting document, if one was captured.
    pub document_sha256: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retailer {
    pub id: String,
    pub name: String,
    pub domain: String,
}

/// How the record was obtained, which governs what we may republish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcquisitionMethod {
    /// Affiliate network product feed (Awin, Rakuten, Partnerize).
    AffiliateFeed,
    /// Open Beauty Facts dump or API.
    OpenBeautyFacts,
    /// Parsed from a retailer product page.
    PageParse,
    /// Entered by hand, including the bundled sample set.
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// Stable id in the form `retailer:sku`.
    pub id: String,
    pub name: String,
    pub brand: String,
    /// Legal manufacturer where known, used to reconcile certificate holders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    pub retailer: Retailer,
    pub retailer_sku: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gtin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shade: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Money>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<ProductImage>,
    pub taxonomy: TaxonomyAssignment,
    /// Breadcrumb trail as the retailer published it, kept for auditing the classifier.
    #[serde(default)]
    pub retailer_breadcrumbs: Vec<String>,
    /// Ingredient declaration exactly as scraped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingredients_raw: Option<String>,
    #[serde(default)]
    pub claims: Vec<LabelClaim>,
    #[serde(default)]
    pub certificate_claims: Vec<CertificateClaim>,
    #[serde(default)]
    pub attestations: Vec<SourceAttestation>,
    pub acquisition: AcquisitionMethod,
    pub captured_at: DateTime<Utc>,
}

impl Product {
    /// Parse the raw declaration. Returns an empty list when there is no declaration, which the
    /// engine treats as an absence of data rather than as a clean bill of health.
    pub fn ingredient_list(&self) -> IngredientList {
        self.ingredients_raw
            .as_deref()
            .map(parse_ingredient_list)
            .unwrap_or_default()
    }

    pub fn has_ingredient_data(&self) -> bool {
        !self.ingredient_list().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_converts_to_major_units() {
        assert_eq!(Money::gbp(1299).as_major(), 12.99);
    }

    #[test]
    fn vegan_certification_supplies_stronger_evidence_than_a_bare_claim() {
        assert_eq!(
            LabelClaim::VeganCertified.evidence(),
            Some(EvidenceKind::VeganCertification)
        );
        assert_eq!(
            LabelClaim::Vegan.evidence(),
            Some(EvidenceKind::PlantSourceDeclaration)
        );
        assert_eq!(LabelClaim::CrueltyFree.evidence(), None);
    }
}
