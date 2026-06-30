## 1. CLI

- [x] 1.1 Add `Init` variant to `Commands` enum with optional `--project-root` flag
- [x] 1.2 Add `run_init()` function that reads/writes `openspec/config.yaml`
- [x] 1.3 Define `SIGGROUND_MARKER`, `BOOTSTRAP_SUFFIX`, `BOOTSTRAP_CONFIG` constants
- [x] 1.4 Implement `merge_config()` that checks for marker and appends suffix
- [x] 1.5 Wire `Commands::Init` to `run_init()` in main match

## 2. Rules Content

- [x] 2.1 Define context block: grounding rules, explicit IDs in specs, descriptive tasks in tasks.md
- [x] 2.2 Define specs rules: explicit IDs, parenthetical syntax, BEFORE/ALWAYS requirements
- [x] 2.3 Define tasks rules: write descriptive descriptions as aliases

## 3. Testing

- [x] 3.1 Test: init on existing config without marker appends rules
- [x] 3.2 Test: init on config with marker is no-op
- [x] 3.3 Test: init on non-existent config creates file
- [x] 3.4 Test: rules content matches expected strings
