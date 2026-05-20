//! Regex-based language parsers for smell detection.
//!
//! Each parser finds function/class definitions, extracts bodies via brace
//! matching, computes `CodeMetrics`, and delegates to `detect_all`.

mod generic;
mod go;
mod helpers;
mod python;
mod ruby;
#[cfg(test)]
mod tests;
mod typescript;

use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use crate::ports::parser::CodeParser;

pub use generic::{
    GenericParser, cpp_parser, csharp_parser, java_parser, kotlin_parser, php_parser, rust_parser,
};
pub use go::GoFullParser;
pub(crate) use helpers::*;
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

/// Maximum number of entries in the owned regex cache before eviction.
const OWNED_CACHE_CAPACITY: usize = 256;

/// Return a cached `Regex` for a dynamically constructed pattern string.
/// The returned Regex is cloned out of the cache to avoid lifetime issues
/// with non-static strings.
///
/// When the cache exceeds [`OWNED_CACHE_CAPACITY`] entries, the oldest half
/// (by insertion order) is evicted to prevent unbounded memory growth.
pub(crate) fn cached_regex_owned(pattern: &str) -> Regex {
    struct OwnedCache {
        map: HashMap<String, Regex>,
        order: VecDeque<String>,
    }

    static OWNED_CACHE: OnceLock<Mutex<OwnedCache>> = OnceLock::new();
    let cache = OWNED_CACHE.get_or_init(|| {
        Mutex::new(OwnedCache {
            map: HashMap::new(),
            order: VecDeque::new(),
        })
    });

    // Fast path: check cache under lock, return immediately on hit.
    let key = pattern.to_string();
    {
        let guard = cache.lock().unwrap();
        if let Some(re) = guard.map.get(&key) {
            return re.clone();
        }
    }

    // Compile outside the lock to avoid blocking other threads.
    let compiled = Regex::new(pattern).unwrap();

    let mut guard = cache.lock().unwrap();

    // Double-check: another thread may have inserted between our first unlock and now.
    if let Some(re) = guard.map.get(&key) {
        return re.clone();
    }

    // Evict oldest half if at capacity. Insertion order is tracked in `order`.
    if guard.map.len() >= OWNED_CACHE_CAPACITY {
        let evict_count = OWNED_CACHE_CAPACITY / 2;
        for _ in 0..evict_count {
            if let Some(old_key) = guard.order.pop_front() {
                guard.map.remove(&old_key);
            }
        }
    }

    guard.order.push_back(key.clone());
    guard.map.insert(key, compiled);
    // Return a clone of the just-inserted Regex.
    guard
        .order
        .back()
        .and_then(|k| guard.map.get(k))
        .unwrap()
        .clone()
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
