use pine_ast::{
    Argument, BinOp, EnumField, ExportItem, Expr, FunctionParam, Literal, MethodParam, Program,
    Stmt, TypeField, TypeQualifier, UnOp, VarKind,
};

use crate::comments::{Comments, Lead};
use crate::doc::{concat, group, hardline, join, line, nest, softline, text, Doc};

const BLOCK_INDENT: usize = 4;

pub(crate) struct Rules {
    comments: Comments,
}

impl Rules {
    pub(crate) fn new(comments: Comments) -> Self {
        Self { comments }
    }

    pub(crate) fn program(&mut self, program: &Program) -> Doc {
        let mut entries = self.list_items(&program.statements);

        // Trivia after the last statement (e.g. an `Expected output` block),
        // keeping a blank line before it.
        let mut blank = false;
        for lead in self.comments.take_leading(Some(u32::MAX)) {
            match lead {
                Lead::Blank => blank = true,
                Lead::Comment(comment) => {
                    entries.push((blank, text(comment)));
                    blank = false;
                }
            }
        }
        for comment in self.comments.drain_trailing() {
            entries.push((false, text(comment)));
        }
        join_entries(entries)
    }

    /// A statement list as `(blank_before, doc)` entries — one per statement and
    /// per leading comment. `blank_before` marks a preserved paragraph break.
    fn list_items(&mut self, stmts: &[Stmt]) -> Vec<(bool, Doc)> {
        let mut entries = Vec::new();
        for stmt in stmts {
            let line = stmt_line(stmt);
            let mut blank = false;
            for lead in self.comments.take_leading(line) {
                match lead {
                    Lead::Blank => blank = true,
                    Lead::Comment(comment) => {
                        entries.push((blank, text(comment)));
                        blank = false;
                    }
                }
            }
            let trailing = self
                .comments
                .take_trailing(line)
                .map(|c| text(format!(" {c}")))
                .unwrap_or(Doc::Nil);
            entries.push((blank, self.stmt(stmt, trailing)));
        }
        entries
    }

    /// A block body indented one level below its header.
    fn block(&mut self, body: &[Stmt]) -> Doc {
        let entries = self.list_items(body);
        nest(
            BLOCK_INDENT,
            concat(vec![hardline(), join_entries(entries)]),
        )
    }

    /// An indented body of one-per-line items (type/enum fields), attaching each
    /// item's leading and trailing comments the way [`Self::list_items`] does for
    /// statements. Each item is `(source line, rendered text)`.
    fn field_body(&mut self, items: Vec<(Option<u32>, String)>) -> Doc {
        let mut entries: Vec<(bool, Doc)> = Vec::new();
        for (line, mut rendered) in items {
            let mut blank = false;
            for lead in self.comments.take_leading(line) {
                match lead {
                    Lead::Blank => blank = true,
                    Lead::Comment(comment) => {
                        entries.push((blank, text(comment)));
                        blank = false;
                    }
                }
            }
            if let Some(comment) = self.comments.take_trailing(line) {
                rendered.push_str(&format!(" {comment}"));
            }
            entries.push((blank, text(rendered)));
        }
        nest(
            BLOCK_INDENT,
            concat(vec![hardline(), join_entries(entries)]),
        )
    }

