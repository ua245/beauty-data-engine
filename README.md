# Halal Beauty Engine

A Rust-backed halal beauty product engine for UK retailers (Boots, Superdrug, Lookfantastic, Space NK, Beauty Bay, and more), with EU **CosIng** ingredient enrichment, halal likelihood scoring, and a **certification trust-ladder** for IFANCA, HCE, JAKIM, BPJPH and other bodies.

This repository ships a **sample dashboard** ready to deploy on **Vercel**, plus the Rust core and scraper foundations.

## Stack

| Layer | Tech |
|-------|------|
| Core engine | Rust (`halal-core`) — INCI parsing, taxonomy, lexicon, scoring, certification ladder |
| Ingestion | Rust (`halal-scraper`) — sample export + optional live scrape (`--features live`) |
| Dashboard | React + Vite (`web/`) |
| Deploy | Vercel (static) |

## Palette

```css
--cotton-rose: #efc7c2;
--powder-petal: #ffe5d4;
--ash-grey: #bfd3c1;
--muted-teal: #68a691;
--mauve-shadow: #694f5d;
```

## Quick start

### 1. Export sample catalogue (Rust)

```bash
cargo run -p halal-scraper -- export-sample
# writes web/public/data/catalog.json
```

### 2. Run the dashboard locally

```bash
cd web
npm install
npm run dev
```

Open http://localhost:5173

### 3. Deploy to Vercel

1. Import this repo in [Vercel](https://vercel.com/new)
2. Root `vercel.json` is already configured (`web/dist` output)
3. Deploy — the committed `web/public/data/catalog.json` powers the sample UI

To refresh data before deploy:

```bash
cargo run -p halal-scraper -- export-sample
git add web/public/data/catalog.json && git commit -m "Refresh sample catalogue"
```

## Live scraping (optional)

Enable the probe command (requires Rust 1.85+ toolchain for full dependency tree, or build with `--features live` on a newer host):

```bash
cargo run -p halal-scraper --features live -- probe \
  --retailer beautybay \
  --sku bb-3310 \
  "https://www.beautybay.com/p/beauty-bay/brighten-hydrate-serum/"
```

**Legal note:** Prefer affiliate feeds (Awin, Rakuten, Partnerize) for SKU metadata and images. Full INCI lists usually require PDP parsing — respect retailer T&Cs and database rights.

## Certification trust ladder

Certificates are not treated as a boolean. Each claim is scored through rungs:

1. **Regulator registry listed** (BPJPH `cek halal`, JAKIM MYeHALAL)
2. **Certifier directory listed** (IFANCA product search)
3. **Document checked** (number + holder + validity + scope)
4. **Claimed** (logo only)
5. **Failures:** expired, scope mismatch, body out of scope, unknown body

## MCP servers & APIs (recommended)

### Data ingestion
| MCP / API | Use |
|-----------|-----|
| [Firecrawl MCP](https://github.com/mendableai/firecrawl-mcp-server) | Structured PDP extraction |
| [Playwright MCP](https://github.com/microsoft/playwright-mcp) | JS-rendered retailer pages |
| [MCP Fetch](https://github.com/modelcontextprotocol/servers/tree/main/src/fetch) | robots.txt, JSON-LD |
| **Awin Publisher API** | Boots, LF, Cult Beauty, Space NK, Beauty Bay feeds |
| **Rakuten Product Catalog** | Superdrug |
| **Partnerize Feeds** | Sephora UK |
| **Open Beauty Facts API** | Barcode-linked INCI (ODbL) |
| **EU CosIng** | Ingredient glossary + annexes (CC BY 4.0) |
| **PubChem PUG-REST** | CAS / synonym resolution |

### Storage & ops
| MCP | Use |
|-----|-----|
| [DBHub](https://github.com/bytebase/dbhub) | Postgres / SQLite gateway |
| [Neon MCP](https://github.com/neondatabase/mcp-server-neon) | Serverless Postgres |
| [Qdrant MCP](https://github.com/qdrant/mcp-server-qdrant) | Ingredient similarity search |
| [Sentry MCP](https://docs.sentry.io/product/sentry-mcp/) | Scraper error tracking |
| [Vercel MCP](https://github.com/vercel/mcp-handler) | Dashboard deploys |

## Project layout

```
crates/halal-core/     Domain, lexicon, scoring, certification
crates/halal-scraper/  Sample export + live scrape (feature flag)
web/                   Vite React dashboard
web/public/data/       Generated catalog.json
```

## Tests

```bash
cargo test -p halal-core
```

## Licence

AGPL-3.0-or-later
