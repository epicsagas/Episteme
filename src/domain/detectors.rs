//! All 23 code-smell detector functions.
//!
//! Ported faithfully from `episteme.parsers.base` -- identical thresholds and
//! confidence formulas.
//!
//! ## Detector categories
//!
//! **Fully functional** (work from `CodeMetrics` alone):
//! SMELL-01, 02, 03, 04, 06, 07, 10, 11, 14, 16, 18, 20, 21, 22
//!
//! **Functional with heuristic** (uses available metrics as proxy):
//! SMELL-05 (Data Clumps -- parameter grouping heuristic)
//!
//! **Require external parameters** (caller must supply additional data):
//! SMELL-09 (Shotgun Surgery -- `dependency_count`),
//! SMELL-12 (Speculative Generality -- `subclass_count`, `usage_count`),
//! SMELL-13 (Duplicate Code -- `ast_hash` + `all_hashes` map)
//!
//! **Placeholder** (requires cross-class or whole-program analysis not
//! available from single-function metrics):
//! SMELL-15 (Parallel Inheritance Hierarchies),
//! SMELL-17 (Dead Code),
//! SMELL-19 (Inappropriate Intimacy),
//! SMELL-23 (Alternative Classes with Different Interfaces)

use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};

// -- Shared helpers ---------------------------------------------------------

fn build_detection(
    id: &str,
    name: &str,
    confidence: f64,
    location: &str,
    fn_name: &str,
    metrics: &CodeMetrics,
    reasons: Vec<String>,
) -> SmellDetection {
    SmellDetection {
        smell_id: id.into(),
        smell_name: name.into(),
        confidence,
        location: location.into(),
        function_name: fn_name.into(),
        metrics: metrics.clone(),
        reasons,
    }
}

/// Accumulator for detectors that sum confidence from multiple tiered checks.
struct TieredAccum {
    confidence: f64,
    reasons: Vec<String>,
}

impl TieredAccum {
    fn new() -> Self {
        Self {
            confidence: 0.0,
            reasons: Vec::new(),
        }
    }

    /// Two-tier threshold: value > high -> high_w + high_msg, else value > low -> low_w + low_msg.
    #[allow(clippy::too_many_arguments)]
    fn tier(
        &mut self,
        value: usize,
        high: usize,
        high_w: f64,
        high_msg: String,
        low: usize,
        low_w: f64,
        low_msg: String,
    ) {
        if value > high {
            self.reasons.push(high_msg);
            self.confidence += high_w;
        } else if value > low {
            self.reasons.push(low_msg);
            self.confidence += low_w;
        }
    }

    /// Flat (non-tiered) contribution.
    fn add(&mut self, weight: f64, reason: String) {
        self.reasons.push(reason);
        self.confidence += weight;
    }

    /// Build detection if confidence >= threshold (capped at 1.0).
    fn into_detection(
        self,
        id: &str,
        name: &str,
        location: &str,
        fn_name: &str,
        metrics: &CodeMetrics,
        threshold: f64,
    ) -> Option<SmellDetection> {
        if self.confidence >= threshold {
            Some(build_detection(
                id,
                name,
                self.confidence.min(1.0),
                location,
                fn_name,
                metrics,
                self.reasons,
            ))
        } else {
            None
        }
    }
}

// -- SMELL-01  Long Method --------------------------------------------------
// LOC>50 +0.30 | LOC>30 +0.15 | CC>15 +0.40 | CC>10 +0.25
// nesting>4 +0.20 | nesting>3 +0.10 | fires at >= 0.5

pub fn detect_long_method(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    let mut a = TieredAccum::new();
    a.tier(
        metrics.loc,
        50,
        0.3,
        format!("LOC={} exceeds 50", metrics.loc),
        30,
        0.15,
        format!("LOC={} exceeds 30", metrics.loc),
    );
    a.tier(
        metrics.cyclomatic_complexity,
        15,
        0.4,
        format!("CC={} exceeds 15", metrics.cyclomatic_complexity),
        10,
        0.25,
        format!("CC={} exceeds 10", metrics.cyclomatic_complexity),
    );
    a.tier(
        metrics.nesting_depth,
        4,
        0.2,
        format!("Nesting depth={} exceeds 4", metrics.nesting_depth),
        3,
        0.1,
        format!("Nesting depth={} exceeds 3", metrics.nesting_depth),
    );
    a.into_detection("SMELL-01", "Long Method", location, name, metrics, 0.5)
}

// -- SMELL-02  Long Parameter List ------------------------------------------
// >8 -> 0.95 | >6 -> 0.85 | >5 -> 0.70

pub fn detect_long_parameter_list(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.parameter_count <= 5 {
        return None;
    }
    let (confidence, reason) = if metrics.parameter_count > 8 {
        (
            0.95,
            format!("Parameter count={} exceeds 8", metrics.parameter_count),
        )
    } else if metrics.parameter_count > 6 {
        (
            0.85,
            format!("Parameter count={} exceeds 6", metrics.parameter_count),
        )
    } else {
        (
            0.70,
            format!("Parameter count={} exceeds 5", metrics.parameter_count),
        )
    };
    Some(build_detection(
        "SMELL-02",
        "Long Parameter List",
        confidence,
        location,
        name,
        metrics,
        vec![reason],
    ))
}

// -- SMELL-03  Primitive Obsession ------------------------------------------
// >=5 primitives AND ratio >= 0.80 -> 0.85 | otherwise None
// Tightened from >=4/0.75 to reduce false positives.

pub fn detect_primitive_obsession(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.primitive_params < 5 {
        return None;
    }
    let ratio = metrics.primitive_params as f64 / metrics.parameter_count.max(1) as f64;
    if metrics.primitive_params >= 5 && ratio >= 0.80 {
        let reasons = vec![
            format!("{} primitive parameters (>=5)", metrics.primitive_params),
            format!("{:.0}% of parameters are primitives", ratio * 100.0),
        ];
        Some(build_detection(
            "SMELL-03",
            "Primitive Obsession",
            0.85,
            location,
            name,
            metrics,
            reasons,
        ))
    } else {
        None
    }
}

