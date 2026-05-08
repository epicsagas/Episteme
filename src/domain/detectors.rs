//! All 16 code-smell detector functions.
//!
//! Ported faithfully from `episteme.parsers.base` -- identical thresholds and
//! confidence formulas.

use crate::domain::metrics::{CodeMetrics, SmellDetection};

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
// >7 -> 0.95 | >5 -> 0.80 | >4 -> 0.65

pub fn detect_long_parameter_list(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.parameter_count <= 4 {
        return None;
    }
    let (confidence, reason) = if metrics.parameter_count > 7 {
        (
            0.95,
            format!("Parameter count={} exceeds 7", metrics.parameter_count),
        )
    } else if metrics.parameter_count > 5 {
        (
            0.80,
            format!("Parameter count={} exceeds 5", metrics.parameter_count),
        )
    } else {
        (
            0.65,
            format!("Parameter count={} exceeds 4", metrics.parameter_count),
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
// >=5 primitives AND ratio >= 0.8 -> 0.85 | >=4 AND >= 0.75 -> 0.70 | >=3 AND >= 0.7 -> 0.55

pub fn detect_primitive_obsession(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.primitive_params < 3 {
        return None;
    }
    let ratio = metrics.primitive_params as f64 / metrics.parameter_count.max(1) as f64;
    let (confidence, reasons) = if metrics.primitive_params >= 5 && ratio >= 0.8 {
        (
            0.85,
            vec![
                format!("{} primitive parameters (>=5)", metrics.primitive_params),
                format!("{:.0}% of parameters are primitives", ratio * 100.0),
            ],
        )
    } else if metrics.primitive_params >= 4 && ratio >= 0.75 {
        (
            0.70,
            vec![
                format!("{} primitive parameters", metrics.primitive_params),
                format!("High primitive ratio {:.0}%", ratio * 100.0),
            ],
        )
    } else if metrics.primitive_params >= 3 && ratio >= 0.7 {
        (
            0.55,
            vec![format!(
                "{} primitive parameters suggest domain object needed",
                metrics.primitive_params
            )],
        )
    } else {
        return None;
    };
    Some(build_detection(
        "SMELL-03",
        "Primitive Obsession",
        confidence,
        location,
        name,
        metrics,
        reasons,
    ))
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

// -- SMELL-05  Data Clumps (stub) ------------------------------------------

pub fn detect_data_clumps(
    _metrics: &CodeMetrics,
    _location: &str,
    _name: &str,
) -> Option<SmellDetection> {
    None
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
// field/method ratio >= 2.0 AND fields >= 5 -> 0.75

pub fn detect_data_class(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.method_count == 0 {
        return None;
    }
    let ratio = metrics.field_count as f64 / metrics.method_count as f64;
    if ratio >= 2.0 && metrics.field_count >= 5 {
        Some(build_detection(
            "SMELL-07",
            "Data Class",
            0.75,
            location,
            name,
            metrics,
            vec![
                format!("High field-to-method ratio ({ratio:.1})"),
                format!("Field count={}, few behavior methods", metrics.field_count),
            ],
        ))
    } else {
        None
    }
}

// -- SMELL-09  Shotgun Surgery (stub) --------------------------------------
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
// LOC < 20 AND methods <= 2 -> 0.70

pub fn detect_lazy_class(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.loc < 20 && metrics.method_count <= 2 {
        Some(build_detection(
            "SMELL-11",
            "Lazy Class",
            0.70,
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
        ))
    } else {
        None
    }
}

// -- SMELL-12  Speculative Generality (stub) --------------------------------
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

// -- SMELL-13  Duplicate Code (stub) ----------------------------------------

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

pub fn detect_middle_man(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.method_count == 0 || metrics.delegation_methods == 0 {
        return None;
    }
    let ratio = metrics.delegation_methods as f64 / metrics.method_count as f64;
    if ratio > 0.7 && metrics.method_count >= 3 {
        let confidence = if ratio > 0.85 { 0.85 } else { 0.70 };
        Some(build_detection(
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
        ))
    } else {
        None
    }
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
            0.4,
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
// >5 -> 0.90 | >4 -> 0.75 | >2 -> 0.60

pub fn detect_message_chains(
    metrics: &CodeMetrics,
    location: &str,
    name: &str,
) -> Option<SmellDetection> {
    if metrics.method_call_chains <= 2 {
        return None;
    }
    let (confidence, reasons) = if metrics.method_call_chains > 5 {
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
    } else if metrics.method_call_chains > 4 {
        (
            0.75,
            vec![
                format!("Long call chains (depth={})", metrics.method_call_chains),
                "Consider introducing intermediate methods".into(),
            ],
        )
    } else {
        (
            0.60,
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
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Run every detector against the given metrics and return all non-None hits.
pub fn detect_all(metrics: &CodeMetrics, location: &str, name: &str) -> Vec<SmellDetection> {
    let mut r = detect_function_smells(metrics, location, name);
    r.extend(detect_class_smells(metrics, location, name));
    r.extend(
        [
            detect_data_clumps(metrics, location, name),
            detect_shotgun_surgery(metrics, location, name, 0),
            detect_speculative_generality(metrics, location, name, 0, 0),
            detect_duplicate_code(metrics, location, name, None),
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
    fn long_params_5() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 5), "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-02");
        assert!((d.confidence - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_6() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 6), "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_8() {
        let d = detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 8), "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn long_params_ok() {
        assert!(detect_long_parameter_list(&make_fn_metrics(10, 1, 0, 3), "t.py:1", "f").is_none());
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
    fn lazy_class_detected() {
        let m = CodeMetrics {
            loc: 10,
            method_count: 1,
            ..Default::default()
        };
        let d = detect_lazy_class(&m, "t.py:1", "Useless").unwrap();
        assert_eq!(d.smell_id, "SMELL-11");
        assert!((d.confidence - 0.70).abs() < f64::EPSILON);
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
    fn message_chains_detected() {
        let m = CodeMetrics {
            method_call_chains: 4,
            ..Default::default()
        };
        let d = detect_message_chains(&m, "t.py:1", "f").unwrap();
        assert_eq!(d.smell_id, "SMELL-20");
        assert!((d.confidence - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn message_chains_long() {
        let m = CodeMetrics {
            method_call_chains: 5,
            ..Default::default()
        };
        let d = detect_message_chains(&m, "t.py:1", "f").unwrap();
        assert!((d.confidence - 0.75).abs() < f64::EPSILON);
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
    fn data_clumps_stub() {
        assert!(detect_data_clumps(&CodeMetrics::default(), "t.py:1", "f").is_none());
    }

    #[test]
    fn shotgun_surgery_stub_zero() {
        assert!(detect_shotgun_surgery(&CodeMetrics::default(), "t.py:1", "f", 0).is_none());
    }

    #[test]
    fn speculative_generality_stub_zero() {
        assert!(
            detect_speculative_generality(&CodeMetrics::default(), "t.py:1", "f", 0, 0).is_none()
        );
    }

    #[test]
    fn duplicate_code_stub_no_hashes() {
        let m = CodeMetrics {
            ast_hash: "abc".into(),
            ..Default::default()
        };
        assert!(detect_duplicate_code(&m, "t.py:1", "f", None).is_none());
    }
}
