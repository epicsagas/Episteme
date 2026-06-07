//! Generic brace-based parser with configurable per-language settings.

use regex::Regex;
use std::sync::OnceLock;

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    build_func_metrics_full, calculate_cc, calculate_cc_cpp, calculate_cc_csharp,
    calculate_cc_java, calculate_cc_kotlin, calculate_cc_php, calculate_cc_rust,
    count_block_comment_lines, count_delegation_methods, count_line_comment_lines, count_loc,
    count_local_vars, count_local_vars_cpp, count_local_vars_csharp, count_local_vars_kotlin,
    count_local_vars_php, count_overrides, count_primitive_params_csharp,
    count_primitive_params_go, count_primitive_params_java, count_primitive_params_kotlin,
    count_primitive_params_none, count_primitive_params_php, count_primitive_params_rust,
    find_matching_brace, line_number, remove_block_comments, remove_hash_comments,
    remove_line_comments,
};

/// Configuration for a brace-based language parser.
pub(crate) struct ParserConfig {
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
    primitive_fn: fn(&str) -> usize,
    /// Comment-line prefix used for counting (e.g. "//", "#"). Empty = no line comments.
    comment_prefix: &'static str,
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
    pub(crate) fn new(config: ParserConfig) -> Self {
        Self {
            config,
            func_re: OnceLock::new(),
            class_re: OnceLock::new(),
            class_method_re: OnceLock::new(),
            class_field_re: OnceLock::new(),
        }
    }

    pub(crate) fn get_func_re(&self) -> &Regex {
        self.func_re
            .get_or_init(|| Regex::new(self.config.func_regex).unwrap())
    }

    fn get_class_re(&self) -> Option<&Regex> {
        self.config
            .class_regex
            .map(|pat| self.class_re.get_or_init(|| Regex::new(pat).unwrap()))
    }

    fn get_class_method_re(&self) -> Option<&Regex> {
        self.config.class_method_regex.map(|pat| {
            self.class_method_re
                .get_or_init(|| Regex::new(pat).unwrap())
        })
    }

    fn get_class_field_re(&self) -> Option<&Regex> {
        self.config
            .class_field_regex
            .map(|pat| self.class_field_re.get_or_init(|| Regex::new(pat).unwrap()))
    }

    /// Strip comments according to config.
    pub(crate) fn strip_comments<'a>(&self, code: &'a str) -> std::borrow::Cow<'a, str> {
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
            primitive_fn: count_primitive_params_none,
            comment_prefix: "",
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
        let primitive_fn = self.config.primitive_fn;
        let comment_prefix = self.config.comment_prefix;
        let has_block_comments = self.config.strip_block_comments;
        let skip = self.config.skip_names;

        // Pre-collect raw function bodies for comment counting.
        let raw_func_comments: std::collections::HashMap<String, usize> =
            collect_raw_func_comment_counts(
                func_re,
                code,
                skip,
                comment_prefix,
                has_block_comments,
            );

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
            let comment_count = *raw_func_comments.get(name).unwrap_or(&0);
            let metrics =
                build_func_metrics_full(body, sig, cc_fn, vars_fn, primitive_fn, comment_count);

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
                let delegation_methods = count_delegation_methods(body);
                let override_count = count_overrides(body);

                let metrics = CodeMetrics {
                    loc: count_loc(body),
                    method_count,
                    field_count,
                    delegation_methods,
                    override_count,
                    item_type: ItemType::Class,
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
        primitive_fn: count_primitive_params_java,
        comment_prefix: "//",
        skip_names: &[],
    })
}

/// Basic Go parser (brace-based).
///
/// This parser cannot detect Go struct receiver methods. Use [`super::GoFullParser`]
/// instead for full Go support including struct method counting.
/// Marked `pub(crate)` because external callers should use [`super::GoFullParser`].
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
        primitive_fn: count_primitive_params_go,
        comment_prefix: "//",
        skip_names: &[],
    })
}

