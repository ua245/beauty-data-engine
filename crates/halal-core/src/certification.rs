//! Certificate verification: the layer that turns "the box has a halal logo on it" into a claim
//! that can be checked, dated and scoped.
//!
//! # Why a ladder rather than a boolean
//!
//! Consumer halal apps almost universally reduce certification to a badge. That loses the four
//! facts that decide whether a certificate actually means anything:
//!
//! 1. **Scope.** A body accredited for poultry slaughter tells you nothing about a lipstick.
//!    JAKIM and BPJPH both publish recognition that is explicitly per-scope, so a body recognised
//!    for food only is not recognised for cosmetics.
//! 2. **Validity window.** Certificates expire, and are suspended between audits.
//! 3. **Holder identity.** Certificates are issued to a legal entity and often to a named
//!    manufacturing site, not to a brand name on a shelf. A certificate held by a contract
//!    manufacturer does not automatically cover every brand it fills for.
//! 4. **Recognition.** A certificate from a body no regulator recognises is a private assurance.
//!
//! [`TrustRung`] encodes those facts as an ordered ladder, so the UI can say *how* strongly a
//! product is evidenced rather than just *whether* a logo exists.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A certification scope category. Kept coarse because that is the granularity at which
/// regulators publish recognition lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    Cosmetics,
    PersonalCare,
    Fragrance,
    Food,
    Beverages,
    Pharmaceutical,
    ChemicalIngredients,
    Slaughtering,
    Logistics,
}

impl Scope {
    /// Scopes that can carry a finished beauty product.
    pub fn covers_beauty(self) -> bool {
        matches!(
            self,
            Scope::Cosmetics | Scope::PersonalCare | Scope::Fragrance
        )
    }
}

/// A regulator or accreditation authority whose recognition lists we treat as authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Authority {
    /// Department of Islamic Development Malaysia, which gazettes recognised foreign bodies.
    Jakim,
    /// Indonesian halal authority; publishes the LHLN foreign-body list and a product registry.
    Bpjph,
    /// UAE Ministry of Industry and Advanced Technology; registers certification bodies.
    Moiat,
    /// Turkish Halal Accreditation Agency; accredits bodies rather than certifying products.
    Hak,
    /// Standards and Metrology Institute for Islamic Countries; publishes OIC/SMIIC standards.
    Smiic,
}

impl Authority {
    pub fn label(self) -> &'static str {
        match self {
            Authority::Jakim => "JAKIM (Malaysia)",
            Authority::Bpjph => "BPJPH (Indonesia)",
            Authority::Moiat => "MOIAT (UAE)",
            Authority::Hak => "HAK (Türkiye)",
            Authority::Smiic => "OIC/SMIIC",
        }
    }

    /// Public verification or recognition-list entry point.
    pub fn registry_url(self) -> &'static str {
        match self {
            Authority::Jakim => "https://myehalal.halal.gov.my/portal-halal/v1/index.php",
            Authority::Bpjph => "http://app.halal.go.id/dashboard/html/lhlnlist",
            Authority::Moiat => {
                "https://moiat.gov.ae/en/programs/halal/registered-halal-certification-bodies"
            }
            Authority::Hak => "https://english.hak.gov.tr/accredited-hcabs",
            Authority::Smiic => "https://smiic.org/en/project/30",
        }
    }
}

/// One authority's recognition of a certification body, valid for a scope set and a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recognition {
    pub authority: Authority,
    pub scopes: Vec<Scope>,
    /// Recognition lists are gazetted with an expiry; JAKIM's run for two years.
    pub valid_until: Option<NaiveDate>,
    /// Where the recognition was read from, so a reviewer can re-check it.
    pub source_url: String,
}

impl Recognition {
    pub fn is_current(&self, as_of: NaiveDate) -> bool {
        self.valid_until.map_or(true, |d| d >= as_of)
    }

    pub fn covers(&self, scope: Scope) -> bool {
        self.scopes.contains(&scope)
    }
}

