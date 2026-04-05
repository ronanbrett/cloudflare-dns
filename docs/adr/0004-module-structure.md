# ADR 0004: Organize Code into api/ui/tasks/utils Modules

**Status:** Accepted  
**Date:** 2026-04-05  
**Context:** Project growth and code organization

## Decision

Reorganize the source code from a flat module structure into logical directories:

```
src/
├── api/          # Cloudflare API client, models, and errors
├── ui/           # TUI components, state, events, and theme
├── tasks/        # Background async tasks with parameter objects
├── utils/        # Pure utility functions (formatting, parsing)
├── config.rs     # Configuration loading
├── constants.rs  # Global constants (re-export)
├── main.rs       # Application entry point
└── lib.rs        # Library interface for testing
```

## Rationale

### Why this structure?

- **Separation of concerns**: Clear boundaries between API, UI, and business logic
- **Scalability**: Each module can grow independently without becoming unwieldy
- **Discoverability**: New contributors can quickly find relevant code
- **Testability**: Clear interfaces between modules enable focused unit and integration tests
- **Parameter objects**: Tasks use struct parameters instead of long argument lists to improve readability

### Alternatives Considered

- **Flat structure**: All modules in `src/` - becomes unmanageable as project grows
- **Feature-based**: Group by feature (dns, config, etc.) - creates more coupling
- **Layered**: Strict layering with dependency injection - overkill for this project size

## Consequences

### Positive
- Clean module boundaries improve code navigation
- Easier to locate related code
- Parameter objects reduce function signature complexity
- Legacy re-exports maintain backward compatibility during migration

### Negative
- More files and directories to navigate initially
- Some indirection with re-exports for backward compatibility

### Migration Strategy
Old module files (e.g., `src/cloudflare.rs`) remain as thin re-exports to maintain compatibility with existing tests and imports.
