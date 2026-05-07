use crate::domain::detectors::detect_all;
use crate::domain::metrics::{CodeMetrics, SmellDetection};
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
    let loc = range_len_lines(code, range);
    let cyclomatic_complexity = 1 + count_decisions(body);
    let nesting_depth = max_nesting(body, 0);
    let local_variables = count_local_assignments(body);
    let return_statements = count_returns(body);
    let external_calls = count_external_calls(body);
    let branch_count = count_branches(body);
    let method_call_chains = count_call_chains(body);

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
        .count();
    let metrics = CodeMetrics {
        loc: range_len_lines(code, c.range()),
        method_count,
        field_count,
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

fn range_len_lines(code: &str, range: rustpython_parser::text_size::TextRange) -> usize {
    let start = range.start().to_usize().min(code.len());
    let end = range.end().to_usize().min(code.len());
    let slice = &code[start..end];
    slice
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .max(1)
}