/// A halal certification body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifierBody {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub country: String,
    pub website: String,
    /// Public directory of certified companies or products, where one exists.
    pub directory_url: Option<String>,
    /// Whether that directory can be queried programmatically. None of the bodies we surveyed
    /// publish an open API, which is why the ledger below exists.
    pub machine_readable_directory: bool,
    /// Scopes the body itself operates in.
    pub scopes: Vec<Scope>,
    /// Recognition held from regulators and accreditation authorities.
    pub recognitions: Vec<Recognition>,
    /// Regex-like shape of the body's certificate numbers, used as a cheap format check.
    /// Expressed as a human-readable hint plus an optional strict pattern.
    pub certificate_number_hint: Option<String>,
    #[serde(default)]
    pub certificate_number_pattern: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl CertifierBody {
    /// Whether any current recognition covers beauty products.
    pub fn recognised_for_beauty(&self, as_of: NaiveDate) -> Vec<&Recognition> {
        self.recognitions
            .iter()
            .filter(|r| r.is_current(as_of) && r.scopes.iter().any(|s| s.covers_beauty()))
            .collect()
    }

    pub fn operates_in_beauty(&self) -> bool {
        self.scopes.iter().any(|s| s.covers_beauty())
    }
}

/// Evidence attached to a certificate claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateEvidence {
    /// Link to the certificate document or the directory entry.
    pub url: Option<String>,
    /// SHA-256 of the certificate document as fetched, so later edits are detectable.
    pub document_sha256: Option<String>,
    /// Free-text note on how the evidence was obtained.
    pub method: String,
    pub captured_at: DateTime<Utc>,
}

impl CertificateEvidence {
    /// Hash a certificate document so the ledger entry is tied to a specific file.
    pub fn hash_document(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }
}

/// A halal certification claim attached to a product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateClaim {
    /// Id of the [`CertifierBody`] the claim names.
    pub certifier_id: String,
    pub certificate_number: Option<String>,
    /// Legal entity named on the certificate.
    pub holder: Option<String>,
    pub scopes: Vec<Scope>,
    pub issued_on: Option<NaiveDate>,
    pub expires_on: Option<NaiveDate>,
    /// True when the claim was found in the certifier's or a regulator's own directory rather
    /// than only on the brand's packaging or website.
    #[serde(default)]
    pub found_in_directory: bool,
    /// True when a regulator's product registry returned this product or holder.
    #[serde(default)]
    pub found_in_regulator_registry: bool,
    pub evidence: Option<CertificateEvidence>,
}

/// Rungs of the certificate trust ladder, ordered from weakest to strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustRung {
    /// No certification claim of any kind.
    NoClaim,
    /// A claim exists but names a body we do not hold a record for.
    UnknownBody,
    /// The named body is known but does not operate in cosmetics or personal care.
    BodyOutOfScope,
    /// The certificate's own validity window has passed.
    Expired,
    /// The certificate exists and is current, but its scope does not cover beauty products.
    ScopeMismatch,
    /// The certificate holder does not appear to be the product's brand or its manufacturer.
    HolderUnmatched,
    /// A claim naming a known, in-scope body, with no document or directory entry behind it.
    Claimed,
    /// Certificate document on file: number present, unexpired, scope covers beauty, holder
    /// reconciles with the brand.
    DocumentChecked,
    /// The certifier's own public directory lists the holder or product.
    CertifierDirectoryListed,
    /// A national regulator's registry returns the product or its holder.
    RegulatorRegistryListed,
}

impl TrustRung {
    /// Whether the rung is strong enough to describe a product as certified in the UI.
    pub fn is_certified(self) -> bool {
        self >= TrustRung::DocumentChecked
    }

    /// Whether the rung represents a failed check rather than an absence of evidence.
    pub fn is_failure(self) -> bool {
        matches!(
            self,
            TrustRung::BodyOutOfScope
                | TrustRung::Expired
                | TrustRung::ScopeMismatch
                | TrustRung::HolderUnmatched
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            TrustRung::NoClaim => "No certification claim",
            TrustRung::UnknownBody => "Certifier not recognised in our register",
            TrustRung::BodyOutOfScope => "Certifier does not cover cosmetics",
            TrustRung::Expired => "Certificate expired",
            TrustRung::ScopeMismatch => "Certificate scope excludes cosmetics",
            TrustRung::HolderUnmatched => "Certificate holder does not match the brand",
            TrustRung::Claimed => "Certification claimed, not yet evidenced",
            TrustRung::DocumentChecked => "Certificate document checked",
            TrustRung::CertifierDirectoryListed => "Listed in the certifier's directory",
            TrustRung::RegulatorRegistryListed => "Confirmed in a national halal registry",
        }
    }
}