/// Rust parser (brace-based).
pub fn rust_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "rust",
        extensions: &["rs"],
        func_regex: r"(?m)(?:pub\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+(\w+)\s*[\(<]",
        class_regex: Some(r"(?m)struct\s+(\w+)"),
        class_method_regex: Some(r"(?m)(?:pub\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+\w+"),
        class_field_regex: Some(r"(?m)\s+\w+\s*:\s*[A-Za-z]"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_rust,
        count_local_vars_fn: count_local_vars,
        primitive_fn: count_primitive_params_rust,
        comment_prefix: "//",
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
        class_method_regex: Some(
            r"(?m)(?:(?:public|private|protected|virtual|static)\s+)*[\w:*&<>,\s]+\s+\w+\s*\(",
        ),
        class_field_regex: Some(r"(?m)(?:public|private|protected)\s+[\w:*&<>,\s]+\s+\w+\s*;"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_cpp,
        count_local_vars_fn: count_local_vars_cpp,
        primitive_fn: count_primitive_params_none,
        comment_prefix: "//",
        skip_names: &[
            "if", "for", "while", "switch", "catch", "return", "class", "struct",
        ],
    })
}

/// C# parser (brace-based).
pub fn csharp_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "csharp",
        extensions: &["cs"],
        func_regex: r"(?m)(?:(?:public|private|protected|internal|static|virtual|override|async|abstract)\s+)+[\w<>\[\]?]+\s+(\w+)\s*\(",
        class_regex: Some(
            r"(?m)(?:(?:public|private|protected|internal|static|abstract|sealed)\s+)*(?:class|struct|record)\s+(\w+)",
        ),
        class_method_regex: Some(
            r"(?m)(?:public|private|protected|internal)\s+[\w<>\[\]?]+\s+\w+\s*\(",
        ),
        class_field_regex: Some(
            r"(?m)(?:public|private|protected|internal|readonly)\s+[\w<>\[\]?]+\s+\w+\s*[;=]",
        ),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_csharp,
        count_local_vars_fn: count_local_vars_csharp,
        primitive_fn: count_primitive_params_csharp,
        comment_prefix: "//",
        skip_names: &["if", "for", "while", "switch", "catch", "using", "lock"],
    })
}

/// Kotlin parser (brace-based).
pub fn kotlin_parser() -> GenericParser {
    GenericParser::new(ParserConfig {
        name: "kotlin",
        extensions: &["kt", "kts"],
        func_regex: r"(?m)(?:(?:public|private|protected|internal|suspend|inline|open|override|abstract)\s+)*fun\s+(?:<[^>]*>\s*)?(\w+)\s*\(",
        class_regex: Some(
            r"(?m)(?:(?:public|private|protected|internal|open|abstract|sealed|data|inner)\s+)*class\s+(\w+)",
        ),
        class_method_regex: Some(r"(?m)fun\s+(?:<[^>]*>\s*)?\w+\s*\("),
        class_field_regex: Some(r"(?:val|var)\s+\w+"),
        strip_line_comment: "//",
        strip_block_comments: true,
        strip_hash_comments: false,
        cc_fn: calculate_cc_kotlin,
        count_local_vars_fn: count_local_vars_kotlin,
        primitive_fn: count_primitive_params_kotlin,
        comment_prefix: "//",
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
        primitive_fn: count_primitive_params_php,
        comment_prefix: "//",
        skip_names: &[],
    })
}

/// Scan raw (unstripped) code for functions, count comment lines in each raw body.
/// Returns a map from function name to comment line count (first occurrence per name).
fn collect_raw_func_comment_counts(
    func_re: &Regex,
    raw_code: &str,
    skip: &[&str],
    comment_prefix: &str,
    has_block_comments: bool,
) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for cap in func_re.captures_iter(raw_code) {
        let name = cap[1].to_string();
        if skip.contains(&name.as_str()) || map.contains_key(&name) {
            continue;
        }
        let full = cap.get(0).unwrap();
        let start = full.start();
        let Some(off) = raw_code[start..].find('{') else {
            continue;
        };
        let brace_pos = start + off;
        let Some(end_pos) = find_matching_brace(raw_code, brace_pos) else {
            continue;
        };
        let raw_body = &raw_code[start..=end_pos];
        let mut count = 0;
        if !comment_prefix.is_empty() {
            count += count_line_comment_lines(raw_body, comment_prefix);
        }
        if has_block_comments {
            count += count_block_comment_lines(raw_body);
        }
        map.insert(name, count);
    }
    map
}
