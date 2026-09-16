//! Testes de integração do motor de expressões (`rustcalc::eval`).

use rustcalc::eval::{evaluate, format_value, AngleMode, EvalError, Options};

fn rt(expr: &str, expected: f64) {
    let got = evaluate(expr, Options::default()).unwrap();
    assert!(
        (got - expected).abs() < 1e-9,
        "evaluate({expr:?}) = {got}, esperava {expected}"
    );
}

fn is_err(expr: &str, variant: &dyn Fn(&EvalError) -> bool) {
    let e = evaluate(expr, Options::default()).unwrap_err();
    assert!(
        variant(&e),
        "evaluate({expr:?}) deveria falhar, mas deu {e:?}"
    );
}

#[test]
fn arithmetic() {
    rt("2+3*4", 14.0);
    rt("(2+3)*4", 20.0);
    rt("10/4", 2.5);
    rt("2^10", 1024.0);
    rt("7", 7.0);
    rt("0.5+0.5", 1.0);
}

#[test]
fn precedence_and_associativity() {
    rt("2^3^2", 512.0);
    rt("-2^2", -4.0);
    rt("2^-2", 0.25);
    rt("2*3+4*5", 26.0);
    rt("100-50-20", 30.0);
    rt("100/10/2", 5.0);
}

#[test]
fn implicit_multiplication() {
    rt("2(3+4)", 14.0);
    rt("2pi", 2.0 * std::f64::consts::PI);
    rt("(1+2)(3+4)", 21.0);
    rt("2(3)", 6.0);
    rt_p("4ans", 7.0, 28.0);
}

#[test]
fn percent() {
    rt("50%", 0.5);
    rt("200%+1", 3.0);
    rt("8%3", 2.0);
    rt("(1+2)%", 0.03);
}

#[test]
fn factorial() {
    rt("5!", 120.0);
    rt("0!", 1.0);
    rt("3!*2", 12.0);
    rt("2^3!", 64.0);
}

#[test]
fn constants() {
    rt("pi", std::f64::consts::PI);
    rt("e", std::f64::consts::E);
    rt("tau", std::f64::consts::TAU);
    rt("2e3", 2000.0);
    rt("1.5e-3", 0.0015);
}

#[test]
fn functions() {
    rt("sin(pi/2)", 1.0);
    rt("cos(0)", 1.0);
    rt("ln(e)", 1.0);
    rt("log(1000)", 3.0);
    rt("log2(8)", 3.0);
    rt("sqrt(9)", 3.0);
    rt("abs(-5)", 5.0);
    rt("floor(2.9)", 2.0);
    rt("ceil(2.1)", 3.0);
    rt("exp(1)", std::f64::consts::E);
    rt("pow(2,8)", 256.0);
    rt("min(1,8)", 1.0);
    rt("max(1,8)", 8.0);
    rt("hypot(3,4)", 5.0);
}

#[test]
fn functions_in_degrees() {
    let opts = Options {
        angle: AngleMode::Degrees,
        ans: 0.0,
    };
    assert!((evaluate("sin(90)", opts).unwrap() - 1.0).abs() < 1e-9);
    assert!((evaluate("asin(1)", opts).unwrap() - 90.0).abs() < 1e-9);
    assert!((evaluate("tan(45)", opts).unwrap() - 1.0).abs() < 1e-9);
}

#[test]
fn typed_chars() {
    rt("2×3", 6.0);
    rt("8÷2", 4.0);
    rt("π", std::f64::consts::PI);
    rt("√9", 3.0);
    rt("4²", 16.0);
    rt("4³", 64.0);
}

#[test]
fn auto_closed_parentheses() {
    rt("(1+2", 3.0);
    rt("sqrt(9", 3.0);
}

#[test]
fn last_answer() {
    let first = evaluate("6*7", Options::default()).unwrap();
    assert!((first - 42.0).abs() < 1e-9);
    rt_p("ans*2", first, 84.0);
}

/// Como `ans` é avaliado a partir de `Options`, avaliamos com `ans` fixado.
fn rt_p(expr: &str, ans: f64, expected: f64) {
    let got = evaluate(
        expr,
        Options {
            ans,
            ..Options::default()
        },
    )
    .unwrap();
    assert!((got - expected).abs() < 1e-9, "evaluate({expr:?}) = {got}");
}

#[test]
fn errors() {
    is_err("5/0", &|e| matches!(e, EvalError::DivisionByZero));
    is_err("sqrt(-1)", &|e| matches!(e, EvalError::Domain(_)));
    is_err("foo(2)", &|e| matches!(e, EvalError::UnknownName(_)));
    is_err("", &|e| matches!(e, EvalError::Empty));
    is_err("(1+2))", &|e| matches!(e, EvalError::Unexpected(_)));
    is_err("2+", &|e| matches!(e, EvalError::NeedValue));
    is_err("sin()", &|e| matches!(e, EvalError::ArgCount(_)));
    is_err("sin(1,2)", &|e| matches!(e, EvalError::ArgCount(_)));
    is_err("3.5!", &|e| matches!(e, EvalError::Factorial(_)));
    is_err("171!", &|e| matches!(e, EvalError::Overflow));
    is_err("(-1)^0.5", &|e| matches!(e, EvalError::Domain(_)));
    is_err("2^2000", &|e| matches!(e, EvalError::Overflow));
    is_err("ln(0)", &|e| matches!(e, EvalError::Domain(_)));
    is_err("@", &|e| matches!(e, EvalError::Unexpected(_)));
}

#[test]
fn formatting() {
    assert_eq!(format_value(5.0), "5");
    assert_eq!(format_value(0.1 + 0.2), "0.3");
    assert_eq!(format_value(-0.0), "0");
    assert_eq!(format_value(2.5), "2.5");
    assert_eq!(format_value(1e13), "10000000000000");
    assert_eq!(format_value(1e16), "1e16");
    assert_eq!(format_value(1e-10), "1e-10");
    assert_eq!(format_value(123456789012.0), "123456789012");
}
