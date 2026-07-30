//! Parsing of on-pack / on-page cosmetic ingredient declarations into structured INCI tokens.
//!
//! Retailer product pages are not a clean data source. The same declaration appears as
//! `Ingredients: Aqua, Glycerin...`, as an all-caps block, with `[+/-]` shade-variant colourant
//! lists, with L'Oréal `F.I.L.` batch codes, with `(nano)` suffixes required by Article 19(1)(g)
//! of Regulation (EC) No 1223/2009, and with HTML entities left in place. This module reduces all
//! of that to a positional list of normalised names that can be matched against CosIng.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

/// A single ingredient token recovered from a declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedIngredient {
    /// Zero-based position within the declaration. Descending-weight order is only meaningful
    /// above roughly 1% w/w, so consumers should not over-interpret later positions.
    pub position: usize,
    /// The token exactly as it appeared, after whitespace and entity clean-up only.
    pub raw: String,
    /// Upper-cased, punctuation-normalised name used as the CosIng lookup key.
    pub normalised: String,
    /// Text found in parentheses that reads as a common/trade name rather than an INCI name,
    /// e.g. `Oryza Sativa (Rice) Bran Water` yields `RICE`.
    pub qualifiers: Vec<String>,
    /// Colour Index number without the `CI` prefix, e.g. `75470` for carmine.
    pub ci_number: Option<String>,
    /// Declared as a nanomaterial.
    pub nano: bool,
    /// Part of a `may contain` / `+/-` shade-variant block, so presence in a specific unit is
    /// not guaranteed.
    pub may_contain: bool,
    /// Concentration stated alongside the name, e.g. `Niacinamide 10%`.
    pub declared_percent: Option<f32>,
}

/// The outcome of parsing one declaration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IngredientList {
    pub ingredients: Vec<ParsedIngredient>,
    /// Tokens that survived splitting but do not look like ingredient names at all. Kept so that
    /// ingestion quality can be audited rather than silently degrading.
    pub discarded: Vec<String>,
    /// True when a `may contain` / `+/-` block was present.
    pub has_shade_variants: bool,
    /// True when the declaration hides composition behind an umbrella fragrance term.
    pub has_undisclosed_fragrance: bool,
}

impl IngredientList {
    pub fn is_empty(&self) -> bool {
        self.ingredients.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ingredients.len()
    }

    /// Ingredients guaranteed present in the unit purchased, i.e. excluding `+/-` blocks.
    pub fn definite(&self) -> impl Iterator<Item = &ParsedIngredient> {
        self.ingredients.iter().filter(|i| !i.may_contain)
    }

    pub fn contains_normalised(&self, name: &str) -> bool {
        self.ingredients.iter().any(|i| i.normalised == name)
    }
}

static HEADER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^\s*(?:full\s+)?(?:list\s+of\s+)?(?:key\s+|active\s+|inactive\s+|other\s+|core\s+)?(?:ingredients?|inci(?:\s+list)?|composition|contains|zutaten|ingr[ée]dients)\s*(?:list|declaration)?\s*:\s*",
    )
    .expect("header regex")
});

static MAY_CONTAIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)[\[\(]?\s*(?:may\s+(?:also\s+)?contain(?:s)?|\+\s*/\s*[-\u{2212}\u{2013}]|\u{00b1})\s*[\]\)]?\s*[:\-]?\s*",
    )
    .expect("may-contain regex")
});

static FIL_CODE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\(?\s*F\.?\s*I\.?\s*L\.?\s*(?:CODE)?\s*[#:]?\s*[A-Z0-9]+\s*/?\s*\d*\s*\)?")
        .expect("fil regex")
});

static PERCENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(?\s*(\d{1,3}(?:[.,]\d{1,2})?)\s*%\s*\)?").expect("percent regex"));

static CI_NUMBER: Lazy<Regex> = Lazy::new(|| {
    // `CI 75470`, `C.I.75470`, `Cl 77491` (lower-case L substituted for I by OCR), `CI77491:2`.
    Regex::new(r"(?i)\bC\.?\s*[IL1]\.?\s*(\d{5})(?::\s*\d+)?\b").expect("ci regex")
});

static NANO: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\(\s*nano\s*\)").expect("nano regex"));

static HTML_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<[^>]+>").expect("html regex"));

static WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").expect("ws regex"));

/// Trailing marketing annotations: asterisks, daggers, trademark marks, footnote digits.
static NOISE_EDGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^[\s\-\u{2013}\u{2022}*\u{2020}\u{2021}\d.)\]]+)|(?:[\s.*\u{2020}\u{2021}\u{00ae}\u{2122}\u{00a9}]+$)")
        .expect("noise regex")
});

