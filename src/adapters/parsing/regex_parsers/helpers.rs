//! Shared helper functions for regex-based language parsers.
//!
//! Brace matching, LOC counting, cyclomatic complexity, local variable counting,
//! comment stripping, and metric building utilities.

use std::borrow::Cow;

use super::{cached_regex, cached_regex_owned};
use crate::domain::metrics::CodeMetrics;

// ===========================================================================
// Brace matching
// ===========================================================================

/// Find the position of the matching `}` for the `{` at `start`.
/// Returns `None` on failure (unbalanced braces or string-literal confusion).
pub(crate) fn find_matching_brace(code: &str, start: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    let mut depth: i32 = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_triple_single = false;
    let mut in_triple_double = false;
    let mut i = start;

    while i < bytes.len() {
        let c = bytes[i];
        let prev = if i > 0 { bytes[i - 1] } else { b' ' };

        // --- Python triple-quoted strings ---
        if !in_double && !in_single {
            if !in_triple_double && i + 2 < bytes.len() && &bytes[i..i + 3] == b"\"\"\"" {
                in_triple_double = true;
                i += 3;
                continue;
            }
            if in_triple_double && i + 2 < bytes.len() && &bytes[i..i + 3] == b"\"\"\"" {
                in_triple_double = false;
                i += 3;
                continue;
            }
            if !in_triple_single && i + 2 < bytes.len() && &bytes[i..i + 3] == b"'''" {
                in_triple_single = true;
                i += 3;
                continue;
            }
            if in_triple_single && i + 2 < bytes.len() && &bytes[i..i + 3] == b"'''" {
                in_triple_single = false;
                i += 3;
                continue;
            }
        }

        if in_triple_single || in_triple_double {
            i += 1;
            continue;
        }

        // --- Single / double quotes (simple, not escape-aware for perf) ---
        if c == b'"' && prev != b'\\' {
            in_double = !in_double;
        } else if c == b'\'' && prev != b'\\' {
            in_single = !in_single;
        }

        if !in_single && !in_double {
            if c == b'{' {
                depth += 1;
            } else if c == b'}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

// ===========================================================================
// LOC / metric counters
// ===========================================================================

/// Count non-blank, non-brace-only lines.
pub(crate) fn count_loc(body: &str) -> usize {
    body.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && t != "{" && t != "}"
        })
        .count()
}

/// Compute cyclomatic complexity from common control-flow keywords.
pub(crate) fn calculate_cc(body: &str) -> usize {
    let mut cc: usize = 1;
    cc += count_keyword(body, r"\bif\b");
    cc += count_keyword(body, r"\belif\b");
    cc += count_keyword(body, r"\bfor\b");
    cc += count_keyword(body, r"\bwhile\b");
    cc += count_keyword(body, r"\bcatch\b");
    cc += count_keyword(body, r"\bexcept\b");
    cc += count_keyword(body, r"\bswitch\b");
    cc += count_keyword(body, r"\bcase\b");
    cc += count_keyword(body, r"\bselect\b");
    cc += count_keyword(body, r"\bmatch\b");
    cc += count_keyword(body, r"\b&&\b");
    cc += count_keyword(body, r"\|\|");
    cc
}

/// Compute maximum nesting depth from brace pairs.
pub(crate) fn calculate_nesting(body: &str) -> usize {
    let mut max_d: usize = 0;
    let mut cur: usize = 0;
    for ch in body.chars() {
        if ch == '{' {
            cur += 1;
            max_d = max_d.max(cur);
        } else if ch == '}' {
            cur = cur.saturating_sub(1);
        }
    }
    max_d
}

/// Count regex matches using the global regex cache.
pub(crate) fn count_keyword(code: &str, pattern: &'static str) -> usize {
    cached_regex(pattern).find_iter(code).count()
}

/// Count return statements.
pub(crate) fn count_returns(body: &str) -> usize {
    count_keyword(body, r"\breturn\b")
}

/// Count local variables: `var x` and `x :=` and simple `x = y`.
pub(crate) fn count_local_vars(body: &str) -> usize {
    count_keyword(body, r"\bvar\s+\w+")
        + count_keyword(body, r"\w+\s*:=")
        + count_keyword(body, r"\blet\s+\w+")
        + count_keyword(body, r"\bconst\s+\w+")
}

/// Count local variables in C/C++: typed declarations like `int x =`, `auto x;`.
pub(crate) fn count_local_vars_cpp(body: &str) -> usize {
    count_keyword(
        body,
        r"\b(?:int|double|float|bool|char|void|auto|long|short|unsigned|signed|size_t)\s+\w+\s*[=;]",
    )
}

/// Count local variables in C#: typed and `var` declarations like `int x =`, `var x;`.
pub(crate) fn count_local_vars_csharp(body: &str) -> usize {
    count_keyword(
        body,
        r"\b(?:int|string|bool|double|float|var|decimal|long|byte|char|short|uint|ulong|ushort)\s+\w+\s*[=;]",
    )
}

/// Count local variables in PHP: `$x =` (PHP variables always start with $).
pub(crate) fn count_local_vars_php(body: &str) -> usize {
    count_keyword(body, r"\$\w+\s*=")
}

/// Count local variables in Kotlin: `val x` and `var x` declarations.
pub(crate) fn count_local_vars_kotlin(body: &str) -> usize {
    count_keyword(body, r"\b(?:val|var)\s+\w+")
}

/// Count parameters inside the first balanced paren group in `sig`.
pub(crate) fn count_params(sig: &str) -> usize {
    let start = match sig.find('(') {
        Some(i) => i + 1,
        None => return 0,
    };
    let mut depth: i32 = 1;
    let mut end = start;
    for (idx, ch) in sig[start..].char_indices() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                end = idx;
                break;
            }
        }
    }
    if end == start {
        return 0;
    }
    let inner = &sig[start..start + end];
    if inner.trim().is_empty() {
        return 0;
    }
    inner.split(',').filter(|p| !p.trim().is_empty()).count()
}

