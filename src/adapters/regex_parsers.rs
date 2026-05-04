//! Regex-based language parsers for smell detection.
//!
//! Each parser finds function/class definitions, extracts bodies via brace
//! matching, computes `CodeMetrics`, and delegates to `detect_all`.

use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, SmellDetection};
use crate::ports::parser::CodeParser;

// ===========================================================================
// Global regex cache
// ===========================================================================

/// Global cache for compiled regexes used by `count_keyword` and helper functions.
/// Avoids recompiling the same patterns on every call.
static REGEX_CACHE: OnceLock<Mutex<HashMap<&'static str, &'static Regex>>> = OnceLock::new();

/// Return a cached `&'static Regex` for a static pattern, compiling it on first use.
/// Compiled regexes are leaked and live for the process lifetime. The set of
/// distinct patterns is small and bounded, so this is an acceptable trade-off.
fn cached_regex(pattern: &'static str) -> &'static Regex {
    let cache = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    {
        let guard = cache.lock().unwrap();
        if let Some(&re) = guard.get(pattern) {
            return re;
        }
    }
    // Compile and leak the Regex outside the lock.
    let re: &'static Regex = Box::leak(Box::new(Regex::new(pattern).unwrap()));
    let mut guard = cache.lock().unwrap();
    guard.insert(pattern, re);
    re
}

/// Return a cached `Regex` for a dynamically constructed pattern string.
/// The returned Regex is cloned out of the cache to avoid lifetime issues
/// with non-static strings.
fn cached_regex_owned(pattern: &str) -> Regex {
    static OWNED_CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();
    let cache = OWNED_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();
    guard
        .entry(pattern.to_string())
        .or_insert_with(|| Regex::new(pattern).unwrap())
        .clone()
}

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Find the position of the matching `}` for the `{` at `start`.
/// Returns `None` on failure (unbalanced braces or string-literal confusion).
fn find_matching_brace(code: &str, start: usize) -> Option<usize> {
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

/// Count non-blank, non-brace-only lines.
fn count_loc(body: &str) -> usize {
    body.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && t != "{" && t != "}"
        })
        .count()
}

/// Compute cyclomatic complexity from common control-flow keywords.
fn calculate_cc(body: &str) -> usize {
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
fn calculate_nesting(body: &str) -> usize {
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
fn count_keyword(code: &str, pattern: &'static str) -> usize {
    cached_regex(pattern).find_iter(code).count()
}

/// Count return statements.
fn count_returns(body: &str) -> usize {
    count_keyword(body, r"\breturn\b")
}

/// Count local variables: `var x` and `x :=` and simple `x = y`.
fn count_local_vars(body: &str) -> usize {
    count_keyword(body, r"\bvar\s+\w+")
        + count_keyword(body, r"\w+\s*:=")
        + count_keyword(body, r"\blet\s+\w+")
        + count_keyword(body, r"\bconst\s+\w+")
}

/// Count local variables in C/C++: typed declarations like `int x =`, `auto x;`.
fn count_local_vars_cpp(body: &str) -> usize {
    count_keyword(
        body,
        r"\b(?:int|double|float|bool|char|void|auto|long|short|unsigned|signed|size_t)\s+\w+\s*[=;]",
    )
}

/// Count local variables in C#: typed and `var` declarations like `int x =`, `var x;`.
fn count_local_vars_csharp(body: &str) -> usize {
    count_keyword(
        body,
        r"\b(?:int|string|bool|double|float|var|decimal|long|byte|char|short|uint|ulong|ushort)\s+\w+\s*[=;]",
    )
}

/// Count local variables in PHP: `$x =` (PHP variables always start with $).
fn count_local_vars_php(body: &str) -> usize {
    count_keyword(body, r"\$\w+\s*=")
}

/// Count local variables in Kotlin: `val x` and `var x` declarations.
fn count_local_vars_kotlin(body: &str) -> usize {
    count_keyword(body, r"\b(?:val|var)\s+\w+")
}

/// Count parameters inside the first balanced paren group in `sig`.
fn count_params(sig: &str) -> usize {
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
fn count_external_calls(body: &str) -> usize {
    count_keyword(body, r"\w+\.\w+\s*\(")
}

/// Count branches: `if`, `elif`, `else if`, `case`, `match` arms.
fn count_branches(body: &str) -> usize {
    count_keyword(body, r"\bif\b")
        + count_keyword(body, r"\belif\b")
        + count_keyword(body, r"\belse\s+if\b")
        + count_keyword(body, r"\bcase\b")
        + count_keyword(body, r"\bmatch\b")
}

/// Count method call chains like `a.b().c().d()`.
/// Heuristic: count sequences of `.identifier(`.
fn count_method_call_chains(body: &str) -> usize {
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
fn line_number(code: &str, byte_offset: usize) -> usize {
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

fn calculate_cc_java(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bdo\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:", r"\btry\b"])
}

fn calculate_cc_cpp(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bdo\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:", r"\btry\b"])
}

fn calculate_cc_csharp(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bforeach\b", r"\bfrom\b", r"\bwhere\b", r"\bselect\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:"])
}

fn calculate_cc_php(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\belseif\b", r"\bforeach\b", r"\bdo\b", r"\b\w+\s*\?\s*[^:\n]{1,50}:"])
}

fn calculate_cc_kotlin(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bwhen\b", r"\bis\b"])
}

fn calculate_cc_rust(body: &str) -> usize {
    calculate_cc_ext(body, &[r"\bloop\b", r"=>"])
}

// ===========================================================================
// Comment stripping
// ===========================================================================

fn remove_line_comments<'a>(code: &'a str, prefix: &str) -> std::borrow::Cow<'a, str> {
    let re = cached_regex_owned(&format!(r"(?m){prefix}.*$"));
    re.replace_all(code, "")
}

fn remove_block_comments(code: &str) -> std::borrow::Cow<'_, str> {
    cached_regex(r"/\*.*?\*/").replace_all(code, "")
}

/// Strip Ruby `=begin`/`=end` block comments.
fn remove_ruby_block_comments(code: &str) -> std::borrow::Cow<'_, str> {
    cached_regex(r"(?m)^=begin\b.*?^=end\b").replace_all(code, "")
}

/// Strip Python/Ruby `#` line comments.
fn remove_hash_comments(code: &str) -> std::borrow::Cow<'_, str> {
    cached_regex(r"(?m)#.*$").replace_all(code, "")
}

// ===========================================================================
// Python Parser (kept separate: indentation-based body extraction)
// ===========================================================================

