//! Core types: Signature, GroundedAtom, GroundingResult, Directive.

use serde::{Deserialize, Serialize};

// ── System Signature (GinSign §2.2) ──

/// A many-sorted system signature: the vocabulary of a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub types: Vec<TypeDef>,
    pub predicates: Vec<PredicateDef>,
    pub constants: Vec<ConstantDef>,
}

impl Signature {
    pub fn summary(&self) -> SignatureSummary {
        SignatureSummary {
            tasks: self.constants.len(),
            predicates: self.predicates.len(),
            types: self.types.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureSummary {
    pub tasks: usize,
    pub predicates: usize,
    pub types: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateDef {
    pub name: String,
    pub arguments: Vec<ArgSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSlot {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: String,
    pub type_name: String,
    /// Alternative NL forms for fuzzy matching (from task descriptions).
    pub aliases: Vec<String>,
}

// ── Grounding ──

/// A grounded atom: predicate + concrete arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAtom {
    pub predicate: String,
    pub arguments: Vec<String>,
    pub confidence: f64,
}

/// Overall status of a grounding attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroundingStatus {
    /// At least one candidate with confidence >= 0.8.
    Grounded,
    /// Candidates exist but all below 0.8.
    Ambiguous,
    /// No candidates found.
    Ungroundable,
}

/// Result of grounding one NL span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingResult {
    pub candidates: Vec<GroundedAtom>,
    pub status: GroundingStatus,
}

// ── Directives (feedback to the LLM) ──

/// A close match suggestion for ungroundable text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMatch {
    pub constant: String,
    pub alias: String,
    pub similarity: f64,
}

/// A structured directive telling the LLM what to fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub severity: String,
    pub element: String,
    pub detail: String,
    pub suggested_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_matches: Option<Vec<CloseMatch>>,
}

// ── Check output ──

/// One grounded input with its directives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedInput {
    pub source: String,
    pub text: String,
    pub groundings: Vec<GroundedAtom>,
    pub status: GroundingStatus,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directives: Vec<Directive>,
}

/// Summary counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSummary {
    pub total: usize,
    pub grounded: usize,
    pub ambiguous: usize,
    pub ungroundable: usize,
}

/// Full check output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOutput {
    pub plan_name: String,
    pub signature: SignatureSummary,
    pub inputs: Vec<GroundedInput>,
    pub summary: CheckSummary,
}

// ── Grounder trait ──

/// Pluggable grounding strategy.
pub trait Grounder: Send + Sync {
    /// Ground one NL span against a signature.
    fn ground(&self, nl: &str, sig: &Signature) -> GroundingResult;

    /// Ground multiple NL spans.
    fn ground_batch(&self, inputs: &[&str], sig: &Signature) -> Vec<GroundingResult> {
        inputs.iter().map(|nl| self.ground(nl, sig)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_json_roundtrip() {
        let sig = Signature {
            types: vec![TypeDef {
                name: "task_id".into(),
            }],
            predicates: vec![PredicateDef {
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
            }],
            constants: vec![ConstantDef {
                name: "T1.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["setup".into()],
            }],
        };
        let json = serde_json::to_string_pretty(&sig).unwrap();
        let back: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(back.types.len(), 1);
        assert_eq!(back.predicates.len(), 1);
        assert_eq!(back.constants.len(), 1);
        assert_eq!(back.constants[0].name, "T1.1");
    }

    #[test]
    fn test_signature_summary() {
        let sig = Signature {
            types: vec![TypeDef { name: "t".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T1.1".into(),
                    type_name: "t".into(),
                    aliases: vec![],
                },
                ConstantDef {
                    name: "T1.2".into(),
                    type_name: "t".into(),
                    aliases: vec![],
                },
            ],
        };
        let s = sig.summary();
        assert_eq!(s.tasks, 2);
        assert_eq!(s.predicates, 0);
        assert_eq!(s.types, 1);
    }

    #[test]
    fn test_grounding_status_equality() {
        assert_eq!(GroundingStatus::Grounded, GroundingStatus::Grounded);
        assert_ne!(GroundingStatus::Grounded, GroundingStatus::Ambiguous);
        assert_ne!(GroundingStatus::Ambiguous, GroundingStatus::Ungroundable);
    }

    #[test]
    fn test_grounded_atom_construction() {
        let atom = GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec!["T1.1".into(), "T1.2".into()],
            confidence: 0.95,
        };
        assert_eq!(atom.predicate, "BEFORE");
        assert_eq!(atom.arguments.len(), 2);
        assert!(atom.confidence > 0.9);
    }

    #[test]
    fn test_directive_construction() {
        let d = Directive {
            severity: "blocker".into(),
            element: "spec.md#L3".into(),
            detail: "No match found".into(),
            suggested_action: "Add task ID".into(),
            close_matches: Some(vec![CloseMatch {
                constant: "T1.1".into(),
                alias: "setup".into(),
                similarity: 0.85,
            }]),
        };
        assert_eq!(d.severity, "blocker");
        assert!(d.close_matches.is_some());
        assert_eq!(d.close_matches.unwrap().len(), 1);
    }

    #[test]
    fn test_check_output_serialization() {
        let output = CheckOutput {
            plan_name: "test".into(),
            signature: SignatureSummary {
                tasks: 1,
                predicates: 1,
                types: 1,
            },
            inputs: vec![],
            summary: CheckSummary {
                total: 0,
                grounded: 0,
                ambiguous: 0,
                ungroundable: 0,
            },
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("plan_name"));
        assert!(json.contains("signature"));
        assert!(json.contains("summary"));
    }
}
