## Context

`veriplan init` already embeds temporal-constraint rules into `openspec/config.yaml`. `sigground init` follows the same pattern but for grounding-specific rules. Both sections coexist in the same file — the model sees both sets of rules.

## Goals / Non-Goals

**Goals:**

- `sigground init` reads `openspec/config.yaml`, appends grounding rules if `# sigground init` marker is missing
- Rules cover: explicit task IDs in specs, descriptive task descriptions in tasks.md, BEFORE/ALWAYS argument requirements, parenthetical syntax
- Idempotent: re-running is a no-op

**Non-Goals:**

- No modification of veriplan's existing rules
- No validation of the config file structure

## Decisions

1. **Same merge pattern as veriplan** — check for `# sigground init` marker, append `BOOTSTRAP_SUFFIX` if missing. Two independent markers in the same file.

2. **Context block for grounding** — explains the *why*: sigground checks that task references can be mapped to real tasks. Rules are split by artifact type (specs vs tasks).

3. **No new dependencies** — uses `serde_yaml` for parsing (already in Cargo.toml if needed), or simple string-based merge like veriplan.

## Risks / Trade-offs

- [Config file grows] → Two init sections is acceptable. Each is ~20 lines.
- [Rules may conflict with veriplan] → They're complementary, not conflicting. veriplan says "use task IDs", sigground says "use explicit IDs not descriptions".