pub struct PythonParser;

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonParser {
    pub fn new() -> Self {
        Self
    }
}

impl CodeParser for PythonParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let cleaned = remove_hash_comments(code);
        let cleaned = strip_python_docstrings(&cleaned);
        let mut detections: Vec<SmellDetection> = Vec::new();

        // --- Functions (def / async def) ---
        let fn_re = cached_regex(r"(?m)^(?:async\s+)?def\s+(\w+)\s*\(");
        for cap in fn_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let sig_start = full.start();

            let sig_line_start = cleaned[..sig_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let indent = cleaned[sig_line_start..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);
            let body_end = find_python_block_end(&cleaned, sig_start, indent);
            let body = &cleaned[sig_start..body_end];

            let sig_text = &cleaned[sig_start..];
            let loc = count_python_loc(body);
            let params = count_params(sig_text);
            let primitive_params = count_primitive_params_python(sig_text);
            let cc = calculate_cc_python(body);
            let nesting = calculate_nesting_python(body);
            let returns = count_returns(body);
            let local_vars = count_keyword(body, r"(?m)^\s*(\w+)\s*=")
                + count_keyword(body, r"(?m)^\s*(\w+)\s*:");
            let ext_calls = count_external_calls(body);
            let branches = count_branches_python(body);
            let chains = count_method_call_chains(body);

            let metrics = CodeMetrics {
                loc,
                cyclomatic_complexity: cc,
                nesting_depth: nesting,
                parameter_count: params,
                local_variables: local_vars,
                return_statements: returns,
                external_calls: ext_calls,
                primitive_params,
                branch_count: branches,
                method_call_chains: chains,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, sig_start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Classes ---
        let cls_re = cached_regex(r"(?m)^class\s+(\w+)");
        let python_method_re = cached_regex(r"(?m)^\s+def\s+\w+");
        for cap in cls_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let cls_start = full.start();

            let sig_line_start = cleaned[..cls_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let indent = cleaned[sig_line_start..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);
            let body_end = find_python_block_end(&cleaned, cls_start, indent);
            let body = &cleaned[cls_start..body_end];

            let method_count = python_method_re.find_iter(body).count();
            let field_count = count_keyword(body, r"self\.\w+\s*=");

            let metrics = CodeMetrics {
                loc: count_python_loc(body),
                method_count,
                field_count,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, cls_start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["py"]
    }
}

fn find_python_block_end(code: &str, start: usize, base_indent: usize) -> usize {
    let lines = code[start..].lines().enumerate();
    let mut end = code.len();
    for (i, line) in lines {
        if i == 0 {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cur_indent = line.len() - trimmed.len();
        if cur_indent <= base_indent {
            let mut offset = start;
            for (j, l) in code[start..].lines().enumerate() {
                if j == i {
                    break;
                }
                offset += l.len() + 1;
            }
            end = offset;
            break;
        }
    }
    end
}

fn count_python_loc(body: &str) -> usize {
    body.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with("'''") && !t.starts_with("\"\"\"")
        })
        .count()
}

fn calculate_cc_python(body: &str) -> usize {
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

fn calculate_nesting_python(body: &str) -> usize {
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

fn count_branches_python(body: &str) -> usize {
    count_keyword(body, r"\bif\b")
        + count_keyword(body, r"\belif\b")
        + count_keyword(body, r"\bcase\b")
        + count_keyword(body, r"\bmatch\b")
}

fn strip_python_docstrings(code: &str) -> std::borrow::Cow<'_, str> {
    let triple_double = cached_regex(r#"(?s)""".*?""""#);
    let no_double = triple_double.replace_all(code, "");
    let triple_single = cached_regex(r"(?s)'''.*?'''");
    triple_single.replace_all(&no_double, "").into_owned().into()
}

fn count_primitive_params_python(sig: &str) -> usize {
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
// TypeScript Parser (kept separate: arrow functions need special handling)
// ===========================================================================

pub struct TypeScriptParser;

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self
    }
}

impl CodeParser for TypeScriptParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let cleaned = remove_line_comments(code, "//");
        let cleaned = remove_block_comments(&cleaned);
        let mut detections: Vec<SmellDetection> = Vec::new();

        // --- Functions: function declarations ---
        let fn_re =
            cached_regex(r"(?m)(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(");
        for cap in fn_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            let brace_pos = match cleaned[start..].find('{') {
                Some(off) => start + off,
                None => continue,
            };
            let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                Some(p) => p,
                None => continue,
            };

            let body = &cleaned[start..=end_pos];
            let sig = &cleaned[start..];

            let metrics = build_func_metrics(body, sig, calculate_cc);

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Arrow functions: const/let/var name = (params) => { ... } or expr ---
        let arrow_re = cached_regex(
            r"(?m)(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s*)?(?:\([^)]*\)|\w+)\s*=>",
        );
        for cap in arrow_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            let after_arrow = match cleaned[start..].find("=>") {
                Some(off) => start + off + 2,
                None => continue,
            };

            let body_start = cleaned[after_arrow..]
                .find(|c: char| !c.is_whitespace())
                .map(|off| after_arrow + off)
                .unwrap_or(after_arrow);

            let body_end = if cleaned.as_bytes().get(body_start) == Some(&b'{') {
                match find_matching_brace(&cleaned, body_start) {
                    Some(p) => p,
                    None => continue,
                }
            } else {
                find_ts_expression_end(&cleaned, after_arrow)
            };

            let body = &cleaned[start..=body_end];
            let sig = &cleaned[start..];

            let metrics = build_func_metrics(body, sig, calculate_cc);

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Classes ---
        let cls_re = cached_regex(r"(?m)(?:export\s+)?(?:abstract\s+)?class\s+(\w+)");
        let ts_method_re = cached_regex(
            r"(?m)(?:public|private|protected|static|\s)+\w+\s*\([^)]*\)\s*(?::\s*[\w<>\[\]]+\s*)?\{",
        );
        for cap in cls_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            let brace_pos = match cleaned[start..].find('{') {
                Some(off) => start + off,
                None => continue,
            };
            let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                Some(p) => p,
                None => continue,
            };

            let body = &cleaned[start..=end_pos];
            let method_count = ts_method_re.find_iter(body).count();
            let field_count = count_keyword(
                body,
                r"(?:public|private|protected|readonly)\s+\w+\s*[:=]",
            );

            let metrics = CodeMetrics {
                loc: count_loc(body),
                method_count,
                field_count,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx"]
    }
}

