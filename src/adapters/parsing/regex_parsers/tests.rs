//! Integration and unit tests for regex-based language parsers.

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
const myFunc = (a: number, b: number) => {
    let result = a + b;
    let doubled = result * 2;
    return doubled;
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
const fetchData = async (url: string) => {
    const response = await fetch(url);
    const data = await response.json();
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
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"add"),
        "should detect expression arrow 'add', got: {names:?}"
    );
    assert!(
        names.contains(&"multiply"),
        "should detect expression arrow 'multiply', got: {names:?}"
    );
}

#[test]
fn typescript_exported_arrow_function() {
    let code = r#"
export const handler = (req: Request) => {
    const body = req.body;
    const result = process(body);
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
pub unsafe fn dangerous(a: i32) -> i32 {
    let mut result = a;
    if a > 0 { result += 1; }
    if a > 10 { result += 2; }
    if a > 100 { result += 3; }
    if a > 1000 { result += 4; }
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
const fn factorial(n: u64) -> u64 {
    let mut result = 1u64;
    let mut i = 2u64;
    while i <= n {
        result *= i;
        i += 1;
    }
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
pub unsafe async fn complex(a: i32, b: i32) -> i32 {
    let x = a + b;
    let y = x * 2;
    if x > 0 { y + 1 } else { y - 1 }
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

// --- GenericParser tests -----------------------------------------------

#[test]
fn generic_parser_java_simple_function() {
    let code = r#"
public class Service {
    public int compute(int a, int b) {
        return a + b;
    }
}
"#;
    let parser = java_parser();
    let results = parser.parse_code(code, "Service.java");
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"compute"),
        "GenericParser (Java) should detect 'compute', got: {names:?}"
    );
}

#[test]
fn generic_parser_rust_simple_function() {
    let code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    let parser = rust_parser();
    let results = parser.parse_code(code, "lib.rs");
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"add"),
        "GenericParser (Rust) should detect 'add', got: {names:?}"
    );
}

#[test]
fn generic_parser_kotlin_function() {
    let code = r#"
fun multiply(a: Int, b: Int): Int {
    return a * b
}
"#;
    let parser = kotlin_parser();
    let results = parser.parse_code(code, "Calc.kt");
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"multiply"),
        "GenericParser (Kotlin) should detect 'multiply', got: {names:?}"
    );
}

#[test]
fn generic_parser_php_function() {
    let code = r#"
<?php
function greet($name) {
    return "Hello, " . $name;
}
"#;
    let parser = php_parser();
    let results = parser.parse_code(code, "hello.php");
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"greet"),
        "GenericParser (PHP) should detect 'greet', got: {names:?}"
    );
}

#[test]
fn generic_parser_csharp_method() {
    let code = r#"
public class Calculator {
    public int Add(int a, int b) {
        return a + b;
    }
}
"#;
    let parser = csharp_parser();
    let results = parser.parse_code(code, "Calc.cs");
    let names: Vec<&str> = results.iter().map(|d| d.function_name.as_str()).collect();
    assert!(
        names.contains(&"Add"),
        "GenericParser (C#) should detect 'Add', got: {names:?}"
    );
}

// --- Regex cache tests --------------------------------------------------

#[test]
fn regex_cache_returns_same_instance_for_static_pattern() {
    let r1 = cached_regex(r"\bif\b");
    let r2 = cached_regex(r"\bif\b");
    assert!(
        std::ptr::eq(r1, r2),
        "cached_regex should return the same static reference for identical patterns"
    );
}

#[test]
fn regex_cache_owned_returns_equivalent_regex() {
    let r1 = cached_regex_owned(r"(?m)//.*$");
    let r2 = cached_regex_owned(r"(?m)//.*$");
    let sample = "let x = 1; // comment\nlet y = 2;";
    assert_eq!(r1.replace_all(sample, ""), r2.replace_all(sample, ""));
}

#[test]
fn regex_cache_owned_handles_many_unique_patterns() {
    // Generate more patterns than OWNED_CACHE_CAPACITY to exercise eviction.
    for i in 0..300 {
        let pattern = format!(r"(?m)unique_marker_{}\b", i);
        let re = cached_regex_owned(&pattern);
        assert!(
            re.is_match(&format!("unique_marker_{} foo", i)),
            "pattern {i} should match its own marker"
        );
    }
}
