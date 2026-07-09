// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Pine semantics: division or modulo by zero yields `na` (no error).
// `x / 0` and `x % 0` evaluate to `na`, exactly as an na operand does —
// arithmetic never aborts. An na dividend short-circuits to `na` before
// the divisor is inspected, so `na / 0` is `na` for that reason too.

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

#[test]
fn div_by_zero_is_na() {
    assert_eq!(run_and_get("x = 1 / 0", "x"), Value::Na);
}

#[test]
fn zero_div_by_zero_is_na() {
    assert_eq!(run_and_get("x = 0 / 0", "x"), Value::Na);
}

#[test]
fn mod_by_zero_is_na() {
    assert_eq!(run_and_get("x = -3 % 0", "x"), Value::Na);
}

#[test]
fn na_div_by_zero_is_na() {
    assert_eq!(run_and_get("x = na / 0", "x"), Value::Na);
}

#[test]
fn nonzero_division_is_unchanged() {
    assert_eq!(run_and_get("x = 6 / 3", "x"), Value::Number(2.0));
}

#[test]
fn nonzero_modulo_is_unchanged() {
    assert_eq!(run_and_get("x = 7 % 4", "x"), Value::Number(3.0));
}
