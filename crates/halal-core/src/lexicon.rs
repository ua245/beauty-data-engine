//! The halal ingredient lexicon: rules that map INCI names onto a halal status, an origin class
//! and — critically — the evidence that would resolve an ambiguous case.
//!
//! The design point that separates this from a keyword blocklist is that most flagged cosmetic
//! ingredients are not prohibited substances. They are *source-ambiguous*: stearic acid, glycerin
//! and cetyl alcohol are all routinely made from palm, from petrochemicals, or from animal fat,
//! and the INCI name is identical in every case. A rule therefore carries an
//! `animal_route_likelihood` (how much of the commercial supply is animal-derived) rather than a
//! verdict, and a list of evidence types that collapse the ambiguity.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Halal status of a single ingredient under a given rule profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HalalStatus {
    /// Permissible; no animal or intoxicant route of concern.
    Halal,
    /// Doubtful (mashbooh). The name does not determine the source.
    Mashbooh,
    /// Impermissible under the active profile.
    Haram,
    /// Not present in the lexicon and not resolvable from reference data.
    Unknown,
}

impl HalalStatus {
    pub fn label(self) -> &'static str {
        match self {
            HalalStatus::Halal => "Halal",
            HalalStatus::Mashbooh => "Doubtful",
            HalalStatus::Haram => "Not permissible",
            HalalStatus::Unknown => "Unknown",
        }
    }
}

/// Where the substance comes from in practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OriginClass {
    /// Made by chemical synthesis from non-animal feedstock.
    Synthetic,
    /// Mined or inorganic.
    Mineral,
    /// Botanical.
    Plant,
    /// Fermentation or microbial biosynthesis.
    Microbial,
    /// Land animal; status depends on species and slaughter.
    AnimalLand,
    /// Fish or other marine animal.
    AnimalMarine,
    /// Insect or insect secretion.
    AnimalInsect,
    /// Human-derived.
    AnimalHuman,
    /// Could be any of several routes; the INCI name does not say.
    Ambiguous,
    /// Ethanol and ethanol derivatives.
    Ethanol,
}

/// Nature of the alcohol, because fatty alcohols share the word but not the ruling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlcoholKind {
    /// Drinkable-type ethanol, including denatured and SD grades.
    Ethanol,
    /// Long-chain fatty alcohol used as an emollient; not an intoxicant.
    FattyAlcohol,
    /// Polyol such as glycerin or propylene glycol.
    Polyol,
}

/// Why an ingredient is doubtful, which determines how the doubt can be removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ambiguity {
    /// Not doubtful.
    None,
    /// The INCI name is shared by animal, plant and synthetic routes, so only supply-chain
    /// evidence settles it.
    Source,
    /// The source is known but scholars and certification schemes differ on the ruling. Choosing
    /// a rule profile settles it; no amount of supplier evidence will.
    Ruling,
}

impl Default for Ambiguity {
    fn default() -> Self {
        Ambiguity::None
    }
}

/// Evidence that can lift an ambiguous ingredient to `Halal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// Product carries a halal certificate from a body whose scope covers cosmetics.
    HalalCertificate,
    /// Product carries a credible vegan certification or claim, which excludes animal routes.
    VeganCertification,
    /// Brand or supplier states the botanical origin of this specific ingredient.
    PlantSourceDeclaration,
    /// Brand or supplier states a synthetic or petrochemical origin.
    SyntheticSourceDeclaration,
    /// Declared as fish or other marine origin, which most schemes accept.
    MarineSourceDeclaration,
    /// Declared as sourced from an animal slaughtered to halal requirements.
    HalalSlaughterDeclaration,
    /// Declared as fermentation-derived.
    FermentationDeclaration,
}

impl EvidenceKind {
    pub fn label(self) -> &'static str {
        match self {
            EvidenceKind::HalalCertificate => "halal certificate covering cosmetics",
            EvidenceKind::VeganCertification => "vegan certification",
            EvidenceKind::PlantSourceDeclaration => "plant-origin declaration",
            EvidenceKind::SyntheticSourceDeclaration => "synthetic-origin declaration",
            EvidenceKind::MarineSourceDeclaration => "marine-origin declaration",
            EvidenceKind::HalalSlaughterDeclaration => "halal slaughter declaration",
            EvidenceKind::FermentationDeclaration => "fermentation-origin declaration",
        }
    }
}

