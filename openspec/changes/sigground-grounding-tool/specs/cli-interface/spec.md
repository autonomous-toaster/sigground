## ADDED Requirements

### Requirement: CLI subcommand `sigground check`

T7.1 SHALL implement the check subcommand AFTER T6.1 SHALL parse spec files. T7.1 SHALL provide a `check` subcommand that parses a plan and checks grounding quality of all spec requirement statements. T7.1 SHALL complete BEFORE T8.1 SHALL run dogfooding.

#### Scenario: Check by change name

- **WHEN** running `sigground check my-change`
- **THEN** T7.1 SHALL look for tasks.md in openspec/changes/my-change/
- **AND** T7.1 SHALL parse all specs/**/*.md in that directory
- **AND** T7.1 SHALL output grounding results

#### Scenario: Check by path

- **WHEN** running `sigground check ./path/to/change`
- **THEN** T7.1 SHALL use the given path directly

#### Scenario: Auto-detect single change

- **WHEN** running `sigground check` with no arguments and exactly one change in openspec/changes/
- **THEN** T7.1 SHALL auto-detect and check that change

#### Scenario: Check with JSON output

- **WHEN** running `sigground check my-change --format json`
- **THEN** T7.4 SHALL output JSON to stdout with CheckOutput structure

#### Scenario: Check with human output

- **WHEN** running `sigground check my-change` (default format)
- **THEN** T7.5 SHALL output human-readable text with icons (✓/⚠/✗), grounded atoms, and directives

### Requirement: Exit codes

T7.6 SHALL set exit codes AFTER T7.1 SHALL complete checking. T7.6 SHALL use exit codes to signal grounding status to the caller (pi). T7.6 SHALL complete BEFORE T8.1 SHALL run dogfooding.

#### Scenario: All grounded exits 0

- **WHEN** all NL statements are Grounded (confidence >= 0.8)
- **THEN** T7.6 SHALL exit with code 0

#### Scenario: Some issues exits 1

- **WHEN** any NL statement is Ambiguous or Ungroundable
- **THEN** T7.6 SHALL exit with code 1

#### Scenario: Error exits 2

- **WHEN** T7.6 encounters a parse error, missing file, or invalid input
- **THEN** T7.6 SHALL exit with code 2

### Requirement: CLI subcommand `sigground signature`

T7.2 SHALL implement the signature subcommand AFTER T3.6 SHALL serialize the Signature. T7.2 SHALL provide a `signature` subcommand that emits the Signature JSON for a plan. T7.2 SHALL complete BEFORE T8.2 SHALL verify signature output.

#### Scenario: Signature by change name

- **WHEN** running `sigground signature my-change`
- **THEN** T7.2 SHALL output the Signature JSON to stdout

#### Scenario: Signature by path

- **WHEN** running `sigground signature ./path/to/change`
- **THEN** T7.2 SHALL use the given path directly