// -- SMELL-04  Large Class --------------------------------------------------
// methods>20 +0.40 | >15 +0.20 | fields>15 +0.30 | >10 +0.15
// LOC>300 +0.30 | >200 +0.15 | fires at >= 0.5

pub fn detect_large_class(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    let mut a = TieredAccum::new();
    a.tier(
        metrics.method_count,
        20,
        0.4,
        format!("Method count={} exceeds 20", metrics.method_count),
        15,
        0.2,
        format!("Method count={} exceeds 15", metrics.method_count),
    );
    a.tier(
        metrics.field_count,
        15,
        0.3,
        format!("Field count={} exceeds 15", metrics.field_count),
        10,
        0.15,
        format!("Field count={} exceeds 10", metrics.field_count),
    );
    a.tier(
        metrics.loc,
        300,
        0.3,
        format!("LOC={} exceeds 300", metrics.loc),
        200,
        0.15,
        format!("LOC={} exceeds 200", metrics.loc),
    );
    a.into_detection("SMELL-04", "Large Class", location, name, metrics, 0.5)
}

// -- SMELL-05  Data Clumps -------------------------------------------------
// Heuristic: functions that take many primitive parameters often indicate
// data clumps -- groups of parameters that should be extracted into a
// dedicated object.  We use primitive_params count, parameter_count, and
// loc as proxies.  This is a conservative heuristic; true data-clump
// detection requires cross-function parameter-set overlap analysis.
// >=7 params AND >=5 primitives -> 0.80 | >=6 AND >=4 -> 0.65
// Removed the low-confidence tier (50%) to reduce noise on borderline cases.

pub fn detect_data_clumps(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.parameter_count < 6 || metrics.primitive_params < 4 {
        return None;
    }
    let (confidence, reasons) = if metrics.parameter_count >= 7 && metrics.primitive_params >= 5 {
        (
            0.80,
            vec![
                format!(
                    "{} parameters with {} primitives suggest data clumps",
                    metrics.parameter_count, metrics.primitive_params
                ),
                "Consider extracting related parameters into a parameter object".into(),
            ],
        )
    } else {
        (
            0.65,
            vec![
                format!(
                    "High parameter count ({}) with many primitives ({})",
                    metrics.parameter_count, metrics.primitive_params
                ),
                "Some parameters likely belong together".into(),
            ],
        )
    };
    Some(build_detection(
        "SMELL-05",
        "Data Clumps",
        confidence,
        location,
        name,
        metrics,
        reasons,
    ))
}

// -- SMELL-06  Switch Statements --------------------------------------------
// >10 -> 0.90 | >7 -> 0.75 | else -> 0.60 | +0.15 when CC>15

pub fn detect_switch_statements(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.branch_count <= 5 {
        return None;
    }
    let (mut confidence, mut reasons) = if metrics.branch_count > 10 {
        (
            0.90,
            vec![format!(
                "Excessive branching with {} branches (>10)",
                metrics.branch_count
            )],
        )
    } else if metrics.branch_count > 7 {
        (
            0.75,
            vec![format!(
                "High branching with {} branches (>7)",
                metrics.branch_count
            )],
        )
    } else {
        (
            0.60,
            vec![format!(
                "Many branches ({}) suggest need for polymorphism",
                metrics.branch_count
            )],
        )
    };
    if metrics.cyclomatic_complexity > 15 {
        reasons.push(format!(
            "Combined with high CC={}",
            metrics.cyclomatic_complexity
        ));
        confidence = (confidence + 0.15_f64).min(1.0_f64);
    }
    Some(build_detection(
        "SMELL-06",
        "Switch Statements",
        confidence,
        location,
        name,
        metrics,
        reasons,
    ))
}

// -- SMELL-07  Data Class ---------------------------------------------------
// fields >= 5 AND (methods == 0 OR field/method ratio >= 2.0)
// Pure data structs (method_count == 0) use lower confidence 0.60 to avoid
// flagging legitimate Rust/Go data containers; this falls below the default
// --min-confidence 0.65 and is effectively filtered out in practice.

pub fn detect_data_class(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.field_count < 5 {
        return None;
    }
    let ratio = metrics.field_count as f64 / metrics.method_count.max(1) as f64;
    if metrics.method_count == 0 || ratio >= 2.0 {
        let confidence = if metrics.method_count == 0 {
            0.60
        } else {
            0.75
        };
        Some(build_detection(
            "SMELL-07",
            "Data Class",
            confidence,
            location,
            name,
            metrics,
            vec![
                if metrics.method_count == 0 {
                    format!("{} fields with no behavior methods", metrics.field_count)
                } else {
                    format!("High field-to-method ratio ({ratio:.1})")
                },
                format!("Field count={}, few behavior methods", metrics.field_count),
            ],
        ))
    } else {
        None
    }
}

// -- SMELL-09  Shotgun Surgery ---------------------------------------------
// EXTERNAL PARAMETER REQUIRED: `dependency_count` -- the number of files that
// depend on this function/class.  Cannot be derived from CodeMetrics alone;
// requires project-wide dependency analysis.  Returns None when
// `dependency_count == 0` or when no threshold is met.
// dep_count >= 10 -> 0.80 | >= 7 -> 0.65

pub fn detect_shotgun_surgery(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
    dependency_count: usize,
) -> Option<SmellDetection> {
    if dependency_count == 0 {
        return None;
    }
    if dependency_count >= 10 {
        Some(build_detection(
            "SMELL-09",
            "Shotgun Surgery",
            0.80,
            location,
            name,
            metrics,
            vec![
                format!("Used by {dependency_count} different files"),
                "Changes here will require widespread modifications".into(),
            ],
        ))
    } else if dependency_count >= 7 {
        Some(build_detection(
            "SMELL-09",
            "Shotgun Surgery",
            0.65,
            location,
            name,
            metrics,
            vec![
                format!("Used by {dependency_count} files"),
                "Moderate coupling suggests refactoring risk".into(),
            ],
        ))
    } else {
        None
    }
}