/// How a rule recognises an ingredient token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleMatch {
    /// Exact normalised INCI names.
    #[serde(default)]
    pub inci: Vec<String>,
    /// Colour Index numbers without the `CI` prefix.
    #[serde(default)]
    pub ci: Vec<String>,
    /// Matches when the normalised name contains all of the given fragments. Each entry is an
    /// AND-group, and the groups are OR-ed together.
    #[serde(default)]
    pub contains_all: Vec<Vec<String>>,
    /// Normalised names that must not match even if a fragment rule would otherwise fire.
    #[serde(default)]
    pub except: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientRule {
    pub id: String,
    /// Human-facing name shown in the product breakdown.
    pub label: String,
    #[serde(default)]
    pub matches: RuleMatch,
    pub origin: OriginClass,
    /// Status under the default profile.
    pub status: HalalStatus,
    #[serde(default)]
    pub ambiguity: Ambiguity,
    /// Share of commercial supply that plausibly follows an animal route, in `0.0..=1.0`.
    /// Used to weight doubt rather than to make a binary call.
    #[serde(default)]
    pub animal_route_likelihood: f32,
    /// Ingredients whose origin is decided by the same supply-chain choice share a family, so the
    /// engine counts the family's worst case once instead of compounding correlated doubts. A
    /// moisturiser listing five palm-or-tallow derivatives reflects one sourcing decision, not
    /// five independent gambles.
    #[serde(default)]
    pub risk_family: Option<String>,
    /// One or two sentences explaining the concern to a shopper.
    pub concern: String,
    /// Evidence types that resolve the ambiguity for this ingredient.
    #[serde(default)]
    pub resolvable_by: Vec<EvidenceKind>,
    /// Forms a water-impermeable film, relevant to ablution rather than to ingestion.
    #[serde(default)]
    pub wudu_barrier: bool,
    #[serde(default)]
    pub alcohol_kind: Option<AlcoholKind>,
    /// Per-profile status overrides keyed by profile id.
    #[serde(default)]
    pub profile_status: HashMap<String, HalalStatus>,
    /// Supporting references for the ruling.
    #[serde(default)]
    pub citations: Vec<String>,
}

impl IngredientRule {
    /// Status of this rule under a named profile, falling back to the default status.
    pub fn status_for(&self, profile_id: &str) -> HalalStatus {
        self.profile_status
            .get(profile_id)
            .copied()
            .unwrap_or(self.status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lexicon {
    pub version: String,
    pub rules: Vec<IngredientRule>,
}

/// Indexed lexicon supporting constant-time exact lookups and a linear fragment pass.
#[derive(Debug)]
pub struct LexiconIndex {
    lexicon: Lexicon,
    by_inci: HashMap<String, usize>,
    by_ci: HashMap<String, usize>,
    fragment_rules: Vec<usize>,
}

/// The outcome of matching one ingredient token against the lexicon.
#[derive(Debug, Clone, Copy)]
pub struct RuleHit<'a> {
    pub rule: &'a IngredientRule,
    /// True when the match came from an exact INCI or CI lookup rather than a fragment rule.
    pub exact: bool,
}

const LEXICON_JSON: &str = include_str!("../data/lexicon.json");

static LEXICON: Lazy<LexiconIndex> = Lazy::new(|| {
    let lexicon: Lexicon =
        serde_json::from_str(LEXICON_JSON).expect("bundled lexicon.json is valid");
    LexiconIndex::new(lexicon)
});

/// The lexicon bundled with the crate.
pub fn lexicon() -> &'static LexiconIndex {
    &LEXICON
}

impl LexiconIndex {
    pub fn new(lexicon: Lexicon) -> Self {
        let mut by_inci = HashMap::new();
        let mut by_ci = HashMap::new();
        let mut fragment_rules = Vec::new();
        for (idx, rule) in lexicon.rules.iter().enumerate() {
            for name in &rule.matches.inci {
                by_inci.insert(crate::inci::normalise_inci(name), idx);
            }
            for ci in &rule.matches.ci {
                by_ci.insert(ci.trim().to_string(), idx);
            }
            if !rule.matches.contains_all.is_empty() {
                fragment_rules.push(idx);
            }
        }
        Self {
            lexicon,
            by_inci,
            by_ci,
            fragment_rules,
        }
    }

    pub fn version(&self) -> &str {
        &self.lexicon.version
    }

    pub fn rules(&self) -> &[IngredientRule] {
        &self.lexicon.rules
    }

    pub fn rule(&self, id: &str) -> Option<&IngredientRule> {
        self.lexicon.rules.iter().find(|r| r.id == id)
    }

    /// Look up a normalised ingredient name and optional Colour Index number.
    ///
    /// Exact INCI matches win over CI matches, which win over fragment matches; among fragment
    /// matches the most specific rule (most fragments matched) wins, so that
    /// `HYDROLYZED SILK PROTEIN` prefers the silk rule over the generic hydrolysed-protein rule.
    pub fn lookup(&self, normalised: &str, ci_number: Option<&str>) -> Option<RuleHit<'_>> {
        if let Some(&idx) = self.by_inci.get(normalised) {
            return Some(RuleHit {
                rule: &self.lexicon.rules[idx],
                exact: true,
            });
        }
        if let Some(ci) = ci_number {
            if let Some(&idx) = self.by_ci.get(ci) {
                return Some(RuleHit {
                    rule: &self.lexicon.rules[idx],
                    exact: true,
                });
            }
        }

