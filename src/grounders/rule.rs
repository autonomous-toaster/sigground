//! Rule-based grounder: keyword matching + positional heuristics.
//!
//! No ML dependency. Works well for structured OpenSpec specs
//! that use explicit task IDs (T1.3) and temporal keywords.

use crate::types::{
    ConstantDef, GroundedAtom, Grounder, GroundingResult, GroundingStatus, Signature,
};

/// Predicate keyword groups and their argument ordering.
const PREDICATE_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "BEFORE",
        &["before", "complete before", "must finish before"],
    ),
    ("AFTER", &["after", "only after"]),
    (
        "CONCURRENTLY",
        &[
            "concurrently",
            "in parallel",
            "simultaneously",
            "at the same time",
        ],
    ),
    ("IF_THEN", &["if", "unless", "in case of", "when ... then"]),
    ("ALWAYS", &["always", "throughout", "at all times"]),
    (
        "AT_MOST_ONE",
        &[
            "at most one",
            "mutually exclusive",
            "not concurrently",
            "not together",
        ],
    ),
];

pub struct RuleGrounder;

impl Grounder for RuleGrounder {
    fn ground(&self, nl: &str, sig: &Signature) -> GroundingResult {
        let lower = nl.to_lowercase();
        let mut candidates = Vec::new();

        for (pred_name, keywords) in PREDICATE_KEYWORDS {
            // 1. Predicate matching: does the text contain any keyword for this predicate?
            let matched = keywords.iter().any(|kw| lower.contains(kw));
            if !matched {
                continue;
            }

            // 2. Argument extraction: find all matching constants in the text
            let matched_constants: Vec<&ConstantDef> = sig
                .constants
                .iter()
                .filter(|c| appears_in(c, nl, &lower))
                .collect();

            // 3. Find the predicate definition
            let pred_def = sig.predicates.iter().find(|p| p.name == *pred_name);

            // 4. Assign arguments based on predicate-specific positional heuristics
            let args = assign_arguments(pred_name, &matched_constants, nl, &lower);

            // 5. Confidence scoring
            let expected = pred_def.map(|p| p.arguments.len()).unwrap_or(0);
            let confidence = if args.len() == expected && expected > 0 {
                0.95
            } else if !args.is_empty() {
                0.5
            } else {
                0.0
            };

            if confidence > 0.0 {
                candidates.push(GroundedAtom {
                    predicate: pred_name.to_string(),
                    arguments: args,
                    confidence,
                });
            }
        }

        // Sort by confidence descending
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let status = if candidates.iter().any(|c| c.confidence >= 0.8) {
            GroundingStatus::Grounded
        } else if !candidates.is_empty() {
            GroundingStatus::Ambiguous
        } else {
            GroundingStatus::Ungroundable
        };

        GroundingResult { candidates, status }
    }
}

/// Check if a constant appears in the NL text (by name or alias).
fn appears_in(c: &ConstantDef, nl: &str, lower: &str) -> bool {
    // Exact name match (e.g., "T2.1")
    if nl.contains(&c.name) {
        return true;
    }
    // Alias match (e.g., "migration" from task description)
    c.aliases.iter().any(|a| lower.contains(a))
}

