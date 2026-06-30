# groundcontrol — Ground Natural Language Into System Signatures

**groundcontrol** checks whether natural-language requirement statements can be grounded to a plan's task vocabulary. It is a companion to [veriplan](https://github.com/autonomous-toaster/veriplan): veriplan verifies plan *structure* (task ordering, temporal constraints), while groundcontrol verifies that the *language* used in requirements actually refers to real tasks.

Based on the paper [*GinSign: Grounding Natural Language Into System Signatures for Temporal Logic Translation*](https://arxiv.org/abs/2512.16770) (arXiv:2512.16770).

---

## The Problem

veriplan reads OpenSpec plans and checks whether temporal constraints hold. But it relies on regex to extract task IDs from requirement text:

```
"T2.1 SHALL complete BEFORE T3.1 SHALL run"  →  regex finds T2.1, T3.1  ✓
"The migration must finish before testing starts"  →  regex finds nothing  ✗
```

When a requirement uses descriptions instead of explicit task IDs, veriplan marks it `PatternUngrounded` and blocks. The LLM (pi) gets a generic error but no guidance on *what* to fix.

groundcontrol solves this by formalizing the plan's vocabulary as a **system signature** and checking whether each NL statement can be grounded to it.

---

## How It Works

### 1. Build a Signature from tasks.md

```
tasks.md:
  ## 1. Setup
  - [ ] 1.1 Create project
  - [ ] 1.2 Add dependencies

  → Signature with 2 constants:
      T1.1  aliases: ["create project", "1.1"]
      T1.2  aliases: ["add dependencies", "1.2"]
```

Each task becomes a constant with aliases derived from its description. Six predicates are always included: `BEFORE`, `AFTER`, `CONCURRENTLY`, `IF_THEN`, `ALWAYS`, `AT_MOST_ONE`.

### 2. Ground NL Statements Against the Signature

```
Input:  "The migration must finish before testing starts"
        ↓
Predicate matching:  "before" → BEFORE
Argument extraction: "migration" → alias match → T2.1
                     "testing"   → alias match → T2.2
Positional:          "X before Y" → earlier=T2.1, later=T2.2
        ↓
Output: BEFORE(T2.1, T2.2) @ 0.72  (ambiguous — used aliases, not explicit IDs)
```

### 3. Produce Directives for the LLM

When grounding is ambiguous or fails, groundcontrol outputs structured directives telling the LLM exactly what to fix:

```
✗ specs/ordering/spec.md#L12
   "The migration must finish before testing starts"
   → BEFORE(T2.1, T2.2) @ 0.72
   Suggestion: Replace 'the migration' with 'T2.1' to raise confidence.
```

---

## Relationship to veriplan

```
┌──────────────────────────────────────────────────────────────┐
│  pi (LLM coding assistant)                                   │
│                                                               │
│  writes plan → groundcontrol check → veriplan check → loop or done
│                                                               │
│  groundcontrol:  "these NL specs are ambiguous, fix them first"   │
│  veriplan:   "these temporal constraints are violated"         │
└──────────────────────────────────────────────────────────────┘
```

| | veriplan | groundcontrol |
|---|---|---|
| **What it checks** | Plan structure (task ordering, phases, LTL) | NL phrasing (can specs be grounded to tasks?) |
| **Input** | OpenSpec markdown (tasks.md, specs/) | Same OpenSpec markdown |
| **Output** | Violations + counterexamples | Grounded atoms + directives for the LLM |
| **When it runs** | After specs are well-grounded | Before veriplan, as a pre-check |
| **Exit codes** | 0 = valid, 1 = violations | 0 = all grounded, 1 = issues found |

The two tools are independent — groundcontrol has no dependency on veriplan. They share the same input format (OpenSpec) but have separate parsers and output different things.

---

## Usage

```bash
# Check grounding quality (default: human-readable)
groundcontrol check my-change

# JSON output for machine consumption
groundcontrol check my-change --format json

# Emit the signature JSON
groundcontrol signature my-change

# Check by path instead of change name
groundcontrol check ./openspec/changes/my-change
```

### Exit Codes

| Code | Meaning | Action |
|---|---|---|
| 0 | All specs grounded | Proceed to veriplan check |
| 1 | Some specs ambiguous or ungroundable | Fix NL phrasing, re-run |
| 2 | Error (missing file, parse error) | Check input |

### Output Format

On success (exit 0): **completely silent**. No stdout, just exit code.

On error (exit 1): successes grouped first (quiet one-liners), then errors grouped after (loud with full details):

```
✓ specs/ordering/spec.md#L5 → BEFORE(T2.1, T2.2)
✓ specs/safety/spec.md#L3 → ALWAYS(T6.1)

✗ specs/safety/spec.md#L7
   "The rollback SHALL always be available"
   Could not ground: no task reference found
   Close matches: 'T6.1' (alias: 'rollback'), 'T1.5' (alias: 'rollback trigger')
   Suggestion: Use 'T6.1 SHALL ALWAYS be available' or add '(T6.1)' after 'rollback'.

Summary: 3 total | 2 grounded | 0 ambiguous | 1 ungroundable
```

---

## How It Differs From the Paper

[GinSign](https://arxiv.org/abs/2512.16770) uses a BERT-based classifier for hierarchical predicate-then-argument grounding. groundcontrol uses a **rule-based grounder** (keyword matching + positional heuristics) with no ML dependency. This is sufficient for structured OpenSpec specs that use explicit task IDs and temporal keywords. A learned grounder can be added later via the `Grounder` trait.

---

## Project Structure

```
src/
├── main.rs              # CLI, spec parsing, directive generation
├── types.rs             # Core types: Signature, GroundedAtom, Grounder trait
├── parse/
│   ├── mod.rs           # tasks.md parser (tree-sitter)
│   └── helpers.rs       # Tree-sitter helpers
└── grounders/
    ├── mod.rs
    └── rule.rs          # Rule-based grounder (6 predicate types)
```