/// Phrases that indicate the block is prose, not a declaration.
const PROSE_MARKERS: [&str; 12] = [
    "we recommend",
    "apply to",
    "suitable for",
    "please note",
    "for best results",
    "patch test",
    "dermatologically tested",
    "how to use",
    "clinically proven",
    "our formula",
    "if irritation",
    "keep out of reach",
];

/// Umbrella fragrance terms that legally stand in for an undisclosed mixture.
const FRAGRANCE_UMBRELLA: [&str; 4] = ["PARFUM", "FRAGRANCE", "AROMA", "FLAVOR"];

/// Expand HTML entities that commonly survive scraping without pulling in a full HTML crate.
fn decode_entities(input: &str) -> String {
    let mut out = input.to_string();
    for (from, to) in [
        ("&amp;", "&"),
        ("&nbsp;", " "),
        ("&#160;", " "),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&apos;", "'"),
        ("&rsquo;", "'"),
        ("&ndash;", "-"),
        ("&mdash;", "-"),
        ("&hellip;", "..."),
        ("&plusmn;", "±"),
    ] {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

/// Bring punctuation used interchangeably by different label printers to one form.
fn canonical_punctuation(input: &str) -> String {
    input
        .nfkc()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{201b}' | '\u{02bc}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' | '\u{2010}' | '\u{2011}' => '-',
            '\u{00a0}' | '\u{2007}' | '\u{202f}' | '\n' | '\r' | '\t' => ' ',
            '\u{ff0c}' | '\u{060c}' => ',',
            '\u{ff1b}' => ';',
            '\u{2027}' | '\u{00b7}' | '\u{2022}' => ',',
            other => other,
        })
        .collect()
}

/// Normalise a single ingredient name to the form used as a CosIng lookup key.
///
/// CosIng publishes glossary (INCI) names in upper case, so upper-casing is the correct
/// canonicalisation rather than an arbitrary choice. Internal slashes are preserved because
/// `CAPRYLIC/CAPRIC TRIGLYCERIDE` is one substance, not two.
pub fn normalise_inci(name: &str) -> String {
    let cleaned = canonical_punctuation(&decode_entities(name));
    let cleaned = NOISE_EDGE.replace_all(&cleaned, "");
    let cleaned = WHITESPACE.replace_all(cleaned.trim(), " ");
    let mut out = String::with_capacity(cleaned.len());
    for ch in cleaned.chars() {
        match ch {
            c if c.is_alphanumeric() => out.extend(c.to_uppercase()),
            ' ' | '-' | '/' | '(' | ')' | ',' | '\'' | '.' | '+' | ':' => out.push(ch),
            _ => {}
        }
    }
    let out = WHITESPACE.replace_all(out.trim(), " ").to_string();
    out.trim_matches(|c: char| c == '.' || c == ',' || c == ' ')
        .to_string()
}

/// Split on `,` and `;` while ignoring separators nested inside brackets, so that
/// `Rosa Canina (Rosehip, Wild Rose) Extract` stays intact.
fn split_top_level(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    for ch in input.chars() {
        match ch {
            '(' => {
                depth_paren += 1;
                current.push(ch);
            }
            ')' => {
                depth_paren = (depth_paren - 1).max(0);
                current.push(ch);
            }
            '[' => {
                depth_bracket += 1;
                current.push(ch);
            }
            ']' => {
                depth_bracket = (depth_bracket - 1).max(0);
                current.push(ch);
            }
            ',' | ';' if depth_paren == 0 && depth_bracket == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            // A period followed by a space is a separator on labels that use full stops, but
            // `F.I.L.` and `C.I.` have already been rewritten by the time we get here.
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// Pull parenthetical content out of a token, returning the bare name and the qualifiers.
fn extract_qualifiers(token: &str) -> (String, Vec<String>) {
    let mut bare = String::with_capacity(token.len());
    let mut qualifiers = Vec::new();
    let mut buffer = String::new();
    let mut depth = 0i32;
    for ch in token.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth == 1 {
                    buffer.clear();
                } else {
                    buffer.push(ch);
                }
            }
            ')' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let q = buffer.trim();
                    if !q.is_empty() {
                        qualifiers.push(q.to_string());
                    }
                } else {
                    buffer.push(ch);
                }
            }
            _ if depth > 0 => buffer.push(ch),
            _ => bare.push(ch),
        }
    }
    (bare, qualifiers)
}