    fn stmt(&mut self, stmt: &Stmt, trailing: Doc) -> Doc {
        match stmt {
            // A bare `name(params) => body` parses as a var whose initializer is
            // a function; Pine has no assignable-lambda form, so render it as a
            // function declaration.
            Stmt::VarDecl {
                name,
                initializer: Some(Expr::Function { params, body }),
                ..
            } => self.function(name, params, body, false, trailing),
            Stmt::VarDecl {
                name,
                type_qualifier,
                type_annotation,
                initializer,
                var_kind,
                ..
            } => {
                let mut head = String::new();
                match var_kind {
                    VarKind::Var => head.push_str("var "),
                    VarKind::Varip => head.push_str("varip "),
                    VarKind::Plain => {}
                }
                if let Some(q) = type_qualifier {
                    head.push_str(qualifier(q));
                    head.push(' ');
                }
                if let Some(t) = type_annotation {
                    head.push_str(t);
                    head.push(' ');
                }
                head.push_str(name);
                match initializer {
                    Some(init) => self.assign(head, "=", init, trailing),
                    None => concat(vec![text(head), trailing]),
                }
            }
            Stmt::Assignment { target, value } => {
                self.assign(self.expr_str(target), ":=", value, trailing)
            }
            Stmt::TupleAssignment { names, value, .. } => {
                self.assign(format!("[{}]", names.join(", ")), "=", value, trailing)
            }
            Stmt::Expression(expr) => concat(vec![self.value(expr), trailing]),
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                let mut parts = vec![
                    text(format!("if {}", self.expr_str(condition))),
                    trailing,
                    self.block(then_branch),
                ];
                for (cond, body) in else_if_branches {
                    parts.push(hardline());
                    parts.push(text(format!("else if {}", self.expr_str(cond))));
                    parts.push(self.block(body));
                }
                if let Some(body) = else_branch {
                    parts.push(hardline());
                    parts.push(text("else"));
                    parts.push(self.block(body));
                }
                concat(parts)
            }
            Stmt::For {
                var_name,
                from,
                to,
                step,
                body,
                ..
            } => {
                let by = match step {
                    Some(step) => format!(" by {}", self.expr_str(step)),
                    None => String::new(),
                };
                let head = format!(
                    "for {var_name} = {} to {}{by}",
                    self.expr_str(from),
                    self.expr_str(to)
                );
                concat(vec![text(head), trailing, self.block(body)])
            }
            Stmt::ForIn {
                index_var,
                item_var,
                collection,
                body,
                ..
            } => {
                let binding = match index_var {
                    Some(index) => format!("[{index}, {item_var}]"),
                    None => item_var.clone(),
                };
                let head = format!("for {binding} in {}", self.expr_str(collection));
                concat(vec![text(head), trailing, self.block(body)])
            }
            Stmt::While { condition, body } => {
                let head = format!("while {}", self.expr_str(condition));
                concat(vec![text(head), trailing, self.block(body)])
            }
            Stmt::Break { .. } => concat(vec![text("break"), trailing]),
            Stmt::Continue { .. } => concat(vec![text("continue"), trailing]),
            Stmt::TypeDecl {
                name,
                fields,
                export,
                ..
            } => {
                let head = text(format!("{}type {name}", export_prefix(*export)));
                let items: Vec<(Option<u32>, String)> = fields
                    .iter()
                    .map(|f| (f.loc.line(), self.type_field(f)))
                    .collect();
                let body = self.field_body(items);
                concat(vec![head, trailing, body])
            }
            Stmt::EnumDecl {
                name,
                fields,
                export,
                ..
            } => {
                let head = text(format!("{}enum {name}", export_prefix(*export)));
                let items: Vec<(Option<u32>, String)> = fields
                    .iter()
                    .map(|f| (f.loc.line(), enum_field(f)))
                    .collect();
                let body = self.field_body(items);
                concat(vec![head, trailing, body])
            }
            Stmt::FunctionDecl {
                name,
                params,
                body,
                export,
                ..
            } => self.function(name, params, body, *export, trailing),
            Stmt::MethodDecl {
                name,
                params,
                body,
                export,
                ..
            } => {
                let head = text(format!(
                    "{}method {name}({}) =>",
                    export_prefix(*export),
                    self.method_params(params)
                ));
                concat(vec![head, trailing, self.block(body)])
            }
            Stmt::Export { item } => {
                let head = match item {
                    ExportItem::Type(name) => format!("export type {name}"),
                    ExportItem::Function(name) => format!("export {name}"),
                };
                concat(vec![text(head), trailing])
            }
            Stmt::Import { path, alias, .. } => {
                let head = if alias.is_empty() {
                    format!("import {path}")
                } else {
                    format!("import {path} as {alias}")
                };
                concat(vec![text(head), trailing])
            }
        }
    }

    /// A function declaration `name(params) => body`, inline when the body is a
    /// single non-block expression, otherwise an indented block.
    fn function(
        &mut self,
        name: &str,
        params: &[FunctionParam],
        body: &[Stmt],
        exported: bool,
        trailing: Doc,
    ) -> Doc {
        let sig = format!(
            "{}{name}({}) =>",
            export_prefix(exported),
            self.params(params)
        );
        match single_expr(body) {
            Some(e) if !is_block_expr(e) => {
                concat(vec![text(format!("{sig} ")), self.expr(e), trailing])
            }
            _ => concat(vec![text(sig), trailing, self.block(body)]),
        }
    }

    /// `head op value`, with `value` possibly a multi-line block expression.
    /// The trailing comment sits at the end of the line for inline values.
    fn assign(&mut self, head: String, op: &str, value: &Expr, trailing: Doc) -> Doc {
        concat(vec![
            text(format!("{head} {op} ")),
            self.value(value),
            trailing,
        ])
    }

    /// A statement-level value: block expressions render multi-line, everything
    /// else stays on one line (with wrapping handled inside brackets).
    fn value(&mut self, expr: &Expr) -> Doc {
        match expr {
            Expr::Switch { value, cases } => {
                let head = text(format!("switch {}", self.expr_str(value)));
                let arms: Vec<Doc> = cases
                    .iter()
                    .map(|(pattern, result)| {
                        concat(vec![
                            text(format!("{} => ", self.expr_str(pattern))),
                            self.value(result),
                        ])
                    })
                    .collect();
                concat(vec![
                    head,
                    nest(BLOCK_INDENT, concat(interleave_hardlines(arms))),
                ])
            }
            Expr::IfExpr {
                condition,
                then_expr,
                else_if_branches,
                else_expr,
            } => {
                let mut parts = vec![
                    text(format!("if {}", self.expr_str(condition))),
                    self.branch(then_expr),
                ];
                for (cond, body) in else_if_branches {
                    parts.push(hardline());
                    parts.push(text(format!("else if {}", self.expr_str(cond))));
                    parts.push(self.branch(body));
                }
                if let Some(body) = else_expr {
                    parts.push(hardline());
                    parts.push(text("else"));
                    parts.push(self.branch(body));
                }
                concat(parts)
            }
            Expr::Function { params, body } if single_expr(body).is_none() => {
                let head = text(format!("({}) =>", self.params(params)));
                concat(vec![head, self.block(body)])
            }
            Expr::Call {
                callee,
                type_args,
                args,
                ..
            } => self.call_value(callee, type_args, args),
            Expr::Array(elements) => self.array_value(elements),
            _ => self.expr(expr),
        }
    }

    /// `callee(args)` at statement level, claiming any comments anchored to an
    /// argument's line. With no such comment this is byte-identical to the
    /// inline [`Self::expr`] form (a group that may still wrap on width); with
    /// one, the group is dropped so the ambient break lays each argument on its
    /// own line, leading comments above and trailing comments after the comma.
    fn call_value(&mut self, callee: &Expr, type_args: &[String], args: &[Argument]) -> Doc {
        let type_args = if type_args.is_empty() {
            String::new()
        } else {
            format!("<{}>", type_args.join(", "))
        };
        let head = concat(vec![self.postfix_operand(callee), text(type_args)]);
        if args.is_empty() {
            return concat(vec![head, text("()")]);
        }
        let items: Vec<(Option<u32>, Doc)> = args
            .iter()
            .map(|a| (expr_line(argument_expr(a)), self.argument(a)))
            .collect();
        let (inner, any_comment) = self.comment_aware_items(items);
        let doc = concat(vec![
            head,
            text("("),
            nest(BLOCK_INDENT, inner),
            softline(),
            text(")"),
        ]);
        if any_comment {
            doc
        } else {
            group(doc)
        }
    }

    /// `[elements]` at statement level, with the same comment-aware behaviour as
    /// [`Self::call_value`].
    fn array_value(&mut self, elements: &[Expr]) -> Doc {
        if elements.is_empty() {
            return text("[]");
        }
        let items: Vec<(Option<u32>, Doc)> = elements
            .iter()
            .map(|e| (expr_line(e), self.expr(e)))
            .collect();
        let (inner, any_comment) = self.comment_aware_items(items);
        let doc = concat(vec![
            text("["),
            nest(BLOCK_INDENT, inner),
            softline(),
            text("]"),
        ]);
        if any_comment {
            doc
        } else {
            group(doc)
        }
    }

    /// Lay out comma-separated `items` (`(source line, rendered doc)`), pulling
    /// in each item's leading and trailing comments. Returns the bracket-interior
    /// doc (a leading softline, the items, comments interleaved) and whether any
    /// comment was found — the caller drops the enclosing group when it was, so
    /// the break is forced.
    fn comment_aware_items(&mut self, items: Vec<(Option<u32>, Doc)>) -> (Doc, bool) {
        let mut parts: Vec<Doc> = vec![softline()];
        let mut any_comment = false;
        let last = items.len().saturating_sub(1);
        for (i, (item_line, item)) in items.into_iter().enumerate() {
            for lead in self.comments.take_leading(item_line) {
                if let Lead::Comment(comment) = lead {
                    parts.push(text(comment));
                    parts.push(hardline());
                    any_comment = true;
                }
            }
            parts.push(item);
            let trailing = self.comments.take_trailing(item_line);
            any_comment |= trailing.is_some();
            if i < last {
                parts.push(text(","));
                if let Some(comment) = trailing {
                    parts.push(text(format!(" {comment}")));
                }
                parts.push(line());
            } else if let Some(comment) = trailing {
                parts.push(text(format!(" {comment}")));
            }
        }
        (concat(parts), any_comment)
    }

    /// The indented body of an `if`-expression branch.
    fn branch(&mut self, expr: &Expr) -> Doc {
        nest(BLOCK_INDENT, concat(vec![hardline(), self.value(expr)]))
    }

    /// The inline rendering of an expression (calls and arrays may still wrap
    /// inside their brackets).
    fn expr(&self, expr: &Expr) -> Doc {
        match expr {
            Expr::Literal(lit) => text(literal(lit)),
            Expr::Variable { name, .. } => text(name.clone()),
            Expr::Binary {
                left, op, right, ..
            } => {
                let prec = bin_prec(op);
                concat(vec![
                    self.operand(left, prec, false),
                    text(format!(" {} ", bin_op(op))),
                    self.operand(right, prec, true),
                ])
            }
            Expr::Unary { op, expr } => match op {
                UnOp::Neg => concat(vec![text("-"), self.unary_operand(expr)]),
                UnOp::Not => concat(vec![text("not "), self.unary_operand(expr)]),
            },
            Expr::Call {
                callee,
                type_args,
                args,
                ..
            } => {
                let type_args = if type_args.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", type_args.join(", "))
                };
                let head = concat(vec![self.postfix_operand(callee), text(type_args)]);
                if args.is_empty() {
                    return concat(vec![head, text("()")]);
                }
                let arg_docs: Vec<Doc> = args.iter().map(|a| self.argument(a)).collect();
                group(concat(vec![
                    head,
                    text("("),
                    nest(
                        BLOCK_INDENT,
                        concat(vec![
                            softline(),
                            join(concat(vec![text(","), line()]), arg_docs),
                        ]),
                    ),
                    softline(),
                    text(")"),
                ]))
            }
            Expr::Index { expr, index, .. } => concat(vec![
                self.postfix_operand(expr),
                text("["),
                self.expr(index),
                text("]"),
            ]),
            Expr::MemberAccess { object, member, .. } => concat(vec![
                self.postfix_operand(object),
                text(format!(".{member}")),
            ]),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => concat(vec![
                self.ternary_part(condition),
                text(" ? "),
                self.ternary_part(then_expr),
                text(" : "),
                self.expr(else_expr),
            ]),
            Expr::Array(elements) => {
                if elements.is_empty() {
                    return text("[]");
                }
                let items: Vec<Doc> = elements.iter().map(|e| self.expr(e)).collect();
                group(concat(vec![
                    text("["),
                    nest(
                        BLOCK_INDENT,
                        concat(vec![
                            softline(),
                            join(concat(vec![text(","), line()]), items),
                        ]),
                    ),
                    softline(),
                    text("]"),
                ]))
            }
            Expr::Function { params, body } => match single_expr(body) {
                Some(e) => concat(vec![
                    text(format!("({}) => ", self.params(params))),
                    self.expr(e),
                ]),
                None => text(format!("({}) => ...", self.params(params))),
            },
            // Only reachable via pathological nesting; block forms go through `value`.
            Expr::Switch { .. } | Expr::IfExpr { .. } => text("..."),
        }
    }

    /// The inline string of an expression (no wrapping); used where a value is
    /// known to stay on one line.
    fn expr_str(&self, expr: &Expr) -> String {
        crate::doc::layout(&self.expr(expr), usize::MAX)
    }

    fn argument(&self, arg: &Argument) -> Doc {
        match arg {
            Argument::Positional(e) => self.expr(e),
            Argument::Named { name, value } => {
                concat(vec![text(format!("{name}=")), self.expr(value)])
            }
        }
    }

    fn operand(&self, expr: &Expr, parent: u8, right: bool) -> Doc {
        let inner = self.expr(expr);
        let wrap = match child_prec(expr) {
            Some(child) => child < parent || (right && child == parent),
            None => false,
        };
        parenthesize(inner, wrap)
    }

    fn unary_operand(&self, expr: &Expr) -> Doc {
        let wrap = matches!(child_prec(expr), Some(child) if child < UNARY_PREC);
        parenthesize(self.expr(expr), wrap)
    }

    fn postfix_operand(&self, expr: &Expr) -> Doc {
        let wrap = matches!(
            expr,
            Expr::Binary { .. } | Expr::Unary { .. } | Expr::Ternary { .. }
        );
        parenthesize(self.expr(expr), wrap)
    }

    fn ternary_part(&self, expr: &Expr) -> Doc {
        let wrap = matches!(expr, Expr::Ternary { .. } | Expr::IfExpr { .. });
        parenthesize(self.expr(expr), wrap)
    }

    fn params(&self, params: &[FunctionParam]) -> String {
        params
            .iter()
            .map(|p| {
                let mut s = String::new();
                if let Some(q) = &p.type_qualifier {
                    s.push_str(qualifier(q));
                    s.push(' ');
                }
                if let Some(t) = &p.type_annotation {
                    s.push_str(t);
                    s.push(' ');
                }
                s.push_str(&p.name);
                if let Some(default) = &p.default_value {
                    s.push_str(&format!(" = {}", self.expr_str(default)));
                }
                s
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn method_params(&self, params: &[MethodParam]) -> String {
        params
            .iter()
            .map(|p| {
                let mut s = String::new();
                if let Some(q) = &p.type_qualifier {
                    s.push_str(qualifier(q));
                    s.push(' ');
                }
                if let Some(t) = &p.type_annotation {
                    s.push_str(t);
                    s.push(' ');
                }
                s.push_str(&p.name);
                if let Some(default) = &p.default_value {
                    s.push_str(&format!(" = {}", self.expr_str(default)));
                }
                s
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn type_field(&self, field: &TypeField) -> String {
        let mut s = String::new();
        if let Some(q) = &field.type_qualifier {
            s.push_str(qualifier(q));
            s.push(' ');
        }
        s.push_str(&field.type_annotation);
        s.push(' ');
        s.push_str(&field.name);
        if let Some(default) = &field.default_value {
            s.push_str(&format!(" = {}", self.expr_str(default)));
        }
        s
    }
}

/// Join `(blank_before, doc)` entries with a hardline, doubling it where a
/// preserved blank line precedes an entry (never before the first).
fn join_entries(entries: Vec<(bool, Doc)>) -> Doc {
    let mut parts = Vec::with_capacity(entries.len() * 3);
    for (i, (blank, doc)) in entries.into_iter().enumerate() {
        if i > 0 {
            parts.push(hardline());
            if blank {
                parts.push(hardline());
            }
        }
        parts.push(doc);
    }
    concat(parts)
}

fn parenthesize(doc: Doc, wrap: bool) -> Doc {
    if wrap {
        concat(vec![text("("), doc, text(")")])
    } else {
        doc
    }
}

/// `docs` separated by hardlines, each on its own line under a block header.
fn interleave_hardlines(docs: Vec<Doc>) -> Vec<Doc> {
    let mut out = Vec::with_capacity(docs.len() * 2);
    for doc in docs {
        out.push(hardline());
        out.push(doc);
    }
    out
}

/// An expression that renders across multiple lines (so it cannot sit inline
/// after `=>`).
fn is_block_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Switch { .. } | Expr::IfExpr { .. } => true,
        Expr::Function { body, .. } => single_expr(body).is_none(),
        _ => false,
    }
}

fn single_expr(body: &[Stmt]) -> Option<&Expr> {
    match body {
        [Stmt::Expression(e)] => Some(e),
        _ => None,
    }
}

fn export_prefix(exported: bool) -> &'static str {
    if exported {
        "export "
    } else {
        ""
    }
}

fn qualifier(q: &TypeQualifier) -> &'static str {
    match q {
        TypeQualifier::Const => "const",
        TypeQualifier::Input => "input",
        TypeQualifier::Simple => "simple",
        TypeQualifier::Series => "series",
    }
}

fn enum_field(field: &EnumField) -> String {
    match &field.title {
        Some(title) => format!("{} = {}", field.name, quote(title)),
        None => field.name.clone(),
    }
}

fn literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n) => n.to_string(),
        Literal::Number(f) => number(*f),
        Literal::String(s) => quote(s),
        Literal::Bool(b) => b.to_string(),
        Literal::Na => "na".to_string(),
        Literal::HexColor(c) => c.clone(),
    }
}

