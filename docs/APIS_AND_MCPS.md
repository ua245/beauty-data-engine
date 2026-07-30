# APIs & MCPs Required

## External APIs

### 1. EU CosIng (Ingredient Database)

| Provider | Endpoint | Auth | Use |
|----------|----------|------|-----|
| **CosIng Checker** (recommended) | `https://cosingchecker.com/api/v1/ingredients/?q={inci}` | None | INCI lookup, CAS/EC numbers, regulatory status |
| **CosIng Checker** (detail) | `https://cosingchecker.com/api/v1/ingredients/{slug}/` | None | Full ingredient profile |
| **CosIng Checker** (annexes) | `https://cosingchecker.com/api/v1/entries/?q={name}&annex=II` | None | Banned/restricted substances |
| **EC Official** (export) | `https://api.tech.ec.europa.eu/cosing20/1.0/api/annexes/export` | None | Official annex exports (II–VI) |
| **EC Web UI** (fallback) | `https://ec.europa.eu/growth/tools-databases/cosing/` | None | Manual verification |

**Integration notes:**
- Cache all CosIng responses in Redis (TTL: 30 days)
- Batch-resolve INCI names during enrichment pipeline
- Store CosIng slug, CAS, EC number, function, and restriction status per ingredient

---

### 2. Halal Certification APIs

| Body | API / Source | Auth | Coverage |
|------|-------------|------|----------|
| **BPJPH / LPPOM MUI** | [HalalCheck API](https://rapidapi.com/diofikriyanto3321/api/halalcheck-api) | RapidAPI key | Indonesian cosmetics certs (LPPOM MUI inspection body) |
| **BPJPH Public Portal** | `https://sertifikasi.halal.go.id/sertifikat/publik` | None (scrape) | Official Indonesian halal registry |
| **Halal MUI** | `https://www.halalmui.org` | None (scrape) | MUI-certified product lookup |
| **IFANCA** | `https://www.ifanca.org` | None (scrape) | US/global halal certified products |
| **AFIC** | `https://www.afic.org.au` | None (scrape) | Australian halal certified products |
| **HCE** | `https://www.halalcertification.eu` | None (scrape) | EU halal certification registry |

**HalalCheck API example:**
```http
GET https://halalcheck-api.p.rapidapi.com/v1/certificates/search
  ?query={brand_name}
  &type=product_name
  &fetch_detail=true
Headers:
  X-RapidAPI-Key: {key}
  X-RapidAPI-Host: halalcheck-api.p.rapidapi.com
```

---

### 3. UK Retailer Data (No Public APIs — Scraping Required)

| Retailer | Method | Key Endpoints |
|----------|--------|---------------|
| **Boots** | POST + HTML | `ProductListingViewRedesign`, category PDPs |
| **Superdrug** | HTML + JSON-LD | Category pages, product detail pages |
| **Look Fantastic** | HTML | `/c/makeup/`, `/c/fragrance/`, `/c/hair/` |
| **Cult Beauty** | HTML / Shopify JSON | Product pages with INCI in description |
| **Space NK** | HTML | Beauty category tree |

**Managed scraping alternatives (for dev/prototyping):**

| Service | Actor / Product | Notes |
|---------|----------------|-------|
| **Apify** | `dromb/boots-uk-product-search-catalog-unofficial` | Boots categories, search, item detail |
| **Apify** | `stealth_mode/boots-product-search-scraper` | Boots search with variants, images |
| **ScrapeIt** | Boots Data Scraper | Managed service with ingredients |

---

### 4. Supporting APIs

| API | Purpose | Auth |
|-----|---------|------|
| **Open Food Facts** | Barcode → ingredients (cross-ref) | None |
| **EAN Search** | Barcode lookup | API key (optional) |
| **Cloudinary / S3** | Image caching for scraped product photos | API key |
| **Meilisearch** | Full-text product search | Master key |

---

## MCP Servers (Recommended)

These MCP servers integrate with Cursor agents for development and ongoing maintenance:

| MCP Server | Use in This Project |
|------------|---------------------|
| **Browser** | Live retailer page inspection, scrape selector discovery, certification portal verification |
| **cursor-cloud** | Cloud agent orchestration, CI monitoring |
| **Apify MCP** (add) | Managed Boots/Superdrug scraping without custom infra |
| **PostgreSQL MCP** (add) | Direct DB queries during development |
| **Fetch / Web MCP** (add) | CosIng and certification page fetching |

### Browser MCP — Primary Use Cases

1. **Scraper development** — Navigate Boots category pages, inspect network requests (`ProductListingViewRedesign`), identify CSS selectors for ingredients/images.
2. **Certification verification** — Search IFANCA/AFIC/HCE product databases interactively.
3. **UI testing** — Screenshot product cards, verify halal badge rendering, test filters.

### Suggested Custom MCP (Build Later)

A `halal-beauty-mcp` server exposing tools:

```
search_products(query, filters)     → Query product catalogue
get_product(id)                     → Full product with ingredients + halal score
check_ingredient(inci_name)           → CosIng + halal status for single ingredient
verify_certification(brand, product)  → Cross-ref all cert bodies
trigger_scrape(retailer, category)    → Enqueue scrape job
```

---

## Environment Variables

```env
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/halal_beauty
REDIS_URL=redis://localhost:6379
MEILI_URL=http://localhost:7700
MEILI_MASTER_KEY=...

# External APIs
RAPIDAPI_KEY=...                    # HalalCheck API
COSING_API_BASE=https://cosingchecker.com/api/v1

# Image storage
S3_BUCKET=halal-beauty-images
S3_REGION=eu-west-2

# Scraping
SCRAPE_RATE_LIMIT_MS=2000
SCRAPE_USER_AGENT=HalalBeautyBot/1.0

# Server
API_HOST=0.0.0.0
API_PORT=8080
```

---

## Rate Limits & Compliance

- **CosIng Checker**: No auth required; cache aggressively (max 1 req/ingredient/day)
- **HalalCheck API**: RapidAPI quota; batch brand lookups nightly
- **Retailer scraping**: 2s delay between requests; respect `robots.txt`; use rotating user agents
- **Certification scrapes**: Weekly refresh; store cert expiry dates
- **Legal**: Display disclaimer that ingredient analysis is informational, not a fatwa; always prefer official certification
