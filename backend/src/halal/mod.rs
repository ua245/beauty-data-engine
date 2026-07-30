mod rules;
mod scorer;

pub use rules::{classify_ingredient, HalalStandard};
pub use scorer::score_product;

use crate::models::{HalalClassification, HalalFlag, HalalStatus, Product};

pub fn evaluate_product(product: &mut Product, standard: HalalStandard) {
    let result = score_product(&product.ingredients, &product.certifications, standard);
    product.halal_score = result.score;
    product.halal_status = result.status;
    product.halal_flags = result.flags;

    for ingredient in &mut product.ingredients {
        ingredient.halal_classification =
            classify_ingredient(&ingredient.inci_name, standard).classification;
    }
}

pub fn status_label(status: &HalalStatus) -> &'static str {
    match status {
        HalalStatus::Halal => "Certified Halal",
        HalalStatus::LikelyHalal => "Likely Halal",
        HalalStatus::Mashbooh => "Mashbooh (Doubtful)",
        HalalStatus::LikelyHaram => "Likely Not Halal",
        HalalStatus::Haram => "Not Halal",
    }
}

pub fn classification_color(classification: &HalalClassification) -> &'static str {
    match classification {
        HalalClassification::Halal => "#68a691",
        HalalClassification::Mashbooh => "#bfd3c1",
        HalalClassification::Haram => "#694f5d",
    }
}
