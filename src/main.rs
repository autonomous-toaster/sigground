//! sigground — Ground Natural Language Into System Signatures for Temporal Logic.
//!
//! Standalone CLI that parses OpenSpec plans and checks whether NL requirement
//! statements can be grounded to the plan's task vocabulary.
//!
//! Usage:
//!   sigground check <change-name>        # by name (openspec/changes/<name>)
//!   sigground check ./path/to/change     # by path
//!   sigground check                      # auto-detect
//!   sigground check --stdin             # read plan from stdin
//!
//!   sigground signature <change-name>    # just emit the signature JSON

mod grounders;
mod parse;
mod types;

use std::path::Path;

use clap::{Parser, Subcommand};
use types::{
    CheckOutput, CheckSummary, CloseMatch, Directive, GroundedInput, Grounder, GroundingStatus,
};

#[derive(Parser)]
#[command(name = "sigground", about = "Ground NL into system signatures for temporal logic")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check grounding quality of a plan's specs
    Check {
        /// Change name, path, or omit for auto-detect
        change: Option<String>,
        /// Output format: json or human (default: human)
        #[arg(long, default_value = "human")]
        format: String,
        /// Read plan from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Emit the signature JSON for a plan
    Signature {
        /// Change name, path, or omit for auto-detect
        change: Option<String>,
        /// Read plan from stdin
        #[arg(long)]
        stdin: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { change, format, stdin } => run_check(change, &format, stdin)?,
        Commands::Signature { change, stdin } => run_signature(change, stdin)?,
    }

    Ok(())
}

fn run_check(change: Option<String>, format: &str, stdin: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;
    let changes = resolve_changes(change.as_deref(), &project_root, stdin)?;

    let mut all_outputs = Vec::new();

    if changes.is_empty() {
        // No changes found — warn on stderr, exit 0
        eprintln!("No active changes found.");
        return Ok(());
    }

    for (change_dir, change_name) in &changes {
        let output = check_single(change_dir, change_name)?;
        all_outputs.push(output);
    }

    // Merge outputs
    let merged = merge_outputs(&all_outputs);
    let has_issues = merged.summary.ungroundable > 0 || merged.summary.ambiguous > 0;

    if has_issues {
        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&merged)?),
            _ => print_human(&merged),
        }
        std::process::exit(1);
    }

    Ok(())
}

