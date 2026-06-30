## Context

The project is currently named `sigground` in Cargo.toml, src/main.rs, and README.md. The GitHub remote has already been renamed to `groundcontrol`. This is a pure rename — no functional changes to any command.

## Goals / Non-Goals

**Goals:**

- Rename the package name from `sigground` to `groundcontrol`
- Update all user-facing references (CLI name, docs, init marker)
- Keep the init marker backward-incompatible by design (clean break — no configs in the wild)

**Non-Goals:**

- No functional changes to check, signature, or init commands
- No directory rename (local path stays `sigground/`)
- No changes to archived changes or git history

## Decisions

1. **Clean break on init marker** — `# sigground init` → `# groundcontrol init`. No backward compat needed since the init was archived before being run on this repo.

2. **No directory rename** — The local checkout directory name is cosmetic and doesn't affect compilation or usage.

3. **README full rewrite** — Every reference to `sigground` in the README gets replaced. This is the largest text change but mechanically simple.

## Risks / Trade-offs

- [Old binary name lingers] → `cargo build` produces `groundcontrol` after rename. Old `sigground` binary in PATH won't be updated automatically — user needs to rebuild/reinstall.
- [Init marker mismatch] → If someone manually wrote `# sigground init` into a config, `groundcontrol init` won't detect it and will append a duplicate. Acceptable given no configs exist with the old marker.
