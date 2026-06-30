## Context

sigground is a standalone Rust CLI that formalizes the plan vocabulary as a system signature and checks whether NL requirement statements can be grounded to it. It is independent of veriplan — it parses the same OpenSpec markdown format using tree-sitter, but produces a Signature instead of PlanIR.

The design follows GinSign's hierarchical grounding approach (predicate then argument) but uses rule-based matching instead of a BERT model, keeping the tool dependency-free and deterministic.

## Goals / Non-Goals

**Goals:**
- Parse OpenSpec tasks.md into a Signature (types, predicates, constants with aliases)
- Ground NL requirement statements against the Signature using rule-based matching
- Output structured JSON with grounded atoms, confidence scores, and directives for the LLM
- Exit codes: 0 = all grounded, 1 = some ambiguous/ungroundable
- Dogfood: sigground's own change specs must pass `sigground check`

**Non-Goals:**
- No ML/LLM dependency inside sigground (the LLM is pi, which calls sigground)
- No dependency on veriplan's PlanIR or translator
- No LTL generation (veriplan's job)
- No model checking (veriplan's job)

## Decisions

1. **Standalone CLI, not a library** — sigground is a binary that communicates via stdin/stdout JSON. veriplan can optionally consume its output, but there's no shared code. This keeps the tools decoupled.

2. **Rule-based grounder as default** — keyword matching + positional heuristics. No ML dependency. Works well for structured OpenSpec specs that use explicit task IDs and temporal keywords. A fuzzy/embedding-based grounder can be added later as an optional feature.

3. **Aliases from task descriptions** — each task's description text is tokenized into aliases (full description, first N words, individual significant words). This lets "the migration must finish before testing starts" match T2.1 (alias: "migration") and T2.2 (alias: "testing").

4. **Directives mirror veriplan's rephrase_directives** — structured feedback with severity, element (file:line), detail, suggested_action, and close_matches. pi reads these and asks the LLM to fix.

5. **Tree-sitter for tasks.md parsing** — same approach as veriplan. Parses markdown AST to extract `## Phase` headings and `- [x] N.M Description` list items. Spec parsing is simpler: line-based extraction of `### Requirement:` blocks.

6. **Exit codes as protocol** — 0 = all grounded, 1 = some issues, 2 = error. pi reads the exit code to decide whether to proceed or loop.

## Risks / Trade-offs

- [Rule-based grounder fragility] → The keyword list and positional heuristics may miss edge cases. Mitigation: start with the 6 veriplan temporal categories and expand as needed. The Grounder trait makes it easy to add a fuzzy/embedding-based strategy later.
- [Alias collision] → Two tasks with similar descriptions could produce false matches. Mitigation: confidence scoring penalizes alias matches vs exact name matches. The directive tells the LLM to add explicit task IDs.
- [Spec parsing is line-based] → The simple line-based spec parser may miss complex requirement structures. Mitigation: if needed, switch to tree-sitter for spec parsing too (same as veriplan).
