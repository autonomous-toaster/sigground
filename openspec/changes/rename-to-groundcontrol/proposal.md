## Why

The name "sigground" is a placeholder that never clicked. "groundcontrol" is more evocative and memorable. The GitHub remote has already been renamed; the local codebase still references the old name.

## What Changes

- **Cargo.toml**: Package name `sigground` → `groundcontrol`
- **src/main.rs**: Doc comment, clap `#[command(name)]`, `SIGGROUND_MARKER` constant, and print message updated to `groundcontrol`
- **README.md**: All references to `sigground` replaced with `groundcontrol`
- The init marker written by `groundcontrol init` changes from `# sigground init` to `# groundcontrol init` (clean break — no configs in the wild with the old marker)

## Capabilities

### New Capabilities

- `rename`: Rename the project from sigground to groundcontrol across all source files and documentation

### Modified Capabilities
<!-- No existing capabilities to modify -->

## Impact

- Binary name changes from `sigground` to `groundcontrol`
- No functional changes to any command (check, signature, init)
- No new dependencies
- Archived changes and git history retain the old name — that's expected
