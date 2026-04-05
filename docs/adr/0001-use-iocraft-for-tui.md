# ADR 0001: Use iocraft for TUI Framework

**Status:** Accepted  
**Date:** 2026-04-05  
**Context:** Initial project setup

## Decision

Use the `iocraft` crate as the TUI (Terminal User Interface) framework for the Cloudflare DNS Manager application.

## Rationale

### Why iocraft?

- **Modern Rust TUI**: iocraft provides a React-like component model that makes UI code more declarative and easier to reason about
- **Composable Components**: The component system allows building reusable UI elements (forms, lists, status bars)
- **Hook System**: Built-in hooks for state management and event handling reduce boilerplate
- **Terminal Size Awareness**: Automatic terminal resize handling built-in
- **Active Development**: The library is actively maintained and evolving

### Alternatives Considered

- **ratatui** (formerly tui-rs): More mature and feature-rich, but requires more imperative code for layout and state management
- **crossterm + custom rendering**: Maximum control but significantly more code to maintain
- **textual** (Python): Would have required Python runtime dependency

## Consequences

### Positive
- Clean, declarative UI code with component composition
- Built-in terminal event handling
- Automatic layout management

### Negative
- Smaller ecosystem and community compared to ratatui
- Less documentation and examples available
- Some limitations in custom rendering (had to work around API constraints)

### Risks Mitigated
- The UI layer is isolated in `src/ui/` modules, making it easier to swap frameworks if needed in the future
