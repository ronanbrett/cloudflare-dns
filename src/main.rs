/// Cloudflare DNS Manager - Main entry point.
///
/// This application provides a TUI for managing Cloudflare DNS records.
mod api;
mod config;
mod tasks;
mod ui;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    let config = config::Config::load().map_err(|e| {
        eprintln!("❌ Failed to load configuration: {}", e);
        eprintln!("\n📝 Please create a config file at:");
        eprintln!("   {}", config::Config::config_path().display());
        eprintln!("\n📖 See README.md for setup instructions.");
        e
    })?;

    println!("Starting Cloudflare DNS Manager...");
    println!("Zone ID: {}", config.cloudflare_zone_id);
    println!();

    ui::run_app(config.cloudflare_api_token, config.cloudflare_zone_id)
}
