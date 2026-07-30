use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyNode {
    pub slug: String,
    pub label: String,
    pub path: String,
    pub children: Vec<TaxonomyNode>,
}

pub fn boots_taxonomy() -> Vec<TaxonomyNode> {
    vec![
        TaxonomyNode {
            slug: "makeup".into(),
            label: "Makeup".into(),
            path: "beauty/makeup".into(),
            children: vec![
                leaf("face", "Face", "beauty/makeup/face"),
                leaf("foundation", "Foundation", "beauty/makeup/face/foundation"),
                leaf("concealer", "Concealer", "beauty/makeup/face/concealer"),
                leaf("lipstick", "Lipstick", "beauty/makeup/lips/lipstick"),
                leaf("mascara", "Mascara", "beauty/makeup/eyes/mascara"),
                leaf("eyeshadow", "Eyeshadow", "beauty/makeup/eyes/eyeshadow"),
                leaf("nail-polish", "Nail Polish", "beauty/makeup/nails/nail-polish"),
            ],
        },
        TaxonomyNode {
            slug: "fragrance".into(),
            label: "Fragrance".into(),
            path: "beauty/fragrance".into(),
            children: vec![
                leaf("womens-perfume", "Women's Perfume", "beauty/fragrance/womens-perfume"),
                leaf("mens-aftershave", "Men's Aftershave", "beauty/fragrance/mens-aftershave"),
                leaf("eau-de-parfum", "Eau de Parfum", "beauty/fragrance/eau-de-parfum"),
                leaf("eau-de-toilette", "Eau de Toilette", "beauty/fragrance/eau-de-toilette"),
                leaf("body-mist", "Body Mist", "beauty/fragrance/body-mist"),
            ],
        },
        TaxonomyNode {
            slug: "haircare".into(),
            label: "Haircare".into(),
            path: "beauty/haircare".into(),
            children: vec![
                leaf("shampoo", "Shampoo", "beauty/haircare/shampoo"),
                leaf("conditioner", "Conditioner", "beauty/haircare/conditioner"),
                leaf("hair-mask", "Hair Mask", "beauty/haircare/hair-mask"),
                leaf("hair-oil", "Hair Oil", "beauty/haircare/hair-oil"),
                leaf("styling", "Styling", "beauty/haircare/styling"),
                leaf("hair-colour", "Hair Colour", "beauty/haircare/hair-colour"),
                leaf("treatments", "Treatments", "beauty/haircare/treatments"),
            ],
        },
        TaxonomyNode {
            slug: "skincare".into(),
            label: "Skincare".into(),
            path: "beauty/skincare".into(),
            children: vec![
                leaf("cleansers", "Cleansers", "beauty/skincare/cleansers"),
                leaf("moisturisers", "Moisturisers", "beauty/skincare/moisturisers"),
                leaf("serums", "Serums", "beauty/skincare/serums"),
                leaf("sun-care", "Sun Care", "beauty/skincare/sun-care"),
                leaf("face-masks", "Face Masks", "beauty/skincare/face-masks"),
            ],
        },
    ]
}

fn leaf(slug: &str, label: &str, path: &str) -> TaxonomyNode {
    TaxonomyNode {
        slug: slug.into(),
        label: label.into(),
        path: path.into(),
        children: vec![],
    }
}

pub fn all_sub_types() -> Vec<String> {
    boots_taxonomy()
        .into_iter()
        .flat_map(|node| {
            node.children
                .into_iter()
                .map(|child| child.slug)
                .collect::<Vec<_>>()
        })
        .collect()
}
