# AGENTS.md

## Cursor Cloud specific instructions

Halal Beauty Data Engine — a Rust (Axum) backend serving an analysis API and a React + TypeScript + Vite frontend. Standard prerequisites, layout, and run commands are in `README.md`; only the non-obvious caveats are captured below.

### Services

| Service | Dir | Run (dev) | Port | Lint / Test / Build |
|---|---|---|---|---|
| Backend API | `backend/` | `cargo run` | 8080 | `cargo clippy`, `cargo test` |
| Frontend | `frontend/` | `npm run dev` | 3000 | `npm run build` (runs `tsc -b` then `vite build`) |

Both services must be running to use the app: the Vite dev server on `:3000` proxies `/api` to the backend on `:8080` (see `frontend/vite.config.ts`). Open http://localhost:3000.

### Non-obvious caveats

- The backend is fully self-contained and serves in-memory sample data (`backend/src/api/sample_data.rs`). The `postgres`, `redis`, and `meilisearch` services in `docker-compose.yml` are declared for the intended production architecture but are **not** used by the current backend code (there are no DB/cache crates in `backend/Cargo.toml`). You do **not** need Docker or any database to build, run, or test locally.
- A transitive dependency requires Rust edition 2024, so the toolchain must be **Rust 1.85+**. Some base images ship an older `cargo` (e.g. 1.83) which fails with `feature 'edition2024' is required`. Fix by selecting a recent stable toolchain: `rustup default stable` (the update script does this).
- The frontend has no lint or test scripts defined; `npm run build` is the type-check + build gate.
- Sample product `image_url`s point to arbitrary Unsplash photos, so a card's photo may not match the product name. This is expected placeholder data, not a bug.
