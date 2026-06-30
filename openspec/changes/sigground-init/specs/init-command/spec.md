## ADDED Requirements

### Requirement: Init subcommand

T2.1 SHALL complete AFTER T1.1 SHALL add the CLI subcommand. T2.1 SHALL read `openspec/config.yaml` and check for a `# sigground init` marker. When the marker is absent, T2.1 SHALL append grounding rules.

#### Scenario: Config exists without sigground marker

- **GIVEN** `openspec/config.yaml` exists without `# sigground init`
- **WHEN** T2.1 runs init
- **THEN** T2.1 SHALL append the grounding rules context and rules to the config
- **AND** T2.1 SHALL add a `# sigground init` marker

#### Scenario: Config already has sigground marker

- **GIVEN** `openspec/config.yaml` contains `# sigground init`
- **WHEN** T2.1 runs init
- **THEN** T2.1 SHALL NOT modify the file

#### Scenario: Config does not exist

- **GIVEN** `openspec/config.yaml` does not exist
- **WHEN** T2.1 runs init
- **THEN** T2.1 SHALL create the file with the grounding rules

### Requirement: Grounding rules content

T2.2 SHALL define the grounding rules content BEFORE T2.1 SHALL write them. T2.2 SHALL include context about using explicit task IDs in specs and descriptive descriptions in tasks.md, plus per-artifact rules for specs and tasks. T2.2 SHALL complete AFTER T1.1 SHALL add the CLI subcommand.

#### Scenario: Rules mention explicit IDs in specs

- **WHEN** T2.2 defines rules
- **THEN** the specs rules SHALL say: "Reference tasks by explicit ID: 'T2.1' not 'the migration step'"

#### Scenario: Rules mention descriptive tasks

- **WHEN** T2.2 defines rules
- **THEN** the tasks rules SHALL say: "Write descriptive task descriptions — they become aliases for grounding"

#### Scenario: Rules mention BEFORE/ALWAYS requirements

- **WHEN** T2.2 defines rules
- **THEN** the specs rules SHALL mention that BEFORE needs two task IDs and ALWAYS needs one