/// For expression-bodied TypeScript arrow functions, find the end of the
/// expression by scanning forward from `start`.
fn find_ts_expression_end(code: &str, start: usize) -> usize {
    let line_start = code[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let base_indent = code[line_start..]
        .find(|c: char| !c.is_whitespace())
        .unwrap_or(0);

    let suffix = &code[start..];
    let mut offset = start;

    for (i, line) in suffix.lines().enumerate() {
        if i == 0 {
            offset += line.len();
            if i < suffix.lines().count() || suffix.ends_with('\n') {
                offset += 1;
            }
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            offset += line.len();
            offset += 1;
            continue;
        }
        let cur_indent = line.len() - trimmed.len();
        if trimmed.starts_with('}')
            || (cur_indent <= base_indent
                && (trimmed.starts_with("const ")
                    || trimmed.starts_with("let ")
                    || trimmed.starts_with("var ")
                    || trimmed.starts_with("function ")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("export ")))
        {
            break;
        }
        offset += line.len();
        offset += 1;
    }

    while offset > start && code.as_bytes().get(offset - 1) == Some(&b'\n') {
        offset -= 1;
    }
    if offset >= code.len() {
        code.len() - 1
    } else if offset <= start {
        start
    } else {
        offset
    }
}

// ===========================================================================
// Ruby Parser (kept separate: end-block-based body extraction)
// ===========================================================================

pub struct RubyParser;

impl Default for RubyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyParser {
    pub fn new() -> Self {
        Self
    }
}

impl CodeParser for RubyParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let cleaned = remove_hash_comments(code);
        let cleaned = remove_ruby_block_comments(&cleaned);
        let mut detections: Vec<SmellDetection> = Vec::new();

        // --- Methods (def / define_method) ---
        let fn_re = cached_regex(r"(?m)def\s+(?:self\.)?(\w+)[?!]?");
        for cap in fn_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let sig_start = full.start();

            let sig_line_start = code[..sig_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let indent = code[sig_line_start..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);
            let body_end = find_ruby_block_end(&cleaned, sig_start, indent);
            let body = &cleaned[sig_start..body_end];

            let sig_text = &cleaned[sig_start..];
            let params = count_params(sig_text);

            let metrics = CodeMetrics {
                loc: count_python_loc(body),
                cyclomatic_complexity: calculate_cc_ruby(body),
                nesting_depth: calculate_nesting_ruby(body),
                parameter_count: params,
                local_variables: count_keyword(body, r"(?m)^\s*(\w+)\s*="),
                return_statements: count_returns(body),
                external_calls: count_external_calls(body),
                primitive_params: params,
                branch_count: count_branches_python(body),
                method_call_chains: count_method_call_chains(body),
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, sig_start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Classes ---
        let cls_re = cached_regex(r"(?m)class\s+(\w+)");
        let ruby_method_re = cached_regex(r"(?m)^\s+def\s+\w+");
        for cap in cls_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let cls_start = full.start();

            let sig_line_start = code[..cls_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let indent = code[sig_line_start..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);
            let body_end = find_ruby_block_end(&cleaned, cls_start, indent);
            let body = &cleaned[cls_start..body_end];

            let method_count = ruby_method_re.find_iter(body).count();
            let field_count = count_keyword(body, r"@\w+");

            let metrics = CodeMetrics {
                loc: count_python_loc(body),
                method_count,
                field_count,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, cls_start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rb"]
    }
}

fn calculate_cc_ruby(body: &str) -> usize {
    let mut cc: usize = 1;
    cc += count_keyword(body, r"\bif\b");
    cc += count_keyword(body, r"\belsif\b");
    cc += count_keyword(body, r"\bunless\b");
    cc += count_keyword(body, r"\bfor\b");
    cc += count_keyword(body, r"\bwhile\b");
    cc += count_keyword(body, r"\buntil\b");
    cc += count_keyword(body, r"\bcase\b");
    cc += count_keyword(body, r"\bwhen\b");
    cc += count_keyword(body, r"\brescue\b");
    cc += count_keyword(body, r"\band\b");
    cc += count_keyword(body, r"\bor\b");
    cc += count_keyword(body, r"&&");
    cc += count_keyword(body, r"\|\|");
    cc
}

fn calculate_nesting_ruby(body: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    let open_re = cached_regex(r"^\s*(class|module|def|if|unless|case|while|until|for|begin|do)\b");
    let close_re = cached_regex(r"^\s*end\b");
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if close_re.is_match(t) {
            depth = depth.saturating_sub(1);
            continue;
        }
        if open_re.is_match(t) {
            depth += 1;
            max_depth = max_depth.max(depth);
        }
    }
    max_depth
}

fn find_ruby_block_end(code: &str, start: usize, _base_indent: usize) -> usize {
    let open_re = cached_regex(r"^\s*(class|module|def|if|unless|case|while|until|for|begin|do)\b");
    let close_re = cached_regex(r"^\s*end\b");
    let mut depth = 0i32;
    let mut offset = start;
    for line in code[start..].lines() {
        let t = line.trim();
        if open_re.is_match(t) {
            depth += 1;
        } else if close_re.is_match(t) {
            depth -= 1;
            if depth <= 0 {
                return (offset + line.len()).min(code.len());
            }
        }
        offset = offset.saturating_add(line.len() + 1);
    }
    code.len()
}

// ===========================================================================
// Generic brace-based parser
// ===========================================================================

/// Configuration for a brace-based language parser.
struct ParserConfig {
    name: &'static str,
    extensions: &'static [&'static str],
    func_regex: &'static str,
    class_regex: Option<&'static str>,
    class_method_regex: Option<&'static str>,
    class_field_regex: Option<&'static str>,
    strip_line_comment: &'static str,
    strip_block_comments: bool,
    strip_hash_comments: bool,
    cc_fn: fn(&str) -> usize,
    count_local_vars_fn: fn(&str) -> usize,
    /// Keywords to skip when they appear as captured function names.
    skip_names: &'static [&'static str],
}

impl std::fmt::Debug for ParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserConfig")
            .field("name", &self.name)
            .field("extensions", &self.extensions)
            .finish_non_exhaustive()
    }
}

pub struct GenericParser {
    config: ParserConfig,
    func_re: OnceLock<Regex>,
    class_re: OnceLock<Regex>,
    class_method_re: OnceLock<Regex>,
    class_field_re: OnceLock<Regex>,
}

impl GenericParser {
    fn new(config: ParserConfig) -> Self {
        Self {
            config,
            func_re: OnceLock::new(),
            class_re: OnceLock::new(),
            class_method_re: OnceLock::new(),
            class_field_re: OnceLock::new(),
        }
    }