fn check_single(change_dir: &std::path::Path, change_name: &str) -> anyhow::Result<CheckOutput> {
    let tasks_path = change_dir.join("tasks.md");
    let tasks_source = std::fs::read_to_string(&tasks_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", tasks_path.display(), e))?;
    let sig = parse::signature_from_tasks(&tasks_source, &tasks_path)?;

    let specs_dir = change_dir.join("specs");
    let statements = parse_specs(&specs_dir)?;

    let grounder = grounders::RuleGrounder;
    let mut inputs = Vec::new();
    for stmt in &statements {
        let result = grounder.ground(&stmt.text, &sig);
        let directives = generate_directives(&result, stmt, &sig);
        inputs.push(GroundedInput {
            source: stmt.source.clone(),
            text: stmt.text.clone(),
            groundings: result.candidates,
            status: result.status,
            directives,
        });
    }

    let summary = CheckSummary {
        total: inputs.len(),
        grounded: inputs.iter().filter(|i| i.status == GroundingStatus::Grounded).count(),
        ambiguous: inputs.iter().filter(|i| i.status == GroundingStatus::Ambiguous).count(),
        ungroundable: inputs.iter().filter(|i| i.status == GroundingStatus::Ungroundable).count(),
    };

    Ok(CheckOutput {
        plan_name: change_name.to_string(),
        signature: sig.summary(),
        inputs,
        summary,
    })
}

fn merge_outputs(outputs: &[CheckOutput]) -> CheckOutput {
    if outputs.is_empty() {
        return CheckOutput {
            plan_name: String::new(),
            signature: types::SignatureSummary { tasks: 0, predicates: 0, types: 0 },
            inputs: vec![],
            summary: CheckSummary { total: 0, grounded: 0, ambiguous: 0, ungroundable: 0 },
        };
    }
    if outputs.len() == 1 {
        return outputs[0].clone();
    }

    let names: Vec<&str> = outputs.iter().map(|o| o.plan_name.as_str()).collect();
    let mut all_inputs = Vec::new();
    let mut total = 0;
    let mut grounded = 0;
    let mut ambiguous = 0;
    let mut ungroundable = 0;

    for o in outputs {
        for input in &o.inputs {
            let mut prefixed = input.clone();
            prefixed.source = format!("{}: {}", o.plan_name, prefixed.source);
            all_inputs.push(prefixed);
        }
        total += o.summary.total;
        grounded += o.summary.grounded;
        ambiguous += o.summary.ambiguous;
        ungroundable += o.summary.ungroundable;
    }

    CheckOutput {
        plan_name: names.join(", "),
        signature: types::SignatureSummary { tasks: 0, predicates: 0, types: 0 },
        inputs: all_inputs,
        summary: CheckSummary { total, grounded, ambiguous, ungroundable },
    }
}

fn run_signature(change: Option<String>, stdin: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;
    let changes = resolve_changes(change.as_deref(), &project_root, stdin)?;
    let (change_dir, _) = &changes[0];

    let tasks_path = change_dir.join("tasks.md");
    let tasks_source = std::fs::read_to_string(&tasks_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", tasks_path.display(), e))?;
    let sig = parse::signature_from_tasks(&tasks_source, &tasks_path)?;

    println!("{}", serde_json::to_string_pretty(&sig)?);
    Ok(())
}

// ── Input resolution ──

fn resolve_changes(
    change: Option<&str>,
    project_root: &Path,
    stdin: bool,
) -> anyhow::Result<Vec<(std::path::PathBuf, String)>> {
    if stdin {
        anyhow::bail!("stdin mode not yet implemented");
    }

    match change {
        Some(name) => {
            // Try as a change name in openspec/changes/<name>
            let as_change = project_root.join("openspec").join("changes").join(name);
            if as_change.join("tasks.md").exists() {
                return Ok(vec![(as_change, name.to_string())]);
            }
            // Try as a direct path
            let as_path = Path::new(name);
            if as_path.join("tasks.md").exists() {
                let label = as_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(name)
                    .to_string();
                return Ok(vec![(as_path.to_path_buf(), label)]);
            }
            // Try as a project root with openspec/changes/
            let changes_dir = as_path.join("openspec").join("changes");
            if changes_dir.exists() {
                let mut changes: Vec<_> = std::fs::read_dir(&changes_dir)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", changes_dir.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                return Ok(changes); // empty vec is valid
            }
            // Try as a changes directory (contains subdirs with tasks.md)
            if as_path.is_dir() {
                let mut changes: Vec<_> = std::fs::read_dir(as_path)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", as_path.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                if !changes.is_empty() {
                    return Ok(changes);
                }
            }
            anyhow::bail!("Change '{}' not found (no tasks.md)", name);
        }
        None => {
            // Auto-detect: return ALL changes in openspec/changes/
            let changes_dir = project_root.join("openspec").join("changes");
            if changes_dir.exists() {
                let mut changes: Vec<_> = std::fs::read_dir(&changes_dir)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", changes_dir.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                return Ok(changes); // empty vec is valid — caller handles it
            }
            Ok(vec![])// no openspec/changes/ — empty is valid
        }
    }
}

// ── Spec parsing ──

struct Statement {
    source: String,
    text: String,
}

fn parse_specs(specs_dir: &Path) -> anyhow::Result<Vec<Statement>> {
    let mut statements = Vec::new();

    if !specs_dir.exists() {
        return Ok(statements);
    }

    collect_spec_files(specs_dir, specs_dir, &mut statements)?;
    Ok(statements)
}

fn collect_spec_files(
    base: &Path,
    dir: &Path,
    out: &mut Vec<Statement>,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_spec_files(base, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md")
            && path.file_stem().and_then(|s| s.to_str()) == Some("spec")
        {
            let source = std::fs::read_to_string(&path)?;
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            extract_requirements(&source, &rel, out);
        }
    }
    Ok(())
}

/// Simple line-based extraction of requirement SHALL statements from spec.md.
/// This is a lightweight alternative to full tree-sitter parsing of specs.
fn extract_requirements(source: &str, rel_path: &str, out: &mut Vec<Statement>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    let mut in_requirement = false;
    let mut req_line = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("### Requirement:") || line.starts_with("### requirement:") {
            in_requirement = true;
            req_line = i + 1; // 1-indexed
            i += 1;
            continue;
        }

        if in_requirement {
            // Collect body paragraphs until next heading
            let mut body = String::new();
            while i < lines.len() {
                let bl = lines[i].trim();
                if bl.starts_with("###") || bl.starts_with("####") {
                    break;
                }
                if !bl.is_empty() {
                    if !body.is_empty() {
                        body.push(' ');
                    }
                    body.push_str(bl);
                }
                i += 1;
            }

            // Extract SHALL statement from body
            let statement = extract_shall(&body);
            if !statement.is_empty() {
                out.push(Statement {
                    source: format!("{}#L{}", rel_path, req_line),
                    text: statement,
                });
            }
            in_requirement = false;
            continue;
        }

        i += 1;
    }
}

