## 1. Project Scaffold

- [x] 1.1 Create Rust project with Cargo: `cargo init --bin` for the `sigground` CLI
- [x] 1.2 Add dependencies to Cargo.toml: tree-sitter, tree-sitter-language-pack, clap, serde, serde_json, anyhow
- [x] 1.3 Set up CI pipeline: Justfile with check, lint, test, fmt, check-file-sizes targets
- [x] 1.4 Configure .cargo/config.toml with RUST_BACKTRACE=0, RUST_LOG=error, incremental build
- [x] 1.5 Create module structure: `parse/`, `grounders/`, `types.rs`, `main.rs`

## 9. Documentation

- [x] 9.1 Write README.md with paper reference, veriplan comparison, usage, output format

## 2. Core Types

- [x] 2.1 Define Signature struct with types, predicates, constants
- [x] 2.2 Define TypeDef, PredicateDef, ArgSlot, ConstantDef structs
- [x] 2.3 Define GroundedAtom, GroundingResult, GroundingStatus enums
- [x] 2.4 Define Directive, CloseMatch, GroundedInput, CheckOutput, CheckSummary structs
- [x] 2.5 Define Grounder trait with ground() and ground_batch() methods
- [x] 2.6 Implement Serialize/Deserialize for all types

## 3. Signature Generation (tasks.md parser)

- [x] 3.1 Initialize tree-sitter markdown grammar via tree-sitter-language-pack
- [x] 3.2 Implement tree-sitter walker that extracts `## Phase` headings and `- [x] N.M Description` list items
- [x] 3.3 Implement extract_task_id() for N.M and T-prefixed patterns
- [x] 3.4 Implement build_aliases() that generates aliases from task descriptions
- [x] 3.5 Implement build_signature() that assembles Signature from parsed tasks
- [x] 3.6 Implement signature_from_tasks() public function
- [x] 3.7 Handle empty tasks.md with error

## 4. Rule-Based Grounder

- [x] 4.1 Implement predicate keyword matching (BEFORE, AFTER, CONCURRENTLY, IF_THEN, ALWAYS, AT_MOST_ONE)
- [x] 4.2 Implement argument extraction by scanning text for constant names and aliases
- [x] 4.3 Implement positional heuristics for BEFORE (X before Y → earlier=X, later=Y)
- [x] 4.4 Implement positional heuristics for AFTER (X after Y → earlier=Y, later=X)
- [x] 4.5 Implement positional heuristics for IF_THEN (IF X fails THEN Y → trigger=X, consequent=Y)
- [x] 4.6 Implement confidence scoring (exact match = 0.95, alias match = 0.7-0.85, missing args = 0.0)
- [x] 4.7 Implement Grounder trait for RuleGrounder

## 5. Directive Generation

- [x] 5.1 Implement generate_directives() for Ambiguous status (warning + suggestion to add explicit task ID)
- [x] 5.2 Implement generate_directives() for Ungroundable status (blocker + close matches)
- [x] 5.3 Implement find_close_matches() using Dice coefficient on bigrams
- [x] 5.4 Implement strsim() helper function

## 6. Spec Parsing

- [x] 6.1 Implement line-based extraction of `### Requirement:` blocks from spec.md files
- [x] 6.2 Implement extract_shall() that finds the first SHALL/MUST sentence in requirement body
- [x] 6.3 Implement collect_spec_files() that walks specs/**/*.md directories
- [x] 6.4 Handle missing specs/ directory gracefully (empty results, not error)

## 7. CLI Interface

- [x] 7.1 Implement `sigground check <change>` subcommand with input resolution
- [x] 7.2 Implement `sigground signature <change>` subcommand
- [x] 7.3 Implement input resolution: change name → openspec/changes/<name>, path, auto-detect
- [x] 7.4 Implement JSON output format for check subcommand
- [x] 7.5 Implement human-readable output format for check subcommand
- [x] 7.6 Implement exit codes: 0 = all grounded, 1 = some issues, 2 = error
- [x] 7.7 Wire up full pipeline: resolve → parse tasks → build signature → parse specs → ground → output

## 8. Dogfooding

- [x] 8.1 Run `sigground check sigground-grounding-tool` and verify all specs are grounded
- [x] 8.2 Run `sigground signature sigground-grounding-tool` and verify JSON output
- [x] 8.3 Fix any grounding issues found in sigground's own specs
- [x] 8.4 Verify exit codes: 0 when all grounded, 1 when issues exist