/// True when a token plausibly names an ingredient. Rejects sentences, addresses and
/// leftover navigation text.
fn looks_like_ingredient(token: &str) -> bool {
    let trimmed = token.trim();
    if trimmed.len() < 2 || trimmed.len() > 160 {
        return false;
    }
    if !trimmed.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    let lower = trimmed.to_lowercase();
    if PROSE_MARKERS.iter().any(|m| lower.contains(m)) {
        return false;
    }
    // INCI names are noun phrases. More than eight words is almost always a sentence,
    // with the exception of long botanical or polymer names which still stay under twelve.
    let words = trimmed.split_whitespace().count();
    if words > 12 {
        return false;
    }
    // Sentences contain verbs we can cheaply detect via common function words.
    const FUNCTION_WORDS: [&str; 10] = [
        " the ", " is ", " are ", " your ", " you ", " and/or the ", " with a ", " this ", " that ",
        " from our ",
    ];
    let padded = format!(" {lower} ");
    if FUNCTION_WORDS.iter().any(|w| padded.contains(w)) {
        return false;
    }
    true
}

/// Parse a raw ingredient declaration.
///
/// Returns an empty list rather than an error for unusable input: retailers frequently put
/// "See packaging for ingredients" in the same field, and that is a data-coverage fact rather
/// than a parse failure.
pub fn parse_ingredient_list(raw: &str) -> IngredientList {
    let mut list = IngredientList::default();

    let text = HTML_TAG.replace_all(raw, " ");
    let text = canonical_punctuation(&decode_entities(&text));
    let text = FIL_CODE.replace_all(&text, " ");
    let text = HEADER.replace(&text, "").to_string();
    if text.trim().is_empty() {
        return list;
    }

    // Split the declaration into the always-present part and the shade-variant part.
    let (definite_part, variant_part) = match MAY_CONTAIN.find(&text) {
        Some(m) => {
            list.has_shade_variants = true;
            (text[..m.start()].to_string(), Some(text[m.end()..].to_string()))
        }
        None => (text.clone(), None),
    };

    let mut position = 0usize;
    for (segment, may_contain) in [(Some(definite_part), false), (variant_part, true)] {
        let Some(segment) = segment else { continue };
        // Strip any repeated `may contain` markers inside the variant block.
        let segment = MAY_CONTAIN.replace_all(&segment, ", ").to_string();
        for token in split_top_level(&segment) {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if !looks_like_ingredient(token) {
                list.discarded.push(token.to_string());
                continue;
            }

            let nano = NANO.is_match(token);
            let token_no_nano = NANO.replace_all(token, " ").to_string();

            let declared_percent = PERCENT.captures(&token_no_nano).and_then(|c| {
                c.get(1)
                    .and_then(|m| m.as_str().replace(',', ".").parse::<f32>().ok())
            });
            let token_no_pct = PERCENT.replace_all(&token_no_nano, " ").to_string();

            let ci_number = CI_NUMBER
                .captures(&token_no_pct)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

            let (bare, qualifiers) = extract_qualifiers(&token_no_pct);
            // For a pure colourant token the CI number *is* the INCI name, so prefer it.
            let normalised = match (&ci_number, normalise_inci(&bare)) {
                (Some(ci), bare_norm) if bare_norm.is_empty() || bare_norm.starts_with("CI ") => {
                    format!("CI {ci}")
                }
                (Some(ci), bare_norm) if bare_norm.len() <= 3 => format!("CI {ci}"),
                (_, bare_norm) => bare_norm,
            };
            if normalised.is_empty() {
                list.discarded.push(token.to_string());
                continue;
            }

            let qualifiers: Vec<String> = qualifiers
                .iter()
                .map(|q| normalise_inci(q))
                .filter(|q| !q.is_empty() && !q.starts_with("CI "))
                .collect();

            if FRAGRANCE_UMBRELLA.contains(&normalised.as_str()) {
                list.has_undisclosed_fragrance = true;
            }

            list.ingredients.push(ParsedIngredient {
                position,
                raw: WHITESPACE.replace_all(token, " ").trim().to_string(),
                normalised,
                qualifiers,
                ci_number,
                nano,
                may_contain,
                declared_percent,
            });
            position += 1;
        }
    }

    list
}