// -- SMELL-10  Divergent Change ---------------------------------------------
// CC>25 AND methods>15 -> 0.80 | CC>20 AND >12 -> 0.65 | CC>15 AND >8 -> 0.55
// +0.10 bonus when fields > 10

pub fn detect_divergent_change(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.cyclomatic_complexity <= 15 || metrics.method_count <= 8 {
        return None;
    }
    let (mut confidence, mut reasons) =
        if metrics.cyclomatic_complexity > 25 && metrics.method_count > 15 {
            (
                0.80,
                vec![
                    format!(
                        "High CC={} with {} methods",
                        metrics.cyclomatic_complexity, metrics.method_count
                    ),
                    "Multiple responsibilities suggest multiple change reasons".into(),
                ],
            )
        } else if metrics.cyclomatic_complexity > 20 && metrics.method_count > 12 {
            (
                0.65,
                vec![
                    format!(
                        "CC={} with {} methods",
                        metrics.cyclomatic_complexity, metrics.method_count
                    ),
                    "Likely has multiple change reasons".into(),
                ],
            )
        } else if metrics.cyclomatic_complexity > 15 && metrics.method_count > 8 {
            (
                0.55,
                vec![format!(
                    "Moderate CC={} and method count",
                    metrics.cyclomatic_complexity
                )],
            )
        } else {
            return None;
        };
    if confidence >= 0.55 && metrics.field_count > 10 {
        reasons.push(format!(
            "Many fields ({}) reinforce multiple concerns",
            metrics.field_count
        ));
        confidence = (confidence + 0.1_f64).min(1.0_f64);
    }
    if confidence >= 0.55 {
        Some(build_detection(
            "SMELL-10",
            "Divergent Change",
            confidence,
            location,
            name,
            metrics,
            reasons,
        ))
    } else {
        None
    }
}

// -- SMELL-11  Lazy Class ---------------------------------------------------
// LOC < 8 AND methods == 0 AND fields == 0 -> 0.80 (truly empty, no data or behavior)
// LOC < 10 AND methods <= 1 AND fields < 3 -> 0.75 (nearly empty class)
// Data structs (Class/Struct with >= 2 fields and no methods) are NOT flagged --
// they are legitimate data containers common in Rust/Go.

pub fn detect_lazy_class(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    // Data struct guard: classes/structs with fields but no methods are not lazy
    if matches!(metrics.item_type, ItemType::Class)
        && metrics.method_count == 0
        && metrics.field_count >= 2
    {
        return None;
    }
    // Class with no methods AND no fields → truly empty
    if metrics.loc < 8 && metrics.method_count == 0 && metrics.field_count == 0 {
        return Some(build_detection(
            "SMELL-11",
            "Lazy Class",
            0.80,
            location,
            name,
            metrics,
            vec![
                format!("LOC={} is very small", metrics.loc),
                "No methods or fields, minimal functionality".into(),
            ],
        ));
    }
    // Class with almost no behavior but some structure.
    // Exclude data containers: a class with fields + a constructor/accessor
    // is a legitimate value object, not a lazy class.
    if metrics.loc < 10
        && metrics.method_count <= 1
        && metrics.field_count < 3
        && !(metrics.field_count >= 1 && metrics.method_count == 1)
    {
        return Some(build_detection(
            "SMELL-11",
            "Lazy Class",
            0.75,
            location,
            name,
            metrics,
            vec![
                format!("LOC={} is very small", metrics.loc),
                format!(
                    "Method count={}, minimal functionality",
                    metrics.method_count
                ),
            ],
        ));
    }
    None
}

// -- SMELL-12  Speculative Generality --------------------------------------
// EXTERNAL PARAMS REQUIRED: `subclass_count` (number of subclasses) and
// `usage_count` (number of call sites).  Cannot be derived from CodeMetrics
// alone; requires project-wide inheritance and usage analysis.
// subclass==1 -> 0.75 | usage==0 -> 0.85 | usage==1 AND methods>3 -> 0.60
// subclass==1 AND usage<=1 -> 0.90 | fires at >= 0.6

pub fn detect_speculative_generality(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
    subclass_count: usize,
    usage_count: usize,
) -> Option<SmellDetection> {
    let mut reasons: Vec<String> = Vec::new();
    let mut confidence: f64 = 0.0;
    if subclass_count == 1 {
        reasons.push("Abstract class/interface with only one implementation".into());
        reasons.push("Abstraction may be premature/unnecessary".into());
        confidence = 0.75;
    }
    if usage_count == 0 && metrics.method_count > 0 {
        reasons.push("Class is defined but never used".into());
        confidence = confidence.max(0.85);
    } else if usage_count == 1 && metrics.method_count > 3 {
        reasons.push("Complex class with only one usage point".into());
        confidence = confidence.max(0.60);
    }
    if subclass_count == 1 && usage_count <= 1 {
        confidence = 0.90;
    }
    if confidence >= 0.6 {
        if reasons.is_empty() {
            reasons.push("Unused or over-engineered abstraction".into());
        }
        Some(build_detection(
            "SMELL-12",
            "Speculative Generality",
            confidence,
            location,
            name,
            metrics,
            reasons,
        ))
    } else {
        None
    }
}

// -- SMELL-13  Duplicate Code -----------------------------------------------
// EXTERNAL PARAMS REQUIRED: `metrics.ast_hash` must be populated, and caller
// must supply `all_hashes: HashMap<String, Vec<String>>` mapping each AST hash
// to the locations where it appears.  Requires project-wide AST hashing.
// Returns None when hashes are not provided or no duplicates are found.

