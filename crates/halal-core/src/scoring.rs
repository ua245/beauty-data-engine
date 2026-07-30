//! The assessment engine: turns a product record into a halal likelihood, a confidence, an
//! ablution verdict and a per-ingredient breakdown.
//!
//! # The scoring model
//!
//! Likelihood is the estimated probability that every source-ambiguous ingredient in the product
//! took a permissible route. It is computed multiplicatively over *risk families* rather than over
//! individual ingredients:
//!
//! ```text
//! likelihood = Π over families ( 1 - max_risk_in_family )
//! ```
//!
//! Grouping by family matters. A moisturiser listing glycerin, cetearyl alcohol, glyceryl stearate
//! and stearic acid is not four independent gambles — it reflects one sourcing decision about
//! palm versus tallow feedstock, taken once by the formulator. Multiplying four independent
//! penalties would push an ordinary supermarket moisturiser below 60% for no defensible reason.
//! Taking the family's worst case once keeps the number meaningful.
//!
//! A categorically prohibited ingredient does not reduce the score, it ends it: the verdict
//! becomes [`Verdict::NotPermissible`] and the likelihood is zero, because no amount of
//! counter-evidence makes carmine permissible.
//!
//! Confidence is deliberately a separate axis. A product with no ingredient list is not "likely
//! halal", it is unknown, and the two must never be conflated in the UI.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::certification::{register, CertificateVerification, TrustRung};
use crate::inci::ParsedIngredient;
use crate::lexicon::{
    lexicon, Ambiguity, AlcoholKind, EvidenceKind, HalalStatus, IngredientRule, OriginClass,
};
use crate::product::{LabelClaim, Product};
use crate::taxonomy::{taxonomy, Application, FilmRisk};

/// A named set of rulings, so a shopper can pick the standard they follow rather than being given
/// one opinion presented as fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Ethanol tolerance for leave-on and evaporative products, as a fraction.
    pub ethanol_leave_on_tolerated: bool,
    /// Whether doubtful ingredients are reported as failures rather than as needing verification.
    pub doubt_is_disqualifying: bool,
}

/// The profiles shipped with the engine.
pub fn profiles() -> Vec<RuleProfile> {
    vec![
        RuleProfile {
            id: "uk-general".to_string(),
            name: "UK mainstream".to_string(),
            description: "The default. Insect colourants are excluded, topical ethanol from a \
                          non-khamr source is treated as doubtful rather than prohibited, and \
                          wool and bee products are permitted."
                .to_string(),
            ethanol_leave_on_tolerated: true,
            doubt_is_disqualifying: false,
        }
        .named(),
        RuleProfile {
            id: "jakim-ms2634".to_string(),
            name: "JAKIM / MS 2634".to_string(),
            description: "Follows the Malaysian halal cosmetics standard, which permits \
                          non-intoxicating alcohol in cosmetics but requires documented halal \
                          sourcing for animal-derived material."
                .to_string(),
            ethanol_leave_on_tolerated: true,
            doubt_is_disqualifying: false,
        }
        .named(),
        RuleProfile {
            id: "bpjph-mui".to_string(),
            name: "BPJPH / MUI".to_string(),
            description: "Indonesian regulator position. Ethanol is acceptable when not derived \
                          from the intoxicating-liquor industry, and water-impermeable cosmetics \
                          are permitted but must be removed before ablution."
                .to_string(),
            ethanol_leave_on_tolerated: true,
            doubt_is_disqualifying: false,
        }
        .named(),
        RuleProfile {
            id: "strict".to_string(),
            name: "Strict".to_string(),
            description: "Avoids all ambiguity. Any ethanol, any insect derivative and any \
                          undocumented animal route is treated as disqualifying."
                .to_string(),
            ethanol_leave_on_tolerated: false,
            doubt_is_disqualifying: true,
        }
        .named(),
        RuleProfile {
            id: "lenient-istihalah".to_string(),
            name: "Istihalah-permissive".to_string(),
            description: "Accepts the transformation argument, so chemically transformed \
                          derivatives and weathered animal secretions are treated as permissible."
                .to_string(),
            ethanol_leave_on_tolerated: true,
            doubt_is_disqualifying: false,
        }
        .named(),
    ]
}

impl RuleProfile {
    fn named(self) -> Self {
        self
    }
}

/// Default profile id.
pub const DEFAULT_PROFILE: &str = "uk-general";