    fn get_func_re(&self) -> &Regex {
        self.func_re.get_or_init(|| Regex::new(self.config.func_regex).unwrap())
    }

    fn get_class_re(&self) -> Option<&Regex> {
        self.config.class_regex.map(|pat| {
            self.class_re.get_or_init(|| Regex::new(pat).unwrap())
        })
    }

    fn get_class_method_re(&self) -> Option<&Regex> {
        self.config.class_method_regex.map(|pat| {
            self.class_method_re.get_or_init(|| Regex::new(pat).unwrap())
        })
    }

    fn get_class_field_re(&self) -> Option<&Regex> {
        self.config.class_field_regex.map(|pat| {
            self.class_field_re.get_or_init(|| Regex::new(pat).unwrap())
        })
    }

    /// Strip comments according to config.
    fn strip_comments<'a>(&self, code: &'a str) -> std::borrow::Cow<'a, str> {
        let mut cleaned: std::borrow::Cow<'_, str> = if self.config.strip_line_comment.is_empty() {
            std::borrow::Cow::Borrowed(code)
        } else {
            remove_line_comments(code, self.config.strip_line_comment)
        };
        if self.config.strip_block_comments {
            cleaned = remove_block_comments(&cleaned).into_owned().into();
        }
        if self.config.strip_hash_comments {
            cleaned = remove_hash_comments(&cleaned).into_owned().into();
        }
        cleaned
    }
}

impl Default for GenericParser {
    fn default() -> Self {
        Self::new(ParserConfig {
            name: "",
            extensions: &[],
            func_regex: "",
            class_regex: None,
            class_method_regex: None,
            class_field_regex: None,
            strip_line_comment: "",
            strip_block_comments: false,
            strip_hash_comments: false,
            cc_fn: calculate_cc,
            count_local_vars_fn: count_local_vars,
            skip_names: &[],
        })
    }
}

impl CodeParser for GenericParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let cleaned = self.strip_comments(code);
        let mut detections: Vec<SmellDetection> = Vec::new();
        let func_re = self.get_func_re();
        let cc_fn = self.config.cc_fn;
        let vars_fn = self.config.count_local_vars_fn;
        let skip = self.config.skip_names;

        // --- Functions ---
        for cap in func_re.captures_iter(&cleaned) {
            let name = &cap[1];
            if skip.contains(&name) {
                continue;
            }
            let full = cap.get(0).unwrap();
            let start = full.start();

            let brace_pos = match cleaned[start..].find('{') {
                Some(off) => start + off,
                None => continue,
            };
            let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                Some(p) => p,
                None => continue,
            };

            let body = &cleaned[start..=end_pos];
            let sig = &cleaned[start..];
            let metrics = build_func_metrics_ext(body, sig, cc_fn, vars_fn);

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Classes ---
        if let (Some(class_re), Some(class_method_re)) =
            (self.get_class_re(), self.get_class_method_re())
        {
            for cap in class_re.captures_iter(&cleaned) {
                let name = &cap[1];
                let full = cap.get(0).unwrap();
                let start = full.start();

                let brace_pos = match cleaned[start..].find('{') {
                    Some(off) => start + off,
                    None => continue,
                };
                let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                    Some(p) => p,
                    None => continue,
                };

                let body = &cleaned[start..=end_pos];
                let method_count = class_method_re.find_iter(body).count();
                let field_count = self
                    .get_class_field_re()
                    .map(|re| re.find_iter(body).count())
                    .unwrap_or(0);

                let metrics = CodeMetrics {
                    loc: count_loc(body),
                    method_count,
                    field_count,
                    ..Default::default()
                };

                let location = format!("{}:{}", file_name, line_number(&cleaned, start));
                detections.extend(detect_all(&metrics, &location, name));
            }
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        self.config.extensions
    }
}

/// Build function metrics with the default `count_local_vars`.
fn build_func_metrics(
    body: &str,
    sig: &str,
    cc_fn: fn(&str) -> usize,
) -> CodeMetrics {
    build_func_metrics_ext(body, sig, cc_fn, count_local_vars)
}

/// Build function metrics with a custom local-var counter.
fn build_func_metrics_ext(
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

// ===========================================================================
// Concrete parser instances
// ===========================================================================

/// Java parser (brace-based).
pub fn java_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "java",
        extensions: &["java"],
        func_regex: r"(?m)(?:public|private|protected|static|\s)+[\w<>\[\]]+\s+(\w+)\s*\(",
        class_regex: Some(r"(?m)(?:public\s+)?(?:abstract\s+)?(?:class|interface|enum)\s+(\w+)"),
        class_method_regex: Some(r"(?m)(?:public|private|protected)\s+[\w<>\[\]]+\s+\w+\s*\("),
        class_field_regex: Some(r"(?m)(?:public|private|protected)\s+[\w<>\[\]]+\s+\w+\s*;"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_java,
        count_local_vars_fn: count_local_vars,
        skip_names: &[],
    })
}

/// Basic Go parser (brace-based).
///
/// This parser cannot detect Go struct receiver methods. Use [`GoFullParser`]
/// instead for full Go support including struct method counting.
/// Marked `pub(crate)` because external callers should use [`GoFullParser`].
pub(crate) fn go_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "go",
        extensions: &["go"],
        func_regex: r"(?m)func\s+(?:\([^)]*\)\s*)?(\w+)\s*\(",
        class_regex: Some(r"(?m)type\s+(\w+)\s+struct\s*\{"),
        class_method_regex: None, // handled specially in GoFullParser
        class_field_regex: None,
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc,
        count_local_vars_fn: count_local_vars,
        skip_names: &[],
    })
}

/// Rust parser (brace-based).
pub fn rust_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "rust",
        extensions: &["rs"],
        func_regex: r"(?m)(?:pub\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+(\w+)\s*[\(<]",
        class_regex: Some(r"(?m)impl\s+(?:<[^>]*>\s*)?(\w+)"),
        class_method_regex: Some(r"(?m)(?:pub\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+\w+"),
        class_field_regex: None,
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_rust,
        count_local_vars_fn: count_local_vars,
        skip_names: &[],
    })
}

