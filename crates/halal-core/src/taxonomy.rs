//! Product taxonomy modelled on the three-level department / category / product-type structure
//! used by UK high-street beauty retailers, plus the physical attributes the halal engine needs.
//!
//! Two attributes on each product type are load-bearing rather than cosmetic:
//! `application` decides which ethanol tolerance applies (rinse-off formulations are treated more
//! leniently by most schemes than leave-on), and `film_risk` drives the wudu-compatibility axis
//! independently of ingredient status.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// How the product sits on the body, which changes how alcohol content is assessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Application {
    /// Stays on the skin, hair or nails until deliberately removed.
    LeaveOn,
    /// Washed off within the same use, e.g. shampoo or cleanser.
    RinseOff,
    /// Sprayed or dabbed on and left to evaporate, e.g. eau de parfum.
    Evaporative,
    /// Not applied to the body at all, e.g. brushes and applicators.
    NonContact,
}

/// Likelihood that the product type forms a water-impermeable layer, which is the question
/// MUI Fatwa 60/2020 addresses for ablution validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilmRisk {
    /// No barrier expected, e.g. a toner.
    None,
    /// A barrier is possible depending on the formulation.
    Possible,
    /// A barrier is inherent to the format, e.g. nail lacquer or waterproof mascara.
    Inherent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductType {
    pub id: String,
    pub name: String,
    pub application: Application,
    pub film_risk: FilmRisk,
    /// Lower-case terms matched against product titles when no usable breadcrumb exists.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Facet groups meaningful for this product type, used to drive the UI filter rail.
    #[serde(default)]
    pub facets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub product_types: Vec<ProductType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    pub departments: Vec<Department>,
    /// Maps retailer breadcrumb fragments (lower-case) onto product type ids. Retailers use their
    /// own wording, so this is the bridge between their tree and ours.
    #[serde(default)]
    pub breadcrumb_map: HashMap<String, String>,
}

/// Where a taxonomy assignment came from, so that low-confidence guesses stay auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassificationMethod {
    /// The retailer's own breadcrumb mapped directly onto a product type.
    Breadcrumb,
    /// Inferred from words in the product title.
    Title,
    /// Set by hand in seed or override data.
    Manual,
    /// Nothing matched; the product sits at department level only.
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxonomyAssignment {
    pub department_id: String,
    pub category_id: String,
    pub product_type_id: String,
    pub method: ClassificationMethod,
    pub confidence: f32,
}

/// A resolved path through the tree, used for display and for engine lookups.
#[derive(Debug, Clone, Copy)]
pub struct TaxonomyNode<'a> {
    pub department: &'a Department,
    pub category: &'a Category,
    pub product_type: &'a ProductType,
}

const TAXONOMY_JSON: &str = include_str!("../data/taxonomy.json");

static TAXONOMY: Lazy<Taxonomy> =
    Lazy::new(|| serde_json::from_str(TAXONOMY_JSON).expect("bundled taxonomy.json is valid"));

/// The taxonomy bundled with the crate.
pub fn taxonomy() -> &'static Taxonomy {
    &TAXONOMY
}