/// Headline verdict for a product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    /// A categorically prohibited ingredient is present.
    NotPermissible,
    /// Doubtful ingredients dominate and the animal route is more likely than not.
    LikelyNotPermissible,
    /// No ingredient data, so nothing can be said.
    Unknown,
    /// Doubtful ingredients present; a source declaration or certificate would settle it.
    NeedsVerification,
    /// No prohibited ingredients and only low-risk ambiguity.
    LikelyHalal,
    /// Backed by a certificate that passed the trust-ladder checks.
    CertifiedHalal,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::NotPermissible => "Not permissible",
            Verdict::LikelyNotPermissible => "Likely not permissible",
            Verdict::Unknown => "Insufficient data",
            Verdict::NeedsVerification => "Needs verification",
            Verdict::LikelyHalal => "Likely halal",
            Verdict::CertifiedHalal => "Certified halal",
        }
    }
}

/// Whether the product blocks water from reaching the skin or nail during ablution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WuduVerdict {
    /// No barrier expected.
    NoBarrier,
    /// A barrier is plausible from the format or a film-forming ingredient.
    PossibleBarrier,
    /// A continuous film is inherent; it must be removed before ablution.
    RemoveBeforeWudu,
    /// The brand states the film is water-permeable. Recorded as a claim, not as a finding,
    /// because permeability testing is not something an ingredient list can show.
    ClaimedPermeable,
    Unknown,
}

impl WuduVerdict {
    pub fn label(self) -> &'static str {
        match self {
            WuduVerdict::NoBarrier => "No barrier to ablution",
            WuduVerdict::PossibleBarrier => "May form a barrier",
            WuduVerdict::RemoveBeforeWudu => "Remove before wudu",
            WuduVerdict::ClaimedPermeable => "Water-permeable (brand claim)",
            WuduVerdict::Unknown => "Unknown",
        }
    }
}

/// Assessment of one ingredient in the context of one product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientAssessment {
    pub position: usize,
    pub raw: String,
    pub normalised: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_number: Option<String>,
    pub may_contain: bool,
    pub status: HalalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<OriginClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concern: Option<String>,
    /// Residual risk after evidence, in `0.0..=1.0`.
    pub residual_risk: f32,
    /// Evidence that resolved this ingredient, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<EvidenceKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_family: Option<String>,
    /// CosIng enrichment, filled in by the ingest pipeline when a match was found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cosing: Option<CosIngFacts>,
    pub wudu_barrier: bool,
}

/// The subset of CosIng fields the dashboard shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosIngFacts {
    pub inci_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cas_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ec_number: Option<String>,
    #[serde(default)]
    pub functions: Vec<String>,
    /// Annex reference such as `IV/115`, when the substance is regulated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annex_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restriction: Option<String>,
}

/// A note worth surfacing above the ingredient table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentNote {
    pub severity: NoteSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteSeverity {
    Info,
    Caution,
    Blocker,
}

/// The full assessment attached to a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalalAssessment {
    pub profile_id: String,
    pub verdict: Verdict,
    /// Probability, as a percentage, that every ambiguous ingredient took a permissible route.
    pub likelihood: u8,
    /// How much the likelihood can be relied on, as a percentage.
    pub confidence: u8,
    pub wudu: WuduVerdict,
    /// Strongest certificate rung reached.
    pub certification: TrustRung,
    pub certificate_verifications: Vec<CertificateVerification>,
    pub ingredients: Vec<IngredientAssessment>,
    pub notes: Vec<AssessmentNote>,
    /// Ingredients driving the verdict, most significant first.
    pub drivers: Vec<String>,
    pub lexicon_version: String,
    /// Count of ingredients with no lexicon and no CosIng match.
    pub unrecognised_count: usize,
}

impl HalalAssessment {
    pub fn is_shoppable(&self) -> bool {
        matches!(
            self.verdict,
            Verdict::CertifiedHalal | Verdict::LikelyHalal | Verdict::NeedsVerification
        )
    }
}

/// Enrichment supplied by the ingest pipeline: normalised INCI name to CosIng facts.
pub type CosIngLookup<'a> = &'a dyn Fn(&str, Option<&str>) -> Option<CosIngFacts>;

