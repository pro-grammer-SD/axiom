/// Integration tests for Axiom closure correctness and runtime behaviour.
///
/// These tests verify:
///   • Nested function closure capture (Local → Enclosing → Global)
///   • Lambda returning lambda (currying pattern)
///   • Shadowed variable resolution
///   • Multiple environment layers
///   • `ret` / `return` keyword parity
///   • Nil handling and typed arithmetic
///   • Arity mismatch produces RuntimeError::ArityMismatch
///   • Stack overflow produces a meaningful error
use axiom::{Parser, Runtime};
use axiom::core::value::AxValue;
use axiom::errors::RuntimeError;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn run_script(src: &str) -> Result<Runtime, RuntimeError> {
    let mut parser = Parser::new(src, 0);
    let items = parser.parse().expect("parse should succeed");
    let mut rt = Runtime::new();
    rt.run(items)?;
    Ok(rt)
}

fn eval_expr(src: &str) -> AxValue {
    // Wrap in a top-level `let result = <expr>` so we can inspect the value
    let wrapper = format!("let result = {}", src);
    let mut parser = Parser::new(&wrapper, 0);
    let items = parser.parse().expect("parse should succeed");
    let mut rt = Runtime::new();
    rt.run(items).expect("runtime should succeed");
    rt.globals.get("result").cloned().unwrap_or(AxValue::Nil)
}

// ─── Closure capture ──────────────────────────────────────────────────────────

#[test]
fn test_closure_captures_outer_variable() {
    let src = r#"
        fn make_adder(x) {
            fn adder(y) {
                ret x + y
            }
            ret adder
        }
        let add5 = make_adder(5)
        let result = add5(10)
    "#;
    let rt = run_script(src).expect("should succeed");
    let result = rt.globals.get("result").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(result, AxValue::Num(n) if n == 15.0), "add5(10) should be 15");
}

#[test]
fn test_multiple_closures_independent() {
    let src = r#"
        fn make_adder(x) {
            fn adder(y) { ret x + y }
            ret adder
        }
        let add5  = make_adder(5)
        let add10 = make_adder(10)
        let r1 = add5(3)
        let r2 = add10(7)
    "#;
    let rt = run_script(src).expect("should succeed");
    let r1 = rt.globals.get("r1").cloned().unwrap_or(AxValue::Nil);
    let r2 = rt.globals.get("r2").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(r1, AxValue::Num(n) if n == 8.0),  "add5(3)  should be 8");
    assert!(matches!(r2, AxValue::Num(n) if n == 17.0), "add10(7) should be 17");
}

// ─── Lambda returning lambda (currying) ───────────────────────────────────────

#[test]
fn test_lambda_returning_lambda() {
    let src = r#"
        let multiply = fn(x) {
            ret fn(y) {
                ret x * y
            }
        }
        let triple = multiply(3)
        let result = triple(7)
    "#;
    let rt = run_script(src).expect("should succeed");
    let result = rt.globals.get("result").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(result, AxValue::Num(n) if n == 21.0), "triple(7) should be 21");
}

// ─── Shadowed variables ────────────────────────────────────────────────────────

#[test]
fn test_shadowed_variable_in_nested_scope() {
    let src = r#"
        let x = 100
        fn shadow() {
            let x = 42
            ret x
        }
        let outer_x = x
        let inner_x = shadow()
    "#;
    let rt = run_script(src).expect("should succeed");
    let outer = rt.globals.get("outer_x").cloned().unwrap_or(AxValue::Nil);
    let inner = rt.globals.get("inner_x").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(outer, AxValue::Num(n) if n == 100.0), "outer x should be 100");
    assert!(matches!(inner, AxValue::Num(n) if n == 42.0),  "shadow() should return 42");
}

// ─── Multiple environment layers ──────────────────────────────────────────────

#[test]
fn test_three_level_closure() {
    let src = r#"
        fn outer(a) {
            fn middle(b) {
                fn inner(c) {
                    ret a + b + c
                }
                ret inner
            }
            ret middle
        }
        let m  = outer(1)
        let i  = m(2)
        let result = i(3)
    "#;
    let rt = run_script(src).expect("should succeed");
    let result = rt.globals.get("result").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(result, AxValue::Num(n) if n == 6.0), "1+2+3 should be 6");
}

// ─── ret / return keyword parity ─────────────────────────────────────────────

