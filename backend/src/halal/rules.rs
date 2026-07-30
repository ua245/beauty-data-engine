use crate::models::HalalClassification;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalalStandard {
    Strict,
    Jakim,
    Bpjph,
    Permissive,
}

pub struct IngredientRule {
    pub classification: HalalClassification,
    pub reason: String,
    pub penalty: u8,
}

pub struct ClassificationResult {
    pub classification: HalalClassification,
    pub reason: String,
    pub penalty: u8,
}

fn haram_rules() -> HashMap<&'static str, (&'static str, u8)> {
    HashMap::from([
        ("carmine", ("Insect-derived red pigment (CI 75470)", 40)),
        ("ci 75470", ("Insect-derived red pigment", 40)),
        ("cochineal", ("Insect-derived colorant", 40)),
        ("natural red 4", ("Insect-derived colorant", 40)),
        ("gelatin", ("Often porcine-derived unless certified", 40)),
        ("lard", ("Pig fat — haram", 40)),
        ("placenta extract", ("Human/animal origin", 40)),
        ("porcine collagen", ("Pig-derived collagen", 40)),
    ])
}

fn mashbooh_rules() -> HashMap<&'static str, (&'static str, u8)> {
    HashMap::from([
        ("glycerin", ("Animal or plant source — verify origin", 15)),
        ("glycerol", ("Animal or plant source — verify origin", 15)),
        ("stearic acid", ("May be animal or plant derived", 15)),
        ("collagen", ("Source unspecified — bovine/porcine/marine", 15)),
        ("hydrolyzed collagen", ("Source unspecified", 15)),
        ("keratin", ("Animal protein — check source", 15)),
        ("hydrolyzed keratin", ("Animal protein — check source", 15)),
        ("elastin", ("Often bovine-derived", 15)),
        ("lanolin", ("Sheep wool wax — majority view permissible", 10)),
        ("alcohol denat", ("Ethanol — disputed in cosmetics", 15)),
        ("ethanol", ("Alcohol — check source and concentration", 15)),
        ("sd alcohol", ("Denatured ethanol", 15)),
        ("isopropyl alcohol", ("Synthetic alcohol — disputed", 10)),
        ("parfum", ("Fragrance composition unknown", 15)),
        ("fragrance", ("May contain alcohol or animal musk", 15)),
        ("beeswax", ("Insect product — majority permissible", 5)),
        ("cera alba", ("Beeswax — majority permissible", 5)),
        ("shellac", ("Insect resin — mashbooh", 15)),
        ("squalene", ("May be shark-derived", 15)),
    ])
}

fn halal_safe() -> Vec<&'static str> {
    vec![
        "aqua",
        "water",
        "cetyl alcohol",
        "stearyl alcohol",
        "cetearyl alcohol",
        "niacinamide",
        "hyaluronic acid",
        "tocopherol",
        "aloe barbadensis",
        "kaolin",
        "mica",
        "talc",
        "caprylic/capric triglyceride",
        "butylene glycol",
        "panthenol",
    ]
}

pub fn classify_ingredient(inci_name: &str, standard: HalalStandard) -> ClassificationResult {
    let normalized = inci_name.trim().to_lowercase();

    if halal_safe().iter().any(|safe| normalized.contains(safe)) {
        return ClassificationResult {
            classification: HalalClassification::Halal,
            reason: "Known halal-safe ingredient".into(),
            penalty: 0,
        };
    }

    let haram = haram_rules();
    for (key, (reason, penalty)) in &haram {
        if normalized.contains(key) {
            return ClassificationResult {
                classification: HalalClassification::Haram,
                reason: reason.to_string(),
                penalty: *penalty,
            };
        }
    }

    let mashbooh = mashbooh_rules();
    for (key, (reason, penalty)) in &mashbooh {
        if normalized.contains(key) {
            let adjusted_penalty = match standard {
                HalalStandard::Permissive if *key == "lanolin" || *key == "beeswax" => 0,
                HalalStandard::Strict if key.contains("alcohol") => 25,
                _ => *penalty,
            };
            let classification = if adjusted_penalty == 0 {
                HalalClassification::Halal
            } else {
                HalalClassification::Mashbooh
            };
            return ClassificationResult {
                classification,
                reason: reason.to_string(),
                penalty: adjusted_penalty,
            };
        }
    }

    ClassificationResult {
        classification: HalalClassification::Halal,
        reason: "No known haram or mashbooh flags".into(),
        penalty: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carmine_is_haram() {
        let result = classify_ingredient("Carmine (CI 75470)", HalalStandard::Strict);
        assert_eq!(result.classification, HalalClassification::Haram);
    }

    #[test]
    fn cetyl_alcohol_is_halal() {
        let result = classify_ingredient("Cetyl Alcohol", HalalStandard::Strict);
        assert_eq!(result.classification, HalalClassification::Halal);
    }

    #[test]
    fn glycerin_is_mashbooh() {
        let result = classify_ingredient("Glycerin", HalalStandard::Strict);
        assert_eq!(result.classification, HalalClassification::Mashbooh);
    }
}