/// C/C++ parser (brace-based).
pub fn cpp_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "cpp",
        extensions: &["cpp", "cxx", "cc", "c", "hpp", "h"],
        func_regex: r"(?m)(?:(?:static|inline|virtual|const|extern)\s+)*(?:[\w:*&<>,\s]+)\s+(\w+)\s*\(",
        class_regex: Some(r"(?m)(?:class|struct)\s+(\w+)\s*(?::\s*[^\{]*)?\{"),
        class_method_regex: Some(r"(?m)(?:(?:public|private|protected|virtual|static)\s+)*[\w:*&<>,\s]+\s+\w+\s*\("),
        class_field_regex: Some(r"(?m)(?:public|private|protected)\s+[\w:*&<>,\s]+\s+\w+\s*;"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_cpp,
        count_local_vars_fn: count_local_vars_cpp,
        skip_names: &["if", "for", "while", "switch", "catch", "return", "class", "struct"],
    })
}

/// C# parser (brace-based).
pub fn csharp_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "csharp",
        extensions: &["cs"],
        func_regex: r"(?m)(?:(?:public|private|protected|internal|static|virtual|override|async|abstract)\s+)+[\w<>\[\]?]+\s+(\w+)\s*\(",
        class_regex: Some(r"(?m)(?:(?:public|private|protected|internal|static|abstract|sealed)\s+)*(?:class|struct|record)\s+(\w+)"),
        class_method_regex: Some(r"(?m)(?:public|private|protected|internal)\s+[\w<>\[\]?]+\s+\w+\s*\("),
        class_field_regex: Some(r"(?m)(?:public|private|protected|internal|readonly)\s+[\w<>\[\]?]+\s+\w+\s*[;=]"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_csharp,
        count_local_vars_fn: count_local_vars_csharp,
        skip_names: &["if", "for", "while", "switch", "catch", "using", "lock"],
    })
}

/// Kotlin parser (brace-based).
pub fn kotlin_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "kotlin",
        extensions: &["kt", "kts"],
        func_regex: r"(?m)(?:(?:public|private|protected|internal|suspend|inline|open|override|abstract)\s+)*fun\s+(?:<[^>]*>\s*)?(\w+)\s*\(",
        class_regex: Some(r"(?m)(?:(?:public|private|protected|internal|open|abstract|sealed|data|inner)\s+)*class\s+(\w+)"),
        class_method_regex: Some(r"(?m)fun\s+(?:<[^>]*>\s*)?\w+\s*\("),
        class_field_regex: Some(r"(?:val|var)\s+\w+"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_kotlin,
        count_local_vars_fn: count_local_vars_kotlin,
        skip_names: &[],
    })
}

/// PHP parser (brace-based).
pub fn php_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "php",
        extensions: &["php"],
        func_regex: r"(?m)function\s+(\w+)\s*\(",
        class_regex: Some(r"(?m)(?:final\s+)?(?:abstract\s+)?class\s+(\w+)"),
        class_method_regex: Some(r"(?m)(?:public|private|protected|static)\s+function\s+\w+"),
        class_field_regex: Some(r"(?m)(?:public|private|protected|static)\s+(?:\$)\w+"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: true,
        cc_fn: calculate_cc_php,
        count_local_vars_fn: count_local_vars_php,
        skip_names: &[],
    })
}

// ===========================================================================
// Go parser (special struct method counting)
// ===========================================================================

/// Extended Go parser that counts struct receiver methods across the full file.
pub struct GoFullParser {
    inner: GenericParser,
}

impl GoFullParser {
    pub fn new() -> Self {
        Self {
            inner: go_parser(),
        }
    }
}

impl Default for GoFullParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeParser for GoFullParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let cleaned = self.inner.strip_comments(code);
        let mut detections: Vec<SmellDetection> = Vec::new();
        let func_re = self.inner.get_func_re();

        // --- Functions ---
        for cap in func_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            let brace_pos = match cleaned[start..].find('{') {
                Some(off) => start + off,
                None => continue,
            };
            let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                Some(p) => p,
                None => continue,
            };

            let body = &cleaned[start..=end_pos];
            let sig = &cleaned[start..];
            let metrics = build_func_metrics(body, sig, calculate_cc);

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Structs (with receiver method counting) ---
        let struct_re = cached_regex(r"(?m)type\s+(\w+)\s+struct\s*\{");
        for cap in struct_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            let brace_pos = match cleaned[start..].find('{') {
                Some(off) => start + off,
                None => continue,
            };
            let end_pos = match find_matching_brace(&cleaned, brace_pos) {
                Some(p) => p,
                None => continue,
            };

            let body = &cleaned[start..=end_pos];
            let field_count = body
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && t != "{" && t != "}" && !t.starts_with("//")
                })
                .count();

            let method_re =
                cached_regex_owned(&format!(r"(?m)func\s+\([^)]*\s+\*?{name}\)\s+\w+"));
            let method_count = method_re.find_iter(&cleaned).count();

            let metrics = CodeMetrics {
                loc: count_loc(body),
                method_count,
                field_count,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["go"]
    }
}

// ===========================================================================
// Factory
// ===========================================================================

