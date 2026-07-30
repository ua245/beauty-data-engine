# Product Taxonomy (Boots-Aligned)

Canonical category tree used for filtering and navigation. All retailer products are mapped to this taxonomy.

## Top-Level Categories

```
beauty/
├── makeup/
├── skincare/
├── fragrance/
├── haircare/
├── bath-body/
├── mens-grooming/
├── nails/
└── tools-accessories/
```

## Makeup (`beauty/makeup/`)

| Sub-type | Slug | Boots Path Example |
|----------|------|------------------|
| Face | `face` | beauty/makeup/face |
| Foundation | `foundation` | beauty/makeup/face/foundation |
| Concealer | `concealer` | beauty/makeup/face/concealer |
| Powder | `powder` | beauty/makeup/face/powder |
| Blusher | `blusher` | beauty/makeup/face/blusher |
| Bronzer | `bronzer` | beauty/makeup/face/bronzer |
| Primer | `primer` | beauty/makeup/face/primer |
| Eyes | `eyes` | beauty/makeup/eyes |
| Mascara | `mascara` | beauty/makeup/eyes/mascara |
| Eyeshadow | `eyeshadow` | beauty/makeup/eyes/eyeshadow |
| Eyeliner | `eyeliner` | beauty/makeup/eyes/eyeliner |
| Eyebrows | `eyebrows` | beauty/makeup/eyes/eyebrows |
| Lips | `lips` | beauty/makeup/lips |
| Lipstick | `lipstick` | beauty/makeup/lips/lipstick |
| Lip Gloss | `lip-gloss` | beauty/makeup/lips/lip-gloss |
| Lip Liner | `lip-liner` | beauty/makeup/lips/lip-liner |
| Lip Balm | `lip-balm` | beauty/makeup/lips/lip-balm |
| Nails | `nails` | beauty/makeup/nails |
| Nail Polish | `nail-polish` | beauty/makeup/nails/nail-polish |
| Palettes & Sets | `palettes-sets` | beauty/makeup/palettes-sets |

## Fragrance (`beauty/fragrance/`)

| Sub-type | Slug | Halal Notes |
|----------|------|-------------|
| Women's Perfume | `womens-perfume` | High alcohol content — check ethanol source |
| Men's Aftershave | `mens-aftershave` | Alcohol-based; mashbooh without cert |
| Body Mist | `body-mist` | Lower alcohol concentration |
| Eau de Parfum | `eau-de-parfum` | 15–20% fragrance oil |
| Eau de Toilette | `eau-de-toilette` | 5–15% fragrance oil |
| Gift Sets | `fragrance-gift-sets` | May contain non-halal items |

## Haircare (`beauty/haircare/`)

| Sub-type | Slug | Halal Notes |
|----------|------|-------------|
| Shampoo | `shampoo` | Check keratin, collagen, gelatin |
| Conditioner | `conditioner` | Check keratin treatments |
| Hair Mask | `hair-mask` | Often contains hydrolyzed proteins |
| Hair Oil | `hair-oil` | Generally lower risk |
| Styling | `styling` | Check alcohol in sprays/gels |
| Hair Colour | `hair-colour` | Check ammonia, resorcinol |
| Treatments | `treatments` | Keratin treatments often animal-derived |
| Scalp Care | `scalp-care` | |

## Skincare (`beauty/skincare/`)

| Sub-type | Slug |
|----------|------|
| Cleansers | `cleansers` |
| Moisturisers | `moisturisers` |
| Serums | `serums` |
| Sun Care | `sun-care` |
| Eye Care | `eye-care` |
| Toners | `toners` |
| Face Masks | `face-masks` |
| Anti-Ageing | `anti-ageing` |

## Filter Dimensions

The frontend exposes these filter axes:

```typescript
interface ProductFilters {
  product_type: ProductType[];       // makeup, fragrance, haircare, skincare
  sub_type: string[];                // lipstick, shampoo, eau-de-parfum, etc.
  retailer: Retailer[];              // boots, superdrug, lookfantastic
  brand: string[];
  halal_status: HalalStatus[];       // halal, likely_halal, mashbooh, etc.
  certified_only: boolean;           // Has official IFANCA/AFIC/MUI/HCE cert
  cert_body: CertBody[];             // Filter by specific certifier
  price_min: number;
  price_max: number;
  query: string;                       // Full-text search
}
```

## Retailer → Taxonomy Mapping

Each scraper includes a mapping table from retailer-specific category IDs/slugs to canonical taxonomy paths. Example:

```json
{
  "boots": {
    "2300180": "beauty/skincare",
    "1595015": "beauty/makeup/lips/lipstick"
  },
  "superdrug": {
    "makeup/lips": "beauty/makeup/lips",
    "fragrance/womens": "beauty/fragrance/womens-perfume"
  }
}
```
