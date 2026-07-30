//! Bundled sample products for the dashboard and for offline development.

use chrono::Utc;
use halal_core::certification::{CertificateClaim, CertificateEvidence, Scope};
use halal_core::product::{
    AcquisitionMethod, Currency, ImageRights, LabelClaim, Money, Product, ProductImage, Retailer,
    SourceAttestation,
};
use halal_core::scoring::CosIngFacts;
use halal_core::taxonomy::{taxonomy, ClassificationMethod};
use halal_core::{assess, EvidenceKind, DEFAULT_PROFILE};

use crate::catalog::{Catalog, CatalogProduct};

fn retailer(id: &str, name: &str, domain: &str) -> Retailer {
    Retailer {
        id: id.to_string(),
        name: name.to_string(),
        domain: domain.to_string(),
    }
}

fn img(url: &str, alt: &str) -> ProductImage {
    ProductImage {
        url: url.to_string(),
        alt: alt.to_string(),
        rights: ImageRights::Placeholder,
        attribution: None,
    }
}

/// Sample CosIng facts keyed by normalised INCI name.
pub fn sample_cosing() -> Vec<(String, CosIngFacts)> {
    vec![
        (
            "AQUA".into(),
            CosIngFacts {
                inci_name: "AQUA".into(),
                cas_number: Some("7732-18-5".into()),
                ec_number: Some("231-791-2".into()),
                functions: vec!["SOLVENT".into()],
                annex_reference: None,
                restriction: None,
            },
        ),
        (
            "GLYCERIN".into(),
            CosIngFacts {
                inci_name: "GLYCERIN".into(),
                cas_number: Some("56-81-5".into()),
                ec_number: Some("200-289-5".into()),
                functions: vec!["HUMECTANT".into(), "SKIN CONDITIONING".into()],
                annex_reference: None,
                restriction: None,
            },
        ),
        (
            "CARMINE".into(),
            CosIngFacts {
                inci_name: "CI 75470".into(),
                cas_number: Some("1260-17-9".into()),
                ec_number: None,
                functions: vec!["COLOUR".into()],
                annex_reference: Some("IV/115".into()),
                restriction: None,
            },
        ),
        (
            "CI 75470".into(),
            CosIngFacts {
                inci_name: "CI 75470".into(),
                cas_number: Some("1260-17-9".into()),
                ec_number: None,
                functions: vec!["COLOUR".into()],
                annex_reference: Some("IV/115".into()),
                restriction: None,
            },
        ),
        (
            "TITANIUM DIOXIDE".into(),
            CosIngFacts {
                inci_name: "TITANIUM DIOXIDE".into(),
                cas_number: Some("13463-67-7".into()),
                ec_number: Some("236-675-5".into()),
                functions: vec!["COLOUR".into(), "OPACIFYING".into()],
                annex_reference: Some("IV/143".into()),
                restriction: None,
            },
        ),
        (
            "SODIUM HYALURONATE".into(),
            CosIngFacts {
                inci_name: "SODIUM HYALURONATE".into(),
                cas_number: Some("9067-32-7".into()),
                ec_number: None,
                functions: vec!["HUMECTANT".into(), "SKIN CONDITIONING".into()],
                annex_reference: None,
                restriction: None,
            },
        ),
        (
            "NITROCELLULOSE".into(),
            CosIngFacts {
                inci_name: "NITROCELLULOSE".into(),
                cas_number: Some("9004-70-0".into()),
                ec_number: None,
                functions: vec!["FILM FORMING".into()],
                annex_reference: None,
                restriction: None,
            },
        ),
    ]
}

