//! OpenSpec markdown parser — parse tasks.md into a Signature.
//!
//! Independent of veriplan. Reads the same OpenSpec directory layout
//! but produces a Signature instead of PlanIR.

mod helpers;

use std::path::Path;

use crate::types::{ArgSlot, ConstantDef, PredicateDef, Signature, TypeDef};

/// Parse tasks.md into a Signature.
pub fn signature_from_tasks(source: &str, _path: &Path) -> anyhow::Result<Signature> {
    let mut parser = tree_sitter::Parser::new();
    let lang = tree_sitter_language_pack::get_language("markdown")
        .map_err(|e| anyhow::anyhow!("Grammar error: {}", e))?;
    parser
        .set_language(&lang)
        .map_err(|e| anyhow::anyhow!("Grammar error: {}", e))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse markdown"))?;
    let root = tree.root_node();

    let mut tasks: Vec<(String, String, String)> = Vec::new(); // (id, description, phase)
    let mut phases: Vec<String> = Vec::new();
    let mut current_phase = String::new();
    let bytes = source.as_bytes();

    let mut cursor = root.walk();
    explore_tree(&mut cursor, &root, &mut |node| match node.kind() {
        "atx_heading" => {
            if let Ok(text) = helpers::node_text(node, bytes) {
                let trimmed = text.trim().trim_start_matches('#').trim();
                if !trimmed.is_empty() {
                    let level = node
                        .child(0)
                        .map(|n| n.kind())
                        .filter(|k| k.starts_with("atx_h"))
                        .map(|k| k.chars().filter(|c| c.is_ascii_digit()).collect::<String>())
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    if level == 2 {
                        current_phase = trimmed.to_string();
                        phases.push(current_phase.clone());
                    }
                }
            }
        }
        "list_item" => {
            let checked = helpers::find_child(node, "task_list_marker_unchecked").is_some()
                || helpers::find_child(node, "task_list_marker_checked").is_some();
            if checked
                && let Some(content) = helpers::find_child(node, "paragraph")
                    && let Ok(text) = helpers::node_text(&content, bytes) {
                        let text = text.trim();
                        let (id, desc) = extract_task_id(text);
                        if !id.is_empty() {
                            tasks.push((id, desc.to_string(), current_phase.clone()));
                        }
                    }
        }
        _ => {}
    });

    build_signature(tasks, phases)
}

/// Build a Signature from parsed tasks and phases.
fn build_signature(
    tasks: Vec<(String, String, String)>,
    _phases: Vec<String>,
) -> anyhow::Result<Signature> {
    let types = vec![
        TypeDef {
            name: "task_id".to_string(),
        },
        TypeDef {
            name: "phase_name".to_string(),
        },
    ];

    let predicates = vec![
        PredicateDef {
            name: "BEFORE".to_string(),
            arguments: vec![
                ArgSlot {
                    name: "earlier".to_string(),
                    type_name: "task_id".to_string(),
                },
                ArgSlot {
                    name: "later".to_string(),
                    type_name: "task_id".to_string(),
                },
            ],
        },
        PredicateDef {
            name: "AFTER".to_string(),
            arguments: vec![
                ArgSlot {
                    name: "earlier".to_string(),
                    type_name: "task_id".to_string(),
                },
                ArgSlot {
                    name: "later".to_string(),
                    type_name: "task_id".to_string(),
                },
            ],
        },
        PredicateDef {
            name: "CONCURRENTLY".to_string(),
            arguments: vec![
                ArgSlot {
                    name: "a".to_string(),
                    type_name: "task_id".to_string(),
                },
                ArgSlot {
                    name: "b".to_string(),
                    type_name: "task_id".to_string(),
                },
            ],
        },
        PredicateDef {
            name: "IF_THEN".to_string(),
            arguments: vec![
                ArgSlot {
                    name: "trigger".to_string(),
                    type_name: "task_id".to_string(),
                },
                ArgSlot {
                    name: "consequent".to_string(),
                    type_name: "task_id".to_string(),
                },
            ],
        },
        PredicateDef {
            name: "ALWAYS".to_string(),
            arguments: vec![ArgSlot {
                name: "target".to_string(),
                type_name: "task_id".to_string(),
            }],
        },
        PredicateDef {
            name: "AT_MOST_ONE".to_string(),
            arguments: vec![
                ArgSlot {
                    name: "a".to_string(),
                    type_name: "task_id".to_string(),
                },
                ArgSlot {
                    name: "b".to_string(),
                    type_name: "task_id".to_string(),
                },
            ],
        },
    ];

    let mut constants: Vec<ConstantDef> = tasks
        .into_iter()
        .map(|(id, desc, _phase)| {
            let aliases = build_aliases(&id, &desc);
            ConstantDef {
                name: format!("T{}", id),
                type_name: "task_id".to_string(),
                aliases,
            }
        })
        .collect();
    constants.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Signature {
        types,
        predicates,
        constants,
    })
}

