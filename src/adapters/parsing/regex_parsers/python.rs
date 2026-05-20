//! Python parser — indentation-based body extraction.

use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, SmellDetection};
use crate::ports::parser::CodeParser;

use super::{
    cached_regex, calculate_cc_python, calculate_nesting_python, count_branches_python,
    count_external_calls, count_keyword, count_method_call_chains, count_params,
    count_primitive_params_python, count_python_loc, count_returns, line_number,
    remove_hash_comments, strip_python_docstrings,
};

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