/// Assess a product under a named profile.
pub fn assess(
    product: &Product,
    profile_id: &str,
    as_of: NaiveDate,
    cosing: Option<CosIngLookup<'_>>,
) -> HalalAssessment {
    let profile = profiles()
        .into_iter()
        .find(|p| p.id == profile_id)
        .unwrap_or_else(|| {
            profiles()
                .into_iter()
                .find(|p| p.id == DEFAULT_PROFILE)
                .expect("default profile exists")
        });

    let lx = lexicon();
    let list = product.ingredient_list();
    let mut notes = Vec::new();

    // Certificates first: a passing certificate changes how ingredient doubt is reported.
    let verifications = register().verify_all(
        &product.certificate_claims,
        &product.brand,
        product.manufacturer.as_deref(),
        as_of,
    );
    let certification = verifications
        .first()
        .map(|v| v.rung)
        .unwrap_or(TrustRung::NoClaim);
    for v in &verifications {
        if v.rung.is_failure() {
            notes.push(AssessmentNote {
                severity: NoteSeverity::Caution,
                message: format!(
                    "{}: {}",
                    v.certifier_name.as_deref().unwrap_or(&v.certifier_id),
                    v.rung.label()
                ),
            });
        }
    }

    // Evidence available to resolve ambiguous ingredients.
    let mut evidence: Vec<EvidenceKind> = product
        .claims
        .iter()
        .filter_map(|c| c.evidence())
        .collect();
    if certification.is_certified() {
        evidence.push(EvidenceKind::HalalCertificate);
    }
    let attestation_by_rule: HashMap<&str, EvidenceKind> = product
        .attestations
        .iter()
        .map(|a| (a.rule_id.as_str(), a.evidence))
        .collect();

    let mut assessments: Vec<IngredientAssessment> = Vec::with_capacity(list.len());
    let mut unrecognised = 0usize;
    let mut has_ethanol = false;

    for ing in &list.ingredients {
        let hit = lx.lookup(&ing.normalised, ing.ci_number.as_deref());
        let cosing_facts =
            cosing.and_then(|f| f(&ing.normalised, ing.ci_number.as_deref()));

        match hit {
            Some(hit) => {
                let rule = hit.rule;
                let status = rule.status_for(&profile.id);
                if rule.alcohol_kind == Some(AlcoholKind::Ethanol) {
                    has_ethanol = true;
                }

                let resolved_by = resolve(rule, &evidence, &attestation_by_rule);
                let residual = residual_risk(rule, status, resolved_by.is_some(), ing);

                assessments.push(IngredientAssessment {
                    position: ing.position,
                    raw: ing.raw.clone(),
                    normalised: ing.normalised.clone(),
                    ci_number: ing.ci_number.clone(),
                    may_contain: ing.may_contain,
                    status: if resolved_by.is_some() && status == HalalStatus::Mashbooh {
                        HalalStatus::Halal
                    } else {
                        status
                    },
                    rule_id: Some(rule.id.clone()),
                    rule_label: Some(rule.label.clone()),
                    origin: Some(rule.origin),
                    concern: Some(rule.concern.clone()),
                    residual_risk: residual,
                    resolved_by,
                    risk_family: rule.risk_family.clone(),
                    cosing: cosing_facts,
                    wudu_barrier: rule.wudu_barrier,
                });
            }
            None => {
                // No lexicon rule. A CosIng match tells us the substance is a recognised
                // cosmetic ingredient, which is weak but real evidence that it is not an
                // undeclared animal derivative; without one we simply do not know.
                let recognised = cosing_facts.is_some();
                if !recognised {
                    unrecognised += 1;
                }
                assessments.push(IngredientAssessment {
                    position: ing.position,
                    raw: ing.raw.clone(),
                    normalised: ing.normalised.clone(),
                    ci_number: ing.ci_number.clone(),
                    may_contain: ing.may_contain,
                    status: if recognised {
                        HalalStatus::Halal
                    } else {
                        HalalStatus::Unknown
                    },
                    rule_id: None,
                    rule_label: None,
                    origin: None,
                    concern: None,
                    residual_risk: 0.0,
                    resolved_by: None,
                    risk_family: None,
                    cosing: cosing_facts,
                    wudu_barrier: false,
                });
            }
        }
    }

    // Worst case per risk family, then multiply across families.
    let mut family_risk: HashMap<String, f32> = HashMap::new();
    let mut blocked: Vec<&IngredientAssessment> = Vec::new();
    for a in &assessments {
        if a.status == HalalStatus::Haram {
            blocked.push(a);
            continue;
        }
        if a.residual_risk <= 0.0 {
            continue;
        }
        let key = a
            .risk_family
            .clone()
            .unwrap_or_else(|| format!("solo:{}", a.rule_id.clone().unwrap_or_default()));
        let entry = family_risk.entry(key).or_insert(0.0);
        if a.residual_risk > *entry {
            *entry = a.residual_risk;
        }
    }

    let likelihood_fraction: f32 = family_risk.values().map(|r| 1.0 - r).product();
    let mut likelihood = (likelihood_fraction * 100.0).round().clamp(0.0, 100.0) as u8;

    // Confidence: driven by data coverage, not by how good the news is.
    let mut confidence: f32 = if list.is_empty() {
        0.0
    } else {
        let recognised_share = if assessments.is_empty() {
            0.0
        } else {
            1.0 - (unrecognised as f32 / assessments.len() as f32)
        };
        0.35 + 0.5 * recognised_share
    };
    if list.has_undisclosed_fragrance {
        confidence -= 0.1;
        notes.push(AssessmentNote {
            severity: NoteSeverity::Caution,
            message: "The fragrance is declared as a single umbrella term, so ethanol carriers \
                      and animal-derived notes inside it are not visible on the label."
                .to_string(),
        });
    }
    if list.has_shade_variants {
        notes.push(AssessmentNote {
            severity: NoteSeverity::Info,
            message: "This declaration includes a shade-variant colourant block, so some listed \
                      pigments may not be present in the specific shade purchased."
                .to_string(),
        });
    }
    if certification.is_certified() {
        confidence += 0.15;
    }
    if !product.attestations.is_empty() {
        confidence += 0.05;
    }
    let confidence = (confidence.clamp(0.0, 1.0) * 100.0).round() as u8;

    // Ethanol handling is profile-dependent and depends on how the product is applied.
    let node = taxonomy().resolve(&product.taxonomy.product_type_id);
    let application = node.map(|n| n.product_type.application);
    if has_ethanol {
        match (application, profile.ethanol_leave_on_tolerated) {
            (_, false) => notes.push(AssessmentNote {
                severity: NoteSeverity::Blocker,
                message: "Contains ethanol, which the strict profile treats as disqualifying \
                          regardless of source."
                    .to_string(),
            }),
            (Some(Application::RinseOff), true) => notes.push(AssessmentNote {
                severity: NoteSeverity::Info,
                message: "Contains ethanol, but this is a rinse-off product, which the major \
                          standards treat more leniently."
                    .to_string(),
            }),
            (Some(Application::Evaporative), true) => notes.push(AssessmentNote {
                severity: NoteSeverity::Caution,
                message: "Alcohol-based fragrance. Most standards permit topical ethanol that \
                          does not come from the intoxicating-liquor industry, but the feedstock \
                          is not stated on the pack. Attar and oil-based alternatives avoid the \
                          question entirely."
                    .to_string(),
            }),
            (_, true) => notes.push(AssessmentNote {
                severity: NoteSeverity::Caution,
                message: "Contains ethanol in a leave-on product. Permitted by most standards \
                          when synthetically sourced, but the source is undeclared."
                    .to_string(),
            }),
        }
    }
    if product.claims.contains(&LabelClaim::AlcoholFree) && has_ethanol {
        notes.push(AssessmentNote {
            severity: NoteSeverity::Caution,
            message: "The pack claims to be alcohol free, but the declaration lists an ethanol \
                      ingredient. Worth flagging to the retailer."
                .to_string(),
        });
    }

    // Ablution axis, independent of halal status.
    let film_from_ingredients = assessments.iter().any(|a| a.wudu_barrier);
    let film_from_format = node.map(|n| n.product_type.film_risk);
    let wudu = if product.claims.contains(&LabelClaim::WaterPermeable) {
        WuduVerdict::ClaimedPermeable
    } else if list.is_empty() && film_from_format.is_none() {
        WuduVerdict::Unknown
    } else if film_from_ingredients || film_from_format == Some(FilmRisk::Inherent) {
        WuduVerdict::RemoveBeforeWudu
    } else if film_from_format == Some(FilmRisk::Possible) {
        WuduVerdict::PossibleBarrier
    } else {
        WuduVerdict::NoBarrier
    };
    if wudu == WuduVerdict::RemoveBeforeWudu {
        notes.push(AssessmentNote {
            severity: NoteSeverity::Caution,
            message: "Forms a continuous film. A halal certificate covers the ingredients, not \
                      whether water reaches the skin, so this still needs removing before wudu."
                .to_string(),
        });
    }
    if wudu == WuduVerdict::ClaimedPermeable {
        notes.push(AssessmentNote {
            severity: NoteSeverity::Info,
            message: "Water permeability here is a brand claim. UK certifiers including HMC \
                      state that they do not verify permeability, so it is not independently \
                      evidenced."
                .to_string(),
        });
    }

    // Verdict.
    let doubtful: Vec<&IngredientAssessment> = assessments
        .iter()
        .filter(|a| a.status == HalalStatus::Mashbooh)
        .collect();
    let max_doubt = doubtful
        .iter()
        .map(|a| a.residual_risk)
        .fold(0.0f32, f32::max);

    let verdict = if !blocked.is_empty() {
        likelihood = 0;
        Verdict::NotPermissible
    } else if list.is_empty() {
        Verdict::Unknown
    } else if certification.is_certified() {
        Verdict::CertifiedHalal
    } else if profile.doubt_is_disqualifying && !doubtful.is_empty() {
        Verdict::LikelyNotPermissible
    } else if max_doubt >= 0.5 {
        Verdict::LikelyNotPermissible
    } else if doubtful.is_empty() {
        Verdict::LikelyHalal
    } else if likelihood >= 85 {
        Verdict::LikelyHalal
    } else {
        Verdict::NeedsVerification
    };

    if !blocked.is_empty() {
        for a in &blocked {
            notes.push(AssessmentNote {
                severity: NoteSeverity::Blocker,
                message: format!(
                    "{} — {}",
                    a.rule_label.clone().unwrap_or_else(|| a.normalised.clone()),
                    a.concern.clone().unwrap_or_default()
                ),
            });
        }
    }

    let mut drivers: Vec<(&IngredientAssessment, f32)> = blocked
        .iter()
        .map(|a| (*a, 1.0f32))
        .chain(doubtful.iter().map(|a| (*a, a.residual_risk)))
        .collect();
    drivers.sort_by(|a, b| b.1.total_cmp(&a.1));
    let drivers = drivers
        .into_iter()
        .take(5)
        .map(|(a, _)| a.rule_label.clone().unwrap_or_else(|| a.normalised.clone()))
        .collect::<Vec<_>>();

    HalalAssessment {
        profile_id: profile.id,
        verdict,
        likelihood,
        confidence,
        wudu,
        certification,
        certificate_verifications: verifications,
        ingredients: assessments,
        notes,
        drivers,
        lexicon_version: lx.version().to_string(),
        unrecognised_count: unrecognised,
    }
}