/// A float literal, keeping a trailing `.0` so it never re-parses as an int.
fn number(f: f64) -> String {
    if f.is_finite() && f.fract() == 0.0 {
        format!("{f:.1}")
    } else {
        format!("{f}")
    }
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn bin_op(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "==",
        BinOp::NotEq => "!=",
        BinOp::Less => "<",
        BinOp::Greater => ">",
        BinOp::LessEq => "<=",
        BinOp::GreaterEq => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
    }
}

const UNARY_PREC: u8 = 7;

fn bin_prec(op: &BinOp) -> u8 {
    match op {
        BinOp::Or => 1,
        BinOp::And => 2,
        BinOp::Eq | BinOp::NotEq => 3,
        BinOp::Less | BinOp::Greater | BinOp::LessEq | BinOp::GreaterEq => 4,
        BinOp::Add | BinOp::Sub => 5,
        BinOp::Mul | BinOp::Div | BinOp::Mod => 6,
    }
}

/// The precedence of an expression as a binary operand, or `None` for atoms and
/// postfix forms that never need parentheses.
fn child_prec(expr: &Expr) -> Option<u8> {
    match expr {
        Expr::Binary { op, .. } => Some(bin_prec(op)),
        Expr::Unary { .. } => Some(UNARY_PREC),
        Expr::Ternary { .. } | Expr::IfExpr { .. } | Expr::Switch { .. } => Some(0),
        _ => None,
    }
}

