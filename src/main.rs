pub mod app;
pub mod cloudflare;
pub mod colors;
pub mod components;
pub mod config;
pub mod constants;
pub mod hooks;
pub mod status;
pub mod state;
pub mod tasks;
pub mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    let config = config::Config::load()?;

    println!("Starting Cloudflare DNS Manager...");
    println!("Zone ID: {}", config.cloudflare_zone_id);
    println!();

    app::run_app(config.cloudflare_api_token, config.cloudflare_zone_id)
}
