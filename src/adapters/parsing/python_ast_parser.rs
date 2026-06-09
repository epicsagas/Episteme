use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, ItemType, SmellDetection};
use crate::ports::parser::CodeParser;
use rustpython_parser::Parse;
use rustpython_parser::ast::{self, Ranged};

pub struct PythonAstParser;

impl Default for PythonAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonAstParser {
    pub fn new() -> Self {
        Self
    }
}

impl CodeParser for PythonAstParser {
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection> {
        let mut detections = Vec::new();
        let Ok(suite) = ast::Suite::parse(code, file_name) else {
            return detections;
        };

        for stmt in &suite {
            visit_stmt(stmt, code, file_name, &mut detections);
        }
        detections
    }

    fn supported_extensions(&self) -> &[&str] {
        &["py"]
    }
}

fn visit_stmt(stmt: &ast::Stmt, code: &str, file_name: &str, out: &mut Vec<SmellDetection>) {
    match stmt {
        ast::Stmt::FunctionDef(f) => {
            out.extend(detect_function_metrics(
                f.name.as_ref(),
                &f.args,
                &f.body,
                f.range(),
                code,
                file_name,
            ));
            for inner in &f.body {
                visit_stmt(inner, code, file_name, out);
            }
        }
        ast::Stmt::AsyncFunctionDef(f) => {
            out.extend(detect_function_metrics(
                f.name.as_ref(),
                &f.args,
                &f.body,
                f.range(),
                code,
                file_name,
            ));
            for inner in &f.body {
                visit_stmt(inner, code, file_name, out);
            }
        }
        ast::Stmt::ClassDef(c) => {
            out.extend(detect_class_metrics(c, code, file_name));
            for inner in &c.body {
                visit_stmt(inner, code, file_name, out);
            }
        }
        _ => {}
    }
}

fn detect_function_metrics(
    name: &str,
    args: &ast::Arguments,
    body: &[ast::Stmt],
    range: rustpython_parser::text_size::TextRange,
    code: &str,
    file_name: &str,
) -> Vec<SmellDetection> {
    let parameter_count = args.posonlyargs.len() + args.args.len() + args.kwonlyargs.len();
    let primitive_params = count_primitive_params(args);
    let loc = count_code_lines_in_range(code, range);
    let cyclomatic_complexity = 1 + count_decisions(body);
    let nesting_depth = max_nesting(body, 0);
    let local_variables = count_local_assignments(body);
    let return_statements = count_returns(body);
    let external_calls = count_external_calls(body);
    let branch_count = count_branches(body);
    let method_call_chains = count_call_chains(body);
    let doc_comment_lines = count_docstring_lines(body, code);
    let comment_count = count_comment_lines_in_range(code, range) + doc_comment_lines;

    let metrics = CodeMetrics {
        loc,
        cyclomatic_complexity,
        nesting_depth,
        parameter_count,
        local_variables,
        return_statements,
        external_calls,
        primitive_params,
        branch_count,
        method_call_chains,
        comment_count,
        doc_comment_count: doc_comment_lines,
        ..Default::default()
    };
    let line = line_number_at_offset(code, range.start().to_usize());
    let location = format!("{file_name}:{line}");
    detect_all(&metrics, &location, name)
}

