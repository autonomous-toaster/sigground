## ADDED Requirements

### Requirement: Rename package and binary

T1.1 SHALL rename the Cargo.toml package name from `sigground` to `groundcontrol`. T1.1 SHALL complete BEFORE T1.2 SHALL update source references.

#### Scenario: Cargo.toml updated

- **WHEN** T1.1 runs
- **THEN** `cargo build` SHALL produce a binary named `groundcontrol`

### Requirement: Update source code references

T1.2 SHALL update all `sigground` references in `src/main.rs` to `groundcontrol`. T1.2 SHALL update the module doc comment, the clap `#[command(name)]` attribute, the `SIGGROUND_MARKER` constant, and the init print message. T1.2 SHALL complete AFTER T1.1 SHALL rename the package.

#### Scenario: Doc comment updated

- **WHEN** T1.2 runs
- **THEN** the module doc comment SHALL say `groundcontrol` instead of `sigground`

#### Scenario: CLI name updated

- **WHEN** T1.2 runs
- **THEN** `groundcontrol --help` SHALL show the new name

#### Scenario: Init marker renamed

- **WHEN** T1.2 runs
- **THEN** the init marker constant SHALL be `# groundcontrol init`

### Requirement: Update README

T1.3 SHALL replace all `sigground` references in `README.md` with `groundcontrol`. T1.3 SHALL complete AFTER T1.2 SHALL update source references.

#### Scenario: README updated

- **WHEN** T1.3 runs
- **THEN** `grep -c sigground README.md` SHALL return 0
