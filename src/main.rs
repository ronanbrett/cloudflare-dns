pub mod app;
pub mod cloudflare;
pub mod colors;
pub mod components;
pub mod constants;
pub mod hooks;
pub mod status;
pub mod state;
pub mod tasks;
pub mod utils;

use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_token = env::var("CLOUDFLARE_API_TOKEN")
        .context("CLOUDFLARE_API_TOKEN environment variable is not set. Please create a .env file or set this variable.")?;

    let zone_id = env::var("CLOUDFLARE_ZONE_ID")
        .context("CLOUDFLARE_ZONE_ID environment variable is not set. Please create a .env file or set this variable.")?;

    println!("Starting Cloudflare DNS Manager...");
    println!("Zone ID: {}", zone_id);
    println!();

    app::run_app(api_token, zone_id)
}
