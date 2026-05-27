//! Regex-based language parsers for smell detection.
//!
//! Each parser finds function/class definitions, extracts bodies via brace
//! matching, computes `CodeMetrics`, and delegates to `detect_all`.

mod generic;
mod go;
mod python;
mod ruby;
mod typescript;

use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::domain::metrics::CodeMetrics;
use crate::ports::parser::CodeParser;

pub use generic::{
    GenericParser, cpp_parser, csharp_parser, java_parser, kotlin_parser, php_parser, rust_parser,
};
pub use go::GoFullParser;
pub use python::PythonParser;
pub use ruby::RubyParser;
pub use typescript::TypeScriptParser;

// ===========================================================================
// Global regex cache
// ===========================================================================

/// Global cache for compiled regexes used by `count_keyword` and helper functions.
/// Avoids recompiling the same patterns on every call.
static REGEX_CACHE: OnceLock<Mutex<HashMap<&'static str, &'static Regex>>> = OnceLock::new();

/// Return a cached `&'static Regex` for a static pattern, compiling it on first use.
/// Compiled regexes are leaked and live for the process lifetime. The set of
/// distinct patterns is small and bounded, so this is an acceptable trade-off.
pub(crate) fn cached_regex(pattern: &'static str) -> &'static Regex {
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
pub(crate) fn cached_regex_owned(pattern: &str) -> Regex {
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

pub(crate) fn remove_line_comments<'a>(code: &'a str, prefix: &str) -> std::borrow::Cow<'a, str> {
    let re = cached_regex_owned(&format!(r"(?m){prefix}.*$"));
    re.replace_all(code, "")
}

pub(crate) fn remove_block_comments(code: &str) -> std::borrow::Cow<'_, str> {
    cached_regex(r"/\*.*?\*/").replace_all(code, "")
}

/// Strip Ruby `=begin`/`=end` block comments.
pub(crate) fn remove_ruby_block_comments(code: &str) -> std::borrow::Cow<'_, str> {
    cached_regex(r"(?m)^=begin\b.*?^=end\b").replace_all(code, "")
}

/// Strip Python/Ruby `#` line comments.
pub(crate) fn remove_hash_comments(code: &str) -> std::borrow::Cow<'_, str> {
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

pub(crate) fn strip_python_docstrings(code: &str) -> std::borrow::Cow<'_, str> {
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

// ===========================================================================
// Factory
// ===========================================================================

