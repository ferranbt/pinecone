//! A Wadler/Prettier-style document algebra and its layout engine.
//!
//! Rules build a [`Doc`]; [`layout`] renders it, choosing for each [`Doc::Group`]
//! whether it fits the target width flat or must break onto multiple lines.

#[derive(Clone)]
pub(crate) enum Doc {
    Nil,
    Text(String),
    /// A space when flat, a newline (+ indent) when broken.
    Line,
    /// Nothing when flat, a newline (+ indent) when broken.
    Softline,
    /// Always a newline; forces every enclosing group to break.
    Hardline,
    Concat(Vec<Doc>),
    Nest(usize, Box<Doc>),
    Group(Box<Doc>),
}

pub(crate) fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}

pub(crate) fn concat(docs: Vec<Doc>) -> Doc {
    Doc::Concat(docs)
}

pub(crate) fn nest(indent: usize, doc: Doc) -> Doc {
    Doc::Nest(indent, Box::new(doc))
}

pub(crate) fn group(doc: Doc) -> Doc {
    Doc::Group(Box::new(doc))
}

pub(crate) fn line() -> Doc {
    Doc::Line
}

pub(crate) fn softline() -> Doc {
    Doc::Softline
}

pub(crate) fn hardline() -> Doc {
    Doc::Hardline
}

/// `items` interleaved with `sep`.
pub(crate) fn join(sep: Doc, items: Vec<Doc>) -> Doc {
    let mut out = Vec::with_capacity(items.len() * 2);
    for (i, doc) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(sep.clone());
        }
        out.push(doc);
    }
    Doc::Concat(out)
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Flat,
    Break,
}

pub(crate) fn layout(doc: &Doc, width: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    let mut stack: Vec<(usize, Mode, &Doc)> = vec![(0, Mode::Break, doc)];

    while let Some((indent, mode, doc)) = stack.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Text(s) => {
                out.push_str(s);
                col += s.chars().count();
            }
            Doc::Concat(docs) => {
                for child in docs.iter().rev() {
                    stack.push((indent, mode, child));
                }
            }
            Doc::Nest(n, inner) => stack.push((indent + n, mode, inner)),
            Doc::Line => match mode {
                Mode::Flat => {
                    out.push(' ');
                    col += 1;
                }
                Mode::Break => col = newline(&mut out, indent),
            },
            Doc::Softline => match mode {
                Mode::Flat => {}
                Mode::Break => col = newline(&mut out, indent),
            },
            Doc::Hardline => col = newline(&mut out, indent),
            Doc::Group(inner) => {
                let mode = if fits(width.saturating_sub(col), indent, inner, &stack) {
                    Mode::Flat
                } else {
                    Mode::Break
                };
                stack.push((indent, mode, inner));
            }
        }
    }
    out
}

fn newline(out: &mut String, indent: usize) -> usize {
    out.push('\n');
    for _ in 0..indent {
        out.push(' ');
    }
    indent
}

/// Whether `group_inner` laid out flat, followed by whatever is already queued,
/// stays within `remaining` columns before the next line break.
fn fits<'a>(
    remaining: usize,
    indent: usize,
    group_inner: &'a Doc,
    rest: &[(usize, Mode, &'a Doc)],
) -> bool {
    let mut budget = remaining.min(isize::MAX as usize) as isize;
    let mut stack: Vec<(usize, Mode, &Doc)> = rest.to_vec();
    stack.push((indent, Mode::Flat, group_inner));

    while budget >= 0 {
        let (indent, mode, doc) = match stack.pop() {
            Some(item) => item,
            None => return true,
        };
        match doc {
            Doc::Nil => {}
            Doc::Text(s) => budget -= s.chars().count() as isize,
            Doc::Concat(docs) => {
                for child in docs.iter().rev() {
                    stack.push((indent, mode, child));
                }
            }
            Doc::Nest(n, inner) => stack.push((indent + n, mode, inner)),
            Doc::Line => match mode {
                Mode::Flat => budget -= 1,
                Mode::Break => return true,
            },
            Doc::Softline => match mode {
                Mode::Flat => {}
                Mode::Break => return true,
            },
            // A forced break ends this line, so everything up to here fit.
            Doc::Hardline => return true,
            Doc::Group(inner) => stack.push((indent, Mode::Flat, inner)),
        }
    }
    false
}