pub fn detect_duplicate_code(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
    all_hashes: Option<&std::collections::HashMap<String, Vec<String>>>,
) -> Option<SmellDetection> {
    let hashes = all_hashes?;
    if metrics.ast_hash.is_empty() {
        return None;
    }
    let dup_locs = hashes.get(&metrics.ast_hash)?;
    if dup_locs.len() > 1 {
        let others: Vec<&str> = dup_locs
            .iter()
            .filter(|l| l.as_str() != location)
            .take(3)
            .map(|s| s.as_str())
            .collect();
        if !others.is_empty() {
            let confidence = (0.7 + (dup_locs.len() - 1) as f64 * 0.1).min(0.95);
            return Some(build_detection(
                "SMELL-13",
                "Duplicate Code",
                confidence,
                location,
                name,
                metrics,
                vec![
                    format!("Code duplicated in {} locations", dup_locs.len()),
                    format!("Also found at: {}", others.join(", ")),
                ],
            ));
        }
    }
    None
}

// -- SMELL-14  Middle Man ---------------------------------------------------
// delegation ratio > 0.7 AND methods >= 3 | ratio > 0.85 -> 0.85 | else -> 0.70
// Heuristic fallback: when delegation_methods is not populated by parsers,
// detect from external_calls count as a proxy for delegation-heavy classes.

pub fn detect_middle_man(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    // Primary detection: delegation_methods populated by parser
    if metrics.method_count > 0 && metrics.delegation_methods > 0 {
        let ratio = metrics.delegation_methods as f64 / metrics.method_count as f64;
        if ratio > 0.7 && metrics.method_count >= 3 {
            let confidence = if ratio > 0.85 { 0.85 } else { 0.70 };
            return Some(build_detection(
                "SMELL-14",
                "Middle Man",
                confidence,
                location,
                name,
                metrics,
                vec![
                    format!(
                        "{}/{} methods are simple delegations",
                        metrics.delegation_methods, metrics.method_count
                    ),
                    format!("Delegation ratio: {:.0}%", ratio * 100.0),
                    "Class adds little value, consider removing".into(),
                ],
            ));
        }
    }
    // Heuristic fallback: external_calls proxy for delegation
    if metrics.method_count >= 3
        && metrics.external_calls >= metrics.method_count
        && metrics.loc < 50
    {
        return Some(build_detection(
            "SMELL-14",
            "Middle Man",
            0.70,
            location,
            name,
            metrics,
            vec![
                format!(
                    "{} external calls with {} methods suggest delegation",
                    metrics.external_calls, metrics.method_count
                ),
                "Low LOC with high forwarding activity".into(),
            ],
        ));
    }
    None
}

// -- SMELL-18  Feature Envy -------------------------------------------------
// external_calls>5 +0.40 | return_statements>5 +0.30 | CC>8 AND LOC<40 +0.20 | fires at >= 0.5

pub fn detect_feature_envy(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    let mut a = TieredAccum::new();
    if metrics.external_calls > 5 {
        a.add(
            0.7,
            format!("External calls={} exceeds 5", metrics.external_calls),
        );
    }
    if metrics.return_statements > 5 {
        a.add(
            0.3,
            format!("Return statements={} exceeds 5", metrics.return_statements),
        );
    }
    if metrics.cyclomatic_complexity > 8 && metrics.loc < 40 {
        a.add(
            0.2,
            "High CC with moderate LOC suggests complex branching".into(),
        );
    }
    a.into_detection("SMELL-18", "Feature Envy", location, name, metrics, 0.5)
}

// -- SMELL-20  Message Chains -----------------------------------------------
// >4 -> 0.90 | >3 -> 0.70
// Note: chains of 3 or fewer are common with standard library APIs (e.g.
// `.get().and_then().unwrap_or()`) and builder patterns, and are not flagged.
// The chain counter uses AST-based analysis for Python and gap-based heuristic
// for other languages, so conservative thresholds avoid FPs from parser noise.

pub fn detect_message_chains(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.method_call_chains <= 3 {
        return None;
    }
    let (confidence, reasons) = if metrics.method_call_chains > 4 {
        (
            0.90,
            vec![
                format!(
                    "Very long call chains (depth={})",
                    metrics.method_call_chains
                ),
                "Violates Law of Demeter, creates tight coupling".into(),
            ],
        )
    } else {
        (
            0.70,
            vec![format!(
                "Call chain depth={} suggests coupling",
                metrics.method_call_chains
            )],
        )
    };
    Some(build_detection(
        "SMELL-20",
        "Message Chains",
        confidence,
        location,
        name,
        metrics,
        reasons,
    ))
}

// -- SMELL-21  God Object ---------------------------------------------------
// methods>30 +0.35 | >25 +0.20 | fields>20 +0.35 | >15 +0.20
// LOC>500 +0.30 | >400 +0.15 | CC>50 +0.20 | fires at >= 0.6

pub fn detect_god_object(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    let mut a = TieredAccum::new();
    a.tier(
        metrics.method_count,
        30,
        0.35,
        format!("Excessive method count={} (>30)", metrics.method_count),
        25,
        0.2,
        format!("Very high method count={} (>25)", metrics.method_count),
    );
    a.tier(
        metrics.field_count,
        20,
        0.35,
        format!("Excessive field count={} (>20)", metrics.field_count),
        15,
        0.2,
        format!("Very high field count={} (>15)", metrics.field_count),
    );
    a.tier(
        metrics.loc,
        500,
        0.3,
        format!("Excessive LOC={} (>500)", metrics.loc),
        400,
        0.15,
        format!("Very high LOC={} (>400)", metrics.loc),
    );
    if metrics.cyclomatic_complexity > 50 {
        a.add(
            0.2,
            format!("Extreme complexity CC={}", metrics.cyclomatic_complexity),
        );
    }
    a.into_detection("SMELL-21", "God Object", location, name, metrics, 0.6)
}

// -- SMELL-15  Parallel Inheritance Hierarchies (placeholder) ---------------
// PLACEHOLDER: Detecting parallel hierarchies requires cross-class inheritance
// analysis (comparing subclass trees of related base classes).  The basic
// CodeMetrics available per function/class cannot capture this relationship.
// A proper implementation would need a project-wide class hierarchy graph.