#[test]
fn test_ret_and_return_are_equivalent() {
    let src_ret = r#"
        fn f_ret(x) { ret x + 1 }
        let r1 = f_ret(9)
    "#;
    let src_return = r#"
        fn f_return(x) { return x + 1 }
        let r2 = f_return(9)
    "#;
    let rt1 = run_script(src_ret).expect("ret should work");
    let rt2 = run_script(src_return).expect("return should work");
    let r1 = rt1.globals.get("r1").cloned().unwrap_or(AxValue::Nil);
    let r2 = rt2.globals.get("r2").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(r1, AxValue::Num(n) if n == 10.0));
    assert!(matches!(r2, AxValue::Num(n) if n == 10.0));
}

// ─── Nil handling ─────────────────────────────────────────────────────────────

#[test]
fn test_nil_is_falsy() {
    let src = r#"
        let x = nil
        let result = if x { "truthy" } else { "falsy" }
    "#;
    // Use simpler approach: run and check the global
    let src2 = r#"
        fn check_nil() {
            let x = nil
            if x {
                ret "truthy"
            }
            ret "falsy"
        }
        let result = check_nil()
    "#;
    let rt = run_script(src2).expect("should succeed");
    let result = rt.globals.get("result").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(result, AxValue::Str(s) if s == "falsy"), "nil should be falsy");
}

#[test]
fn test_calling_nil_gives_nil_call_error() {
    let src = r#"
        let f = nil
        f()
    "#;
    let result = run_script(src);
    assert!(
        matches!(result, Err(RuntimeError::NilCall { .. })),
        "calling nil should give NilCall error"
    );
}

// ─── Arity mismatch ───────────────────────────────────────────────────────────

#[test]
fn test_arity_mismatch_error() {
    let src = r#"
        fn add(a, b) { ret a + b }
        add(1, 2, 3)
    "#;
    let result = run_script(src);
    assert!(
        matches!(result, Err(RuntimeError::ArityMismatch { expected: 2, found: 3 })),
        "arity mismatch should be reported"
    );
}

// ─── Fibonacci correctness ────────────────────────────────────────────────────

#[test]
fn test_fibonacci_iterative() {
    let src = r#"
        fn fib(n) {
            if n <= 1 { ret n }
            let a = 0
            let b = 1
            let i = 2
            while i <= n {
                let tmp = a + b
                a = b
                b = tmp
                i = i + 1
            }
            ret b
        }
        let r10 = fib(10)
        let r20 = fib(20)
    "#;
    let rt = run_script(src).expect("should succeed");
    let r10 = rt.globals.get("r10").cloned().unwrap_or(AxValue::Nil);
    let r20 = rt.globals.get("r20").cloned().unwrap_or(AxValue::Nil);
    assert!(matches!(r10, AxValue::Num(n) if n == 55.0),   "fib(10) should be 55");
    assert!(matches!(r20, AxValue::Num(n) if n == 6765.0), "fib(20) should be 6765");
}

// ─── Higher-order: alg.map with user-defined function ─────────────────────────

#[test]
fn test_alg_range_returns_list() {
    let src = r#"
        let nums = alg.range(5)
        let s = alg.sum(nums)
    "#;
    let rt = run_script(src).expect("should succeed");
    let s = rt.globals.get("s").cloned().unwrap_or(AxValue::Nil);
    // range(5) = [0,1,2,3,4], sum = 10
    assert!(matches!(s, AxValue::Num(n) if n == 10.0), "alg.sum(range(5)) should be 10");
}

#[test]
fn test_alg_map_with_lambda() {
    let src = r#"
        let nums    = alg.range(4)
        let doubled = alg.map(nums, fn(x) { ret x * 2 })
        let s       = alg.sum(doubled)
    "#;
    let rt = run_script(src).expect("should succeed");
    let s = rt.globals.get("s").cloned().unwrap_or(AxValue::Nil);
    // range(4)=[0,1,2,3] doubled=[0,2,4,6], sum=12
    assert!(matches!(s, AxValue::Num(n) if n == 12.0), "sum of doubled range(4) should be 12");
}

// ─── String operations ────────────────────────────────────────────────────────

#[test]
fn test_string_methods() {
    let src = r#"
        let s      = "hello world"
        let upper  = s.upper()
        let parts  = s.split(" ")
    "#;
    let rt = run_script(src).expect("should succeed");
    let upper = rt.globals.get("upper").cloned().unwrap_or(AxValue::Nil);
    assert!(
        matches!(&upper, AxValue::Str(s) if s == "HELLO WORLD"),
        "upper() should uppercase the string"
    );
}

// ─── Stack overflow detection ─────────────────────────────────────────────────

#[test]
fn test_stack_overflow_detected() {
    let src = r#"
        fn infinite() {
            ret infinite()
        }
        infinite()
    "#;
    let result = run_script(src);
    assert!(
        matches!(result, Err(RuntimeError::GenericError { message, .. }) if message.contains("overflow")),
        "infinite recursion should produce a stack overflow error"
    );
}