/// Result of verifying one claim, including the reasoning so the UI can explain itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateVerification {
    pub rung: TrustRung,
    pub certifier_name: Option<String>,
    pub certifier_id: String,
    /// Authorities that currently recognise this body for beauty scopes.
    pub recognised_by: Vec<Authority>,
    /// Individual checks performed, in order, each with a pass/fail and an explanation.
    pub checks: Vec<VerificationCheck>,
    pub expires_on: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl VerificationCheck {
    fn new(name: &str, passed: bool, detail: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            passed,
            detail: detail.into(),
        }
    }
}

/// Register of certification bodies, loaded from bundled data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifierRegister {
    pub version: String,
    pub bodies: Vec<CertifierBody>,
}

const CERTIFIERS_JSON: &str = include_str!("../data/certifiers.json");

static REGISTER: once_cell::sync::Lazy<CertifierRegister> = once_cell::sync::Lazy::new(|| {
    serde_json::from_str(CERTIFIERS_JSON).expect("bundled certifiers.json is valid")
});

/// The certifier register bundled with the crate.
pub fn register() -> &'static CertifierRegister {
    &REGISTER
}

impl CertifierRegister {
    pub fn body(&self, id: &str) -> Option<&CertifierBody> {
        self.bodies.iter().find(|b| b.id == id)
    }

