//! Halal beauty product engine — core domain, parsing, scoring and certification.

pub mod certification;
pub mod inci;
pub mod lexicon;
pub mod product;
pub mod scoring;
pub mod taxonomy;

pub use certification::{
    register, Authority, CertificateClaim, CertificateEvidence, CertificateVerification,
    CertifierBody, CertifierRegister, Scope, TrustRung,
};
pub use inci::{is_probable_declaration, normalise_inci, parse_ingredient_list, IngredientList};
pub use lexicon::{
    lexicon, AlcoholKind, Ambiguity, EvidenceKind, HalalStatus, IngredientRule, LexiconIndex,
    OriginClass, RuleHit,
};
pub use product::{
    AcquisitionMethod, Currency, ImageRights, LabelClaim, Money, Product, ProductImage, Retailer,
    SourceAttestation,
};
pub use scoring::{
    assess, profiles, CosIngFacts, HalalAssessment, IngredientAssessment, RuleProfile, Verdict,
    WuduVerdict, DEFAULT_PROFILE,
};
pub use taxonomy::{
    taxonomy, Application, Category, ClassificationMethod, Department, FilmRisk, ProductType,
    Taxonomy, TaxonomyAssignment,
};
