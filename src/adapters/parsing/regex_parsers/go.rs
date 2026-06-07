//! Extended Go parser with struct receiver method counting across the full file.

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    GenericParser, build_func_metrics_full, cached_regex, cached_regex_owned, calculate_cc,
    count_block_comment_lines, count_delegation_methods, count_line_comment_lines, count_loc,
    count_local_vars, count_overrides, count_primitive_params_go, find_matching_brace, line_number,
};

/// Extended Go parser that counts struct receiver methods across the full file.
pub struct GoFullParser {
    inner: GenericParser,
}

impl GoFullParser {
    pub fn new() -> Self {
        Self {
            inner: super::generic::go_parser(),
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
            let raw_body = &code[start..=end_pos];
            let comment_count =
                count_line_comment_lines(raw_body, "//") + count_block_comment_lines(raw_body);
            let metrics = build_func_metrics_full(
                body,
                sig,
                calculate_cc,
                count_local_vars,
                count_primitive_params_go,
                comment_count,
            );

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

            let method_re = cached_regex_owned(&format!(r"(?m)func\s+\([^)]*\s+\*?{name}\)\s+\w+"));
            let method_count = method_re.find_iter(&cleaned).count();

            // Collect all receiver method bodies for delegation/override counting
            let mut method_bodies = String::new();
            for m in method_re.find_iter(&cleaned) {
                let m_start = m.start();
                if let Some(off) = cleaned[m_start..].find('{') {
                    let brace_pos = m_start + off;
                    if let Some(end) = find_matching_brace(&cleaned, brace_pos) {
                        method_bodies.push_str(&cleaned[brace_pos..=end]);
                    }
                }
            }

            let delegation_methods = count_delegation_methods(&method_bodies);
            let override_count = count_overrides(&method_bodies);

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

        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["go"]
    }
}
