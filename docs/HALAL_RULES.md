# Halal Ingredient Rules Engine

Rules applied per INCI name during enrichment. Configurable via `HalalStandard` profile.

## Standards Profiles

| Profile | Alcohol Tolerance | Carmine | Animal Derivatives |
|---------|-------------------|---------|-------------------|
| `strict` | Zero ethanol/denat | Haram | Must be certified halal source |
| `jakim` | ≤0.5% non-khamr ethanol | Haram | Halal-slaughtered or plant/marine |
| `bpjph` | Zero ethanol in certified | Haram | Full traceability required |
| `permissive` | Topical ethanol OK if synthetic | Mashbooh | Plant/marine OK; unspecified = mashbooh |

## Ingredient Classification

### Haram (Definite — score penalty: 40)

| INCI / Alias | Reason | Common In |
|--------------|--------|-----------|
| Carmine, CI 75470, Cochineal, Natural Red 4 | Insect-derived | Lipstick, blush, eyeshadow |
| Gelatin (unsourced) | Porcine default | Face masks, nail polish |
| Hydrolyzed Collagen (porcine) | Pig-derived | Anti-ageing creams |
| Lard, Tallow (porcine) | Pig fat | Soaps (rare in UK cosmetics) |
| Placenta Extract | Human/animal origin | Anti-ageing |
| Squalene (shark) | Non-halal animal | Moisturisers |

### Mashbooh (Doubtful — score penalty: 15)

| INCI / Alias | Reason | Resolution |
|--------------|--------|------------|
| Glycerin, Glycerol | Animal or plant source | Check manufacturer; plant = halal |
| Stearic Acid | Animal or plant | Plant-derived = halal |
| Collagen (unsourced) | Bovine/porcine/marine | Marine or certified = halal |
| Keratin | Animal protein | Plant keratin or certified |
| Elastin | Often bovine | Check source |
| Lanolin | Sheep wool wax | Majority view: permissible |
| Alcohol Denat., Ethanol, SD Alcohol | Disputed; often synthetic | Synthetic topical = mashbooh/permissible |
| Isopropyl Alcohol | Synthetic | Generally mashbooh in strict |
| Fragrance, Parfum | May contain alcohol/animal musk | Requires cert or brand disclosure |
| Beeswax, Cera Alba | Insect product | Majority: permissible |

### Halal (Safe — no penalty)

| INCI / Alias | Notes |
|--------------|-------|
| Cetyl Alcohol, Stearyl Alcohol, Cetearyl Alcohol | Fatty alcohols (plant) |
| Caprylic/Capric Triglyceride | Usually coconut/palm |
| Aloe Barbadensis Leaf Juice | Plant |
| Tocopherol (Vitamin E) | Usually plant/soy |
| Niacinamide | Synthetic |
| Hyaluronic Acid | Synthetic/fermentation |
| Aqua | Water |
| Kaolin, Mica, Talc | Minerals |
| Iron Oxides (CI 77491, etc.) | Mineral pigments |

## Category-Specific Rules

### Perfumes & Fragrances
- Flag any product with `Alcohol Denat.` or `Ethanol` in top 3 ingredients
- `Parfum` alone = mashbooh (composition unknown)
- Attar/oil-based fragrances (no alcohol) = likely_halal if no other flags

### Haircare
- Flag `Hydrolyzed Keratin`, `Hydrolyzed Wheat Protein` (usually OK), animal keratin
- Hair dyes: check for resorcinol (synthetic, generally OK)
- "Keratin treatment" products = high mashbooh risk

### Makeup
- Lip products: always check for Carmine/CI 75470
- Mascara: check beeswax (permissible), shellac (insect — mashbooh)
- Foundation: check collagen, glycerin, stearic acid

## Certification Override

When an active certification from IFANCA, AFIC, LPPOM MUI, or HCE matches:

```
halal_status = "halal"
halal_score = max(halal_score, 95)
certifications = [{ body, cert_number, expiry, status: "active" }]
```

Expired or revoked certs are ignored; product falls back to ingredient-based scoring.