pub fn detect_parallel_inheritance(
    _metrics: &CodeMetrics,
    _location: &str,
    _name: &str,
) -> Option<SmellDetection> {
    // TODO: Requires cross-class inheritance tree comparison.
    // Not detectable from per-function CodeMetrics alone.
    None
}

// -- SMELL-16  Comments -----------------------------------------------------
// Heuristic: high comment density relative to code suggests the code is not
// self-documenting.  Uses `comment_count` from CodeMetrics.
// comment_ratio >= 75% -> 0.70 | >= 35% -> 0.40
// Extra: long method + ratio >= 0.35 -> +0.25 | high CC + ratio >= 0.35 -> +0.20
// Fires at >= 0.65.
// NOTE: Low threshold is 35% (not 25%) to avoid flagging well-documented code
// with rich doc comments (Javadoc, ///, etc.) which commonly reach 25-30%.

pub fn detect_comments(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    // Use only inline comments (exclude doc comments like docstrings, ///)
    let inline_count = metrics.comment_count.saturating_sub(metrics.doc_comment_count);
    if inline_count == 0 || metrics.loc == 0 {
        return None;
    }
    let comment_ratio = inline_count as f64 / metrics.loc as f64;
    let mut a = TieredAccum::new();
    a.tier(
        inline_count,
        (metrics.loc as f64 * 0.75) as usize, // > 75% comment lines
        0.70,
        format!(
            "Comment density {:.0}% is very high ({} inline comment lines / {} LOC)",
            comment_ratio * 100.0,
            inline_count,
            metrics.loc
        ),
        // ~35% threshold: (loc * 100 / 285 ≈ 35%). Using integer math: loc * 5 / 14.
        (metrics.loc as f64 * 0.35) as usize,
        0.40,
        format!(
            "Comment density {:.0}% suggests code is not self-documenting",
            comment_ratio * 100.0
        ),
    );
    if metrics.loc > 50 && comment_ratio >= 0.35 {
        a.add(
            0.25,
            "Long method with many comments -- consider extracting named methods".into(),
        );
    }
    if metrics.cyclomatic_complexity > 10 && comment_ratio >= 0.35 {
        a.add(
            0.20,
            format!(
                "High CC={} with many comments suggests complex control flow",
                metrics.cyclomatic_complexity
            ),
        );
    }
    a.into_detection("SMELL-16", "Comments", location, name, metrics, 0.65)
}

// -- SMELL-17  Dead Code (placeholder) --------------------------------------
// PLACEHOLDER: Detecting dead code requires project-wide usage analysis
// (finding functions/classes that are defined but never called/referenced).
// The basic CodeMetrics available per function/class cannot capture call-graph
// information.  A proper implementation would need a whole-program dependency
// graph or AST-based reference analysis.

pub fn detect_dead_code(
    _metrics: &CodeMetrics,
    _location: &str,
    _name: &str,
) -> Option<SmellDetection> {
    // TODO: Requires project-wide call-graph/reference analysis.
    // Not detectable from per-function CodeMetrics alone.
    None
}

// -- SMELL-19  Inappropriate Intimacy (placeholder) -------------------------
// PLACEHOLDER: Detecting inappropriate intimacy requires cross-class access
// analysis (measuring how much one class accesses another's internals).
// The basic CodeMetrics available per function/class cannot capture
// inter-class field/method access patterns.  A proper implementation would
// need a project-wide dependency graph with access-level tracking.

pub fn detect_inappropriate_intimacy(
    _metrics: &CodeMetrics,
    _location: &str,
    _name: &str,
) -> Option<SmellDetection> {
    // TODO: Requires cross-class access analysis with visibility tracking.
    // Not detectable from per-function CodeMetrics alone.
    None
}

// -- SMELL-22  Refused Bequest ----------------------------------------------
// Primary: override_count >= 3 AND methods <= 5 -> 0.75
// Primary: override_count >= 2 AND methods <= 4 -> 0.60
// Primary: field_count >= 8 AND methods <= 2 AND overrides > 0 -> 0.55
// Heuristic fallback: when override_count == 0 (never populated by parser),
// flag small classes with few methods that likely inherit without adding value.

pub fn detect_refused_bequest(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    // Signal 1: Many trivial overrides (empty/stub) in a small class
    if metrics.override_count >= 3 && metrics.method_count <= 5 && metrics.method_count > 0 {
        let ratio = metrics.override_count as f64 / metrics.method_count as f64;
        if ratio >= 0.5 {
            return Some(build_detection(
                "SMELL-22",
                "Refused Bequest",
                0.75,
                location,
                name,
                metrics,
                vec![
                    format!(
                        "{} out of {} methods are trivial overrides",
                        metrics.override_count, metrics.method_count
                    ),
                    "Subclass rejects parent behavior -- consider composition over inheritance"
                        .into(),
                ],
            ));
        }
    }
    // Signal 2: Moderate trivial overrides
    if metrics.override_count >= 2 && metrics.method_count <= 4 && metrics.method_count > 0 {
        return Some(build_detection(
            "SMELL-22",
            "Refused Bequest",
            0.70,
            location,
            name,
            metrics,
            vec![
                format!(
                    "{} trivial overrides suggest rejected parent contract",
                    metrics.override_count
                ),
                "Consider whether inheritance is appropriate".into(),
            ],
        ));
    }
    // Signal 3: Many inherited fields but very few methods (lazy subclass).
    // Requires override_count > 0 to avoid flagging data classes (DTOs, entities)
    // which legitimately have many fields and few methods.
    if metrics.field_count >= 8
        && metrics.method_count <= 2
        && metrics.method_count > 0
        && metrics.override_count > 0
    {
        return Some(build_detection(
            "SMELL-22",
            "Refused Bequest",
            0.55,
            location,
            name,
            metrics,
            vec![
                format!(
                    "{} fields, {} methods, {} trivial overrides -- inherits without adding value",
                    metrics.field_count, metrics.method_count, metrics.override_count
                ),
                "Subclass overrides parent behavior but adds little -- consider composition".into(),
            ],
        ));
    }
    // Heuristic fallback: when override_count is never populated by parser,
    // detect potential refused bequest from class shape alone.
    if metrics.override_count == 0
        && metrics.method_count <= 3
        && metrics.method_count > 0
        && metrics.field_count <= 2
        && metrics.loc < 30
    {
        return Some(build_detection(
            "SMELL-22",
            "Refused Bequest",
            0.55,
            location,
            name,
            metrics,
            vec![
                format!(
                    "Small class (LOC={}, {} methods, {} fields) likely inherits without adding value",
                    metrics.loc, metrics.method_count, metrics.field_count
                ),
                "Consider whether inheritance is appropriate or composition would be better".into(),
            ],
        ));
    }
    None
}

