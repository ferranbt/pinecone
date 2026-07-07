// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Pine `and`/`or` are lazy: when the left operand alone decides the result
// (false-and / true-or), the right operand is not evaluated, so side effects
// inside it (function calls, stateful ta.* calls) must not run.
// TradingView-validated: strategies calling ta.crossover inside `or`
// right-operands only advance that call's internal state on bars where the
// operand is actually evaluated.
//
// Evaluation (not just the result) is probed with an undefined variable in
// the right operand: if the operand is evaluated, execution fails with
// UndefinedVariable; if it is skipped, execution succeeds.
//
// The three-valued na results are unchanged: false absorbs na in `and`,
// true absorbs na in `or`, and an na left operand still evaluates the right
// operand (its value is needed to decide between na and a definite result).

use pine_interpreter::{Interpreter, Value};

fn run(src: &str) -> Result<Interpreter, pine_interpreter::RuntimeError> {
    let tokens = pine_lexer::Lexer::new(src).tokenize().expect("lex failed");
    let stmts = pine_parser::Parser::new(tokens)
        .parse()
        .expect("parse failed");
    let program = pine_parser::Program::new(stmts);
    let mut interp: Interpreter = Interpreter::new();
    interp.execute(&program)?;
    Ok(interp)
}

fn run_and_get(src: &str, var: &str) -> Value {
    run(src)
        .expect("execute failed")
        .get_variable(var)
        .cloned()
        .unwrap_or_else(|| panic!("variable {var} not found"))
}

#[test]
fn or_with_true_left_skips_right() {
    // `boom` is undefined — evaluating the right operand would error.
    assert_eq!(run_and_get("x = true or boom", "x"), Value::Bool(true));
}

#[test]
fn and_with_false_left_skips_right() {
    assert_eq!(run_and_get("x = false and boom", "x"), Value::Bool(false));
}

#[test]
fn or_with_false_left_evaluates_right() {
    assert!(
        run("x = false or boom").is_err(),
        "right operand must be evaluated"
    );
    assert_eq!(run_and_get("x = false or true", "x"), Value::Bool(true));
}

#[test]
fn and_with_true_left_evaluates_right() {
    assert!(
        run("x = true and boom").is_err(),
        "right operand must be evaluated"
    );
    assert_eq!(run_and_get("x = true and false", "x"), Value::Bool(false));
}

#[test]
fn na_left_still_evaluates_right() {
    // An na left operand cannot decide the result alone; the right operand's
    // value is required (and its side effects run), per three-valued logic.
    assert!(run("cond = na\nx = cond or boom").is_err());
    assert!(run("cond = na\nx = cond and boom").is_err());
    assert_eq!(
        run_and_get("cond = na\nx = cond or true", "x"),
        Value::Bool(true)
    );
    assert_eq!(run_and_get("cond = na\nx = cond and true", "x"), Value::Na);
}

#[test]
fn three_valued_results_unchanged() {
    // Laziness must not change any and/or RESULT, only skipped side effects.
    assert_eq!(run_and_get("x = true or na", "x"), Value::Bool(true));
    assert_eq!(run_and_get("x = false or na", "x"), Value::Na);
    assert_eq!(run_and_get("x = na or false", "x"), Value::Na);
    assert_eq!(run_and_get("x = false and na", "x"), Value::Bool(false));
    assert_eq!(run_and_get("x = true and na", "x"), Value::Na);
    assert_eq!(run_and_get("x = na and false", "x"), Value::Bool(false));
}
