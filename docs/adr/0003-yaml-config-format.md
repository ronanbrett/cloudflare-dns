# ADR 0003: Use YAML for Configuration File

**Status:** Accepted  
**Date:** 2026-04-05  
**Context:** Configuration management design

## Decision

Use YAML format stored in `~/.config/cloudflaredns/config.yaml` as the primary configuration file format, with `.env` files and environment variables as fallbacks.

## Rationale

### Why YAML?

- **Human-readable**: Clear, hierarchical structure that's easy to edit
- **Comments**: Supports inline comments explaining each field
- **Standard**: Widely used for configuration files in DevOps tools
- **Rust support**: `serde_yaml` integrates seamlessly with serde serialization

### Configuration Priority

1. `~/.config/cloudflaredns/config.yaml` (YAML config file)
2. `.env` in current directory
3. Environment variables

This allows flexibility for different deployment scenarios.

### Alternatives Considered

- **TOML**: More Rust-idiomatic, but less readable for nested structures
- **JSON**: No comment support, harder for users to edit
- **INI**: Too simple, lacks type safety and nesting support
- **Environment variables only**: Hard to manage multiple values, not user-friendly

## Consequences

### Positive
- User-friendly configuration with comments
- Flexible config loading with fallbacks
- Standard location following XDG conventions

### Negative
- YAML parsing errors can be cryptic for users
- Extra dependency on `serde_yaml`

### Security Note
The config file contains API tokens. Users must ensure proper file permissions (readable only by owner).