pub fn sample_products() -> Vec<Product> {
    let tx = taxonomy();
    let now = Utc::now();

    let mut products = vec![
        Product {
            id: "boots:12345678".into(),
            name: "No7 Lift & Luminate Triple Action Serum".into(),
            brand: "No7".into(),
            manufacturer: Some("The Boots Company PLC".into()),
            retailer: retailer("boots", "Boots", "boots.com"),
            retailer_sku: "12345678".into(),
            url: "https://www.boots.com/no7-lift-luminate-triple-action-serum-10123456".into(),
            gtin: Some("5052191234567".into()),
            shade: None,
            size: Some("30ml".into()),
            price: Some(Money {
                minor_units: 3499,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1620916560428-7fe1343c76fe?w=400",
                "No7 serum bottle",
            )),
            taxonomy: tx.classify(
                &["Skincare".into(), "Serums".into()],
                "No7 Lift & Luminate Triple Action Serum",
                "No7",
            ),
            retailer_breadcrumbs: vec![
                "Skincare".into(),
                "Serums".into(),
                "Face Serum".into(),
            ],
            ingredients_raw: Some(
                "Aqua, Glycerin, Dimethicone, Niacinamide, Sodium Hyaluronate, Cetearyl Alcohol, \
                 Stearic Acid, Phenoxyethanol, Parfum, Citric Acid."
                    .into(),
            ),
            claims: vec![LabelClaim::CrueltyFree],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "superdrug:987654".into(),
            name: "Sleek MakeUP True Colour Lipstick — Ruby".into(),
            brand: "Sleek MakeUP".into(),
            manufacturer: None,
            retailer: retailer("superdrug", "Superdrug", "superdrug.com"),
            retailer_sku: "987654".into(),
            url: "https://www.superdrug.com/sleek-makeup-true-colour-lipstick-ruby/p/987654".into(),
            gtin: Some("5055998765432".into()),
            shade: Some("Ruby".into()),
            size: Some("3.7g".into()),
            price: Some(Money {
                minor_units: 599,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1586495777744-4413f210eafc?w=400",
                "Red lipstick",
            )),
            taxonomy: tx.classify(
                &["Make-up".into(), "Lips".into(), "Lipstick".into()],
                "Sleek MakeUP True Colour Lipstick Ruby",
                "Sleek MakeUP",
            ),
            retailer_breadcrumbs: vec!["Make-up".into(), "Lips".into(), "Lipstick".into()],
            ingredients_raw: Some(
                "Ricinus Communis Seed Oil, Mica, Candelilla Wax, Carnauba Wax, Talc, \
                 CI 77491, CI 77492, CI 75470, Parfum."
                    .into(),
            ),
            claims: vec![],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "lookfantastic:lf-4421".into(),
            name: "Amika The Kure Bond Repair Shampoo".into(),
            brand: "Amika".into(),
            manufacturer: None,
            retailer: retailer("lookfantastic", "Lookfantastic", "lookfantastic.com"),
            retailer_sku: "lf-4421".into(),
            url: "https://www.lookfantastic.com/amika-the-kure-bond-repair-shampoo/12088509.html"
                .into(),
            gtin: Some("8500047101234".into()),
            shade: None,
            size: Some("275ml".into()),
            price: Some(Money {
                minor_units: 2400,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1535585209827-a15fcdbc4c2d?w=400",
                "Shampoo bottle",
            )),
            taxonomy: tx.classify(&["Haircare".into(), "Shampoo".into()], "Amika The Kure Bond Repair Shampoo", "Amika"),
            retailer_breadcrumbs: vec!["Haircare".into(), "Shampoo".into()],
            ingredients_raw: Some(
                "Aqua, Sodium C14-16 Olefin Sulfonate, Cocamidopropyl Betaine, Glycerin, \
                 Hydrolyzed Keratin, Panthenol, Citric Acid, Sodium Benzoate, Parfum."
                    .into(),
            ),
            claims: vec![LabelClaim::VeganCertified, LabelClaim::CrueltyFree],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "spacenk:snk-7788".into(),
            name: "Radiance Pure Glow Foundation — Sand".into(),
            brand: "Radiance".into(),
            manufacturer: Some("Radiance Cosmetics Ltd".into()),
            retailer: retailer("spacenk", "Space NK", "spacenk.com"),
            retailer_sku: "snk-7788".into(),
            url: "https://www.spacenk.com/uk/radiance-pure-glow-foundation-sand".into(),
            gtin: Some("5067890123456".into()),
            shade: Some("Sand".into()),
            size: Some("30ml".into()),
            price: Some(Money {
                minor_units: 4200,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1596462502278-27bfdc403348?w=400",
                "Foundation bottle",
            )),
            taxonomy: tx.classify(
                &["Make-up".into(), "Face".into(), "Foundation".into()],
                "Radiance Pure Glow Foundation Sand",
                "Radiance",
            ),
            retailer_breadcrumbs: vec!["Make-up".into(), "Face".into(), "Foundation".into()],
            ingredients_raw: Some(
                "Aqua, Glycerin, Mica, Titanium Dioxide, Cetearyl Alcohol, Stearic Acid, \
                 Phenoxyethanol, Parfum."
                    .into(),
            ),
            claims: vec![LabelClaim::HalalCertified, LabelClaim::CrueltyFree],
            certificate_claims: vec![CertificateClaim {
                certifier_id: "hce".into(),
                certificate_number: Some("21633-1/2/2/Y1".into()),
                holder: Some("Radiance Cosmetics Ltd".into()),
                scopes: vec![Scope::Cosmetics],
                issued_on: None,
                expires_on: chrono::NaiveDate::from_ymd_opt(2027, 5, 31),
                found_in_directory: false,
                found_in_regulator_registry: false,
                evidence: Some(CertificateEvidence {
                    url: Some("https://example.org/radiance-hce-cert.pdf".into()),
                    document_sha256: Some(CertificateEvidence::hash_document(b"sample-cert")),
                    method: "sample data".into(),
                    captured_at: now,
                }),
            }],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "beautybay:bb-3310".into(),
            name: "Beauty Bay Brighten + Hydrate Serum".into(),
            brand: "Beauty Bay".into(),
            manufacturer: None,
            retailer: retailer("beautybay", "Beauty Bay", "beautybay.com"),
            retailer_sku: "bb-3310".into(),
            url: "https://www.beautybay.com/p/beauty-bay/brighten-hydrate-serum/".into(),
            gtin: None,
            shade: None,
            size: Some("30ml".into()),
            price: Some(Money {
                minor_units: 1295,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1570194065650-d99fb4d38c22?w=400",
                "Serum dropper bottle",
            )),
            taxonomy: tx.classify(&[], "Beauty Bay Brighten Hydrate Serum", "Beauty Bay"),
            retailer_breadcrumbs: vec!["Skincare".into(), "Serums".into()],
            ingredients_raw: Some(
                "Aqua, Glycerin, Niacinamide, Sodium Hyaluronate, Panthenol, Phenoxyethanol, \
                 Ethylhexylglycerin."
                    .into(),
            ),
            claims: vec![LabelClaim::Vegan, LabelClaim::CrueltyFree],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "boots:perf-001".into(),
            name: "YSL Libre Eau de Parfum".into(),
            brand: "Yves Saint Laurent".into(),
            manufacturer: Some("L'Oréal UK Ltd".into()),
            retailer: retailer("boots", "Boots", "boots.com"),
            retailer_sku: "perf-001".into(),
            url: "https://www.boots.com/ysl-libre-eau-de-parfum".into(),
            gtin: Some("3614272045678".into()),
            shade: None,
            size: Some("50ml".into()),
            price: Some(Money {
                minor_units: 8900,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1541643600914-78b084683601?w=400",
                "Perfume bottle",
            )),
            taxonomy: tx.classify(
                &["Fragrance".into(), "Eau de Parfum".into()],
                "YSL Libre Eau de Parfum",
                "Yves Saint Laurent",
            ),
            retailer_breadcrumbs: vec!["Fragrance".into(), "Women's Fragrance".into()],
            ingredients_raw: Some(
                "Alcohol Denat., Parfum, Aqua, Linalool, Limonene, Citronellol, Geraniol, \
                 Benzyl Salicylate."
                    .into(),
            ),
            claims: vec![],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "boots:attar-002".into(),
            name: "Al Haramain Amber Oud Attar".into(),
            brand: "Al Haramain".into(),
            manufacturer: None,
            retailer: retailer("boots", "Boots", "boots.com"),
            retailer_sku: "attar-002".into(),
            url: "https://www.boots.com/al-haramain-amber-oud-attar".into(),
            gtin: None,
            shade: None,
            size: Some("12ml".into()),
            price: Some(Money {
                minor_units: 2499,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1592945403244-b3fbafd7f539?w=400",
                "Attar oil bottle",
            )),
            taxonomy: tx.classify(&["Fragrance".into(), "Attar".into()], "Al Haramain Amber Oud Attar", "Al Haramain"),
            retailer_breadcrumbs: vec!["Fragrance".into(), "Attar".into()],
            ingredients_raw: Some(
                "Aquilaria Malaccensis Wood Oil, Santalum Album Wood Oil, Rosa Damascena Flower Oil."
                    .into(),
            ),
            claims: vec![LabelClaim::AlcoholFree, LabelClaim::HalalCertified],
            certificate_claims: vec![CertificateClaim {
                certifier_id: "ifanca".into(),
                certificate_number: Some("MAR 23823 250001 US".into()),
                holder: Some("Al Haramain Perfumes".into()),
                scopes: vec![Scope::Fragrance, Scope::Cosmetics],
                issued_on: None,
                expires_on: chrono::NaiveDate::from_ymd_opt(2027, 12, 31),
                found_in_directory: true,
                found_in_regulator_registry: false,
                evidence: None,
            }],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "superdrug:nail-003".into(),
            name: "Nails Inc. Breathable Halal Nail Polish — Baker Street".into(),
            brand: "Nails Inc.".into(),
            manufacturer: None,
            retailer: retailer("superdrug", "Superdrug", "superdrug.com"),
            retailer_sku: "nail-003".into(),
            url: "https://www.superdrug.com/nails-inc-breathable-halal-baker-street".into(),
            gtin: None,
            shade: Some("Baker Street".into()),
            size: Some("14ml".into()),
            price: Some(Money {
                minor_units: 1099,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1604654894610-df63bc536371?w=400",
                "Nail polish",
            )),
            taxonomy: tx.classify(
                &["Make-up".into(), "Nails".into(), "Breathable Nail Polish".into()],
                "Nails Inc Breathable Halal Nail Polish Baker Street",
                "Nails Inc.",
            ),
            retailer_breadcrumbs: vec![
                "Make-up".into(),
                "Nails".into(),
                "Breathable Nail Polish".into(),
            ],
            ingredients_raw: Some(
                "Butyl Acetate, Ethyl Acetate, Nitrocellulose, Acetyl Tributyl Citrate, \
                 Isopropyl Alcohol, Adipic Acid/Neopentyl Glycol/Trimellitic Anhydride Copolymer, \
                 Acrylates Copolymer, Mica, CI 77891."
                    .into(),
            ),
            claims: vec![
                LabelClaim::HalalCertified,
                LabelClaim::WaterPermeable,
                LabelClaim::Vegan,
            ],
            certificate_claims: vec![CertificateClaim {
                certifier_id: "hce".into(),
                certificate_number: Some("HCE-NI-2025-44".into()),
                holder: Some("Nails Inc. Ltd".into()),
                scopes: vec![Scope::Cosmetics],
                issued_on: None,
                expires_on: chrono::NaiveDate::from_ymd_opt(2026, 12, 31),
                found_in_directory: false,
                found_in_regulator_registry: false,
                evidence: None,
            }],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "hollandbarrett:hb-5566".into(),
            name: "Faith in Nature Lavender Shampoo".into(),
            brand: "Faith in Nature".into(),
            manufacturer: None,
            retailer: retailer(
                "hollandbarrett",
                "Holland & Barrett",
                "hollandandbarrett.com",
            ),
            retailer_sku: "hb-5566".into(),
            url: "https://www.hollandandbarrett.com/shop/product/faith-in-nature-lavender-shampoo"
                .into(),
            gtin: Some("5028868123456".into()),
            shade: None,
            size: Some("400ml".into()),
            price: Some(Money {
                minor_units: 599,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1527799820374-dcf8d9a4e388?w=400",
                "Natural shampoo",
            )),
            taxonomy: tx.classify(
                &["Haircare".into(), "Shampoo".into()],
                "Faith in Nature Lavender Shampoo",
                "Faith in Nature",
            ),
            retailer_breadcrumbs: vec!["Haircare".into(), "Shampoo".into()],
            ingredients_raw: Some(
                "Aqua, Sodium Laureth Sulfate, Cocamidopropyl Betaine, Lavandula Angustifolia \
                 Flower Extract, Glycerin, Sodium Chloride, Citric Acid, Sodium Benzoate, Parfum."
                    .into(),
            ),
            claims: vec![
                LabelClaim::VeganCertified,
                LabelClaim::CrueltyFree,
                LabelClaim::FragranceFree,
            ],
            certificate_claims: vec![],
            attestations: vec![SourceAttestation {
                rule_id: "glycerin".into(),
                evidence: EvidenceKind::PlantSourceDeclaration,
                statement: "Glycerin is 100% vegetable origin.".into(),
                source_url: Some("https://faithinnature.co.uk/ingredients".into()),
                document_sha256: None,
                recorded_at: now,
            }],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
        Product {
            id: "cultbeauty:cb-9900".into(),
            name: "Drunk Elephant Protini Polypeptide Cream".into(),
            brand: "Drunk Elephant".into(),
            manufacturer: None,
            retailer: retailer("cultbeauty", "Cult Beauty", "cultbeauty.co.uk"),
            retailer_sku: "cb-9900".into(),
            url: "https://www.cultbeauty.co.uk/drunk-elephant-protini-polypeptide-cream".into(),
            gtin: None,
            shade: None,
            size: Some("50ml".into()),
            price: Some(Money {
                minor_units: 6400,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1556228720-195a672e8a03?w=400",
                "Moisturiser jar",
            )),
            taxonomy: tx.classify(
                &["Skincare".into(), "Moisturisers".into()],
                "Drunk Elephant Protini Polypeptide Cream",
                "Drunk Elephant",
            ),
            retailer_breadcrumbs: vec!["Skincare".into(), "Moisturisers".into()],
            ingredients_raw: None,
            claims: vec![LabelClaim::CrueltyFree],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        },
    ];

    // One product with manual taxonomy override for classification demo.
    if let Some(a) = tx.assignment_for("mascara", ClassificationMethod::Manual, 1.0) {
        products.push(Product {
            id: "boots:masc-004".into(),
            name: "Maybelline Lash Sensational Sky High Mascara — Very Black".into(),
            brand: "Maybelline".into(),
            manufacturer: None,
            retailer: retailer("boots", "Boots", "boots.com"),
            retailer_sku: "masc-004".into(),
            url: "https://www.boots.com/maybelline-lash-sensational-sky-high".into(),
            gtin: Some("3017620423456".into()),
            shade: Some("Very Black".into()),
            size: Some("7.2ml".into()),
            price: Some(Money {
                minor_units: 1199,
                currency: Currency::Gbp,
            }),
            image: Some(img(
                "https://images.unsplash.com/photo-1631730486572-226f0b47afdd?w=400",
                "Mascara tube",
            )),
            taxonomy: a,
            retailer_breadcrumbs: vec!["Make-up".into(), "Eyes".into(), "Mascara".into()],
            ingredients_raw: Some(
                "Aqua, Paraffin, Potassium Cetyl Phosphate, Copernicia Cerifera Cera, \
                 Acacia Senegal Gum, Glycerin, Phenoxyethanol, Rayon, CI 77499."
                    .into(),
            ),
            claims: vec![],
            certificate_claims: vec![],
            attestations: vec![],
            acquisition: AcquisitionMethod::Manual,
            captured_at: now,
        });
    }

    products
}

pub fn build_catalog() -> Catalog {
    let cosing_map: std::collections::HashMap<String, CosIngFacts> =
        sample_cosing().into_iter().collect();
    let lookup = |name: &str, ci: Option<&str>| {
        cosing_map
            .get(name)
            .cloned()
            .or_else(|| ci.and_then(|c| cosing_map.get(&format!("CI {c}")).cloned()))
    };
    let as_of = chrono::NaiveDate::from_ymd_opt(2026, 7, 30).expect("date");

    let items: Vec<CatalogProduct> = sample_products()
        .into_iter()
        .map(|product| {
            let assessment = assess(
                &product,
                DEFAULT_PROFILE,
                as_of,
                Some(&|name, ci| lookup(name, ci)),
            );
            CatalogProduct { product, assessment }
        })
        .collect();

    Catalog {
        version: "2026.07-sample".into(),
        generated_at: Utc::now(),
        profile: DEFAULT_PROFILE.into(),
        products: items,
        retailers: vec![
            retailer("boots", "Boots", "boots.com"),
            retailer("superdrug", "Superdrug", "superdrug.com"),
            retailer("lookfantastic", "Lookfantastic", "lookfantastic.com"),
            retailer("spacenk", "Space NK", "spacenk.com"),
            retailer("beautybay", "Beauty Bay", "beautybay.com"),
            retailer("cultbeauty", "Cult Beauty", "cultbeauty.co.uk"),
            retailer(
                "hollandbarrett",
                "Holland & Barrett",
                "hollandandbarrett.com",
            ),
        ],
        taxonomy: taxonomy().clone(),
        profiles: halal_core::profiles(),
    }
}
