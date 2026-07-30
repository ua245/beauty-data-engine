# Halal Beauty Data Engine

A UK beauty product intelligence platform that scrapes retailer catalogues (Boots, Superdrug, and more), enriches ingredients via the EU **CosIng** database, scores halal compliance, and surfaces official certification from **IFANCA**, **AFIC**, **LPPOM MUI**, and **HCE**.

![Halal Beauty Engine](docs/screenshot-placeholder.png)

## Features

- **Retailer scraping** — Boots-aligned taxonomy for makeup, fragrance, haircare, and skincare
- **CosIng EU enrichment** — INCI ingredient resolution with CAS/EC numbers and regulatory data
- **Halal scoring engine** — Rule-based classification (halal / mashbooh / haram) with configurable standards
- **Certification registry** — Cross-reference against IFANCA, AFIC, LPPOM MUI, and HCE
- **Sleek UI** — Product cards with halal score, certification badges, ingredient breakdown, and filters

## Quick Start

### Prerequisites

- Rust 1.84+
- Node.js 22+
- Docker & Docker Compose (optional)

### Run locally

```bash
# Backend
cd backend && cargo run

# Frontend (separate terminal)
cd frontend && npm install && npm run dev
```

Open http://localhost:3000 — the Vite dev server proxies `/api` to the Rust backend on port 8080.

### Run with Docker

```bash
docker compose up --build
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full system design.

```
beauty-data-engine/
├── backend/          # Rust (Axum) API + halal engine
├── frontend/         # React + TypeScript + Vite
├── docs/             # Architecture, APIs, taxonomy, halal rules
└── docker-compose.yml
```

## APIs & MCPs

See [docs/APIS_AND_MCPS.md](docs/APIS_AND_MCPS.md) for all external APIs and recommended MCP servers.

| API | Purpose |
|-----|---------|
| CosIng Checker | EU ingredient lookup |
| HalalCheck (RapidAPI) | BPJPH / LPPOM MUI certification |
| Boots/Superdrug | Scraped (no public API) |
| IFANCA / AFIC / HCE | Certification verification (scrape) |

## Halal Rules

See [docs/HALAL_RULES.md](docs/HALAL_RULES.md) for ingredient classification rules covering makeup, perfumes, and haircare.

## Product Taxonomy

See [docs/TAXONOMY.md](docs/TAXONOMY.md) for the Boots-aligned category tree.

## Development Workflow

| Task | Model |
|------|-------|
| Quick fixes, CSS, small refactors | Composer 2.5 |
| Scraping research, API discovery | Sonnet 5 + Composer 2.5 |
| Core engine, halal rules, architecture | Opus 5 |

## Disclaimer

Ingredient analysis is informational and does not constitute a religious ruling (fatwa). Always verify with official halal certification from recognized bodies.

## License

MIT