fn detect_class_metrics(c: &ast::StmtClassDef, code: &str, file_name: &str) -> Vec<SmellDetection> {
    let method_count = c
        .body
        .iter()
        .filter(|s| {
            matches!(
                s,
                ast::Stmt::FunctionDef(_) | ast::Stmt::AsyncFunctionDef(_)
            )
        })
        .count();
    // Count fields: assignments inside method bodies (self.x = ...)
    // plus class-level annotated assignments (dataclass fields: name: str)
    let field_count = c
        .body
        .iter()
        .filter_map(|s| match s {
            ast::Stmt::FunctionDef(f) => Some(&f.body),
            ast::Stmt::AsyncFunctionDef(f) => Some(&f.body),
            _ => None,
        })
        .flat_map(|body| body.iter())
        .filter(|s| matches!(s, ast::Stmt::Assign(_) | ast::Stmt::AnnAssign(_)))
        .count()
        + c.body
            .iter()
            .filter(|s| matches!(s, ast::Stmt::AnnAssign(_) | ast::Stmt::Assign(_)))
            .count();
    // Aggregate cyclomatic complexity across all methods in the class.
    let cyclomatic_complexity: usize = c
        .body
        .iter()
        .filter_map(|s| match s {
            ast::Stmt::FunctionDef(f) => Some(1 + count_decisions(&f.body)),
            ast::Stmt::AsyncFunctionDef(f) => Some(1 + count_decisions(&f.body)),
            _ => None,
        })
        .sum();
    // For subclasses (classes with bases), non-dunder methods are likely overrides
    let override_count = if !c.bases.is_empty() {
        c.body
            .iter()
            .filter(|s| {
                let name = match s {
                    ast::Stmt::FunctionDef(f) => f.name.as_ref(),
                    ast::Stmt::AsyncFunctionDef(f) => f.name.as_ref(),
                    _ => "",
                };
                !name.is_empty() && !name.starts_with("__")
            })
            .count()
    } else {
        0
    };
    // Count external calls across all methods in the class
    let external_calls: usize = c
        .body
        .iter()
        .filter_map(|s| match s {
            ast::Stmt::FunctionDef(f) => Some(&f.body),
            ast::Stmt::AsyncFunctionDef(f) => Some(&f.body),
            _ => None,
        })
        .map(|body| count_external_calls(body))
        .sum();
    // Count delegation methods: methods whose body is a single return of
    // a self.xxx.method(...) call (pure forwarding).
    let delegation_methods = c
        .body
        .iter()
        .filter(|s| {
            let body = match s {
                ast::Stmt::FunctionDef(f) => &f.body,
                ast::Stmt::AsyncFunctionDef(f) => &f.body,
                _ => return false,
            };
            is_delegation_body(body)
        })
        .count();
    let class_doc_lines = count_docstring_lines(&c.body, code);
    let metrics = CodeMetrics {
        loc: count_code_lines_in_range(code, c.range()),
        cyclomatic_complexity,
        method_count,
        field_count,
        external_calls,
        delegation_methods,
        override_count,
        comment_count: count_comment_lines_in_range(code, c.range()) + class_doc_lines,
        doc_comment_count: class_doc_lines,
        item_type: ItemType::Class,
        ..Default::default()
    };
    let line = line_number_at_offset(code, c.range().start().to_usize());
    let location = format!("{file_name}:{line}");
    detect_all(&metrics, &location, c.name.as_ref())
}

fn count_primitive_params(args: &ast::Arguments) -> usize {
    fn is_primitive(expr: &ast::Expr) -> bool {
        match expr {
            ast::Expr::Name(n) => matches!(
                n.id.as_str(),
                "int" | "float" | "bool" | "str" | "bytes" | "list" | "dict" | "set" | "tuple"
            ),
            _ => false,
        }
    }
    args.posonlyargs
        .iter()
        .chain(args.args.iter())
        .chain(args.kwonlyargs.iter())
        .filter(|a| {
            a.def
                .annotation
                .as_ref()
                .map(|expr| is_primitive(expr))
                .unwrap_or(true)
        })
        .count()
}

fn count_decisions(body: &[ast::Stmt]) -> usize {
    let mut c = 0;
    for stmt in body {
        match stmt {
            ast::Stmt::If(s) => {
                c += 1 + count_decisions(&s.body) + count_decisions(&s.orelse);
            }
            ast::Stmt::For(s) => {
                c += 1 + count_decisions(&s.body) + count_decisions(&s.orelse);
            }
            ast::Stmt::AsyncFor(s) => {
                c += 1 + count_decisions(&s.body) + count_decisions(&s.orelse);
            }
            ast::Stmt::While(s) => {
                c += 1 + count_decisions(&s.body) + count_decisions(&s.orelse);
            }
            ast::Stmt::Try(s) => {
                c += s.handlers.len()
                    + count_decisions(&s.body)
                    + count_decisions(&s.orelse)
                    + count_decisions(&s.finalbody);
            }
            ast::Stmt::Match(s) => {
                c += s.cases.len();
            }
            _ => {}
        }
    }
    c
}