impl Taxonomy {
    pub fn resolve(&self, product_type_id: &str) -> Option<TaxonomyNode<'_>> {
        for department in &self.departments {
            for category in &department.categories {
                for product_type in &category.product_types {
                    if product_type.id == product_type_id {
                        return Some(TaxonomyNode {
                            department,
                            category,
                            product_type,
                        });
                    }
                }
            }
        }
        None
    }

    pub fn product_type(&self, id: &str) -> Option<&ProductType> {
        self.resolve(id).map(|n| n.product_type)
    }

    pub fn assignment_for(&self, product_type_id: &str, method: ClassificationMethod, confidence: f32) -> Option<TaxonomyAssignment> {
        let node = self.resolve(product_type_id)?;
        Some(TaxonomyAssignment {
            department_id: node.department.id.clone(),
            category_id: node.category.id.clone(),
            product_type_id: node.product_type.id.clone(),
            method,
            confidence,
        })
    }

    /// Classify a product from whatever the retailer gave us.
    ///
    /// Breadcrumbs win when they map, because they are the retailer's own merchandising decision.
    /// Title keywords are the fallback and are scored by specificity so that
    /// "Waterproof Liquid Eyeliner" prefers `eyeliner` over `eye-makeup`.
    pub fn classify(&self, breadcrumbs: &[String], title: &str, brand: &str) -> TaxonomyAssignment {
        for crumb in breadcrumbs.iter().rev() {
            let key = crumb.trim().to_lowercase();
            if let Some(pt) = self.breadcrumb_map.get(&key) {
                if let Some(assignment) =
                    self.assignment_for(pt, ClassificationMethod::Breadcrumb, 0.97)
                {
                    return assignment;
                }
            }
        }

        // Strip the brand from the title so that brand names containing product words
        // (for example "Nails Inc") do not dominate the match.
        let stripped = title
            .to_lowercase()
            .replace(&brand.to_lowercase(), " ")
            .replace(['-', '/', '&', ',', '.', '\''], " ");
        let haystack = format!(" {} ", stripped.split_whitespace().collect::<Vec<_>>().join(" "));
        let mut best: Option<(&ProductType, usize)> = None;
        for department in &self.departments {
            for category in &department.categories {
                for pt in &category.product_types {
                    for keyword in &pt.keywords {
                        if haystack.contains(&format!(" {keyword} ")) || haystack.contains(&format!(" {keyword}s ")) {
                            let score = keyword.len();
                            if best.map_or(true, |(_, s)| score > s) {
                                best = Some((pt, score));
                            }
                        }
                    }
                }
            }
        }
        if let Some((pt, score)) = best {
            let confidence = (0.55 + (score as f32 / 40.0)).min(0.9);
            if let Some(assignment) =
                self.assignment_for(&pt.id, ClassificationMethod::Title, confidence)
            {
                return assignment;
            }
        }

        TaxonomyAssignment {
            department_id: "unclassified".to_string(),
            category_id: "unclassified".to_string(),
            product_type_id: "unclassified".to_string(),
            method: ClassificationMethod::Unresolved,
            confidence: 0.0,
        }
    }

    /// Flattened list of every product type with its ancestry, for building filter rails.
    pub fn flatten(&self) -> Vec<(&Department, &Category, &ProductType)> {
        let mut out = Vec::new();
        for d in &self.departments {
            for c in &d.categories {
                for pt in &c.product_types {
                    out.push((d, c, pt));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_taxonomy_loads_and_has_no_duplicate_ids() {
        let tx = taxonomy();
        let mut seen = std::collections::HashSet::new();
        for (_, _, pt) in tx.flatten() {
            assert!(seen.insert(pt.id.clone()), "duplicate product type id {}", pt.id);
        }
        assert!(seen.len() > 60, "expected a broad taxonomy, got {}", seen.len());
    }

    #[test]
    fn breadcrumb_map_points_at_real_product_types() {
        let tx = taxonomy();
        for (crumb, pt) in &tx.breadcrumb_map {
            assert!(
                tx.product_type(pt).is_some(),
                "breadcrumb {crumb} maps to unknown product type {pt}"
            );
        }
    }

    #[test]
    fn classifies_from_breadcrumbs() {
        let tx = taxonomy();
        let a = tx.classify(
            &[
                "Make-up".to_string(),
                "Face".to_string(),
                "Foundation".to_string(),
            ],
            "Infallible 24H Fresh Wear Foundation",
            "L'Oreal Paris",
        );
        assert_eq!(a.product_type_id, "foundation");
        assert_eq!(a.method, ClassificationMethod::Breadcrumb);
    }

    #[test]
    fn falls_back_to_title_keywords() {
        let tx = taxonomy();
        let a = tx.classify(&[], "Volumising Waterproof Mascara Black", "Maybelline");
        assert_eq!(a.product_type_id, "mascara");
        assert_eq!(a.method, ClassificationMethod::Title);
    }

    #[test]
    fn nail_polish_is_inherently_film_forming() {
        let pt = taxonomy().product_type("nail-polish").expect("nail polish");
        assert_eq!(pt.film_risk, FilmRisk::Inherent);
        assert_eq!(pt.application, Application::LeaveOn);
    }

    #[test]
    fn fragrance_is_evaporative() {
        let pt = taxonomy().product_type("eau-de-parfum").expect("edp");
        assert_eq!(pt.application, Application::Evaporative);
    }

    #[test]
    fn unknown_input_is_unresolved_rather_than_wrong() {
        let a = taxonomy().classify(&[], "Gift Card", "Boots");
        assert_eq!(a.method, ClassificationMethod::Unresolved);
    }
}
