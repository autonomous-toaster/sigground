## ADDED Requirements

### Requirement: Ground NL statement against Signature

T4.1 SHALL ground NL statements AFTER T3.1 SHALL parse tasks.md. T4.1 SHALL ground an NL requirement statement against a Signature, producing GroundedAtoms with predicate, arguments, and confidence. T4.1 SHAL complete BEFORE T5.1 SHALL generate directives.

#### Scenario: Ground BEFORE with explicit task IDs

- **GIVEN** a Signature with constants T5.1 and T5.2
- **WHEN** T4.1 grounds "T5.1 SHALL complete BEFORE T5.2 SHALL run"
- **THEN** T4.1 SHALL produce GroundedAtom with predicate "BEFORE", arguments ["T5.1", "T5.2"], and confidence >= 0.8

#### Scenario: Ground BEFORE with aliases

- **GIVEN** a Signature with constant T2.1 (alias: "migration") and T2.2 (alias: "testing")
- **WHEN** T4.1 grounds "The migration must finish before testing starts"
- **THEN** T4.1 SHALL produce GroundedAtom with predicate "BEFORE", arguments ["T2.1", "T2.2"], and confidence < 0.8

#### Scenario: Ground AFTER with positional heuristics

- **GIVEN** a Signature with constants T4.4 and T5.1
- **WHEN** T4.1 grounds "T5.1 SHALL complete AFTER T4.4 SHALL classify"
- **THEN** T4.1 SHALL assign T4.4 as the "earlier" argument and T5.1 as the "later" argument

#### Scenario: Ground IF_THEN

- **GIVEN** a Signature with constants T1.4 and T1.5
- **WHEN** T4.1 grounds "IF smoke tests fail THEN rollback SHALL trigger"
- **THEN** T4.1 SHALL produce GroundedAtom with predicate "IF_THEN", trigger=T1.4, consequent=T1.5

#### Scenario: Ground ALWAYS

- **GIVEN** a Signature with constant T6.1
- **WHEN** T4.1 grounds "Rollback SHALL ALWAYS be available"
- **THEN** T4.1 SHALL produce GroundedAtom with predicate "ALWAYS", target=T6.1

#### Scenario: Ground AT_MOST_ONE

- **GIVEN** a Signature with constants T2.1 and T2.2
- **WHEN** T4.1 grounds "At most one of T2.1, T2.2 SHALL be active"
- **THEN** T4.1 SHALL produce GroundedAtom with predicate "AT_MOST_ONE", arguments ["T2.1", "T2.2"]

#### Scenario: Ungroundable NL returns empty

- **WHEN** T4.1 grounds "The system SHALL be user-friendly"
- **THEN** T4.1 SHALL return status Ungroundable with no candidates

### Requirement: Generate directives for ambiguous grounding

T5.1 SHALL generate directives AFTER T4.1 SHALL ground NL statements. T5.1 SHALL produce structured directives when grounding is ambiguous (confidence < 0.8 but > 0), telling the LLM what to fix. T5.1 SHALL complete BEFORE T7.1 SHALL output results.

#### Scenario: Ambiguous directive

- **GIVEN** a GroundingResult with status Ambiguous
- **WHEN** T5.1 generates directives
- **THEN** T5.1 SHALL produce a directive with severity "warning", element (file:line), detail explaining which argument was matched via alias, and suggested_action to add explicit task ID

### Requirement: Generate directives for ungroundable text

T5.2 SHALL generate ungroundable directives AFTER T4.1 SHALL ground NL statements. T5.2 SHALL produce structured directives when grounding fails, including close matches based on string similarity. T5.2 SHALL complete BEFORE T7.1 SHALL output results.

#### Scenario: Ungroundable directive with close matches

- **GIVEN** a GroundingResult with status Ungroundable
- **WHEN** T5.2 generates directives
- **THEN** T5.2 SHALL produce a directive with severity "blocker", detail explaining no match found, and close_matches with constant name, alias, and similarity score

### Requirement: Batch grounding

T4.7 SHALL support batch grounding AFTER T4.1 SHALL ground single statements. T4.7 SHALL ground multiple NL statements in a single call, returning results in the same order. T4.7 SHALL complete BEFORE T7.1 SHALL output results.

#### Scenario: Batch ground

- **WHEN** T4.7 receives multiple NL statements
- **THEN** T4.7 SHALL return a GroundingResult for each, preserving input order
