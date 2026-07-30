use super::rules::{classify_ingredient, HalalStandard};
use crate::models::{
    Certification, CertStatus, CosingIngredient, HalalClassification, HalalFlag, HalalStatus,
};

pub struct ScoreResult {
    pub score: u8,
    pub status: HalalStatus,
    pub flags: Vec<HalalFlag>,
}

pub fn score_product(
    ingredients: &[CosingIngredient],
    certifications: &[Certification],
    standard: HalalStandard,
) -> ScoreResult {
    let mut score: i16 = 100;
    let mut flags = Vec::new();
    let mut has_haram = false;
    let mut has_mashbooh = false;

    for ingredient in ingredients {
        let result = classify_ingredient(&ingredient.inci_name, standard);
        if result.penalty > 0 {
            flags.push(HalalFlag {
                ingredient: ingredient.inci_name.clone(),
                classification: result.classification.clone(),
                reason: result.reason.clone(),
                penalty: result.penalty,
            });
            score -= result.penalty as i16;

            match result.classification {
                HalalClassification::Haram => has_haram = true,
                HalalClassification::Mashbooh => has_mashbooh = true,
                HalalClassification::Halal => {}
            }
        }
    }

    let active_cert = certifications
        .iter()
        .any(|c| matches!(c.status, CertStatus::Active));

    if active_cert {
        score = score.max(95);
    }

    let score = score.clamp(0, 100) as u8;

    let status = if active_cert {
        HalalStatus::Halal
    } else if has_haram {
        if score <= 24 {
            HalalStatus::Haram
        } else {
            HalalStatus::LikelyHaram
        }
    } else if has_mashbooh {
        if score >= 75 {
            HalalStatus::LikelyHalal
        } else {
            HalalStatus::Mashbooh
        }
    } else if score >= 90 {
        HalalStatus::LikelyHalal
    } else {
        HalalStatus::Mashbooh
    };

    ScoreResult {
        score,
        status,
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HalalClassification;

    fn ingredient(name: &str) -> CosingIngredient {
        CosingIngredient {
            inci_name: name.into(),
            slug: None,
            cas_number: None,
            ec_number: None,
            functions: vec![],
            restriction: None,
            halal_classification: HalalClassification::Halal,
        }
    }

    #[test]
    fn clean_product_scores_high() {
        let ingredients = vec![ingredient("Aqua"), ingredient("Niacinamide")];
        let result = score_product(&ingredients, &[], HalalStandard::Strict);
        assert!(result.score >= 90);
    }

    #[test]
    fn carmine_drops_score() {
        let ingredients = vec![ingredient("Aqua"), ingredient("Carmine (CI 75470)")];
        let result = score_product(&ingredients, &[], HalalStandard::Strict);
        assert!(result.score < 70);
        assert_eq!(result.status, HalalStatus::LikelyHaram);
    }
}
