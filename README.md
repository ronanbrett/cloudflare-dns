# Cloudflare DNS Manager

A TUI for managing Cloudflare DNS records programmatically.

[![Crates.io](https://img.shields.io/crates/v/cloudflaredns.svg)](https://crates.io/crates/cloudflaredns)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Installation

### Option 1: Install from Crates.io (Recommended)

```bash
cargo install cloudflare-dns
```

### Option 2: Install from Source

```bash
git clone https://github.com/ronanbrett/cloudflare-dns.git
cd cloudflare-dns
cargo install --path .
```

### Option 3: Build and Run Directly

```bash
git clone https://github.com/ronanbrett/cloudflare-dns.git
cd cloudflare-dns
cargo run --release
```

## Features

📋 List all DNS records in your Cloudflare zone

<img alt="CleanShot 2026-04-05 at 21 55 14@2x" src="https://github.com/user-attachments/assets/ce24acbb-2fc7-4777-bb09-082aa005d1cb" />

➕ Create new DNS records (A, AAAA, CNAME, MX, TXT, etc.)

<img alt="CleanShot 2026-04-05 at 21 55 25@2x" src="https://github.com/user-attachments/assets/08cb4b5f-4413-4eae-bc47-6531c80fa6b7" />

✏️ Edit existing DNS records

<img alt="CleanShot 2026-04-05 at 21 55 46@2x" src="https://github.com/user-attachments/assets/39d419a8-8bf2-4ee3-8925-14c4a0cdfb76" />

🗑️ Delete DNS records

<img   alt="CleanShot 2026-04-05 at 21 54 55@2x" src="https://github.com/user-attachments/assets/1e810742-95f9-4d8e-9ef5-ce8bfb6da687" />


## Prerequisites

- Rust 1.85+ (with Cargo)
- A Cloudflare account with a domain
- Cloudflare API Token with DNS edit permissions

## Setup

### 1. Get Your Cloudflare API Token

1. Go to [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens)
2. Click "Create Token"
3. Use the "Edit zone DNS" template or create a custom token with:
   - **Permissions**: `Zone` → `DNS` → `Edit`
   - **Permissions**: `Zone` → `Zone` → `Read`
   - **Zone Resources**: Select your domain
4. Copy the token (you won't see it again!)

### 2. Get Your Zone ID

1. Go to your Cloudflare dashboard
2. Select your domain
3. Scroll down to find the **Zone ID** in the API section
4. Copy the Zone ID

### 3. Configure the Application

You have two options for configuration:

#### Option A: YAML Config File (Recommended for installed binary)

```bash
# Create config directory
mkdir -p ~/.config/cloudflaredns

# Download the example config file
curl -o ~/.config/cloudflaredns/config.yaml https://raw.githubusercontent.com/ronanbrett/cloudflare-dns/main/config.example.yaml

# Edit with your credentials
nano ~/.config/cloudflaredns/config.yaml
```

Your `config.yaml` should look like:
```yaml
cloudflare:
  api_token: "your_api_token_here"
  zone_id: "your_zone_id_here"
```


### Option B: Environment Variables

```bash
export CLOUDFLARE_API_TOKEN=your_api_token_here
export CLOUDFLARE_ZONE_ID=your_zone_id_here
```

**Note**: This is not persistent and will be lost when the application is closed.

```bash
# Bash

echo 'export CLOUDFLARE_API_TOKEN=your_api_token_here' >> ~/.bashrc
echo 'export CLOUDFLARE_ZONE_ID=your_zone_id_here' >> ~/.bashrc
```

```zsh
# Zsh 

echo 'export CLOUDFLARE_API_TOKEN=your_api_token_here' >> ~/.zshrc
echo 'export CLOUDFLARE_ZONE_ID=your_zone_id_here' >> ~/.zshrc
```

#### Environment File (Development Only)

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env and add your credentials
nano .env
```

Your `.env` file should look like:
```env
CLOUDFLARE_API_TOKEN=your_api_token_here
CLOUDFLARE_ZONE_ID=your_zone_id_here
```

**Configuration Priority**: The app checks in this order:
1. `~/.config/cloudflaredns/config.yaml` (YAML config file)
2. `.env` in current directory
3. Environment variables

### 4. Run the Application

If you installed via `cargo install`:

```bash
cloudflare-dns
```

If you're running from source:

```bash
cargo run --release
```

## Usage

Once the application starts, you'll see a terminal UI displaying your DNS records.

### Controls

#### Record List View
- **↑/↓** — Navigate records
- **R** — Refresh DNS records
- **C** — Create new DNS record
- **E** — Edit selected record
- **D** — Delete selected record
- **Q** — Quit

#### Create/Edit Form
- **Tab / ↑↓** — Navigate fields
- **Space** on Type — Cycle DNS record type
- **Space** on IP Address — Open existing IP selector
- **Space** on Proxied — Toggle proxy on/off
- **Enter** — Submit
- **Esc** — Cancel

#### Delete Confirmation
- **Enter** — Confirm deletion
- **Esc** — Cancel

#### IP Selector
- **↑/↓** — Navigate IPs
- **Enter** — Select IP
- **Esc** — Back to form

### DNS Record Fields

When creating a record:
- **Type**: DNS record type (A, AAAA, CNAME, MX, TXT, SRV, CAA, NS, PTR)
- **Name**: The record name (e.g., `www` for `www.example.com`)
- **Ip Address**: IP address, hostname, or value
- **TTL**: Time-to-live in seconds (use `1` for automatic)
- **Proxied**: Route through Cloudflare's proxy (orange cloud)

## Project Structure

```
cloudflare-dns/
├── src/
│   ├── api/            # Cloudflare API client, models, errors, cache
│   ├── ui/             # TUI components, state, events, theme
│   ├── tasks/          # Background async tasks
│   ├── utils/          # Pure utility functions
│   ├── config.rs       # Configuration loading
│   ├── main.rs         # Entry point
│   └── lib.rs          # Library interface for testing
├── Cargo.toml
├── config.example.yaml
└── README.md
```

## API Reference

The application uses the Cloudflare API v4:

- **List DNS Records**: `GET /zones/{zone_id}/dns_records`
- **Create DNS Record**: `POST /zones/{zone_id}/dns_records`
- **Delete DNS Record**: `DELETE /zones/{zone_id}/dns_records/{record_id}`

See [Cloudflare API Documentation](https://developers.cloudflare.com/api/resources/dns/subresources/records/) for more details.

## Security Notes

- ⚠️ **Never commit your `.env` file** - it contains your API token
- The `.env` file is already in `.gitignore`
- Use API tokens instead of Global API Keys for better security
- Tokens should have minimal required permissions

## Troubleshooting

### "Failed to load configuration"

Make sure you've set up your config file or environment variables:

```bash
# Create config directory and file
mkdir -p ~/.config/cloudflaredns
nano ~/.config/cloudflaredns/config.yaml
```

Or set environment variables manually:

```bash
export CLOUDFLARE_API_TOKEN=your_token
export CLOUDFLARE_ZONE_ID=your_zone_id
cloudflaredns
```

### "API request failed with status 403"

- Check that your API token has the correct permissions
- Verify the token is valid and not expired
- Ensure the token has access to the specified zone

### "API request failed with status 404"

- Double-check your Zone ID is correct
- Verify the zone exists in your Cloudflare account

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Build release binary
cargo build --release
```

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.
