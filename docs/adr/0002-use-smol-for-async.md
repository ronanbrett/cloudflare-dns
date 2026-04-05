# ADR 0002: Use smol for Async Runtime

**Status:** Accepted  
**Date:** 2026-04-05  
**Context:** Initial project setup

## Decision

Use the `smol` async runtime for handling asynchronous operations in the application.

## Rationale

### Why smol?

- **Lightweight**: Minimal overhead and small binary size compared to tokio
- **Simple API**: Straightforward async/await support without complex macros
- **Good for CLI/TUI**: Well-suited for applications that don't need massive concurrency
- **smol::spawn + detach**: Perfect fire-and-forget pattern for background tasks
- **smol::unblock**: Easy way to run blocking code in thread pools (used previously with reqwest-blocking)

### Alternatives Considered

- **tokio**: Industry standard, but brings significant complexity and binary size overhead for a TUI app
- **async-std**: Similar to smol but less actively maintained
- **Pollster/blocking**: Simpler but lacks task spawning capabilities needed for background API calls

## Consequences

### Positive
- Fast compile times
- Small binary size
- Simple async model that's easy to understand

### Negative
- Smaller ecosystem (fewer crates with smol-specific integrations)
- Less community support and examples

### Note
The application uses `smol::block_on` in main.rs to run the top-level async TUI event loop
