# ADR 0005: Use Blocking reqwest with smol::unblock

**Status:** Accepted  
**Date:** 2026-04-05 (revised)  
**Context:** API client runtime compatibility with iocraft

## Decision

Use `reqwest` with the `blocking` feature, wrapped in `smol::unblock`, for all API client methods.

## Rationale

### Why blocking + unblock?

- **iocraft runtime conflict**: iocraft uses `smol` internally and calls `smol::block_on()` at the top level. Using async `reqwest` creates a nested runtime conflict.
- **Proven pattern**: `smol::unblock(move || { blocking_call() })` safely offloads blocking I/O to a thread pool without blocking the smol executor.
- **Simpler testing**: Sync mockito tests work naturally without async test runners.

### Before and After (same pattern maintained)

```rust
pub async fn list_dns_records(&self) -> CloudflareResult<Vec<DnsRecord>> {
    let inner = self.inner.clone();
    smol::unblock(move || {
        // blocking reqwest call runs on thread pool
        let response = inner.client.get(&url).send()?;
        // ...
    })
    .await
}
```

### Alternatives Considered

- **Native async reqwest**: Causes runtime panic with iocraft's smol executor
- **isahc/surf**: Same async runtime conflict issues
- **Custom HTTP client**: Unnecessary complexity

## Consequences

### Positive
- Stable runtime with no executor conflicts
- Clean separation: async API surface, blocking implementation
- Tests work with `smol::block_on` wrapper

### Negative
- Slight thread pool overhead for each API call
- Error handling uses `map_err` instead of `?` with anyhow context