fn max_nesting(body: &[ast::Stmt], depth: usize) -> usize {
    let mut max_depth = depth;
    for stmt in body {
        let nested = match stmt {
            ast::Stmt::If(s) => {
                max_nesting(&s.body, depth + 1).max(max_nesting(&s.orelse, depth + 1))
            }
            ast::Stmt::For(s) => {
                max_nesting(&s.body, depth + 1).max(max_nesting(&s.orelse, depth + 1))
            }
            ast::Stmt::AsyncFor(s) => {
                max_nesting(&s.body, depth + 1).max(max_nesting(&s.orelse, depth + 1))
            }
            ast::Stmt::While(s) => {
                max_nesting(&s.body, depth + 1).max(max_nesting(&s.orelse, depth + 1))
            }
            ast::Stmt::Try(s) => max_nesting(&s.body, depth + 1)
                .max(max_nesting(&s.orelse, depth + 1))
                .max(max_nesting(&s.finalbody, depth + 1)),
            _ => depth,
        };
        max_depth = max_depth.max(nested);
    }
    max_depth
}

fn count_local_assignments(body: &[ast::Stmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            ast::Stmt::Assign(_) | ast::Stmt::AnnAssign(_) | ast::Stmt::AugAssign(_) => 1,
            ast::Stmt::If(s) => {
                count_local_assignments(&s.body) + count_local_assignments(&s.orelse)
            }
            ast::Stmt::For(s) => {
                count_local_assignments(&s.body) + count_local_assignments(&s.orelse)
            }
            ast::Stmt::AsyncFor(s) => {
                count_local_assignments(&s.body) + count_local_assignments(&s.orelse)
            }
            ast::Stmt::While(s) => {
                count_local_assignments(&s.body) + count_local_assignments(&s.orelse)
            }
            ast::Stmt::Try(s) => {
                count_local_assignments(&s.body)
                    + count_local_assignments(&s.orelse)
                    + count_local_assignments(&s.finalbody)
            }
            _ => 0,
        })
        .sum()
}

fn count_returns(body: &[ast::Stmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            ast::Stmt::Return(_) => 1,
            ast::Stmt::If(s) => count_returns(&s.body) + count_returns(&s.orelse),
            ast::Stmt::For(s) => count_returns(&s.body) + count_returns(&s.orelse),
            ast::Stmt::AsyncFor(s) => count_returns(&s.body) + count_returns(&s.orelse),
            ast::Stmt::While(s) => count_returns(&s.body) + count_returns(&s.orelse),
            ast::Stmt::Try(s) => {
                count_returns(&s.body) + count_returns(&s.orelse) + count_returns(&s.finalbody)
            }
            _ => 0,
        })
        .sum()
}

fn count_external_calls(body: &[ast::Stmt]) -> usize {
    fn count_expr(expr: &ast::Expr) -> usize {
        match expr {
            ast::Expr::Call(c) => match c.func.as_ref() {
                ast::Expr::Attribute(_) => 1 + c.args.iter().map(count_expr).sum::<usize>(),
                f => count_expr(f) + c.args.iter().map(count_expr).sum::<usize>(),
            },
            ast::Expr::BoolOp(b) => b.values.iter().map(count_expr).sum(),
            ast::Expr::BinOp(b) => count_expr(&b.left) + count_expr(&b.right),
            ast::Expr::UnaryOp(u) => count_expr(&u.operand),
            ast::Expr::Compare(c) => {
                count_expr(&c.left) + c.comparators.iter().map(count_expr).sum::<usize>()
            }
            _ => 0,
        }
    }
    body.iter()
        .map(|stmt| match stmt {
            ast::Stmt::Expr(e) => count_expr(&e.value),
            ast::Stmt::Assign(a) => count_expr(&a.value),
            ast::Stmt::AnnAssign(a) => a.value.as_ref().map(|v| count_expr(v)).unwrap_or(0),
            ast::Stmt::If(s) => count_external_calls(&s.body) + count_external_calls(&s.orelse),
            ast::Stmt::For(s) => count_external_calls(&s.body) + count_external_calls(&s.orelse),
            ast::Stmt::AsyncFor(s) => {
                count_external_calls(&s.body) + count_external_calls(&s.orelse)
            }
            ast::Stmt::While(s) => count_external_calls(&s.body) + count_external_calls(&s.orelse),
            ast::Stmt::Try(s) => {
                count_external_calls(&s.body)
                    + count_external_calls(&s.orelse)
                    + count_external_calls(&s.finalbody)
            }
            _ => 0,
        })
        .sum()
}

fn count_branches(body: &[ast::Stmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            ast::Stmt::If(s) => 1 + count_branches(&s.body) + count_branches(&s.orelse),
            ast::Stmt::Match(s) => s.cases.len(),
            _ => 0,
        })
        .sum()
}