/// Assign arguments to slots based on predicate-specific heuristics.
fn assign_arguments(
    pred_name: &str,
    matched: &[&ConstantDef],
    nl: &str,
    lower: &str,
) -> Vec<String> {
    match pred_name {
        "BEFORE" => {
            // "X BEFORE Y" → earlier=X, later=Y
            // Find the split point around "before"
            if let Some(pos) = lower.find("before") {
                let before_text = &nl[..pos];
                let after_text = &nl[pos + 6..];
                let before_match = matched.iter().find(|c| appears_in_slice(c, before_text));
                let after_match = matched.iter().find(|c| appears_in_slice(c, after_text));
                let mut args = Vec::new();
                if let Some(c) = before_match {
                    args.push(c.name.clone());
                }
                if let Some(c) = after_match {
                    args.push(c.name.clone());
                }
                args
            } else {
                matched.iter().take(2).map(|c| c.name.clone()).collect()
            }
        }
        "AFTER" => {
            // "X AFTER Y" → earlier=Y, later=X
            if let Some(pos) = lower.find("after") {
                let before_text = &nl[..pos];
                let after_text = &nl[pos + 5..];
                let before_match = matched.iter().find(|c| appears_in_slice(c, before_text));
                let after_match = matched.iter().find(|c| appears_in_slice(c, after_text));
                let mut args = Vec::new();
                // AFTER: the thing before "after" is the later task
                if let Some(c) = after_match {
                    args.push(c.name.clone()); // earlier
                }
                if let Some(c) = before_match {
                    args.push(c.name.clone()); // later
                }
                args
            } else {
                matched.iter().take(2).map(|c| c.name.clone()).collect()
            }
        }
        "CONCURRENTLY" | "AT_MOST_ONE" => matched.iter().take(2).map(|c| c.name.clone()).collect(),
        "IF_THEN" => {
            // "IF X fails THEN Y" → trigger=X, consequent=Y
            if let Some(if_pos) = lower.find("if") {
                let after_if = &lower[if_pos + 2..];
                let then_pos = after_if.find("then").unwrap_or(usize::MAX);
                let trigger_text = &nl[if_pos + 2..if_pos + 2 + then_pos.min(after_if.len())];
                let consequent_text = if then_pos < after_if.len() {
                    &nl[if_pos + 2 + then_pos + 4..]
                } else {
                    ""
                };
                let trigger = matched.iter().find(|c| appears_in_slice(c, trigger_text));
                let consequent = matched
                    .iter()
                    .find(|c| appears_in_slice(c, consequent_text));
                let mut args = Vec::new();
                if let Some(c) = trigger {
                    args.push(c.name.clone());
                }
                if let Some(c) = consequent {
                    args.push(c.name.clone());
                }
                args
            } else {
                matched.iter().take(2).map(|c| c.name.clone()).collect()
            }
        }
        "ALWAYS" => matched.iter().take(1).map(|c| c.name.clone()).collect(),
        _ => matched.iter().map(|c| c.name.clone()).collect(),
    }
}