/// Heuristic gate used during ingestion to decide whether a scraped block is an ingredient
/// declaration at all. Requires several comma-separated tokens and a recognisable anchor.
pub fn is_probable_declaration(raw: &str) -> bool {
    let parsed = parse_ingredient_list(raw);
    if parsed.ingredients.len() < 3 {
        return false;
    }
    const ANCHORS: [&str; 14] = [
        "AQUA",
        "WATER",
        "GLYCERIN",
        "PARFUM",
        "FRAGRANCE",
        "ALCOHOL DENAT",
        "ALCOHOL",
        "TALC",
        "MICA",
        "DIMETHICONE",
        "CETEARYL ALCOHOL",
        "SODIUM LAURETH SULFATE",
        "TITANIUM DIOXIDE",
        "PHENOXYETHANOL",
    ];
    parsed
        .ingredients
        .iter()
        .any(|i| ANCHORS.contains(&i.normalised.as_str()) || i.ci_number.is_some())
        || parsed.ingredients.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_declaration() {
        let list = parse_ingredient_list("Ingredients: Aqua, Glycerin, Cetearyl Alcohol, Parfum.");
        assert_eq!(list.len(), 4);
        assert_eq!(list.ingredients[0].normalised, "AQUA");
        assert_eq!(list.ingredients[3].normalised, "PARFUM");
        assert!(list.has_undisclosed_fragrance);
        assert!(!list.has_shade_variants);
    }

    #[test]
    fn keeps_slash_compounds_whole() {
        let list = parse_ingredient_list("Caprylic/Capric Triglyceride, Acrylates/C10-30 Alkyl Acrylate Crosspolymer");
        assert_eq!(list.len(), 2);
        assert_eq!(list.ingredients[0].normalised, "CAPRYLIC/CAPRIC TRIGLYCERIDE");
    }

    #[test]
    fn does_not_split_inside_parentheses() {
        let list = parse_ingredient_list("Rosa Canina (Rosehip, Wild Rose) Fruit Oil, Tocopherol");
        assert_eq!(list.len(), 2);
        assert_eq!(list.ingredients[0].normalised, "ROSA CANINA FRUIT OIL");
        assert!(list.ingredients[0]
            .qualifiers
            .contains(&"ROSEHIP, WILD ROSE".to_string()));
    }

    #[test]
    fn separates_shade_variant_block() {
        let list = parse_ingredient_list(
            "Talc, Mica, Silica, May Contain (+/-): CI 77491, CI 77492, CI 75470",
        );
        assert!(list.has_shade_variants);
        assert_eq!(list.definite().count(), 3);
        let carmine = list
            .ingredients
            .iter()
            .find(|i| i.ci_number.as_deref() == Some("75470"))
            .expect("carmine token");
        assert!(carmine.may_contain);
        assert_eq!(carmine.normalised, "CI 75470");
    }

    #[test]
    fn normalises_colour_index_variants() {
        for input in ["C.I. 75470", "CI75470", "Cl 75470", "ci 75470:1"] {
            let list = parse_ingredient_list(&format!("Talc, Mica, Silica, {input}"));
            let last = list.ingredients.last().expect("token");
            assert_eq!(last.ci_number.as_deref(), Some("75470"), "input: {input}");
        }
    }

    #[test]
    fn keeps_named_colourant_and_its_index() {
        let list = parse_ingredient_list("Talc, Mica, Carmine (CI 75470)");
        let last = list.ingredients.last().expect("token");
        assert_eq!(last.normalised, "CARMINE");
        assert_eq!(last.ci_number.as_deref(), Some("75470"));
    }

    #[test]
    fn strips_fil_codes_and_percentages() {
        let list = parse_ingredient_list(
            "Aqua, Niacinamide 10%, Glycerin, Tocopherol (F.I.L. B123456/1)",
        );
        assert_eq!(list.len(), 4);
        assert_eq!(list.ingredients[1].normalised, "NIACINAMIDE");
        assert_eq!(list.ingredients[1].declared_percent, Some(10.0));
        assert_eq!(list.ingredients[3].normalised, "TOCOPHEROL");
    }

    #[test]
    fn flags_nanomaterials() {
        let list = parse_ingredient_list("Aqua, Titanium Dioxide (nano), Glycerin, Silica");
        assert!(list.ingredients[1].nano);
        assert_eq!(list.ingredients[1].normalised, "TITANIUM DIOXIDE");
    }

    #[test]
    fn handles_semicolons_and_html() {
        let list = parse_ingredient_list("<p>Ingredients:</p> Aqua; Glycerin&nbsp;; Parfum");
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn rejects_marketing_prose() {
        let list = parse_ingredient_list(
            "Ingredients: Aqua, Glycerin, We recommend you apply to damp skin, Parfum",
        );
        assert_eq!(list.len(), 3);
        assert_eq!(list.discarded.len(), 1);
    }

    #[test]
    fn declaration_gate_rejects_short_prose() {
        assert!(!is_probable_declaration("See packaging for full ingredients."));
        assert!(is_probable_declaration(
            "Aqua, Glycerin, Cetearyl Alcohol, Parfum, Citric Acid"
        ));
    }
}