/// Build alias list from task description.
/// E.g. "Create Rust project with Cargo" → ["create rust project", "cargo init", "rust project"]
fn build_aliases(id: &str, desc: &str) -> Vec<String> {
    let mut aliases = Vec::new();

    // The full description, lowercased
    let lower = desc.to_lowercase();
    aliases.push(lower.clone());

    // First few words as a short alias
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.len() > 3 {
        aliases.push(words[..3].join(" "));
    }
    if words.len() > 5 {
        aliases.push(words[..5].join(" "));
    }

    // Individual significant words (skip very short ones)
    for w in &words {
        if w.len() > 4 && !aliases.contains(&w.to_string()) {
            aliases.push(w.to_string());
        }
    }

    // The task ID itself as an alias (for matching "T1.3" style refs)
    aliases.push(id.to_string());

    aliases
}

// ── Tree-sitter helpers ──

fn explore_tree<'a>(
    cursor: &mut tree_sitter::TreeCursor<'a>,
    node: &tree_sitter::Node<'a>,
    f: &mut dyn FnMut(&tree_sitter::Node<'a>),
) {
    f(node);
    if cursor.goto_first_child() {
        loop {
            explore_tree(cursor, &cursor.node(), f);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

/// Extract task ID from a checklist item text.
/// "1.3 Add dependencies" → ("1.3", "1.3 Add dependencies")
/// "T1.3 SHALL..." → ("1.3", "1.3")
fn extract_task_id(text: &str) -> (String, String) {
    // Try N.M pattern first (bare, no T prefix — checklist items)
    if let Some(space_pos) = text.find(' ') {
        let candidate = &text[..space_pos];
        if let Some(dot_pos) = candidate.find('.') {
            let left = &candidate[..dot_pos];
            let right = &candidate[dot_pos + 1..];
            if !left.is_empty()
                && !right.is_empty()
                && left.chars().all(|c| c.is_ascii_digit())
                && right.chars().all(|c| c.is_ascii_digit())
            {
                return (candidate.to_string(), text.to_string());
            }
        }
    }

    // Try T-prefixed: "T1.3" → id="1.3"
    // Also handles parenthetical: "(T1.3)" → id="1.3"
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip opening parenthesis before T
        let start_t = if i + 1 < bytes.len() && bytes[i] == b'(' && bytes[i + 1] == b'T' {
            i + 2
        } else if bytes[i] == b'T' {
            i + 1
        } else {
            i += 1;
            continue;
        };
        let start = start_t;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
            end += 1;
        }
        if end > start
            && let Ok(s) = std::str::from_utf8(&bytes[start..end]) {
                return (s.to_string(), text[start..end].to_string());
            }
        i = end;
    }

    (String::new(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_task_id_n_m() {
        let (id, desc) = extract_task_id("1.3 Add dependencies");
        assert_eq!(id, "1.3");
        assert_eq!(desc, "1.3 Add dependencies");
    }

    #[test]
    fn test_extract_task_id_t_prefix() {
        let (id, desc) = extract_task_id("T1.3 SHALL complete");
        assert_eq!(id, "1.3");
        assert_eq!(desc, "1.3");
    }

    #[test]
    fn test_extract_task_id_no_match() {
        let (id, desc) = extract_task_id("no task here");
        assert!(id.is_empty());
        assert!(desc.is_empty());
    }

    #[test]
    fn test_extract_task_id_multi_dot() {
        let (id, desc) = extract_task_id("T10.7 test visualize");
        assert_eq!(id, "10.7");
    }

    #[test]
    fn test_extract_task_id_parenthetical() {
        let (id, desc) = extract_task_id("add connection pooling (T1.3)");
        assert_eq!(id, "1.3");
        assert_eq!(desc, "1.3");
    }

    #[test]
    fn test_extract_task_id_parenthetical_multi() {
        let (id, desc) = extract_task_id("setup (T1.1) before migration (T2.1)");
        assert_eq!(id, "1.1");
    }

    #[test]
    fn test_build_aliases_full() {
        let aliases = build_aliases("1.1", "Create Rust project with Cargo");
        assert!(aliases.contains(&"create rust project with cargo".to_string()));
        assert!(aliases.contains(&"create rust project".to_string()));
        assert!(aliases.contains(&"create".to_string()));
        assert!(aliases.contains(&"project".to_string()));
        assert!(aliases.contains(&"cargo".to_string()));
        assert!(aliases.contains(&"1.1".to_string()));
    }

    #[test]
    fn test_build_aliases_short_desc() {
        let aliases = build_aliases("2.1", "Setup");
        assert!(aliases.contains(&"setup".to_string()));
        assert!(aliases.contains(&"2.1".to_string()));
        // "Setup" is 5 chars, should be included as significant word
        assert!(aliases.contains(&"setup".to_string()));
    }

    #[test]
    fn test_build_aliases_skips_short_words() {
        let aliases = build_aliases("3.1", "a bc def ghij");
        // "a", "bc", "def" are <= 4 chars, should NOT be individual aliases
        assert!(!aliases.contains(&"a".to_string()));
        assert!(!aliases.contains(&"bc".to_string()));
        assert!(!aliases.contains(&"def".to_string()));
        // "ghij" is 4 chars, should NOT be included (len > 4 means >= 5)
        assert!(!aliases.contains(&"ghij".to_string()));
    }

    #[test]
    fn test_signature_from_tasks_basic() {
        let tasks_md = r#"## 1. Setup

- [ ] 1.1 Create project
- [x] 1.2 Add deps
"#;
        let sig = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md")).unwrap();
        assert_eq!(sig.constants.len(), 2);
        assert_eq!(sig.constants[0].name, "T1.1");
        assert_eq!(sig.constants[1].name, "T1.2");
        assert_eq!(sig.predicates.len(), 6);
        assert_eq!(sig.types.len(), 2);
    }

    #[test]
    fn test_signature_from_tasks_with_phases() {
        let tasks_md = r#"## 1. Setup

- [ ] 1.1 Create project

## 2. Core

- [ ] 2.1 Implement core
- [ ] 2.2 Test core
"#;
        let sig = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md")).unwrap();
        assert_eq!(sig.constants.len(), 3);
        assert!(sig.constants.iter().any(|c| c.name == "T1.1"));
        assert!(sig.constants.iter().any(|c| c.name == "T2.1"));
        assert!(sig.constants.iter().any(|c| c.name == "T2.2"));
    }

    #[test]
    fn test_signature_from_tasks_empty() {
        let tasks_md = "";
        let result = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md"));
        // Empty markdown parses to a tree with no checklist items → empty signature
        assert!(result.is_ok());
        assert_eq!(result.unwrap().constants.len(), 0);
    }

    #[test]
    fn test_signature_from_tasks_no_checklist() {
        let tasks_md = "# Just a heading\n\nSome text without checkboxes.";
        let sig = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md")).unwrap();
        assert_eq!(sig.constants.len(), 0);
    }

    #[test]
    fn test_signature_predicates_have_correct_args() {
        let tasks_md = "- [ ] 1.1 Task\n";
        let sig = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md")).unwrap();
        let before = sig.predicates.iter().find(|p| p.name == "BEFORE").unwrap();
        assert_eq!(before.arguments.len(), 2);
        assert_eq!(before.arguments[0].name, "earlier");
        assert_eq!(before.arguments[1].name, "later");

        let always = sig.predicates.iter().find(|p| p.name == "ALWAYS").unwrap();
        assert_eq!(always.arguments.len(), 1);
        assert_eq!(always.arguments[0].name, "target");
    }

    #[test]
    fn test_signature_constants_sorted() {
        let tasks_md = r#"- [ ] 1.3 Third
- [ ] 1.1 First
- [ ] 1.2 Second
"#;
        let sig = signature_from_tasks(tasks_md, std::path::Path::new("tasks.md")).unwrap();
        assert_eq!(sig.constants[0].name, "T1.1");
        assert_eq!(sig.constants[1].name, "T1.2");
        assert_eq!(sig.constants[2].name, "T1.3");
    }
}