/// Find evidence that resolves an ambiguous ingredient.
///
/// Only source ambiguity can be resolved by evidence. A ruling dispute is settled by the choice
/// of profile, so no supplier document changes it.
fn resolve(
    rule: &IngredientRule,
    evidence: &[EvidenceKind],
    attestations: &HashMap<&str, EvidenceKind>,
) -> Option<EvidenceKind> {
    if rule.ambiguity != Ambiguity::Source {
        return None;
    }
    if let Some(kind) = attestations.get(rule.id.as_str()) {
        if rule.resolvable_by.contains(kind) {
            return Some(*kind);
        }
    }
    evidence
        .iter()
        .find(|e| rule.resolvable_by.contains(e))
        .copied()
}

/// Residual probability that this ingredient took an impermissible route.
fn residual_risk(
    rule: &IngredientRule,
    status: HalalStatus,
    resolved: bool,
    ing: &ParsedIngredient,
) -> f32 {
    if status != HalalStatus::Mashbooh {
        return 0.0;
    }
    if resolved {
        return 0.0;
    }
    let base = match rule.ambiguity {
        // A ruling dispute is not a probability. It is reported as a fixed, moderate doubt so it
        // shows up as "needs verification" rather than silently sinking the score.
        Ambiguity::Ruling => 0.3,
        Ambiguity::Source => rule.animal_route_likelihood,
        Ambiguity::None => 0.0,
    };
    // A pigment listed only in a `may contain` block is not certainly present.
    if ing.may_contain {
        base * 0.5
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certification::{CertificateClaim, CertificateEvidence, Scope};
    use crate::product::{AcquisitionMethod, Retailer, SourceAttestation};
    use chrono::Utc;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, 30).expect("date")
    }

    fn product(name: &str, product_type: &str, ingredients: &str) -> Product {
        Product {
            id: format!("test:{name}"),
            name: name.to_string(),
            brand: "Test Brand".to_string(),
            manufacturer: None,
            retailer: Retailer {
                id: "test".to_string(),
                name: "Test".to_string(),
                domain: "example.com".to_string(),
            },
            retailer_sku: "1".to_string(),
            url: "https://example.com/p/1".to_string(),
            gtin: None,
            shade: None,
            size: None,
            price: None,
            image: None,
            taxonomy: taxonomy()
                .assignment_for(product_type, crate::taxonomy::ClassificationMethod::Manual, 1.0)
                .unwrap_or_else(|| panic!("unknown product type {product_type}")),
            retailer_breadcrumbs: vec![],
            ingredients_raw: Some(ingredients.to_string()),
            claims: vec![],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: Utc::now(),
        }
    }

    fn assess_default(p: &Product) -> HalalAssessment {
        assess(p, DEFAULT_PROFILE, today(), None)
    }

    #[test]
    fn a_mineral_only_formula_is_likely_halal_at_full_score() {
        let p = product(
            "Mineral Eyeshadow",
            "eyeshadow",
            "Mica, Talc, Silica, CI 77491, CI 77492, Boron Nitride",
        );
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::LikelyHalal);
        assert_eq!(a.likelihood, 100);
        assert_eq!(a.wudu, WuduVerdict::NoBarrier);
    }

    #[test]
    fn carmine_ends_the_assessment_regardless_of_everything_else() {
        let p = product(
            "Red Lipstick",
            "lipstick",
            "Mica, Talc, Carmine, Candelilla Wax",
        );
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::NotPermissible);
        assert_eq!(a.likelihood, 0);
        assert!(a.notes.iter().any(|n| n.severity == NoteSeverity::Blocker));
        assert!(a.drivers.iter().any(|d| d.contains("Carmine")));
    }

    #[test]
    fn a_may_contain_carmine_still_blocks_but_is_marked_conditional() {
        let p = product(
            "Blusher Palette",
            "blusher",
            "Talc, Mica, Silica, May Contain (+/-): CI 77491, CI 75470",
        );
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::NotPermissible);
        let carmine = a
            .ingredients
            .iter()
            .find(|i| i.rule_id.as_deref() == Some("carmine"))
            .expect("carmine");
        assert!(carmine.may_contain);
    }

    #[test]
    fn correlated_fatty_derivatives_are_counted_once_not_five_times() {
        let p = product(
            "Everyday Moisturiser",
            "face-moisturiser",
            "Aqua, Glycerin, Cetearyl Alcohol, Glyceryl Stearate, Stearic Acid, Sodium Stearate, \
             Phenoxyethanol",
        );
        let a = assess_default(&p);
        // All six flagged ingredients belong to the fatty-chain family, whose worst case is
        // stearic acid at 0.25, so the likelihood must sit near 75 rather than collapsing.
        assert!(
            a.likelihood >= 70 && a.likelihood <= 80,
            "likelihood was {}",
            a.likelihood
        );
        assert_eq!(a.verdict, Verdict::NeedsVerification);
    }

    #[test]
    fn a_vegan_claim_resolves_source_ambiguity() {
        let mut p = product(
            "Everyday Moisturiser",
            "face-moisturiser",
            "Aqua, Glycerin, Cetearyl Alcohol, Stearic Acid, Phenoxyethanol",
        );
        let before = assess_default(&p);
        p.claims.push(LabelClaim::VeganCertified);
        let after = assess_default(&p);
        assert!(after.likelihood > before.likelihood);
        assert_eq!(after.verdict, Verdict::LikelyHalal);
        assert_eq!(after.likelihood, 100);
        let glycerin = after
            .ingredients
            .iter()
            .find(|i| i.rule_id.as_deref() == Some("glycerin"))
            .expect("glycerin");
        assert_eq!(glycerin.status, HalalStatus::Halal);
        assert_eq!(
            glycerin.resolved_by,
            Some(EvidenceKind::VeganCertification)
        );
    }

    #[test]
    fn a_targeted_attestation_resolves_only_its_own_ingredient() {
        let mut p = product(
            "Serum",
            "face-serum",
            "Aqua, Glycerin, Stearic Acid, Sodium Hyaluronate",
        );
        p.attestations.push(SourceAttestation {
            rule_id: "glycerin".to_string(),
            evidence: EvidenceKind::PlantSourceDeclaration,
            statement: "Glycerin is RSPO-certified palm origin.".to_string(),
            source_url: Some("https://example.com/spec".to_string()),
            document_sha256: Some(CertificateEvidence::hash_document(b"spec")),
            recorded_at: Utc::now(),
        });
        let a = assess_default(&p);
        let glycerin = a
            .ingredients
            .iter()
            .find(|i| i.rule_id.as_deref() == Some("glycerin"))
            .expect("glycerin");
        assert_eq!(glycerin.status, HalalStatus::Halal);
        let stearic = a
            .ingredients
            .iter()
            .find(|i| i.rule_id.as_deref() == Some("stearic-acid"))
            .expect("stearic acid");
        assert_eq!(stearic.status, HalalStatus::Mashbooh);
    }

    #[test]
    fn a_verified_certificate_produces_a_certified_verdict() {
        let mut p = product(
            "Certified Foundation",
            "foundation",
            "Aqua, Glycerin, Stearic Acid, Mica, Titanium Dioxide",
        );
        p.brand = "Radiance".to_string();
        p.certificate_claims.push(CertificateClaim {
            certifier_id: "hce".to_string(),
            certificate_number: Some("21633-1/2/2/Y1".to_string()),
            holder: Some("Radiance Cosmetics Ltd".to_string()),
            scopes: vec![Scope::Cosmetics],
            issued_on: None,
            expires_on: Some(NaiveDate::from_ymd_opt(2027, 5, 31).expect("date")),
            found_in_directory: false,
            found_in_regulator_registry: false,
            evidence: Some(CertificateEvidence {
                url: Some("https://example.org/cert.pdf".to_string()),
                document_sha256: Some(CertificateEvidence::hash_document(b"cert")),
                method: "brand press kit".to_string(),
                captured_at: Utc::now(),
            }),
        });
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::CertifiedHalal);
        assert!(a.certification.is_certified());
        assert_eq!(a.likelihood, 100);
    }

    #[test]
    fn an_expired_certificate_does_not_certify_and_is_explained() {
        let mut p = product("Foundation", "foundation", "Aqua, Mica, Titanium Dioxide");
        p.brand = "Radiance".to_string();
        p.certificate_claims.push(CertificateClaim {
            certifier_id: "hce".to_string(),
            certificate_number: Some("X".to_string()),
            holder: Some("Radiance".to_string()),
            scopes: vec![Scope::Cosmetics],
            issued_on: None,
            expires_on: Some(NaiveDate::from_ymd_opt(2024, 1, 1).expect("date")),
            found_in_directory: false,
            found_in_regulator_registry: false,
            evidence: None,
        });
        let a = assess_default(&p);
        assert_eq!(a.certification, TrustRung::Expired);
        assert_ne!(a.verdict, Verdict::CertifiedHalal);
        assert!(a.notes.iter().any(|n| n.message.contains("expired")));
    }

    #[test]
    fn nail_polish_needs_removing_before_wudu_even_when_ingredients_are_fine() {
        let p = product(
            "Classic Nail Polish",
            "nail-polish",
            "Butyl Acetate, Ethyl Acetate, Nitrocellulose, Acetyl Tributyl Citrate, Mica",
        );
        let a = assess_default(&p);
        assert_eq!(a.wudu, WuduVerdict::RemoveBeforeWudu);
        assert_ne!(a.verdict, Verdict::NotPermissible);
    }

    #[test]
    fn a_permeability_claim_is_reported_as_a_claim_not_a_finding() {
        let mut p = product(
            "Breathable Polish",
            "breathable-nail-polish",
            "Aqua, Acrylates Copolymer, Mica",
        );
        p.claims.push(LabelClaim::WaterPermeable);
        let a = assess_default(&p);
        assert_eq!(a.wudu, WuduVerdict::ClaimedPermeable);
        assert!(a.notes.iter().any(|n| n.message.contains("brand claim")));
    }

    #[test]
    fn alcohol_based_perfume_is_flagged_but_not_condemned_by_default() {
        let p = product(
            "Rose Eau de Parfum",
            "eau-de-parfum",
            "Alcohol Denat., Parfum, Aqua, Linalool, Limonene",
        );
        let a = assess_default(&p);
        assert_ne!(a.verdict, Verdict::NotPermissible);
        assert!(a
            .notes
            .iter()
            .any(|n| n.message.contains("Alcohol-based fragrance")));
    }

    #[test]
    fn the_strict_profile_rejects_the_same_perfume() {
        let p = product(
            "Rose Eau de Parfum",
            "eau-de-parfum",
            "Alcohol Denat., Parfum, Aqua, Linalool",
        );
        let a = assess(&p, "strict", today(), None);
        assert_eq!(a.verdict, Verdict::NotPermissible);
    }

    #[test]
    fn an_attar_avoids_the_alcohol_question_entirely() {
        let p = product(
            "Oud Attar",
            "attar",
            "Aquilaria Malaccensis Wood Oil, Santalum Album Wood Oil, Rosa Damascena Flower Oil",
        );
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::LikelyHalal);
        assert!(!a
            .notes
            .iter()
            .any(|n| n.message.contains("ethanol")));
    }

    #[test]
    fn an_empty_declaration_is_unknown_rather_than_optimistic() {
        let mut p = product("Mystery Cream", "face-moisturiser", "");
        p.ingredients_raw = None;
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::Unknown);
        assert_eq!(a.confidence, 0);
    }

    #[test]
    fn unrecognised_ingredients_lower_confidence_not_likelihood() {
        let clean = product("A", "face-serum", "Glycerin, Niacinamide, Phenoxyethanol");
        let odd = product(
            "B",
            "face-serum",
            "Glycerin, Niacinamide, Phenoxyethanol, Xylopentane, Zorbital",
        );
        let a = assess_default(&clean);
        let b = assess_default(&odd);
        assert_eq!(a.likelihood, b.likelihood);
        assert!(b.confidence < a.confidence);
        assert_eq!(b.unrecognised_count, 2);
    }

    #[test]
    fn a_gelatin_mask_is_likely_not_permissible_without_evidence() {
        let p = product(
            "Peel Mask",
            "peel-off-mask",
            "Aqua, Gelatin, Glycerin, Phenoxyethanol",
        );
        let a = assess_default(&p);
        assert_eq!(a.verdict, Verdict::LikelyNotPermissible);
        assert!(a.likelihood < 20);
    }

    #[test]
    fn a_marine_declaration_rescues_the_same_gelatin() {
        let mut p = product(
            "Peel Mask",
            "peel-off-mask",
            "Aqua, Gelatin, Glycerin, Phenoxyethanol",
        );
        p.attestations.push(SourceAttestation {
            rule_id: "gelatin".to_string(),
            evidence: EvidenceKind::MarineSourceDeclaration,
            statement: "Fish gelatin from a certified supplier.".to_string(),
            source_url: None,
            document_sha256: None,
            recorded_at: Utc::now(),
        });
        let a = assess_default(&p);
        assert!(a.likelihood > 70, "likelihood was {}", a.likelihood);
    }

    #[test]
    fn a_ruling_dispute_is_not_resolvable_by_supplier_evidence() {
        let mut p = product("Snail Essence", "toner", "Aqua, Snail Secretion Filtrate, Betaine");
        p.claims.push(LabelClaim::VeganCertified);
        let a = assess_default(&p);
        let snail = a
            .ingredients
            .iter()
            .find(|i| i.rule_id.as_deref() == Some("snail-mucin"))
            .expect("snail");
        assert_eq!(snail.status, HalalStatus::Mashbooh);
        assert!(snail.resolved_by.is_none());
    }

    #[test]
    fn the_lenient_profile_permits_a_disputed_ingredient() {
        let p = product("Shellac Topcoat", "base-top-coat", "Ethyl Acetate, Shellac, Mica");
        let default = assess_default(&p);
        let lenient = assess(&p, "lenient-istihalah", today(), None);
        assert_eq!(default.verdict, Verdict::NeedsVerification);
        assert_eq!(lenient.verdict, Verdict::LikelyHalal);
    }

    #[test]
    fn every_profile_is_addressable_and_unknown_ids_fall_back() {
        for p in profiles() {
            let prod = product("X", "shampoo", "Aqua, Sodium Laureth Sulfate, Cocamidopropyl Betaine");
            let a = assess(&prod, &p.id, today(), None);
            assert_eq!(a.profile_id, p.id);
        }
        let prod = product("X", "shampoo", "Aqua, Sodium Laureth Sulfate");
        assert_eq!(
            assess(&prod, "no-such-profile", today(), None).profile_id,
            DEFAULT_PROFILE
        );
    }

    #[test]
    fn cosing_enrichment_raises_confidence_for_otherwise_unknown_names() {
        let p = product("X", "face-serum", "Aqua, Zzzz Unknownium Complex");
        let without = assess_default(&p);
        let lookup = |name: &str, _ci: Option<&str>| {
            Some(CosIngFacts {
                inci_name: name.to_string(),
                cas_number: Some("7732-18-5".to_string()),
                ec_number: None,
                functions: vec!["SOLVENT".to_string()],
                annex_reference: None,
                restriction: None,
            })
        };
        let with = assess(&p, DEFAULT_PROFILE, today(), Some(&lookup));
        assert!(with.confidence > without.confidence);
        assert_eq!(with.unrecognised_count, 0);
    }
}