/// The source line a statement begins on, derived from the AST's recorded
/// positions (falling back to the statement's leading expression).
fn stmt_line(stmt: &Stmt) -> Option<u32> {
    match stmt {
        Stmt::VarDecl { loc, .. }
        | Stmt::TupleAssignment { loc, .. }
        | Stmt::For { loc, .. }
        | Stmt::ForIn { loc, .. }
        | Stmt::TypeDecl { loc, .. }
        | Stmt::MethodDecl { loc, .. }
        | Stmt::EnumDecl { loc, .. }
        | Stmt::FunctionDecl { loc, .. }
        | Stmt::Import { loc, .. } => loc.line(),
        Stmt::Assignment { target, value } => expr_line(target).or_else(|| expr_line(value)),
        Stmt::Expression(expr) => expr_line(expr),
        Stmt::If { condition, .. } | Stmt::While { condition, .. } => expr_line(condition),
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Export { .. } => None,
    }
}

/// The earliest source line recorded anywhere in an expression, if any.
fn argument_expr(arg: &Argument) -> &Expr {
    match arg {
        Argument::Positional(e) => e,
        Argument::Named { value, .. } => value,
    }
}

fn expr_line(expr: &Expr) -> Option<u32> {
    match expr {
        Expr::Variable { loc, .. } => loc.line(),
        Expr::Binary { loc, left, .. } => loc.line().or_else(|| expr_line(left)),
        Expr::Call { loc, callee, .. } => loc.line().or_else(|| expr_line(callee)),
        Expr::MemberAccess {
            member_loc, object, ..
        } => member_loc.line().or_else(|| expr_line(object)),
        Expr::Index { expr, .. } | Expr::Unary { expr, .. } => expr_line(expr),
        Expr::Ternary { condition, .. } => expr_line(condition),
        _ => None,
    }
}