/// Extract the first sentence containing SHALL/MUST from body text.
/// Avoids splitting on periods within task IDs (e.g., "T3.1").
fn extract_shall(body: &str) -> String {
    // Split on sentence boundaries: ". " (period+space), "! ", "? ", or end of string.
    // This avoids splitting on periods inside task IDs like "T3.1".
    let sentences: Vec<&str> = body
        .split(['!', '?'])
        .flat_map(|part| {
            // Further split on ". " but not on standalone "."
            let mut result = Vec::new();
            let mut start = 0;
            let bytes = part.as_bytes();
            for i in 0..part.len().saturating_sub(1) {
                if bytes[i] == b'.' && bytes[i + 1] == b' ' {
                    result.push(&part[start..i]);
                    start = i + 2;
                }
            }
            result.push(&part[start..]);
            result
        })
        .collect();
    for s in &sentences {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        let upper = trimmed.to_uppercase();
        if upper.contains("SHALL") || upper.contains("MUST") {
            return trimmed.to_string();
        }
    }
    // Fallback: return the first non-empty line
    body.lines().next().unwrap_or("").trim().to_string()
}

// ── Directive generation ──

fn generate_directives(
    result: &types::GroundingResult,
    stmt: &Statement,
    sig: &types::Signature,
) -> Vec<Directive> {
    match result.status {
        GroundingStatus::Grounded => vec![],

        GroundingStatus::Ambiguous => {
            let best = &result.candidates[0];
            let mut dirs = Vec::new();

            // Check for BEFORE with only one argument — common mistake
            if best.predicate == "BEFORE" && best.arguments.len() < 2 {
                dirs.push(Directive {
                    severity: "warning".to_string(),
                    element: stmt.source.clone(),
                    detail: format!(
                        "BEFORE requires two task IDs, but only '{}' was found.",
                        best.arguments.first().map(|s| s.as_str()).unwrap_or("?"),
                    ),
                    suggested_action: "Add a second task ID after 'BEFORE'. Example: 'T2.1 SHALL complete BEFORE T3.1 SHALL run'.".to_string(),
                    close_matches: None,
                });
            }

            // Find which arguments were matched via alias vs exact name
            for (i, arg) in best.arguments.iter().enumerate() {
                if !stmt.text.contains(arg)
                    && let Some(constant) = sig.constants.iter().find(|c| c.name == *arg) {
                        let alias = constant.aliases.first().cloned().unwrap_or_default();
                        dirs.push(Directive {
                            severity: "warning".to_string(),
                            element: stmt.source.clone(),
                            detail: format!(
                                "Argument {} matched '{}' via alias '{}' (confidence: {:.2}). \
                                 Add explicit '{}' to raise confidence.",
                                i + 1,
                                arg,
                                alias,
                                best.confidence,
                                arg,
                            ),
                            suggested_action: format!(
                                "Replace the NL reference with '{}' or add '({})' after it.",
                                arg, arg,
                            ),
                            close_matches: None,
                        });
                    }
            }

            if dirs.is_empty() {
                dirs.push(Directive {
                    severity: "warning".to_string(),
                    element: stmt.source.clone(),
                    detail: format!(
                        "Grounded with low confidence ({:.2}). Consider using explicit task IDs.",
                        best.confidence,
                    ),
                    suggested_action: "Add explicit task ID references (e.g., 'T2.1') to the requirement statement.".to_string(),
                    close_matches: None,
                });
            }

            dirs
        }

        GroundingStatus::Ungroundable => {
            let close_matches = find_close_matches(&stmt.text, sig);
            let suggestions: Vec<String> = close_matches
                .iter()
                .map(|m| format!("'{}' (alias: '{}')", m.constant, m.alias))
                .collect();

            let suggested = if suggestions.is_empty() {
                "Add a task ID reference (e.g., 'T5.1') or a known predicate keyword (BEFORE, AFTER, CONCURRENTLY, IF...THEN, ALWAYS, AT MOST ONE).".to_string()
            } else {
                format!(
                    "Close matches: {}. Use one of these task IDs explicitly in the requirement.",
                    suggestions.join(", ")
                )
            };

            vec![Directive {
                severity: "blocker".to_string(),
                element: stmt.source.clone(),
                detail: format!(
                    "Could not ground: '{}'. No matching task or predicate found.",
                    stmt.text,
                ),
                suggested_action: suggested,
                close_matches: Some(close_matches),
            }]
        }
    }
}