/// Check if a constant appears in a slice of the NL text.
fn appears_in_slice(c: &ConstantDef, slice: &str) -> bool {
    let lower = slice.to_lowercase();
    slice.contains(&c.name) || c.aliases.iter().any(|a| lower.contains(a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArgSlot, ConstantDef, PredicateDef, Signature, TypeDef};

    fn test_sig() -> Signature {
        Signature {
            types: vec![TypeDef {
                name: "task_id".into(),
            }],
            predicates: vec![
                PredicateDef {
                    name: "BEFORE".into(),
                    arguments: vec![
                        ArgSlot {
                            name: "earlier".into(),
                            type_name: "task_id".into(),
                        },
                        ArgSlot {
                            name: "later".into(),
                            type_name: "task_id".into(),
                        },
                    ],
                },
                PredicateDef {
                    name: "AFTER".into(),
                    arguments: vec![
                        ArgSlot {
                            name: "earlier".into(),
                            type_name: "task_id".into(),
                        },
                        ArgSlot {
                            name: "later".into(),
                            type_name: "task_id".into(),
                        },
                    ],
                },
                PredicateDef {
                    name: "CONCURRENTLY".into(),
                    arguments: vec![
                        ArgSlot {
                            name: "a".into(),
                            type_name: "task_id".into(),
                        },
                        ArgSlot {
                            name: "b".into(),
                            type_name: "task_id".into(),
                        },
                    ],
                },
                PredicateDef {
                    name: "IF_THEN".into(),
                    arguments: vec![
                        ArgSlot {
                            name: "trigger".into(),
                            type_name: "task_id".into(),
                        },
                        ArgSlot {
                            name: "consequent".into(),
                            type_name: "task_id".into(),
                        },
                    ],
                },
                PredicateDef {
                    name: "ALWAYS".into(),
                    arguments: vec![ArgSlot {
                        name: "target".into(),
                        type_name: "task_id".into(),
                    }],
                },
                PredicateDef {
                    name: "AT_MOST_ONE".into(),
                    arguments: vec![
                        ArgSlot {
                            name: "a".into(),
                            type_name: "task_id".into(),
                        },
                        ArgSlot {
                            name: "b".into(),
                            type_name: "task_id".into(),
                        },
                    ],
                },
            ],
            constants: vec![
                ConstantDef {
                    name: "T1.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["setup".into(), "create project".into(), "1.1".into()],
                },
                ConstantDef {
                    name: "T1.2".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["deps".into(), "add dependencies".into(), "1.2".into()],
                },
                ConstantDef {
                    name: "T2.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["migrate".into(), "migration".into(), "2.1".into()],
                },
                ConstantDef {
                    name: "T2.2".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["test".into(), "testing".into(), "2.2".into()],
                },
                ConstantDef {
                    name: "T3.1".into(),
                    type_name: "task_id".into(),
                    aliases: vec!["deploy".into(), "deployment".into(), "3.1".into()],
                },
            ],
        }
    }

    #[test]
    fn test_ground_before_explicit_ids() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("T1.1 SHALL complete BEFORE T1.2 SHALL run", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "BEFORE");
        assert_eq!(result.candidates[0].arguments, vec!["T1.1", "T1.2"]);
        assert!(result.candidates[0].confidence >= 0.8);
    }

    #[test]
    fn test_ground_before_with_aliases() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("The migration must finish before testing starts", &sig);
        // Both args found via aliases (migration→T2.1, testing→T2.2) → Grounded
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "BEFORE");
        assert!(result.candidates[0].arguments.contains(&"T2.1".to_string()));
        assert!(result.candidates[0].arguments.contains(&"T2.2".to_string()));
    }

    #[test]
    fn test_ground_after_explicit_ids() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("T2.2 SHALL run AFTER T2.1 SHALL complete", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "AFTER");
        // AFTER: earlier is the task after "after", later is the task before "after"
        assert_eq!(result.candidates[0].arguments, vec!["T2.1", "T2.2"]);
    }

    #[test]
    fn test_ground_concurrently() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("T2.1 and T2.2 SHALL run concurrently", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "CONCURRENTLY");
        assert!(result.candidates[0].arguments.contains(&"T2.1".to_string()));
        assert!(result.candidates[0].arguments.contains(&"T2.2".to_string()));
    }

    #[test]
    fn test_ground_if_then() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("IF T1.1 fails THEN T3.1 SHALL run", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "IF_THEN");
        assert_eq!(result.candidates[0].arguments, vec!["T1.1", "T3.1"]);
    }

    #[test]
    fn test_ground_always() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("T3.1 SHALL ALWAYS be available", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "ALWAYS");
        assert_eq!(result.candidates[0].arguments, vec!["T3.1"]);
    }

    #[test]
    fn test_ground_at_most_one() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("At most one of T2.1, T2.2 SHALL be active", &sig);
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "AT_MOST_ONE");
        assert!(result.candidates[0].arguments.contains(&"T2.1".to_string()));
        assert!(result.candidates[0].arguments.contains(&"T2.2".to_string()));
    }

    #[test]
    fn test_ground_ungroundable() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("The system SHALL be user-friendly", &sig);
        assert_eq!(result.status, GroundingStatus::Ungroundable);
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn test_ground_no_keyword() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("T1.1 does something", &sig);
        assert_eq!(result.status, GroundingStatus::Ungroundable);
    }

    #[test]
    fn test_ground_batch() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let inputs = vec!["T1.1 BEFORE T1.2", "The system SHALL be user-friendly"];
        let results = grounder.ground_batch(&inputs, &sig);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, GroundingStatus::Grounded);
        assert_eq!(results[1].status, GroundingStatus::Ungroundable);
    }

    #[test]
    fn test_ground_after_with_aliases() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("Testing SHALL start after migration completes", &sig);
        // Both args found via aliases → Grounded
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "AFTER");
    }

    #[test]
    fn test_ground_if_then_with_aliases() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        let result = grounder.ground("If setup fails then deployment SHALL run", &sig);
        // Both args found via aliases → Grounded
        assert_eq!(result.status, GroundingStatus::Grounded);
        assert_eq!(result.candidates[0].predicate, "IF_THEN");
    }

    #[test]
    fn test_ground_multiple_predicates() {
        let sig = test_sig();
        let grounder = RuleGrounder;
        // Text contains both BEFORE and ALWAYS keywords
        let result = grounder.ground(
            "T1.1 SHALL complete BEFORE T1.2. T3.1 SHALL ALWAYS be available.",
            &sig,
        );
        // Should find both predicates
        assert!(result.candidates.iter().any(|c| c.predicate == "BEFORE"));
        assert!(result.candidates.iter().any(|c| c.predicate == "ALWAYS"));
    }
}