/// Return the appropriate parser for the given language name.
///
/// Matches the Python `get_parser()` factory: case-insensitive,
/// supports aliases like `"js"` -> TypeScriptParser, `"cpp"` -> CppParser.
pub fn get_parser(language: &str) -> Result<Box<dyn CodeParser>, String> {
    match language.to_ascii_lowercase().as_str() {
        "python" => Ok(Box::new(crate::adapters::python_ast_parser::PythonAstParser::new())),
        "java" => Ok(Box::new(java_parser())),
        "go" => Ok(Box::new(GoFullParser::new())),
        "rust" => Ok(Box::new(rust_parser())),
        "typescript" | "javascript" | "js" | "ts" => {
            Ok(Box::new(TypeScriptParser::new()))
        }
        "c" | "cpp" | "c++" | "cxx" | "cc" | "hpp" => Ok(Box::new(cpp_parser())),
        "c#" | "cs" | "csharp" => Ok(Box::new(csharp_parser())),
        "kotlin" | "kt" => Ok(Box::new(kotlin_parser())),
        "php" => Ok(Box::new(php_parser())),
        "ruby" | "rb" => Ok(Box::new(RubyParser::new())),
        other => Err(format!("Unsupported language: {other}")),
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // --- Python parser tests -----------------------------------------------

    #[test]
    fn python_detects_long_method() {
        let code = r#"
def massive_function(a, b, c, d, e, f, g, h):
    if a:
        for i in range(100):
            if b:
                while c:
                    if d:
                        for j in range(50):
                            if e:
                                x = 1
                                y = 2
                                z = 3
                                w = 4
                                return x + y + z + w
            if b and c or d:
                for k in range(20):
                    if k > 10:
                        val = k * 2
                        if val > 20:
                            result = val + 1
                            if result > 25:
                                extra = result * 3
                                if extra > 80:
                                    return extra
    if f:
        while g:
            if h:
                for m in range(10):
                    if m > 5:
                        n = m + 1
                        if n > 6:
                            return n
    return None
"#;
        let parser = PythonParser::new();
        let results = parser.parse_code(code, "test.py");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-01"),
            "should detect Long Method, got: {ids:?}"
        );
        assert!(
            ids.contains(&"SMELL-02"),
            "should detect Long Parameter List (8 params), got: {ids:?}"
        );
    }

    #[test]
    fn python_clean_code_no_smells() {
        let code = r#"
def add(a, b):
    result = a + b
    intermediate = result * 2
    final_value = intermediate + 1
    another = final_value - 3
    total = another + a
    combined = total + b
    output = combined * 0.5
    scaled = output + 10
    finished = scaled - 1
    adjusted = finished + 2
    finalized = adjusted * 3
    processed = finalized - 4
    transformed = processed + 5
    completed = transformed * 0.8
    enhanced = completed + 6
    refined = enhanced - 7
    polished = refined + 8
    improved = polished * 1.5
    optimized = improved + 9
    return optimized

def greet(name):
    greeting = f"Hello, {name}"
    length = len(greeting)
    message = f"{greeting} (length: {length})"
    upper = message.upper()
    lower = message.lower()
    trimmed = lower.strip()
    final_msg = f"{trimmed}!"
    tagged = f"[{final_msg}]"
    formatted = f"MSG: {tagged}"
    padded = formatted.center(50)
    aligned = padded.ljust(60)
    decorated = f"=={aligned}=="
    finalized = decorated.upper()
    processed = f">> {finalized} <<"
    wrapped = f"({processed})"
    encoded = wrapped.encode('utf-8')
    decoded = encoded.decode('utf-8')
    trimmed2 = decoded.strip()
    finished = f"Result: {trimmed2}"
    return finished
"#;
        let parser = PythonParser::new();
        let results = parser.parse_code(code, "clean.py");
        assert!(
            results.is_empty(),
            "clean code should have no smells, got: {results:?}"
        );
    }

    #[test]
    fn python_class_large_class() {
        let code = r#"
class MegaClass:
    self.x1 = 1
    self.x2 = 2
    self.x3 = 3
    self.x4 = 4
    self.x5 = 5
    self.x6 = 6
    self.x7 = 7
    self.x8 = 8
    self.x9 = 9
    self.x10 = 10
    self.x11 = 11
    self.x12 = 12
    self.x13 = 13
    self.x14 = 14
    self.x15 = 15
    self.x16 = 16

    def m1(self): pass
    def m2(self): pass
    def m3(self): pass
    def m4(self): pass
    def m5(self): pass
    def m6(self): pass
    def m7(self): pass
    def m8(self): pass
    def m9(self): pass
    def m10(self): pass
    def m11(self): pass
    def m12(self): pass
    def m13(self): pass
    def m14(self): pass
    def m15(self): pass
    def m16(self): pass
    def m17(self): pass
    def m18(self): pass
    def m19(self): pass
    def m20(self): pass
    def m21(self): pass
"#;
        let parser = PythonParser::new();
        let results = parser.parse_code(code, "mega.py");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-04"),
            "should detect Large Class, got: {ids:?}"
        );
    }

    // --- Go parser tests ---------------------------------------------------

    #[test]
    fn go_detects_long_function() {
        let code = r#"
package main

func bigFunc(a int, b int, c int, d int, e int, f int, g int) int {
    if a > 0 {
        for i := 0; i < 100; i++ {
            if b > 0 {
                for j := 0; j < 50; j++ {
                    if c > 0 {
                        for k := 0; k < 25; k++ {
                            if d > 0 {
                                if e > 0 {
                                    if f > 0 {
                                        x := a + b
                                        y := c + d
                                        z := e + f
                                        w := g + x
                                        q := y + z
                                        r := w + q
                                        if r > 100 {
                                            return r
                                        }
                                        if r > 50 {
                                            return r / 2
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if g > 0 {
        for m := 0; m < 20; m++ {
            if m > 10 {
                val := m * 2
                if val > 20 {
                    return val
                }
            }
        }
    }
    return 0
}
"#;
        let parser = GoFullParser::new();
        let results = parser.parse_code(code, "big.go");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-01"),
            "should detect Long Method in Go, got: {ids:?}"
        );
    }

    // --- Java parser tests -------------------------------------------------

    #[test]
    fn java_detects_long_method() {
        let code = r#"
public class Foo {
    public int bigMethod(int a, int b, int c, int d, int e, int f, int g, int h) {
        if (a > 0) {
            for (int i = 0; i < 100; i++) {
                if (b > 0) {
                    while (c > 0) {
                        if (d > 0) {
                            for (int j = 0; j < 50; j++) {
                                if (e > 0) {
                                    int x = a + b;
                                    int y = c + d;
                                    int z = e + f;
                                    if (x > 10) {
                                        return x + y + z;
                                    }
                                    if (y > 10) {
                                        return y + z;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if (f > 0) {
            for (int k = 0; k < 20; k++) {
                if (k > 10) {
                    int val = k * 2;
                    if (val > 20) {
                        return val;
                    }
                }
            }
        }
        int extra1 = a + b + c;
        int extra2 = d + e + f;
        int extra3 = g + h + extra1;
        int extra4 = extra2 + extra3;
        int extra5 = extra4 * 2;
        int extra6 = extra5 + 1;
        int extra7 = extra6 - 3;
        int extra8 = extra7 + extra1;
        int extra9 = extra8 * extra2;
        int extra10 = extra9 + extra3;
        int extra11 = extra10 - extra4;
        return extra11;
    }
}
"#;
        let parser = java_parser();
        let results = parser.parse_code(code, "Foo.java");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-01"),
            "should detect Long Method in Java, got: {ids:?}"
        );
        assert!(
            ids.contains(&"SMELL-02"),
            "should detect Long Parameter List (8 params), got: {ids:?}"
        );
    }

    // --- Rust parser tests -------------------------------------------------

    #[test]
    fn rust_detects_long_fn() {
        let code = r#"
pub fn massive(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) -> i32 {
    let mut result = 0;
    if a > 0 { result += 1; }
    if b > 0 { result += 2; }
    if c > 0 { result += 3; }
    if d > 0 { result += 4; }
    if e > 0 { result += 5; }
    if f > 0 { result += 6; }
    if g > 0 { result += 7; }
    if a > 0 && b > 0 { result += 10; }
    if c > 0 && d > 0 { result += 20; }
    if e > 0 && f > 0 { result += 30; }
    if a > 0 && g > 0 { result += 40; }
    if b > 0 && c > 0 { result += 50; }
    for i in 0..100 {
        if i > 50 { result += i; }
    }
    while result > 1000 {
        result -= 1;
    }
    let x1 = a + b;
    let x2 = c + d;
    let x3 = e + f;
    let x4 = g + x1;
    let x5 = x2 + x3;
    let x6 = x4 + x5;
    let x7 = x6 * 2;
    let x8 = x7 + 1;
    let x9 = x8 - 3;
    let x10 = x9 + x1;
    let x11 = x10 * x2;
    let x12 = x11 + x3;
    let x13 = x12 - x4;
    let x14 = x13 + x5;
    let x15 = x14 * x6;
    let x16 = x15 + x7;
    let x17 = x16 - x8;
    let x18 = x17 + x9;
    let x19 = x18 * x10;
    let x20 = x19 + result;
    return x20;
}
"#;
        let parser = rust_parser();
        let results = parser.parse_code(code, "lib.rs");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-01"),
            "should detect Long Method in Rust, got: {ids:?}"
        );
    }

    // --- TypeScript parser tests -------------------------------------------

    #[test]
    fn typescript_detects_long_function() {
        let code = r#"
export function bigFunc(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number): number {
    let mut result = 0;
    if (a > 0) { result += 1; }
    if (b > 0) { result += 2; }
    if (c > 0) { result += 3; }
    if (d > 0) { result += 4; }
    if (e > 0) { result += 5; }
    if (f > 0) { result += 6; }
    if (g > 0) { result += 7; }
    if (a > 0 && b > 0) { result += 10; }
    if (c > 0 && d > 0) { result += 20; }
    if (e > 0 && f > 0) { result += 30; }
    if (a > 0 && g > 0) { result += 40; }
    if (b > 0 && h > 0) { result += 50; }
    for (let i = 0; i < 100; i++) {
        if (i > 50) { result += i; }
    }
    while (result > 1000) {
        result -= 1;
    }
    let x1 = a + b;
    let x2 = c + d;
    let x3 = e + f;
    let x4 = g + h + x1;
    let x5 = x2 + x3;
    let x6 = x4 + x5;
    let x7 = x6 * 2;
    let x8 = x7 + 1;
    let x9 = x8 - 3;
    let x10 = x9 + x1;
    let x11 = x10 * x2;
    let x12 = x11 + x3;
    let x13 = x12 - x4;
    let x14 = x13 + x5;
    let x15 = x14 * x6;
    let x16 = x15 + x7;
    let x17 = x16 - x8;
    let x18 = x17 + x9;
    let x19 = x18 * x10;
    let x20 = x19 + result;
    return x20;
}
"#;
        let parser = TypeScriptParser::new();
        let results = parser.parse_code(code, "app.ts");
        let ids: Vec<&str> = results.iter().map(|d| d.smell_id.as_str()).collect();
        assert!(
            ids.contains(&"SMELL-01"),
            "should detect Long Method in TS, got: {ids:?}"
        );
        assert!(
            ids.contains(&"SMELL-02"),
            "should detect Long Parameter List (8 params), got: {ids:?}"
        );
    }

    // --- Factory tests -----------------------------------------------------

    #[test]
    fn factory_python() {
        let p = get_parser("python").unwrap();
        assert_eq!(p.supported_extensions(), &["py"]);
    }

    #[test]
    fn factory_aliases() {
        assert!(get_parser("JavaScript").is_ok());
        assert!(get_parser("typescript").is_ok());
        assert!(get_parser("js").is_ok());
        assert!(get_parser("GO").is_ok());
        assert!(get_parser("Rust").is_ok());
        assert!(get_parser("java").is_ok());
    }

    #[test]
    fn factory_unsupported() {
        assert!(get_parser("brainfuck").is_err());
    }

    // --- parse_file integration --------------------------------------------

    #[test]
    fn python_parse_file_missing() {
        let parser = PythonParser::new();
        let path = PathBuf::from("/nonexistent/file.py");
        let result = parser.parse_file(&path);
        assert!(result.is_err());
    }

    // --- Language-specific CC function tests --------------------------------

    #[test]
    fn cc_java_counts_do_and_try() {
        let code = "public void foo() { do { } while (x); try { } catch (E e) { } if (a) { } }";
        let cc = calculate_cc_java(code);
        assert!(cc >= 5, "Java CC should count do + try + catch + if, got {cc}");
    }

    #[test]
    fn cc_java_counts_ternary() {
        let code = "int x = a ? b : c;";
        let cc = calculate_cc_java(code);
        assert!(cc >= 2, "Java CC should count ternary, got {cc}");
    }

    #[test]
    fn cc_cpp_counts_do_and_try() {
        let code = "void foo() { do { x++; } while (x < 10); try { } catch (...) { } if (a) { } }";
        let cc = calculate_cc_cpp(code);
        assert!(cc >= 5, "C++ CC should count do + try + catch + if, got {cc}");
    }

    #[test]
    fn cc_cpp_counts_ternary() {
        let code = "int x = flag ? 1 : 0;";
        let cc = calculate_cc_cpp(code);
        assert!(cc >= 2, "C++ CC should count ternary, got {cc}");
    }

    #[test]
    fn cc_csharp_counts_foreach_and_linq() {
        let code = "void Foo() { foreach (var x in xs) { } from y in ys where y > 0 select y; if (a) { } }";
        let cc = calculate_cc_csharp(code);
        assert!(cc >= 5, "C# CC should count foreach + from + where + select + if, got {cc}");
    }

    #[test]
    fn cc_csharp_counts_ternary() {
        let code = "var x = a ? b : c;";
        let cc = calculate_cc_csharp(code);
        assert!(cc >= 2, "C# CC should count ternary, got {cc}");
    }

    #[test]
    fn cc_php_counts_elseif_foreach_do() {
        let code = "function foo() { if (a) { } elseif (b) { } foreach ($xs as $x) { } do { } while (c); }";
        let cc = calculate_cc_php(code);
        assert!(cc >= 5, "PHP CC should count if + elseif + foreach + do, got {cc}");
    }

    #[test]
    fn cc_php_counts_ternary() {
        let code = "$x = $a ? $b : $c;";
        let cc = calculate_cc_php(code);
        assert!(cc >= 2, "PHP CC should count ternary, got {cc}");
    }

    #[test]
    fn cc_kotlin_counts_when_and_is() {
        let code = "fun foo(x: Any) { when (x) { is String -> println(x) is Int -> println(x) } if (a) { } }";
        let cc = calculate_cc_kotlin(code);
        assert!(cc >= 5, "Kotlin CC should count when + is + is + if, got {cc}");
    }

    #[test]
    fn cc_rust_counts_loop_and_match_arms() {
        let code = "fn foo() { loop { x += 1; } match x { 1 => true, 2 => false, _ => true } if (a) { } }";
        let cc = calculate_cc_rust(code);
        assert!(cc >= 6, "Rust CC should count loop + match + 3 arms + if, got {cc}");
    }

    #[test]
    fn cc_ruby_counts_when() {
        let code = "def foo(x)\n  case x\n  when 'a'\n    1\n  when 'b'\n    2\n  end\nend";
        let cc = calculate_cc_ruby(code);
        assert!(cc >= 3, "Ruby CC should count case + 2 when, got {cc}");
    }

    // --- Language-specific local var counting tests --------------------------

    #[test]
    fn local_vars_cpp_counts_typed_declarations() {
        let code = "void foo() { int x = 1; double y = 2.0; auto z = 3; bool flag = true; }";
        let count = count_local_vars_cpp(code);
        assert!(count >= 4, "C++ local vars should count int/double/auto/bool, got {count}");
    }

    #[test]
    fn local_vars_csharp_counts_typed_and_var() {
        let code = "void Foo() { int x = 1; string y = \"hi\"; var z = 3; bool flag = true; }";
        let count = count_local_vars_csharp(code);
        assert!(count >= 4, "C# local vars should count int/string/var/bool, got {count}");
    }

    #[test]
    fn local_vars_php_counts_dollar_vars() {
        let code = "function foo() { $x = 1; $y = 2; $z = $x + $y; }";
        let count = count_local_vars_php(code);
        assert!(count >= 3, "PHP local vars should count $x/$y/$z, got {count}");
    }

    #[test]
    fn local_vars_kotlin_counts_val_and_var() {
        let code = "fun foo() { val x = 1; var y = 2; val z = x + y; }";
        let count = count_local_vars_kotlin(code);
        assert!(count >= 3, "Kotlin local vars should count val/var declarations, got {count}");
    }

    // --- TypeScript arrow function tests -----------------------------------

    #[test]
    fn typescript_arrow_function_block_body() {
        let code = r#"
const myFunc = (a: number, b: number) => {
    let result = a + b;
    let doubled = result * 2;
    return doubled;
};
"#;
        let parser = TypeScriptParser::new();
        let results = parser.parse_code(code, "arrow.ts");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"myFunc"),
            "should detect arrow function 'myFunc', got: {names:?}"
        );
    }

    #[test]
    fn typescript_async_arrow_function() {
        let code = r#"
const fetchData = async (url: string) => {
    const response = await fetch(url);
    const data = await response.json();
    return data;
};
"#;
        let parser = TypeScriptParser::new();
        let results = parser.parse_code(code, "async_arrow.ts");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"fetchData"),
            "should detect async arrow function 'fetchData', got: {names:?}"
        );
    }

    #[test]
    fn typescript_arrow_function_expression_body() {
        let code = r#"
const add = (a: number, b: number) => a + b;
const multiply = (a: number, b: number) => a * b;
"#;
        let parser = TypeScriptParser::new();
        let results = parser.parse_code(code, "expr_arrow.ts");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"add"),
            "should detect expression arrow 'add', got: {names:?}"
        );
        assert!(
            names.contains(&"multiply"),
            "should detect expression arrow 'multiply', got: {names:?}"
        );
    }

    #[test]
    fn typescript_exported_arrow_function() {
        let code = r#"
export const handler = (req: Request) => {
    const body = req.body;
    const result = process(body);
    return result;
};
"#;
        let parser = TypeScriptParser::new();
        let results = parser.parse_code(code, "export_arrow.ts");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"handler"),
            "should detect exported arrow 'handler', got: {names:?}"
        );
    }

    // --- Rust unsafe fn / const fn tests -----------------------------------

    #[test]
    fn rust_detects_unsafe_fn() {
        let code = r#"
pub unsafe fn dangerous(a: i32) -> i32 {
    let mut result = a;
    if a > 0 { result += 1; }
    if a > 10 { result += 2; }
    if a > 100 { result += 3; }
    if a > 1000 { result += 4; }
    result
}
"#;
        let parser = rust_parser();
        let results = parser.parse_code(code, "unsafe.rs");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"dangerous"),
            "should detect unsafe fn 'dangerous', got: {names:?}"
        );
    }

    #[test]
    fn rust_detects_const_fn() {
        let code = r#"
const fn factorial(n: u64) -> u64 {
    let mut result = 1u64;
    let mut i = 2u64;
    while i <= n {
        result *= i;
        i += 1;
    }
    result
}
"#;
        let parser = rust_parser();
        let results = parser.parse_code(code, "const_fn.rs");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"factorial"),
            "should detect const fn 'factorial', got: {names:?}"
        );
    }

    #[test]
    fn rust_detects_pub_unsafe_async_fn() {
        let code = r#"
pub unsafe async fn complex(a: i32, b: i32) -> i32 {
    let x = a + b;
    let y = x * 2;
    if x > 0 { y + 1 } else { y - 1 }
}
"#;
        let parser = rust_parser();
        let results = parser.parse_code(code, "complex.rs");
        let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
        assert!(
            names.contains(&"complex"),
            "should detect pub unsafe async fn 'complex', got: {names:?}"
        );
    }

    #[test]
    fn rust_unsafe_const_fn_metrics() {
        let code = r#"
unsafe fn compute(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut result = a + b;
    if a > 0 { result += c; }
    if b > 0 { result += d; }
    if c > 0 { result *= 2; }
    if d > 0 { result *= 3; }
    result
}
"#;
        let parser = rust_parser();
        let results = parser.parse_code(code, "metrics.rs");
        let fn_result = results.iter().find(|d| d.function_name == "compute");
        assert!(
            fn_result.is_some(),
            "should detect 'compute' function for metrics"
        );
        let m = &fn_result.unwrap().metrics;
        assert!(
            m.loc > 0,
            "unsafe fn should have non-zero LOC, got {}",
            m.loc
        );
        assert!(
            m.cyclomatic_complexity >= 5,
            "unsafe fn CC should be >= 5 (base + 4 ifs), got {}",
            m.cyclomatic_complexity
        );
    }
}
