## Why

The model (pi) writes specs that fail sigground's grounding check because it doesn't know the rules upfront. It uses descriptions instead of task IDs ("the migration step" vs "T2.1"), writes BEFORE with only one argument, and scatters task references across sentences. A `sigground init` command embeds grounding rules into `openspec/config.yaml` so the model sees them from the start — before writing a single spec.

## What Changes

- **New subcommand**: `sigground init` — merges grounding rules into `openspec/config.yaml`
- Reads existing config, checks for `# sigground init` marker
- If missing: appends context block + per-artifact rules
- If present: no-op (idempotent)
- Rules cover: explicit task IDs in specs, descriptive task descriptions in tasks.md, BEFORE/ALWAYS argument requirements, parenthetical syntax

## Capabilities

### New Capabilities

- `init-command`: `sigground init` subcommand that configures openspec/config.yaml with grounding rules

### Modified Capabilities
<!-- No existing capabilities to modify -->

## Impact

- New CLI subcommand in `src/main.rs`
- No new dependencies
- Follows same pattern as `veriplan init` (merge marker-based config)