/// Return the appropriate parser for the given language name.
///
/// Matches the Python `get_parser()` factory: case-insensitive,
/// supports aliases like `"js"` -> TypeScriptParser, `"cpp"` -> CppParser.
pub fn get_parser(language: &str) -> Result<Box<dyn CodeParser>, String> {
    match language.to_ascii_lowercase().as_str() {
        "python" => Ok(Box::new(
            crate::adapters::python_ast_parser::PythonAstParser::new(),
        )),
        "java" => Ok(Box::new(java_parser())),
        "go" => Ok(Box::new(GoFullParser::new())),
        "rust" => Ok(Box::new(rust_parser())),
        "typescript" | "javascript" | "js" | "ts" => Ok(Box::new(TypeScriptParser::new())),
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
        assert!(
            cc >= 5,
            "Java CC should count do + try + catch + if, got {cc}"
        );
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
        assert!(
            cc >= 5,
            "C++ CC should count do + try + catch + if, got {cc}"
        );
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
        assert!(
            cc >= 5,
            "C# CC should count foreach + from + where + select + if, got {cc}"
        );
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
        assert!(
            cc >= 5,
            "PHP CC should count if + elseif + foreach + do, got {cc}"
        );
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
        assert!(
            cc >= 5,
            "Kotlin CC should count when + is + is + if, got {cc}"
        );
    }

    #[test]
    fn cc_rust_counts_loop_and_match_arms() {
        let code =
            "fn foo() { loop { x += 1; } match x { 1 => true, 2 => false, _ => true } if (a) { } }";
        let cc = calculate_cc_rust(code);
        assert!(
            cc >= 6,
            "Rust CC should count loop + match + 3 arms + if, got {cc}"
        );
    }

    #[test]
    fn cc_ruby_counts_when() {
        let code = "def foo(x)\n  case x\n  when 'a'\n    1\n  when 'b'\n    2\n  end\nend";
        let cc = ruby::calculate_cc_ruby(code);
        assert!(cc >= 3, "Ruby CC should count case + 2 when, got {cc}");
    }

    // --- Language-specific local var counting tests --------------------------

    #[test]
    fn local_vars_cpp_counts_typed_declarations() {
        let code = "void foo() { int x = 1; double y = 2.0; auto z = 3; bool flag = true; }";
        let count = count_local_vars_cpp(code);
        assert!(
            count >= 4,
            "C++ local vars should count int/double/auto/bool, got {count}"
        );
    }

    #[test]
    fn local_vars_csharp_counts_typed_and_var() {
        let code = "void Foo() { int x = 1; string y = \"hi\"; var z = 3; bool flag = true; }";
        let count = count_local_vars_csharp(code);
        assert!(
            count >= 4,
            "C# local vars should count int/string/var/bool, got {count}"
        );
    }

    #[test]
    fn local_vars_php_counts_dollar_vars() {
        let code = "function foo() { $x = 1; $y = 2; $z = $x + $y; }";
        let count = count_local_vars_php(code);
        assert!(
            count >= 3,
            "PHP local vars should count $x/$y/$z, got {count}"
        );
    }

    #[test]
    fn local_vars_kotlin_counts_val_and_var() {
        let code = "fun foo() { val x = 1; var y = 2; val z = x + y; }";
        let count = count_local_vars_kotlin(code);
        assert!(
            count >= 3,
            "Kotlin local vars should count val/var declarations, got {count}"
        );
    }

    // --- TypeScript arrow function tests -----------------------------------

    #[test]
    fn typescript_arrow_function_block_body() {
        let code = r#"
const myFunc = (a: number, b: number, c: number, d: number, e: number, f: number) => {
    let result = a + b;
    let doubled = result * 2;
    let tripled = result * 3;
    let quad = result * 4;
    let penta = result * 5;
    let hex = result * 6;
    let hept = result * 7;
    let oct = result * 8;
    let non = result * 9;
    let dec = result * 10;
    let undec = result * 11;
    let duodec = result * 12;
    return doubled + tripled + quad + penta + hex + hept + oct + non + dec + undec + duodec;
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
const fetchData = async (url: string, method: string, headers: string, body: string, timeout: number, retries: number) => {
    const response = await fetch(url);
    const data = await response.json();
    const status = response.status;
    const ok = response.ok;
    const redirected = response.redirected;
    const type = response.type;
    const url2 = response.url;
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
        // Expression-body arrows produce no smells (single-line, no braces).
        // Parsing them is still valid — just no detections.
        assert!(
            results.is_empty(),
            "expression-body arrows should produce no smells, got: {results:?}"
        );
    }

    #[test]
    fn typescript_exported_arrow_function() {
        let code = r#"
export const handler = (req: Request, res: Response, next: Next, ctx: Context, opts: Options, cfg: Config) => {
    const body = req.body;
    const result = process(body);
    const extra1 = result + 1;
    const extra2 = extra1 + 2;
    const extra3 = extra2 + 3;
    const extra4 = extra3 + 4;
    const extra5 = extra4 + 5;
    const extra6 = extra5 + 6;
    const extra7 = extra6 + 7;
    const extra8 = extra7 + 8;
    const extra9 = extra8 + 9;
    const extra10 = extra9 + 10;
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
pub unsafe fn dangerous(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    let mut result = a;
    if a > 0 { result += 1; }
    if a > 10 { result += 2; }
    if a > 100 { result += 3; }
    if a > 1000 { result += 4; }
    if b > 0 && c > 0 { result += 10; }
    if d > 0 || e > 0 { result += 20; }
    if f > 0 { result += 30; }
    if a > 0 && b > 0 { result += 40; }
    if c > 0 && d > 0 { result += 50; }
    if e > 0 && f > 0 { result += 60; }
    if a > 10 && c > 10 { result += 70; }
    if b > 10 && d > 10 { result += 80; }
    if e > 10 && f > 10 { result += 90; }
    if a > 100 { result += 100; }
    if b > 100 { result += 200; }
    if c > 100 { result += 300; }
    if d > 100 { result += 400; }
    if e > 100 { result += 500; }
    if f > 100 { result += 600; }
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
const fn factorial(n: u64, m: u64, k: u64, p: u64, q: u64, r: u64) -> u64 {
    let mut result = 1u64;
    let mut i = 2u64;
    while i <= n {
        result *= i;
        i += 1;
    }
    if m > 0 { result += m; }
    if k > 0 { result += k; }
    if p > 0 { result += p; }
    if q > 0 { result += q; }
    if r > 0 { result += r; }
    if m > 0 && k > 0 { result += 10; }
    if p > 0 && q > 0 { result += 20; }
    if r > 0 && n > 0 { result += 30; }
    if m > 10 { result += 40; }
    if k > 10 { result += 50; }
    if p > 10 { result += 60; }
    if q > 10 { result += 70; }
    if r > 10 { result += 80; }
    if n > 10 { result += 90; }
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
pub unsafe async fn complex(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    let x = a + b;
    let y = x * 2;
    if x > 0 { y + 1 } else { y - 1 }
    let z = c + d;
    let w = e + f;
    if z > 0 { w + 1 } else { w - 1 }
    if a > 0 && b > 0 { x + y } else { x - y }
    if c > 0 && d > 0 { z + w } else { z - w }
    if e > 0 && f > 0 { x + z } else { x - z }
    if a > 0 || c > 0 { y + w } else { y - w }
    if b > 0 || d > 0 { x + z } else { x - z }
    if e > 0 || f > 0 { y + w } else { y - w }
    if a > 10 { x + 100 } else { x - 100 }
    if b > 10 { y + 200 } else { y - 200 }
    if c > 10 { z + 300 } else { z - 300 }
    if d > 10 { w + 400 } else { w - 400 }
    if e > 10 { x + 500 } else { x - 500 }
    if f > 10 { y + 600 } else { y - 600 }
    x + y + z + w
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
