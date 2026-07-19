//! The set of built-in names known to the analyzer.
//!
//! Name resolution needs to know which identifiers exist without a user
//! declaration — price series (`close`), namespaces (`ta`, `math`), and global
//! functions (`plot`, `nz`). This table is deliberately kept as plain string
//! data so `pine-sema` stays decoupled from the interpreter (the runtime's
//! `pine_builtins::register_namespace_objects` returns interpreter `Value`s,
//! which we don't want to depend on here).
//!
//! It errs on the side of **completeness**: a missing name causes a false
//! "undeclared" error (bad), whereas an extra name merely lets a rare typo slip
//! through (minor). Extend freely — this is the single place to teach the
//! analyzer about new built-ins.
//!
//! Sources: `pine_builtins::register_namespace_objects` (namespaces + global
//! functions), the per-bar variables set in `pine::Script::execute`, plus the
//! common Pine v6 built-in variables/namespaces real scripts rely on.

/// Built-in namespaces: valid as the base of a `namespace.member` access. Their
/// members are *not* validated here (that would be Tier 3 signature checking).
const NAMESPACES: &[&str] = &[
    // Registered in this codebase.
    "array",
    "box",
    "color",
    "currency",
    "label",
    "log",
    "math",
    "matrix",
    "str",
    "ta",
    // Common Pine v6 namespaces (not all implemented by the interpreter yet, but
    // valid Pine — listing them avoids false positives on real scripts).
    "line",
    "linefill",
    "table",
    "polyline",
    "map",
    "chart",
    "strategy",
    "request",
    "ticker",
    "syminfo",
    "timeframe",
    "session",
    "barstate",
    "adjustment",
    "alert",
    "barmerge",
    "dayofweek",
    "display",
    "dividends",
    "earnings",
    "extend",
    "font",
    "format",
    "hline",
    "input",
    "location",
    "order",
    "position",
    "scale",
    "settlement_as_close",
    "shape",
    "size",
    "splits",
    "text",
    "xloc",
    "yloc",
    "runtime",
    "month",
    "weekofyear",
];

/// Built-in variables (top-level values, not under a namespace).
const VARIABLES: &[&str] = &[
    // Per-bar series provided by the interpreter.
    "open",
    "high",
    "low",
    "close",
    "volume",
    // Common derived price series / bar state (valid Pine v6).
    "hl2",
    "hlc3",
    "ohlc4",
    "hlcc4",
    "bar_index",
    "last_bar_index",
    "time",
    "time_close",
    "time_tradingday",
    "timenow",
    "last_bar_time",
];

/// Built-in global functions callable without a namespace.
const FUNCTIONS: &[&str] = &[
    // Registered in this codebase.
    "na",
    "bool",
    "int",
    "float",
    "nz",
    "fixnan",
    "plot",
    "plotarrow",
    "plotbar",
    "plotcandle",
    "plotchar",
    "plotshape",
    "dayofmonth",
    "dayofweek",
    "hour",
    "minute",
    "month",
    "second",
    "year",
    // Common bare global functions in Pine v6.
    "fill",
    "bgcolor",
    "barcolor",
    "alertcondition",
    "input",
    "color",
    "string",
    "line",
    "timestamp",
    // v3's unqualified `input(..., type=<tag>)` type constants.
    "integer",
    "source",
    "symbol",
    "resolution",
    "price",
    // Script declarations. Every script opens with exactly one of these.
    "indicator",
    "strategy",
    "library",
];

/// Functions that Pine only permits at **global** scope (never inside `if`,
/// loops, or function bodies). Currently the plot family — the set the
/// interpreter actually registers.
const GLOBAL_ONLY_FUNCTIONS: &[&str] = &[
    "plot",
    "plotshape",
    "plotchar",
    "plotcandle",
    "plotbar",
    "plotarrow",
];

/// Is `name` a known built-in identifier (namespace, variable, or function)?
pub fn is_builtin(name: &str) -> bool {
    NAMESPACES.contains(&name) || VARIABLES.contains(&name) || FUNCTIONS.contains(&name)
}

/// May `name` only be called at global scope?
pub fn is_global_only(name: &str) -> bool {
    GLOBAL_ONLY_FUNCTIONS.contains(&name)
}