/// Find close matches for ungroundable text by comparing against constant names and aliases.
fn find_close_matches(text: &str, sig: &types::Signature) -> Vec<CloseMatch> {
    let lower = text.to_lowercase();
    let mut matches: Vec<CloseMatch> = sig
        .constants
        .iter()
        .filter_map(|c| {
            // Compare against the constant name itself (e.g., "T1.3")
            let name_score = strsim(&lower, &c.name.to_lowercase());
            // Compare against all aliases
            let best_alias = c
                .aliases
                .iter()
                .map(|a| (a, strsim(&lower, a)))
                .max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).unwrap_or(std::cmp::Ordering::Equal));

            let (best_alias_str, best_alias_score) = match best_alias {
                Some((a, s)) if s > name_score => (a.clone(), s),
                _ => (c.name.clone(), name_score),
            };

            if best_alias_score > 0.4 {
                Some(CloseMatch {
                    constant: c.name.clone(),
                    alias: best_alias_str,
                    similarity: best_alias_score,
                })
            } else {
                None
            }
        })
        .collect();
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(5);
    matches
}

/// Simple string similarity (Dice coefficient on bigrams).
fn strsim(a: &str, b: &str) -> f64 {
    let a_bigrams: Vec<Vec<char>> = a
        .as_bytes()
        .windows(2)
        .map(|w| w.iter().map(|&c| c as char).collect())
        .collect();
    let b_bigrams: Vec<Vec<char>> = b
        .as_bytes()
        .windows(2)
        .map(|w| w.iter().map(|&c| c as char).collect())
        .collect();

    if a_bigrams.is_empty() || b_bigrams.is_empty() {
        return 0.0;
    }

    let intersection = a_bigrams
        .iter()
        .filter(|bg| b_bigrams.contains(bg))
        .count();
    (2.0 * intersection as f64) / (a_bigrams.len() + b_bigrams.len()) as f64
}

// ── Human-readable output ──
// Grouped: successes first (quiet), then errors (loud).

