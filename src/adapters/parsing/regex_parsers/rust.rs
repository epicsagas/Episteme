//! Extended Rust parser with impl method counting across the full file.

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    GenericParser, build_func_metrics_full, cached_regex, cached_regex_owned, calculate_cc_rust,
    count_block_comment_lines, count_line_comment_lines, count_loc, count_local_vars,
    count_primitive_params_rust, find_matching_brace, line_number,
};

/// Extended Rust parser that counts `impl` block methods for each struct.
pub struct RustFullParser {
    inner: GenericParser,
}

impl RustFullParser {
    pub fn new() -> Self {
        Self {
            inner: super::generic::rust_parser(),
        }
    }
}

impl Default for RustFullParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeParser for RustFullParser {
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
            let raw_body = &code[start..=end_pos];
            let comment_count =
                count_line_comment_lines(raw_body, "//") + count_block_comment_lines(raw_body);
            let metrics = build_func_metrics_full(
                body,
                sig,
                calculate_cc_rust,
                count_local_vars,
                count_primitive_params_rust,
                comment_count,
            );

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        // --- Structs (with impl method counting) ---
        let struct_re = cached_regex(r"(?m)struct\s+(\w+)\s*(?:\{|;|where)");
        for cap in struct_re.captures_iter(&cleaned) {
            let name = &cap[1];
            let full = cap.get(0).unwrap();
            let start = full.start();

            // Count fields: only if the struct has a brace body (not unit/tuple struct)
            let (loc, field_count) = if cleaned[start..].starts_with("struct") {
                if let Some(off) = cleaned[start..].find('{') {
                    let brace_pos = start + off;
                    if let Some(end_pos) = find_matching_brace(&cleaned, brace_pos) {
                        let body = &cleaned[start..=end_pos];
                        let fields = body
                            .lines()
                            .filter(|l| {
                                let t = l.trim();
                                !t.is_empty()
                                    && t != "{"
                                    && t != "}"
                                    && !t.starts_with("//")
                                    && !t.starts_with("/*")
                                    && !t.starts_with('#')
                            })
                            .count();
                        (count_loc(body), fields)
                    } else {
                        (1, 0)
                    }
                } else {
                    // Unit or tuple struct — no fields
                    (1, 0)
                }
            } else {
                (1, 0)
            };

            // Count methods in `impl Name { ... }` and `impl Trait for Name { ... }`
            let impl_re = cached_regex_owned(&format!(
                r"(?m)impl\s+(?:<[^>]*>\s*)?(?:\w+\s+for\s+)?{name}\s*(?:<[^>]*>\s*)?\{{"
            ));
            let method_re = cached_regex(r"(?m)(?:pub\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+\w+");
            let method_count: usize = impl_re
                .find_iter(&cleaned)
                .map(|m| {
                    let impl_start = m.start();
                    let brace_off = cleaned[impl_start..].find('{').unwrap_or(0);
                    let brace_pos = impl_start + brace_off;
                    match find_matching_brace(&cleaned, brace_pos) {
                        Some(end) => method_re.find_iter(&cleaned[brace_pos..=end]).count(),
                        None => 0,
                    }
                })
                .sum();

            let metrics = CodeMetrics {
                loc,
                method_count,
                field_count,
                item_type: ItemType::Class,
                ..Default::default()
            };

            let location = format!("{}:{}", file_name, line_number(&cleaned, start));
            detections.extend(detect_all(&metrics, &location, name));
        }

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
    }
}
