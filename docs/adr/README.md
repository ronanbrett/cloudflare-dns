# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for the Cloudflare DNS Manager project.

ADRs are lightweight documents that capture important architectural decisions, their context, and consequences. They help current and future team members understand why certain decisions were made.

## ADR Index

| Number | Title | Status |
|--------|-------|--------|
| [0001](0001-use-iocraft-for-tui.md) | Use iocraft for TUI framework | Accepted |
| [0002](0002-use-smol-for-async.md) | Use smol for async runtime | Accepted |
| [0003](0003-yaml-config-format.md) | Use YAML for configuration file | Accepted |
| [0004](0004-module-structure.md) | Organize code into api/ui/tasks/utils modules | Accepted |
| [0005](0005-async-reqwest.md) | Use async reqwest instead of blocking | Accepted |

## Template

To create a new ADR, copy `template.md` and update the number and content:

```bash
cp docs/adr/template.md docs/adr/0006-your-decision.md
```

## References

- [Michael Nygard's ADR format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [Joel Parker Henderson's ADR collection](https://github.com/joelparkerhenderson/architecture-decision-record)
