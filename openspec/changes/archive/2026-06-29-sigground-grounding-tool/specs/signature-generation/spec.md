## ADDED Requirements

### Requirement: Parse tasks.md into a Signature

T3.1 SHALL parse tasks.md BEFORE T4.1 SHALL ground NL statements. T3.1 SHALL produce a Signature containing types, predicates, and constants with aliases derived from task descriptions. T3.1 SHALL complete AFTER T2.1 SHALL define the Signature struct.

#### Scenario: Parse checklist items

- **GIVEN** tasks.md contains `- [x] 1.3 Add dependencies`
- **WHEN** T3.1 parses tasks.md
- **THEN** T3.1 SHALL produce a ConstantDef with name "T1.3", type_name "task_id", and aliases containing "add dependencies"

#### Scenario: Extract phase headings

- **GIVEN** tasks.md contains `## 1. Project Scaffold`
- **WHEN** T3.1 parses tasks.md
- **THEN** T3.1 SHALL include a TypeDef with name "phase_name"

#### Scenario: Include standard predicates

- **WHEN** T3.5 builds a Signature
- **THEN** T3.5 SHALL include predicates: BEFORE, AFTER, CONCURRENTLY, IF_THEN, ALWAYS, AT_MOST_ONE

#### Scenario: Handle empty tasks.md

- **WHEN** T3.1 parses an empty tasks.md or one with no checklist items
- **THEN** T3.1 SHALL return an error

### Requirement: Build aliases from task descriptions

T3.4 SHALL generate aliases BEFORE T3.5 SHALL build the Signature. T3.4 SHALL generate aliases from task descriptions for fuzzy matching, including the full description, first 3-5 words, and individual significant words (length > 4). T3.4 SHALL complete AFTER T3.3 SHALL extract task IDs.

#### Scenario: Alias generation

- **GIVEN** a task with description "Create Rust project with Cargo: cargo init"
- **WHEN** T3.4 builds aliases
- **THEN** T3.4 SHALL include aliases: "create rust project with cargo: cargo init", "create rust project", "create rust project with cargo: cargo", "create", "rust", "project", "cargo", "init"

### Requirement: Serialize Signature to JSON

T3.6 SHALL serialize a Signature to JSON BEFORE T7.2 SHALL emit the signature subcommand output. T3.6 SHALL serialize a Signature to JSON for consumption by other tools and the `signature` subcommand. T3.6 SHALL complete AFTER T3.5 SHALL build the Signature.

#### Scenario: JSON output

- **WHEN** T3.6 serializes a Signature
- **THEN** the JSON SHALL contain "types", "predicates", and "constants" arrays
- **AND** each constant SHALL have "name", "type_name", and "aliases" fields
- **AND** each predicate SHALL have "name" and "arguments" fields
- **AND** each argument SHALL have "name" and "type_name" fields
