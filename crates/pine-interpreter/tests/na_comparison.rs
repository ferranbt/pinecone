// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Pine semantics: comparisons where either operand is `na` yield `na`,
// including `na == na` (testing for na requires the na() function).
//
// Regression guard: Eq/NotEq previously returned a structural bool
// (`1 != na` → true), which leaked truthiness into `if` conditions —
// e.g. `is_new_day = dayofweek != dayofweek[1]` fired on the first bar
// of a series, where `[1]` is na.

use pine_interpreter::{Interpreter, Value};

fn run_and_get(src: &str, var: &str) -> Value {
    let tokens = pine_lexer::Lexer::new(src).tokenize().expect("lex failed");
    let stmts = pine_parser::Parser::new(tokens)
        .parse()
        .expect("parse failed");
    let program = pine_parser::Program::new(stmts);
    let mut interp: Interpreter = Interpreter::new();
    interp.execute(&program).expect("execute failed");
    interp
        .get_variable(var)
        .cloned()
        .unwrap_or_else(|| panic!("variable {var} not found"))
}

// `na` is float NaN; NaN can also reach Eq/NotEq wrapped as a
// `Value::Number(NaN)` (e.g. ta.* functions return `Number(NaN)` for
// all-NaN windows) rather than `Value::Na`. Seed such a value into a
// variable so a script-level comparison exercises that path.
fn run_and_get_with_nan(src: &str, var: &str) -> Value {
    let tokens = pine_lexer::Lexer::new(src).tokenize().expect("lex failed");
    let stmts = pine_parser::Parser::new(tokens)
        .parse()
        .expect("parse failed");
    let program = pine_parser::Program::new(stmts);
    let mut interp: Interpreter = Interpreter::new();
    interp.set_variable("nanv", Value::Number(f64::NAN));
    interp.execute(&program).expect("execute failed");
    interp
        .get_variable(var)
        .cloned()
        .unwrap_or_else(|| panic!("variable {var} not found"))
}

#[test]
fn neq_with_na_operand_is_na() {
    assert_eq!(run_and_get("x = 1 != na", "x"), Value::Na);
    assert_eq!(run_and_get("x = na != 1", "x"), Value::Na);
}

#[test]
fn eq_with_na_operand_is_na() {
    assert_eq!(run_and_get("x = 1 == na", "x"), Value::Na);
    assert_eq!(run_and_get("x = na == 1", "x"), Value::Na);
    assert_eq!(run_and_get("x = na == na", "x"), Value::Na);
}

#[test]
fn eq_neq_without_na_still_boolean() {
    assert_eq!(run_and_get("x = 1 == 1", "x"), Value::Bool(true));
    assert_eq!(run_and_get("x = 1 != 1", "x"), Value::Bool(false));
    assert_eq!(run_and_get("x = 1 != 2", "x"), Value::Bool(true));
    assert_eq!(run_and_get("x = \"a\" == \"a\"", "x"), Value::Bool(true));
}

#[test]
fn eq_neq_with_nan_number_operand_is_na() {
    // Same rule as `na`, but the NaN arrives inside a `Value::Number`.
    assert_eq!(run_and_get_with_nan("x = nanv == nanv", "x"), Value::Na);
    assert_eq!(run_and_get_with_nan("x = nanv != nanv", "x"), Value::Na);
    assert_eq!(run_and_get_with_nan("x = 1 == nanv", "x"), Value::Na);
    assert_eq!(run_and_get_with_nan("x = nanv != 1", "x"), Value::Na);
}

#[test]
fn nan_neq_ternary_takes_else_branch() {
    // With both operands NaN the condition is na (falsy), so the ternary
    // must take the else branch, matching TradingView.
    assert_eq!(
        run_and_get_with_nan("x = nanv != nanv ? 1.0 : 2.0", "x"),
        Value::Number(2.0)
    );
}

#[test]
fn na_neq_condition_is_falsy_in_if() {
    // Mirrors the day-boundary pattern: with `prev` na, the comparison must
    // be na (falsy), so the branch must not run.
    let src = "\
prev = na
flag = 1 != prev
x = 0
if flag
    x := 1
";
    assert_eq!(run_and_get(src, "x"), Value::Number(0.0));
}
