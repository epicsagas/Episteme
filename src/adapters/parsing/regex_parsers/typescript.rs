//! TypeScript parser — handles arrow functions and standard function declarations.

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    build_func_metrics, cached_regex, calculate_cc, count_keyword, count_loc, find_matching_brace,
    line_number, remove_block_comments, remove_line_comments,
};

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
        let fn_re = cached_regex(r"(?m)(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\(");
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
            let field_count =
                count_keyword(body, r"(?:public|private|protected|readonly)\s+\w+\s*[:=]");

            let metrics = CodeMetrics {
                loc: count_loc(body),
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
