#[cfg(feature = "live")]
mod scrape;

mod catalog;
mod sample;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "halal-scraper", about = "Halal beauty data engine — scrape & export")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export the bundled sample catalog as JSON for the web dashboard.
    ExportSample {
        #[arg(short, long, default_value = "web/public/data/catalog.json")]
        output: PathBuf,
    },
    /// Fetch and parse a single product URL (requires `--features live`).
    #[cfg(feature = "live")]
    Probe {
        #[arg(long)]
        retailer: String,
        #[arg(long)]
        sku: String,
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("halal_scraper=info".parse()?))
        .init();

    match Cli::parse().command {
        Commands::ExportSample { output } => {
            let catalog = sample::build_catalog();
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(&catalog)?;
            std::fs::write(&output, json)?;
            tracing::info!(
                "wrote {} products to {}",
                catalog.products.len(),
                output.display()
            );
        }
        #[cfg(feature = "live")]
        Commands::Probe {
            retailer,
            sku,
            url,
        } => {
            let html = scrape::fetch_page(&url).await?;
            let extract = scrape::parse_product_page(&html, &url);
            let product = scrape::product_from_extract(
                halal_core::product::Retailer {
                    id: retailer.clone(),
                    name: retailer,
                    domain: url::Url::parse(&url)
                        .ok()
                        .and_then(|u| u.host_str().map(str::to_string))
                        .unwrap_or_else(|| "unknown".into()),
                },
                &sku,
                &url,
                &extract,
            );
            println!("{}", serde_json::to_string_pretty(&product)?);
        }
    }
    Ok(())
}