/// Count `obj.method()` external calls.
pub(crate) fn count_external_calls(body: &str) -> usize {
    count_keyword(body, r"\w+\.\w+\s*\(")
}

/// Count branches: `if`, `elif`, `else if`, `case`, `match` arms.
pub(crate) fn count_branches(body: &str) -> usize {
    count_keyword(body, r"\bif\b")
        + count_keyword(body, r"\belif\b")
        + count_keyword(body, r"\belse\s+if\b")
        + count_keyword(body, r"\bcase\b")
        + count_keyword(body, r"\bmatch\b")
}

/// Count method call chains like `a.b().c().d()`.
/// Heuristic: count sequences of `.identifier(`.
pub(crate) fn count_method_call_chains(body: &str) -> usize {
    let re = cached_regex(r"\.\w+\s*\(");
    let matches: Vec<_> = re.find_iter(body).collect();
    if matches.is_empty() {
        return 0;
    }
    let mut max_chain: usize = 1;
    let mut cur_chain: usize = 1;
    for i in 1..matches.len() {
        let gap = matches[i].start() - matches[i - 1].end();
        if gap <= 5 {
            cur_chain += 1;
            max_chain = max_chain.max(cur_chain);
        } else {
            cur_chain = 1;
        }
    }
    max_chain
}

/// Line number for a byte offset (1-based).
pub(crate) fn line_number(code: &str, byte_offset: usize) -> usize {
    code[..byte_offset].chars().filter(|&c| c == '\n').count() + 1
}

// ===========================================================================
// Language-specific cyclomatic complexity helpers
// ===========================================================================

/// Extended cyclomatic complexity: base keywords + extra per-language keywords.
fn calculate_cc_ext(body: &str, extras: &[&'static str]) -> usize {
    let mut cc = calculate_cc(body);
    for kw in extras {
        cc += count_keyword(body, kw);
    }
    cc
}

pub(crate) fn calculate_cc_java(body: &str) -> usize {
    calculate_cc_ext(
        body,
        &[r"\bdo\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:", r"\btry\b"],
    )
}

pub(crate) fn calculate_cc_cpp(body: &str) -> usize {
    calculate_cc_ext(
        body,
        &[r"\bdo\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:", r"\btry\b"],
    )
}

pub(crate) fn calculate_cc_csharp(body: &str) -> usize {
    calculate_cc_ext(
        body,
        &[
            r"\bforeach\b",
            r"\bfrom\b",
            r"\bwhere\b",
            r"\bselect\b",
            r"\b\w+\s*\?\s*[^:\n]{1,50}:",
        ],
    )
}

pub(crate) fn calculate_cc_php(body: &str) -> usize {
    calculate_cc_ext(
        body,
        &[
            r"\belseif\b",
            r"\bforeach\b",
            r"\bdo\b",
            r"\b\w+\s*\?\s*[^:\n]{1,50}:",
        ],
    )
}

pub(crate) fn calculate_cc_kotlin(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bwhen\b", r"\bis\b"])
}

pub(crate) fn calculate_cc_rust(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bloop\b", r"=>"])
}

// ===========================================================================
// Comment stripping
// ===========================================================================

pub(crate) fn remove_line_comments<'a>(code: &'a str, prefix: &str) -> Cow<'a, str> {
    let re = cached_regex_owned(&format!(r"(?m){prefix}.*$"));
    re.replace_all(code, "")
}

