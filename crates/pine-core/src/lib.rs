//! Core language primitives shared across the whole Pine toolchain.
//!
//! This crate sits at the very bottom of the dependency graph — it depends on
//! no other `pine` crate — so every other one (lexer, AST, parser, sema,
//! builtins, interpreter) can consume it. Types belong here when they are
//! needed by crates that are otherwise independent siblings, such as
//! [`PineVersion`], which both `pine-lexer` and `pine-ast` need.

mod version;

pub use version::{PineVersion, VersionError};