        let mut best: Option<(usize, usize)> = None;
        for &idx in &self.fragment_rules {
            let rule = &self.lexicon.rules[idx];
            if rule
                .matches
                .except
                .iter()
                .any(|e| normalised == crate::inci::normalise_inci(e))
            {
                continue;
            }
            for group in &rule.matches.contains_all {
                if group
                    .iter()
                    .all(|fragment| normalised.contains(fragment.as_str()))
                {
                    let specificity: usize = group.iter().map(|f| f.len()).sum();
                    if best.map_or(true, |(_, s)| specificity > s) {
                        best = Some((idx, specificity));
                    }
                }
            }
        }
        best.map(|(idx, _)| RuleHit {
            rule: &self.lexicon.rules[idx],
            exact: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inci::normalise_inci;

    fn hit(name: &str) -> Option<&'static IngredientRule> {
        lexicon()
            .lookup(&normalise_inci(name), None)
            .map(|h| h.rule)
    }

    #[test]
    fn bundled_lexicon_loads_with_unique_ids() {
        let lx = lexicon();
        let mut seen = std::collections::HashSet::new();
        for rule in lx.rules() {
            assert!(seen.insert(&rule.id), "duplicate rule id {}", rule.id);
            assert!(
                (0.0..=1.0).contains(&rule.animal_route_likelihood),
                "{} has out-of-range likelihood",
                rule.id
            );
            assert!(!rule.concern.trim().is_empty(), "{} has no concern text", rule.id);
        }
        assert!(seen.len() >= 60, "expected a broad lexicon, got {}", seen.len());
    }

    #[test]
    fn source_ambiguous_rules_offer_a_route_to_resolution() {
        for rule in lexicon().rules() {
            if rule.ambiguity == Ambiguity::Source {
                assert!(
                    !rule.resolvable_by.is_empty(),
                    "{} has source ambiguity but lists no resolving evidence",
                    rule.id
                );
                assert!(
                    rule.animal_route_likelihood > 0.0,
                    "{} has source ambiguity but zero animal-route likelihood",
                    rule.id
                );
            }
            if rule.status == HalalStatus::Mashbooh {
                assert_ne!(
                    rule.ambiguity,
                    Ambiguity::None,
                    "{} is doubtful without saying why",
                    rule.id
                );
            }
        }
    }

    #[test]
    fn carmine_matches_by_name_and_by_colour_index() {
        assert_eq!(hit("Carmine").map(|r| r.id.as_str()), Some("carmine"));
        let by_ci = lexicon().lookup("CI 75470", Some("75470")).expect("ci hit");
        assert_eq!(by_ci.rule.id, "carmine");
        assert_eq!(by_ci.rule.status, HalalStatus::Haram);
        assert_eq!(by_ci.rule.origin, OriginClass::AnimalInsect);
    }

    #[test]
    fn fatty_alcohols_are_not_treated_as_intoxicants() {
        let cetyl = hit("Cetearyl Alcohol").expect("cetearyl alcohol rule");
        assert_eq!(cetyl.alcohol_kind, Some(AlcoholKind::FattyAlcohol));
        assert_ne!(cetyl.status, HalalStatus::Haram);

        let denat = hit("Alcohol Denat.").expect("alcohol denat rule");
        assert_eq!(denat.alcohol_kind, Some(AlcoholKind::Ethanol));
    }

    #[test]
    fn stearic_acid_is_ambiguous_not_prohibited() {
        let rule = hit("Stearic Acid").expect("stearic acid rule");
        assert_eq!(rule.status, HalalStatus::Mashbooh);
        assert_eq!(rule.origin, OriginClass::Ambiguous);
        assert!(rule
            .resolvable_by
            .contains(&EvidenceKind::PlantSourceDeclaration));
    }

    #[test]
    fn fragment_rules_catch_stearate_derivatives() {
        let rule = hit("Magnesium Stearate").expect("stearate fragment rule");
        assert_eq!(rule.status, HalalStatus::Mashbooh);
    }

    #[test]
    fn porcine_names_are_categorical() {
        for name in ["Lard", "Sus Scrofa Placental Protein"] {
            let rule = hit(name).unwrap_or_else(|| panic!("no rule for {name}"));
            assert_eq!(rule.status, HalalStatus::Haram, "{name}");
        }
    }

    #[test]
    fn strict_profile_can_tighten_a_rule() {
        let ethanol = hit("Alcohol Denat.").expect("alcohol denat rule");
        assert_eq!(ethanol.status_for("strict"), HalalStatus::Haram);
        assert_eq!(ethanol.status_for("uk-general"), ethanol.status);
    }

    #[test]
    fn nitrocellulose_is_flagged_as_a_wudu_barrier() {
        let rule = hit("Nitrocellulose").expect("nitrocellulose rule");
        assert!(rule.wudu_barrier);
    }
}