pub(crate) fn remove_block_comments(code: &str) -> Cow<'_, str> {
    cached_regex(r"/\*.*?\*/").replace_all(code, "")
}

/// Strip Ruby `=begin`/`=end` block comments.
pub(crate) fn remove_ruby_block_comments(code: &str) -> Cow<'_, str> {
    cached_regex(r"(?m)^=begin\b.*?^=end\b").replace_all(code, "")
}

/// Strip Python/Ruby `#` line comments.
pub(crate) fn remove_hash_comments(code: &str) -> Cow<'_, str> {
    cached_regex(r"(?m)#.*$").replace_all(code, "")
}

// ===========================================================================
// Python-specific helpers (shared between python.rs and ruby.rs)
// ===========================================================================

pub(crate) fn count_python_loc(body: &str) -> usize {
    body.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('#')
                && !t.starts_with("'''")
                && !t.starts_with("\"\"\"")
        })
        .count()
}

pub(crate) fn calculate_cc_python(body: &str) -> usize {
    let mut cc: usize = 1;
    cc += count_keyword(body, r"\bif\b");
    cc += count_keyword(body, r"\belif\b");
    cc += count_keyword(body, r"\bfor\b");
    cc += count_keyword(body, r"\bwhile\b");
    cc += count_keyword(body, r"\bexcept\b");
    cc += count_keyword(body, r"\bwith\b");
    cc += count_keyword(body, r"\band\b");
    cc += count_keyword(body, r"\bor\b");
    cc
}

pub(crate) fn calculate_nesting_python(body: &str) -> usize {
    let mut max_d: usize = 0;
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let depth = indent / 4;
        max_d = max_d.max(depth);
    }
    max_d
}

pub(crate) fn count_branches_python(body: &str) -> usize {
    count_keyword(body, r"\bif\b")
        + count_keyword(body, r"\belif\b")
        + count_keyword(body, r"\bcase\b")
        + count_keyword(body, r"\bmatch\b")
}

pub(crate) fn strip_python_docstrings(code: &str) -> Cow<'_, str> {
    let triple_double = cached_regex(r#"(?s)""".*?""""#);
    let no_double = triple_double.replace_all(code, "");
    let triple_single = cached_regex(r"(?s)'''.*?'''");
    triple_single
        .replace_all(&no_double, "")
        .into_owned()
        .into()
}

pub(crate) fn count_primitive_params_python(sig: &str) -> usize {
    let start = match sig.find('(') {
        Some(i) => i + 1,
        None => return 0,
    };
    let end = match sig[start..].find(')') {
        Some(i) => start + i,
        None => return 0,
    };
    let params = &sig[start..end];
    if params.trim().is_empty() {
        return 0;
    }
    let primitive_re = cached_regex(
        r":\s*(int|float|bool|str|bytes|list|dict|set|tuple|Optional\[[^\]]+\]|Union\[[^\]]+\])\b",
    );
    params
        .split(',')
        .filter(|p| {
            let t = p.trim();
            !t.is_empty() && (primitive_re.is_match(t) || !t.contains(':'))
        })
        .count()
}

// ===========================================================================
// Metric builder helpers (used by typescript, generic, go)
// ===========================================================================

/// Build function metrics with the default `count_local_vars`.
pub(crate) fn build_func_metrics(body: &str, sig: &str, cc_fn: fn(&str) -> usize) -> CodeMetrics {
    build_func_metrics_ext(body, sig, cc_fn, count_local_vars)
}

/// Build function metrics with a custom local-var counter.
pub(crate) fn build_func_metrics_ext(
    body: &str,
    sig: &str,
    cc_fn: fn(&str) -> usize,
    vars_fn: fn(&str) -> usize,
) -> CodeMetrics {
    let params = count_params(sig);
    CodeMetrics {
        loc: count_loc(body),
        cyclomatic_complexity: cc_fn(body),
        nesting_depth: calculate_nesting(body),
        parameter_count: params,
        local_variables: vars_fn(body),
        return_statements: count_returns(body),
        external_calls: count_external_calls(body),
        primitive_params: params,
        branch_count: count_branches(body),
        method_call_chains: count_method_call_chains(body),
        ..Default::default()
    }
}
