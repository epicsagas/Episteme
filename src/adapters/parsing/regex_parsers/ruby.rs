//! Ruby parser — end-block-based body extraction.

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    cached_regex, count_delegation_methods, count_external_calls, count_keyword,
    count_method_call_chains, count_overrides, count_params, count_python_loc, count_returns,
    line_number, remove_hash_comments, remove_ruby_block_comments,
};

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
                primitive_params: 0,
                branch_count: super::count_branches_python(body),
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
            let delegation_methods = count_delegation_methods(body);
            let override_count = count_overrides(body);

            let metrics = CodeMetrics {
                loc: count_python_loc(body),
                method_count,
                field_count,
                delegation_methods,
                override_count,
                item_type: ItemType::Class,
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

pub(crate) fn calculate_cc_ruby(body: &str) -> usize {
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
