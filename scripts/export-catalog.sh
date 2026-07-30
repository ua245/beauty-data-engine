# Regenerate the dashboard JSON from Rust sample data
set -euo pipefail
cargo run -p halal-scraper -- export-sample
echo "Catalog written to web/public/data/catalog.json"
