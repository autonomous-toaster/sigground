## Why

veriplan's current NL→LTL translation uses regex-based keyword matching and fragile task ID extraction. It cannot handle free-form NL that uses descriptions instead of task IDs ("the migration must finish before testing starts"), and it has no way to give the LLM structured feedback about grounding quality. A standalone grounding tool — sigground — solves this by formalizing the plan vocabulary as a system signature and checking whether each NL spec statement can be grounded to it.

## What Changes

- **New crate**: `sigground` — a standalone Rust CLI for grounding NL into system signatures
- Parses OpenSpec `tasks.md` directly (tree-sitter) to build a `Signature` (types, predicates, constants with aliases)
- `sigground check <change>` — checks all spec requirement statements against the signature, outputs grounded atoms + directives for the LLM
- `sigground signature <change>` — emits the signature JSON for consumption by other tools
- Rule-based grounder (keyword matching + positional heuristics) with no ML dependency
- Structured JSON output with exit codes: 0 = all grounded, 1 = ambiguous/ungroundable
- Directives tell the LLM exactly what to fix (which file, which line, what to change)
- Dogfooding: sigground's own OpenSpec change will be checked with `sigground check`

## Capabilities

### New Capabilities
- `signature-generation`: Parse OpenSpec tasks.md into a system signature (types, predicates, constants with aliases from task descriptions)
- `grounding-check`: Ground NL requirement statements against a signature, produce structured directives for the LLM
- `cli-interface`: CLI subcommands `check` and `signature` with human and JSON output formats

### Modified Capabilities
<!-- No existing capabilities to modify — this is a new tool -->

## Impact

- New Rust crate at `../sigground/` (independent of veriplan, no dependency on it)
- Dependencies: tree-sitter, tree-sitter-language-pack, clap, serde, serde_json, anyhow
- File size target: max 500 lines per source file (matching veriplan's standard)
- CI via Justfile: check, lint, test, fmt, check-file-sizes
