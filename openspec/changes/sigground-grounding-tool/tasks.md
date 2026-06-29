## 1. Project Scaffold

- [ ] 1.1 Create Rust project with Cargo: `cargo init --bin` for the `sigground` CLI
- [ ] 1.2 Add dependencies to Cargo.toml: tree-sitter, tree-sitter-language-pack, clap, serde, serde_json, anyhow
- [ ] 1.3 Set up CI pipeline: Justfile with check, lint, test, fmt, check-file-sizes targets
- [ ] 1.4 Configure .cargo/config.toml with RUST_BACKTRACE=0, RUST_LOG=error, incremental build
- [ ] 1.5 Create module structure: `parse/`, `grounders/`, `types.rs`, `main.rs`

## 2. Core Types

- [ ] 2.1 Define Signature struct with types, predicates, constants
- [ ] 2.2 Define TypeDef, PredicateDef, ArgSlot, ConstantDef structs
- [ ] 2.3 Define GroundedAtom, GroundingResult, GroundingStatus enums
- [ ] 2.4 Define Directive, CloseMatch, GroundedInput, CheckOutput, CheckSummary structs
- [ ] 2.5 Define Grounder trait with ground() and ground_batch() methods
- [ ] 2.6 Implement Serialize/Deserialize for all types

## 3. Signature Generation (tasks.md parser)

- [ ] 3.1 Initialize tree-sitter markdown grammar via tree-sitter-language-pack
- [ ] 3.2 Implement tree-sitter walker that extracts `## Phase` headings and `- [x] N.M Description` list items
- [ ] 3.3 Implement extract_task_id() for N.M and T-prefixed patterns
- [ ] 3.4 Implement build_aliases() that generates aliases from task descriptions
- [ ] 3.5 Implement build_signature() that assembles Signature from parsed tasks
- [ ] 3.6 Implement signature_from_tasks() public function
- [ ] 3.7 Handle empty tasks.md with error

## 4. Rule-Based Grounder

- [ ] 4.1 Implement predicate keyword matching (BEFORE, AFTER, CONCURRENTLY, IF_THEN, ALWAYS, AT_MOST_ONE)
- [ ] 4.2 Implement argument extraction by scanning text for constant names and aliases
- [ ] 4.3 Implement positional heuristics for BEFORE (X before Y → earlier=X, later=Y)
- [ ] 4.4 Implement positional heuristics for AFTER (X after Y → earlier=Y, later=X)
- [ ] 4.5 Implement positional heuristics for IF_THEN (IF X fails THEN Y → trigger=X, consequent=Y)
- [ ] 4.6 Implement confidence scoring (exact match = 0.95, alias match = 0.7-0.85, missing args = 0.0)
- [ ] 4.7 Implement Grounder trait for RuleGrounder

## 5. Directive Generation

- [ ] 5.1 Implement generate_directives() for Ambiguous status (warning + suggestion to add explicit task ID)
- [ ] 5.2 Implement generate_directives() for Ungroundable status (blocker + close matches)
- [ ] 5.3 Implement find_close_matches() using Dice coefficient on bigrams
- [ ] 5.4 Implement strsim() helper function

## 6. Spec Parsing

- [ ] 6.1 Implement line-based extraction of `### Requirement:` blocks from spec.md files
- [ ] 6.2 Implement extract_shall() that finds the first SHALL/MUST sentence in requirement body
- [ ] 6.3 Implement collect_spec_files() that walks specs/**/*.md directories
- [ ] 6.4 Handle missing specs/ directory gracefully (empty results, not error)

## 7. CLI Interface

- [ ] 7.1 Implement `sigground check <change>` subcommand with input resolution
- [ ] 7.2 Implement `sigground signature <change>` subcommand
- [ ] 7.3 Implement input resolution: change name → openspec/changes/<name>, path, auto-detect
- [ ] 7.4 Implement JSON output format for check subcommand
- [ ] 7.5 Implement human-readable output format for check subcommand
- [ ] 7.6 Implement exit codes: 0 = all grounded, 1 = some issues, 2 = error
- [ ] 7.7 Wire up full pipeline: resolve → parse tasks → build signature → parse specs → ground → output

## 8. Dogfooding

- [ ] 8.1 Run `sigground check sigground-grounding-tool` and verify all specs are grounded
- [ ] 8.2 Run `sigground signature sigground-grounding-tool` and verify JSON output
- [ ] 8.3 Fix any grounding issues found in sigground's own specs
- [ ] 8.4 Verify exit codes: 0 when all grounded, 1 when issues exist