    /// Verify a claim against the register.
    ///
    /// `brand` and `manufacturer` are used to reconcile the certificate holder. Contract
    /// manufacturing means an exact string match is too strict, so this uses a similarity
    /// threshold and reports the outcome rather than silently accepting.
    pub fn verify(
        &self,
        claim: &CertificateClaim,
        brand: &str,
        manufacturer: Option<&str>,
        as_of: NaiveDate,
    ) -> CertificateVerification {
        let mut checks = Vec::new();

        let Some(body) = self.body(&claim.certifier_id) else {
            checks.push(VerificationCheck::new(
                "certifier known",
                false,
                format!(
                    "`{}` is not in the certifier register, so nothing about this claim can be \
                     checked.",
                    claim.certifier_id
                ),
            ));
            return CertificateVerification {
                rung: TrustRung::UnknownBody,
                certifier_name: None,
                certifier_id: claim.certifier_id.clone(),
                recognised_by: Vec::new(),
                checks,
                expires_on: claim.expires_on,
            };
        };
        checks.push(VerificationCheck::new(
            "certifier known",
            true,
            format!("{} is in the certifier register.", body.name),
        ));

        let recognised: Vec<Authority> = body
            .recognised_for_beauty(as_of)
            .iter()
            .map(|r| r.authority)
            .collect();
        checks.push(VerificationCheck::new(
            "regulator recognition",
            !recognised.is_empty(),
            if recognised.is_empty() {
                "No current recognition for cosmetics from a national authority we track."
                    .to_string()
            } else {
                format!(
                    "Currently recognised for cosmetics by {}.",
                    recognised
                        .iter()
                        .map(|a| a.label())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        ));

        let finish = |rung: TrustRung, checks: Vec<VerificationCheck>| CertificateVerification {
            rung,
            certifier_name: Some(body.name.clone()),
            certifier_id: body.id.clone(),
            recognised_by: recognised.clone(),
            checks,
            expires_on: claim.expires_on,
        };

        if !body.operates_in_beauty() {
            checks.push(VerificationCheck::new(
                "certifier scope",
                false,
                format!(
                    "{} does not list cosmetics, personal care or fragrance among its scopes.",
                    body.short_name
                ),
            ));
            return finish(TrustRung::BodyOutOfScope, checks);
        }
        checks.push(VerificationCheck::new(
            "certifier scope",
            true,
            format!("{} certifies beauty products.", body.short_name),
        ));

        // A regulator registry hit is the strongest evidence available and short-circuits the
        // document checks, because the regulator has already done them.
        if claim.found_in_regulator_registry {
            checks.push(VerificationCheck::new(
                "national registry",
                true,
                "The product or its holder is listed in a national halal registry.",
            ));
            return finish(TrustRung::RegulatorRegistryListed, checks);
        }

        if let Some(expires) = claim.expires_on {
            let valid = expires >= as_of;
            checks.push(VerificationCheck::new(
                "validity window",
                valid,
                if valid {
                    format!("Certificate valid until {expires}.")
                } else {
                    format!("Certificate expired on {expires}.")
                },
            ));
            if !valid {
                return finish(TrustRung::Expired, checks);
            }
        } else {
            checks.push(VerificationCheck::new(
                "validity window",
                false,
                "No expiry date recorded, so currency cannot be confirmed.",
            ));
        }

        let scope_ok = claim.scopes.iter().any(|s| s.covers_beauty());
        checks.push(VerificationCheck::new(
            "certificate scope",
            scope_ok,
            if scope_ok {
                "Certificate scope covers cosmetics or personal care.".to_string()
            } else {
                format!(
                    "Certificate scope is {:?}, which does not cover beauty products.",
                    claim.scopes
                )
            },
        ));
        if !scope_ok && !claim.scopes.is_empty() {
            return finish(TrustRung::ScopeMismatch, checks);
        }

        if claim.found_in_directory {
            checks.push(VerificationCheck::new(
                "certifier directory",
                true,
                format!(
                    "Listed in {}'s public directory.",
                    body.short_name
                ),
            ));
            return finish(TrustRung::CertifierDirectoryListed, checks);
        }

        let has_number = claim
            .certificate_number
            .as_deref()
            .is_some_and(|n| !n.trim().is_empty());
        checks.push(VerificationCheck::new(
            "certificate number",
            has_number,
            match claim.certificate_number.as_deref() {
                Some(n) if !n.trim().is_empty() => format!("Certificate number {n} recorded."),
                _ => "No certificate number recorded.".to_string(),
            },
        ));

        let has_document = claim
            .evidence
            .as_ref()
            .is_some_and(|e| e.document_sha256.is_some() || e.url.is_some());
        checks.push(VerificationCheck::new(
            "evidence on file",
            has_document,
            if has_document {
                "Certificate document or directory link is on file with a content hash."
                    .to_string()
            } else {
                "No document or link on file; the claim rests on packaging alone.".to_string()
            },
        ));

        let holder_ok = match claim.holder.as_deref() {
            Some(holder) => {
                let matched = entity_matches(holder, brand)
                    || manufacturer.is_some_and(|m| entity_matches(holder, m));
                checks.push(VerificationCheck::new(
                    "holder reconciliation",
                    matched,
                    if matched {
                        format!("Holder `{holder}` reconciles with the brand or its manufacturer.")
                    } else {
                        format!(
                            "Holder `{holder}` does not obviously correspond to `{brand}`. This \
                             is common with contract manufacturing and needs a human check."
                        )
                    },
                ));
                matched
            }
            None => {
                checks.push(VerificationCheck::new(
                    "holder reconciliation",
                    false,
                    "No certificate holder recorded.",
                ));
                false
            }
        };

        if has_number && has_document && holder_ok {
            return finish(TrustRung::DocumentChecked, checks);
        }
        if claim.holder.is_some() && !holder_ok && has_document {
            return finish(TrustRung::HolderUnmatched, checks);
        }
        finish(TrustRung::Claimed, checks)
    }

    /// Verify every claim on a product and return the strongest outcome, keeping the others so
    /// that a failed claim is still visible rather than hidden by a stronger one.
    pub fn verify_all(
        &self,
        claims: &[CertificateClaim],
        brand: &str,
        manufacturer: Option<&str>,
        as_of: NaiveDate,
    ) -> Vec<CertificateVerification> {
        let mut out: Vec<_> = claims
            .iter()
            .map(|c| self.verify(c, brand, manufacturer, as_of))
            .collect();
        out.sort_by(|a, b| b.rung.cmp(&a.rung));
        out
    }
}

/// Loose comparison of two legal-entity or brand strings.
///
/// Strips company suffixes and punctuation before comparing, because certificates name
/// "Al-Noor Cosmetics Manufacturing Sdn. Bhd." for a brand shown as "Al Noor".
fn entity_matches(a: &str, b: &str) -> bool {
    let na = normalise_entity(a);
    let nb = normalise_entity(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    if na == nb || na.contains(&nb) || nb.contains(&na) {
        return true;
    }
    strsim::jaro_winkler(&na, &nb) >= 0.92
}

fn normalise_entity(input: &str) -> String {
    const SUFFIXES: [&str; 14] = [
        "limited", "ltd", "plc", "llc", "inc", "incorporated", "gmbh", "sdn bhd", "sdn", "bhd",
        "pt", "srl", "sa", "bv",
    ];
    let lowered: String = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    let words: Vec<&str> = lowered
        .split_whitespace()
        .filter(|w| !SUFFIXES.contains(w))
        .filter(|w| !matches!(*w, "the" | "co" | "company" | "cosmetics" | "group"))
        .collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("date")
    }

    fn today() -> NaiveDate {
        date("2026-07-30")
    }

    fn claim(certifier: &str) -> CertificateClaim {
        CertificateClaim {
            certifier_id: certifier.to_string(),
            certificate_number: Some("21633-1/2/2/Y1".to_string()),
            holder: Some("Radiance Cosmetics Ltd".to_string()),
            scopes: vec![Scope::Cosmetics],
            issued_on: Some(date("2025-06-01")),
            expires_on: Some(date("2027-05-31")),
            found_in_directory: false,
            found_in_regulator_registry: false,
            evidence: Some(CertificateEvidence {
                url: Some("https://example.org/cert.pdf".to_string()),
                document_sha256: Some(CertificateEvidence::hash_document(b"cert")),
                method: "fetched from brand press kit".to_string(),
                captured_at: Utc::now(),
            }),
        }
    }

    #[test]
    fn bundled_register_loads_with_unique_ids() {
        let reg = register();
        let mut seen = std::collections::HashSet::new();
        for body in &reg.bodies {
            assert!(seen.insert(&body.id), "duplicate certifier id {}", body.id);
        }
        assert!(reg.bodies.len() >= 8, "expected the major bodies, got {}", reg.bodies.len());
        assert!(reg.body("hce").is_some());
        assert!(reg.body("ifanca").is_some());
    }

    #[test]
    fn a_complete_document_claim_reaches_document_checked() {
        let v = register().verify(&claim("hce"), "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::DocumentChecked);
        assert!(v.rung.is_certified());
    }

    #[test]
    fn a_regulator_registry_hit_outranks_everything() {
        let mut c = claim("hce");
        c.found_in_regulator_registry = true;
        let v = register().verify(&c, "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::RegulatorRegistryListed);
    }

    #[test]
    fn an_expired_certificate_is_a_failure_not_a_pass() {
        let mut c = claim("hce");
        c.expires_on = Some(date("2025-01-01"));
        let v = register().verify(&c, "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::Expired);
        assert!(v.rung.is_failure());
        assert!(!v.rung.is_certified());
    }

    #[test]
    fn a_food_only_scope_does_not_certify_a_lipstick() {
        let mut c = claim("hce");
        c.scopes = vec![Scope::Food, Scope::Slaughtering];
        let v = register().verify(&c, "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::ScopeMismatch);
    }

    #[test]
    fn an_unmatched_holder_is_surfaced_rather_than_accepted() {
        let mut c = claim("hce");
        c.holder = Some("Unrelated Contract Fillers GmbH".to_string());
        let v = register().verify(&c, "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::HolderUnmatched);
    }

    #[test]
    fn a_named_manufacturer_can_satisfy_holder_reconciliation() {
        let mut c = claim("hce");
        c.holder = Some("Northern Fillers Ltd".to_string());
        let v = register().verify(&c, "Radiance", Some("Northern Fillers"), today());
        assert_eq!(v.rung, TrustRung::DocumentChecked);
    }

    #[test]
    fn a_bare_claim_without_evidence_stays_on_the_claimed_rung() {
        let mut c = claim("hce");
        c.evidence = None;
        c.certificate_number = None;
        let v = register().verify(&c, "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::Claimed);
        assert!(!v.rung.is_certified());
    }

    #[test]
    fn an_unknown_body_cannot_be_checked() {
        let v = register().verify(&claim("totally-made-up"), "Radiance", None, today());
        assert_eq!(v.rung, TrustRung::UnknownBody);
        assert!(v.certifier_name.is_none());
    }

    #[test]
    fn entity_matching_tolerates_company_suffixes() {
        assert!(entity_matches("Al-Noor Cosmetics Sdn. Bhd.", "Al Noor"));
        assert!(entity_matches("Radiance Cosmetics Ltd", "Radiance"));
        assert!(!entity_matches("Radiance", "Maybelline"));
    }

    #[test]
    fn document_hashing_is_stable_and_distinguishing() {
        assert_eq!(
            CertificateEvidence::hash_document(b"a"),
            CertificateEvidence::hash_document(b"a")
        );
        assert_ne!(
            CertificateEvidence::hash_document(b"a"),
            CertificateEvidence::hash_document(b"b")
        );
    }

    #[test]
    fn recognition_expiry_is_respected() {
        let reg = register();
        let jakim_recognised = reg
            .bodies
            .iter()
            .filter(|b| !b.recognised_for_beauty(date("2050-01-01")).is_empty())
            .count();
        // Every recognition in the bundled data has an expiry well before 2050, so a far-future
        // date must clear the list rather than leaving stale recognitions in place.
        assert_eq!(jakim_recognised, 0);
    }
}