// -- SMELL-23  Alternative Classes with Different Interfaces (placeholder) ---
// PLACEHOLDER: Detecting alternative classes with different interfaces requires
// cross-class comparison (finding classes that do the same thing but have
// different method signatures).  The basic CodeMetrics available per
// function/class cannot capture semantic equivalence of classes.
// A proper implementation would need project-wide interface analysis.

pub fn detect_alternative_classes(
    _metrics: &CodeMetrics,
    _location: &str,
    _name: &str,
) -> Option<SmellDetection> {
    // TODO: Requires cross-class interface comparison.
    // Not detectable from per-function CodeMetrics alone.
    None
}

// -- Convenience orchestrators -----------------------------------------------

/// Run function-level smell detectors.
pub fn detect_function_smells(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Vec<SmellDetection> {
    [
        detect_long_method(metrics, location, name),
        detect_long_parameter_list(metrics, location, name),
        detect_primitive_obsession(metrics, location, name),
        detect_switch_statements(metrics, location, name),
        detect_feature_envy(metrics, location, name),
        detect_message_chains(metrics, location, name),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Run class-level smell detectors.
pub fn detect_class_smells(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Vec<SmellDetection> {
    [
        detect_large_class(metrics, location, name),
        detect_data_class(metrics, location, name),
        detect_lazy_class(metrics, location, name),
        detect_divergent_change(metrics, location, name),
        detect_middle_man(metrics, location, name),
        detect_god_object(metrics, location, name),
        detect_refused_bequest(metrics, location, name),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Run appropriate detectors based on `metrics.item_type`.
///
/// - `Function` items: function-level smells + data clumps + comments.
/// - `Class` items: class-level smells + data clumps + comments.
///
/// External-parameter detectors (SMELL-09, -12, -13) require project-wide
/// data not available at the per-item level. Callers with real data should
/// invoke `detect_shotgun_surgery`, `detect_speculative_generality`, and
/// `detect_duplicate_code` directly.
pub fn detect_all(metrics: &CodeMetrics, location: &str, name: &str) -> Vec<SmellDetection> {
    let mut r = match metrics.item_type {
        ItemType::Function => detect_function_smells(metrics, location, name),
        ItemType::Class => detect_class_smells(metrics, location, name),
    };
    r.extend(
        [
            detect_data_clumps(metrics, location, name),
            detect_comments(metrics, location, name),
        ]
        .into_iter()
        .flatten(),
    );
    r
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fn_metrics(loc: usize, cc: usize, nesting: usize, params: usize) -> CodeMetrics {
        CodeMetrics {
            loc,
            cyclomatic_complexity: cc,
            nesting_depth: nesting,
            parameter_count: params,
            ..Default::default()
        }
    }

    fn make_class_metrics(loc: usize, methods: usize, fields: usize) -> CodeMetrics {
        CodeMetrics {
            loc,
            cyclomatic_complexity: 1,
            method_count: methods,
            field_count: fields,
            ..Default::default()
        }
    }

    #[test]
    fn long_method_high_loc_and_cc() {
        let d = detect_long_method(&make_fn_metrics(80, 20, 5, 2), "test.py:1", "big_fn").unwrap();
        assert_eq!(d.smell_id, "SMELL-01");
        assert!((d.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn long_method_below_threshold() {
        assert!(
            detect_long_method(&make_fn_metrics(10, 2, 1, 0), "test.py:1", "small_fn").is_none()
        );
    }

    #[test]
    fn long_method_moderate() {
        // LOC>30 (+0.15) + CC>10 (+0.25) = 0.40 -- below 0.5
        assert!(
            detect_long_method(&make_fn_metrics(40, 12, 2, 1), "test.py:1", "mid_fn").is_none()
        );
    }

    #[test]
    fn long_params_6() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 6), "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-02");
        assert!((d.confidence - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_7() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 7), "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_9() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 9), "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_ok() {
        assert!(detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 3), "t.py:1", "f").is_none());
    }

    #[test]
    fn long_params_5_not_flagged() {
        assert!(detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 5), "t.py:1", "f").is_none());
    }

    #[test]
    fn primitive_obsession_high() {
        let m = CodeMetrics {
            primitive_params: 5,
            parameter_count: 6,
            ..Default::default()
        };
        let d = detect_primitive_obsession(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-03");
        assert!((d.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn primitive_obsession_low_ratio() {
        let m = CodeMetrics {
            primitive_params: 3,
            parameter_count: 10,
            ..Default::default()
        };
        assert!(detect_primitive_obsession(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn primitive_obsession_below_4_not_flagged() {
        let m = CodeMetrics {
            primitive_params: 3,
            parameter_count: 3,
            ..Default::default()
        };
        assert!(detect_primitive_obsession(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn large_class_high() {
        let d = detect_large_class(&make_class_metrics(350, 25, 18), "t.py:1", "BigCls").unwrap();
        assert_eq!(d.smell_id, "SMELL-04");
        assert!(d.confidence >= 0.5);
    }

    #[test]
    fn large_class_none() {
        assert!(detect_large_class(&make_class_metrics(50, 5, 3), "t.py:1", "SmallCls").is_none());
    }

    #[test]
    fn data_class_detected() {
        let m = CodeMetrics {
            method_count: 3,
            field_count: 10,
            ..Default::default()
        };
        let d = detect_data_class(&m, "t.py:1", "Dto").unwrap();
        assert_eq!(d.smell_id, "SMELL-07");
        assert!((d.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn data_class_not_enough_fields() {
        let m = CodeMetrics {
            method_count: 3,
            field_count: 3,
            ..Default::default()
        };
        assert!(detect_data_class(&m, "t.py:1", "Dto").is_none());
    }

    #[test]
    fn data_class_zero_methods_lower_confidence() {
        // Pure data structs get confidence 0.60 (below typical min_confidence 0.65)
        let m = CodeMetrics {
            method_count: 0,
            field_count: 8,
            ..Default::default()
        };
        let d = detect_data_class(&m, "t.go:1", "Config").unwrap();
        assert_eq!(d.smell_id, "SMELL-07");
        assert!((d.confidence - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn lazy_class_detected() {
        let m = CodeMetrics {
            loc: 5,
            method_count: 0,
            field_count: 0,
            ..Default::default()
        };
        let d = detect_lazy_class(&m, "t.py:1", "Useless").unwrap();
        assert_eq!(d.smell_id, "SMELL-11");
        assert!((d.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn lazy_class_with_one_method_no_fields() {
        // A class with one method but no fields — still suspicious
        let m = CodeMetrics {
            loc: 9,
            method_count: 1,
            field_count: 0,
            ..Default::default()
        };
        let d = detect_lazy_class(&m, "t.py:1", "Tiny").unwrap();
        assert_eq!(d.smell_id, "SMELL-11");
        assert!((d.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn lazy_class_one_method_with_fields_not_flagged() {
        // A class with fields + one method is a value object, not lazy
        let m = CodeMetrics {
            loc: 9,
            method_count: 1,
            field_count: 2,
            ..Default::default()
        };
        assert!(detect_lazy_class(&m, "t.py:1", "Tiny").is_none());
    }

    #[test]
    fn lazy_class_data_struct_not_flagged() {
        // Rust-style data struct with many fields but no methods — should NOT be flagged
        let m = CodeMetrics {
            loc: 15,
            method_count: 0,
            field_count: 10,
            ..Default::default()
        };
        assert!(detect_lazy_class(&m, "t.rs:1", "BuildStats").is_none());
    }

    #[test]
    fn lazy_class_enough_methods_not_flagged() {
        let m = CodeMetrics {
            loc: 30,
            method_count: 3,
            field_count: 2,
            ..Default::default()
        };
        assert!(detect_lazy_class(&m, "t.py:1", "Active").is_none());
    }

    #[test]
    fn switch_detected() {
        let m = CodeMetrics {
            branch_count: 8,
            ..Default::default()
        };
        let d = detect_switch_statements(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-06");
        assert!((d.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn switch_with_high_cc() {
        let m = CodeMetrics {
            branch_count: 6,
            cyclomatic_complexity: 20,
            ..Default::default()
        };
        let d = detect_switch_statements(&m, "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.75).abs() < f64::EPSILON); // 0.60 + 0.15
    }

    #[test]
    fn god_object_detected() {
        let m = CodeMetrics {
            loc: 600,
            method_count: 35,
            field_count: 25,
            ..Default::default()
        };
        let d = detect_god_object(&m, "t.py:1", "God").unwrap();
        assert_eq!(d.smell_id, "SMELL-21");
        assert!((d.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn message_chains_not_detected_at_3() {
        let m = CodeMetrics {
            method_call_chains: 3,
            ..Default::default()
        };
        assert!(detect_message_chains(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn message_chains_detected() {
        let m = CodeMetrics {
            method_call_chains: 4,
            ..Default::default()
        };
        let d = detect_message_chains(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-20");
        assert!((d.confidence - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn message_chains_long() {
        let m = CodeMetrics {
            method_call_chains: 6,
            ..Default::default()
        };
        let d = detect_message_chains(&m, "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn feature_envy_detected() {
        let m = CodeMetrics {
            external_calls: 8,
            return_statements: 7,
            ..Default::default()
        };
        let d = detect_feature_envy(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-18");
        assert!(d.confidence >= 0.5);
    }

    #[test]
    fn detect_all_combines() {
        let m = CodeMetrics {
            loc: 80,
            cyclomatic_complexity: 20,
            nesting_depth: 5,
            parameter_count: 8,
            branch_count: 12,
            ..Default::default()
        };
        let results = detect_all(&m, "t.py:1", "mega");
        assert!(!results.is_empty());
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(ids.contains(&"SMELL-01"), "should detect Long Method");
        assert!(
            ids.contains(&"SMELL-02"),
            "should detect Long Parameter List"
        );
        assert!(ids.contains(&"SMELL-06"), "should detect Switch Statements");
    }

    #[test]
    fn middle_man_detected() {
        let m = CodeMetrics {
            method_count: 5,
            delegation_methods: 4,
            ..Default::default()
        };
        let d = detect_middle_man(&m, "t.py:1", "Proxy").unwrap();
        assert_eq!(d.smell_id, "SMELL-14");
        assert!((d.confidence - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn divergent_change_detected() {
        let m = CodeMetrics {
            cyclomatic_complexity: 30,
            method_count: 20,
            ..Default::default()
        };
        let d = detect_divergent_change(&m, "t.py:1", "SwissArmy").unwrap();
        assert_eq!(d.smell_id, "SMELL-10");
        assert!((d.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn data_clumps_below_threshold() {
        assert!(detect_data_clumps(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }

    #[test]
    fn data_clumps_high_params() {
        let m = CodeMetrics {
            parameter_count: 8,
            primitive_params: 6,
            ..Default::default()
        };
        let d = detect_data_clumps(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-05");
        assert!((d.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn data_clumps_moderate() {
        let m = CodeMetrics {
            parameter_count: 6,
            primitive_params: 4,
            ..Default::default()
        };
        let d = detect_data_clumps(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-05");
        assert!((d.confidence - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn data_clumps_below_new_threshold() {
        // 5 params, 3 primitives — now below threshold after tightening
        let m = CodeMetrics {
            parameter_count: 5,
            primitive_params: 3,
            ..Default::default()
        };
        assert!(detect_data_clumps(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn shotgun_surgery_zero_deps() {
        assert!(detect_shotgun_surgery(&CodeMetrics::default(), "t.py:1", "f", 0).is_none());
    }

    #[test]
    fn speculative_generality_zero() {
        assert!(
            detect_speculative_generality(&CodeMetrics::default(), "t.py:1", "f", 0, 0).is_none()
        );
    }

    #[test]
    fn duplicate_code_no_hashes() {
        let m = CodeMetrics {
            ast_hash: "abc".into(),
            ..Default::default()
        };
        assert!(detect_duplicate_code(&m, "t.py:1", "f", None).is_none());
    }

    // -- SMELL-16 (Comments) -------------------------------------------------

    #[test]
    fn comments_no_comments() {
        let m = CodeMetrics {
            loc: 20,
            comment_count: 0,
            ..Default::default()
        };
        assert!(detect_comments(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn comments_below_threshold() {
        let m = CodeMetrics {
            loc: 100,
            comment_count: 5, // 5% ratio, way below threshold
            ..Default::default()
        };
        assert!(detect_comments(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn comments_doc_comment_not_flagged() {
        // 30% comment ratio — typical for well-documented code (doc comments).
        // Should NOT fire with only the low tier signal (0.25 < 0.50 threshold).
        let m = CodeMetrics {
            loc: 100,
            comment_count: 30, // 30% ratio — below 35% low tier
            ..Default::default()
        };
        assert!(detect_comments(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn comments_35_percent_boundary() {
        // 35% ratio triggers low tier (0.25) but needs another signal to reach 0.50.
        let m = CodeMetrics {
            loc: 100,
            comment_count: 35, // exactly 35%
            ..Default::default()
        };
        // 0.25 alone < 0.50 threshold → None
        assert!(detect_comments(&m, "t.py:1", "f").is_none());
    }

    #[test]
    fn comments_high_density() {
        let m = CodeMetrics {
            loc: 100,
            comment_count: 60, // 60% ratio
            ..Default::default()
        };
        let d = detect_comments(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-16");
        assert!(d.confidence >= 0.4);
    }

    #[test]
    fn comments_with_long_method() {
        let m = CodeMetrics {
            loc: 80,
            comment_count: 45, // > 50% ratio, long method
            ..Default::default()
        };
        let d = detect_comments(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-16");
        // Should include bonus for long method + comments
        assert!(d.confidence > 0.4);
    }

    #[test]
    fn comments_with_high_cc() {
        let m = CodeMetrics {
            loc: 60,
            comment_count: 25,         // ~42% ratio
            cyclomatic_complexity: 15, // high CC
            ..Default::default()
        };
        let d = detect_comments(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-16");
        assert!(d.confidence >= 0.5);
    }

    // -- SMELL-22 (Refused Bequest) ------------------------------------------

    #[test]
    fn refused_bequest_many_overrides() {
        let m = CodeMetrics {
            method_count: 4,
            override_count: 3,
            ..Default::default()
        };
        let d = detect_refused_bequest(&m, "t.py:1", "BadSub").unwrap();
        assert_eq!(d.smell_id, "SMELL-22");
        assert!((d.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn refused_bequest_moderate_overrides() {
        let m = CodeMetrics {
            method_count: 3,
            override_count: 2,
            ..Default::default()
        };
        let d = detect_refused_bequest(&m, "t.py:1", "Sub").unwrap();
        assert_eq!(d.smell_id, "SMELL-22");
        assert!((d.confidence - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn refused_bequest_lazy_subclass() {
        let m = CodeMetrics {
            field_count: 10,
            method_count: 1,
            override_count: 1, // must have overrides — not a pure DTO
            ..Default::default()
        };
        let d = detect_refused_bequest(&m, "t.py:1", "LazySub").unwrap();
        assert_eq!(d.smell_id, "SMELL-22");
        assert!((d.confidence - 0.55).abs() < f64::EPSILON);
    }

    #[test]
    fn refused_bequest_dto_not_flagged() {
        // A data class (DTO/Entity) with many fields, few methods, no overrides.
        // Should NOT be flagged as Refused Bequest.
        let m = CodeMetrics {
            field_count: 12,
            method_count: 2,
            override_count: 0, // no overrides → not refusing bequest
            ..Default::default()
        };
        assert!(detect_refused_bequest(&m, "t.py:1", "UserDTO").is_none());
    }

    #[test]
    fn refused_bequest_none() {
        let m = CodeMetrics {
            method_count: 10,
            override_count: 1,
            field_count: 3,
            ..Default::default()
        };
        assert!(detect_refused_bequest(&m, "t.py:1", "GoodSub").is_none());
    }

    #[test]
    fn refused_bequest_zero_methods() {
        let m = CodeMetrics {
            method_count: 0,
            override_count: 5,
            ..Default::default()
        };
        assert!(detect_refused_bequest(&m, "t.py:1", "Empty").is_none());
    }

    // -- Placeholder detectors return None ------------------------------------

    #[test]
    fn parallel_inheritance_placeholder() {
        assert!(detect_parallel_inheritance(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }

    #[test]
    fn dead_code_placeholder() {
        assert!(detect_dead_code(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }

    #[test]
    fn inappropriate_intimacy_placeholder() {
        assert!(detect_inappropriate_intimacy(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }

    #[test]
    fn alternative_classes_placeholder() {
        assert!(detect_alternative_classes(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }
}