/// Check if a method body is a simple delegation: single statement that
/// returns or directly calls `self.xxx.method(...)`.
fn is_delegation_body(body: &[ast::Stmt]) -> bool {
    // Must have exactly one statement
    if body.len() != 1 {
        return false;
    }
    let expr = match &body[0] {
        ast::Stmt::Return(r) => r.value.as_deref(),
        ast::Stmt::Expr(e) => Some(&*e.value),
        _ => None,
    };
    let Some(expr) = expr else { return false };
    // Check: self.service.method(...) pattern
    is_self_attribute_call(expr)
}

/// Check if an expression is `self.xxx.yyy(...)` (attribute call on self.xxx).
fn is_self_attribute_call(expr: &ast::Expr) -> bool {
    let ast::Expr::Call(call) = expr else {
        return false;
    };
    // func should be Attribute(Attribute(Name("self"), ...), ...)
    let ast::Expr::Attribute(attr) = call.func.as_ref() else {
        return false;
    };
    let ast::Expr::Attribute(inner) = attr.value.as_ref() else {
        // Also accept self.method() — single-level forwarding
        let ast::Expr::Name(name) = attr.value.as_ref() else {
            return false;
        };
        return name.id.as_str() == "self";
    };
    let ast::Expr::Name(name) = inner.value.as_ref() else {
        return false;
    };
    name.id.as_str() == "self"
}

fn count_call_chains(body: &[ast::Stmt]) -> usize {
    fn chain_len(expr: &ast::Expr) -> usize {
        match expr {
            ast::Expr::Call(c) => chain_len(&c.func),
            ast::Expr::Attribute(a) => 1 + chain_len(&a.value),
            _ => 0,
        }
    }
    body.iter()
        .map(|stmt| match stmt {
            ast::Stmt::Expr(e) => chain_len(&e.value),
            ast::Stmt::Assign(a) => chain_len(&a.value),
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

fn line_number_at_offset(code: &str, offset: usize) -> usize {
    code[..offset.min(code.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1
}

/// Count actual code lines (non-blank, non-comment, non-docstring) within a range.
/// Tracks multi-line docstrings (triple-quoted strings) so content lines between
/// opening and closing delimiters are excluded from the count.
fn count_code_lines_in_range(code: &str, range: rustpython_parser::text_size::TextRange) -> usize {
    let start = range.start().to_usize().min(code.len());
    let end = range.end().to_usize().min(code.len());
    let slice = &code[start..end];

    let mut in_docstring = false;
    let mut count: usize = 0;

    for line in slice.lines() {
        let t = line.trim();

        if in_docstring {
            if t.ends_with("\"\"\"") || t.ends_with("'''") {
                in_docstring = false;
            }
            continue;
        }

        // Opening triple-quote (docstring)
        if t.starts_with("\"\"\"") {
            if !t[3..].contains("\"\"\"") {
                in_docstring = true;
            }
            continue;
        }
        if t.starts_with("'''") {
            if !t[3..].contains("'''") {
                in_docstring = true;
            }
            continue;
        }

        if !t.is_empty() && !t.starts_with('#') {
            count += 1;
        }
    }

    count.max(1)
}

/// Count comment lines (# lines) within a range.
fn count_comment_lines_in_range(
    code: &str,
    range: rustpython_parser::text_size::TextRange,
) -> usize {
    let start = range.start().to_usize().min(code.len());
    let end = range.end().to_usize().min(code.len());
    let slice = &code[start..end];
    slice
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with('#')
        })
        .count()
}

/// Count docstring lines in a function/class body.
/// A docstring is the first statement if it is a string constant expression.
fn count_docstring_lines(body: &[ast::Stmt], code: &str) -> usize {
    let Some(first) = body.first() else {
        return 0;
    };
    let ast::Stmt::Expr(expr_stmt) = first else {
        return 0;
    };
    // Check if the first expression is a string constant (docstring)
    match expr_stmt.value.as_ref() {
        ast::Expr::Constant(_) => {
            // Check if the constant is a string by inspecting the source text
            let start = first.range().start().to_usize().min(code.len());
            let end = first.range().end().to_usize().min(code.len());
            let src = code[start..end].trim();
            if src.starts_with('"') || src.starts_with('\'') {
                src.lines().count().max(1)
            } else {
                0
            }
        }
        _ => 0,
    }
}
