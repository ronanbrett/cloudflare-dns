# Cloudflare DNS Manager

An interactive terminal application built with **iocraft** for managing Cloudflare DNS records programmatically.

## Features

- 📋 List all DNS records in your Cloudflare zone
- ➕ Create new DNS records (A, AAAA, CNAME, MX, TXT, etc.)
- 🗑️ Delete DNS records

https://github.com/user-attachments/assets/c61a1d9b-c41d-4d63-822e-90cb93405184


## Prerequisites

- Rust 1.70+ (with Cargo)
- A Cloudflare account with a domain
- Cloudflare API Token with DNS edit permissions

## Setup

### 1. Get Your Cloudflare API Token

1. Go to [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens)
2. Click "Create Token"
3. Use the "Edit zone DNS" template or create a custom token with:
   - **Permissions**: `Zone` → `DNS` → `Edit`
   - **Zone Resources**: Select your domain
4. Copy the token (you won't see it again!)

### 2. Get Your Zone ID

1. Go to your Cloudflare dashboard
2. Select your domain
3. Scroll down to find the **Zone ID** in the API section
4. Copy the Zone ID

### 3. Configure the Application

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

### 4. Build and Run

```bash
# Build the application
cargo build --release

# Run the application
cargo run --release
```

## Usage

Once the application starts, you'll see a terminal UI displaying your DNS records.

### Controls

- **L** - List/refresh DNS records
- **C** - Create new DNS record
- **D** - Delete selected record
- **Q** - Quit the application

### DNS Record Fields

When creating a record:
- **Type**: DNS record type (A, AAAA, CNAME, MX, TXT, SRV, CAA)
- **Name**: The record name (e.g., `example.com` or `sub.example.com`)
- **Content**: IP address, hostname, or value
- **TTL**: Time-to-live in seconds (use `1` for automatic)
- **Proxied**: Route through Cloudflare's proxy (orange cloud)

## Project Structure

```
cloudflare-dns/
├── src/
│   ├── main.rs           # Entry point, configuration loading
│   ├── app.rs            # iocraft TUI components and application logic
│   └── cloudflare.rs     # Cloudflare API client
├── Cargo.toml            # Rust dependencies
├── .env.example          # Example environment file
└── README.md             # This file
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

### "CLOUDFLARE_API_TOKEN environment variable is not set"

Make sure you have a `.env` file in the project root with your API token, or export the variables manually:

```bash
export CLOUDFLARE_API_TOKEN=your_token
export CLOUDFLARE_ZONE_ID=your_zone_id
cargo run
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
# Run in development mode with hot reload
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## License

MIT

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.