fn print_human(output: &CheckOutput) {
    // Pass 1: successes — quiet one-liners
    for input in &output.inputs {
        if input.status != GroundingStatus::Grounded {
            continue;
        }
        if let Some(g) = input.groundings.first() {
            println!(
                "\u{2713} {} \u{2192} {}({})",
                input.source, g.predicate, g.arguments.join(", ")
            );
        }
    }

    // Pass 2: errors — loud with full details
    for input in &output.inputs {
        match input.status {
            GroundingStatus::Grounded => {}
            GroundingStatus::Ambiguous => {
                println!("\u{26A0} {}", input.source);
                println!("   \"{}\"", input.text);
                if let Some(g) = input.groundings.first() {
                    println!(
                        "   \u{2192} {}({}) @ {:.2}",
                        g.predicate,
                        g.arguments.join(", "),
                        g.confidence
                    );
                }
                for d in &input.directives {
                    println!("   Suggestion: {}", d.suggested_action);
                }
            }
            GroundingStatus::Ungroundable => {
                println!("\u{2717} {}", input.source);
                println!("   \"{}\"", input.text);
                for d in &input.directives {
                    println!("   {}", d.detail);
                    if let Some(matches) = &d.close_matches
                        && !matches.is_empty() {
                            println!(
                                "   Close matches: {}",
                                matches
                                    .iter()
                                    .map(|m| format!("'{}' (alias: '{}')", m.constant, m.alias))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                    println!("   Suggestion: {}", d.suggested_action);
                }
            }
        }
    }

    // Summary line only if there are issues
    let has_issues = output.summary.ungroundable > 0 || output.summary.ambiguous > 0;
    if has_issues {
        println!(
            "Summary: {} total | {} grounded | {} ambiguous | {} ungroundable",
            output.summary.total,
            output.summary.grounded,
            output.summary.ambiguous,
            output.summary.ungroundable
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CheckOutput, CheckSummary, CloseMatch, ConstantDef, GroundedAtom, GroundingResult,
        GroundingStatus, Signature, SignatureSummary, TypeDef,
    };
    use crate::GroundingStatus as GS;

    #[test]
    fn test_strsim_identical() {
        let s = strsim("hello", "hello");
        assert!((s - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_strsim_similar() {
        let s = strsim("migration", "migrate");
        assert!(s > 0.4);
    }

    #[test]
    fn test_strsim_different() {
        let s = strsim("abc", "xyz");
        assert!(s < 0.1);
    }

    #[test]
    fn test_strsim_empty() {
        assert_eq!(strsim("", "hello"), 0.0);
        assert_eq!(strsim("hello", ""), 0.0);
        assert_eq!(strsim("", ""), 0.0);
    }

    #[test]
    fn test_strsim_short() {
        let s = strsim("a", "b");
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_extract_shall_simple() {
        let body = "T1.1 SHALL complete BEFORE T1.2 SHALL run.";
        let result = extract_shall(body);
        // Trailing period is preserved (split on ". " not just ".")
        assert_eq!(result, "T1.1 SHALL complete BEFORE T1.2 SHALL run.");
    }

    #[test]
    fn test_extract_shall_with_task_ids() {
        let body = "T3.1 SHALL parse tasks.md BEFORE T4.1 SHALL ground NL statements. T3.1 SHALL produce a Signature.";
        let result = extract_shall(body);
        // Should not split on periods inside task IDs like "T3.1"
        assert!(result.contains("T3.1"));
        assert!(result.contains("BEFORE"));
        assert!(result.contains("T4.1"));
    }

    #[test]
    fn test_extract_shall_multiple_sentences() {
        let body = "First sentence. T5.1 SHALL generate LTL. Third sentence.";
        let result = extract_shall(body);
        assert_eq!(result, "T5.1 SHALL generate LTL");
    }

    #[test]
    fn test_extract_shall_no_shall() {
        let body = "Just some text without any SHALL or MUST keywords.";
        let result = extract_shall(body);
        assert_eq!(result, "Just some text without any SHALL or MUST keywords.");
    }

    #[test]
    fn test_extract_shall_must() {
        let body = "The system MUST handle errors. Other text.";
        let result = extract_shall(body);
        assert_eq!(result, "The system MUST handle errors");
    }

    #[test]
    fn test_find_close_matches() {
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T2.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["migration".into(), "migrate".into()],
                },
                ConstantDef {
                    name: "T3.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["deployment".into(), "deploy".into()],
                },
            ],
        };
        let matches = find_close_matches("The migration step", &sig);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.constant == "T2.1"));
    }

    #[test]
    fn test_find_close_matches_no_match() {
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T1.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["setup".into()],
                },
            ],
        };
        let matches = find_close_matches("xyzzy", &sig);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_close_matches_truncated() {
        let mut constants = Vec::new();
        for i in 0..20 {
            constants.push(ConstantDef {
                name: format!("T{}.1", i),
                type_name: "task_id".into(),
                aliases: vec![format!("task{}", i)],
            });
        }
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants,
        };
        let matches = find_close_matches("task5", &sig);
        assert!(matches.len() <= 5);
        assert!(matches.iter().any(|m| m.constant == "T5.1"));
    }

    #[test]
    fn test_generate_directives_grounded() {
        let result = GroundingResult {
            candidates: vec![GroundedAtom {
                predicate: "BEFORE".into(),
                arguments: vec!["T1.1".into(), "T1.2".into()],
                confidence: 0.95,
            }],
            status: GroundingStatus::Grounded,
        };
        let stmt = Statement {
            source: "test.md#L1".into(),
            text: "T1.1 BEFORE T1.2".into(),
        };
        let sig = Signature {
            types: vec![],
            predicates: vec![],
            constants: vec![],
        };
        let dirs = generate_directives(&result, &stmt, &sig);
        assert!(dirs.is_empty());
    }

    #[test]
    fn test_generate_directives_ungroundable() {
        let result = GroundingResult {
            candidates: vec![],
            status: GroundingStatus::Ungroundable,
        };
        let stmt = Statement {
            source: "test.md#L1".into(),
            text: "The system SHALL be robust".into(),
        };
        let sig = Signature {
            types: vec![],
            predicates: vec![],
            constants: vec![],
        };
        let dirs = generate_directives(&result, &stmt, &sig);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].severity, "blocker");
    }

    #[test]
    fn test_generate_directives_ambiguous() {
        let result = GroundingResult {
            candidates: vec![GroundedAtom {
                predicate: "BEFORE".into(),
                arguments: vec!["T2.1".into(), "T2.2".into()],
                confidence: 0.72,
            }],
            status: GroundingStatus::Ambiguous,
        };
        let stmt = Statement {
            source: "test.md#L1".into(),
            text: "migration before testing".into(),
        };
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T2.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["migration".into()],
                },
                ConstantDef {
                    name: "T2.2".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["testing".into()],
                },
            ],
        };
        let dirs = generate_directives(&result, &stmt, &sig);
        assert!(!dirs.is_empty());
        assert_eq!(dirs[0].severity, "warning");
    }

    #[test]
    fn test_find_close_matches_by_constant_name() {
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T1.3".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["implement appconfig".into()],
                },
            ],
        };
        // The text "T1.3" should match the constant name directly
        let matches = find_close_matches("T1.3", &sig);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.constant == "T1.3"));
    }

    #[test]
    fn test_generate_directives_before_one_arg() {
        let result = GroundingResult {
            candidates: vec![GroundedAtom {
                predicate: "BEFORE".into(),
                arguments: vec!["T3.1".into()],
                confidence: 0.5,
            }],
            status: GroundingStatus::Ambiguous,
        };
        let stmt = Statement {
            source: "test.md#L1".into(),
            text: "T3.1 SHALL complete before any pool is used".into(),
        };
        let sig = Signature {
            types: vec![TypeDef { name: "task_id".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T3.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["setup".into()],
                },
            ],
        };
        let dirs = generate_directives(&result, &stmt, &sig);
        assert!(!dirs.is_empty());
        assert!(dirs.iter().any(|d| d.detail.contains("BEFORE requires two")));
    }
}
