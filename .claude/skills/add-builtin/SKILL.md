---
name: add-builtin
description: Use when adding or changing a Pine built-in function or namespace member in crates/pine-builtins — anything callable from a script like ta.sma, math.max, str.tostring, request.security, or a new namespace value. Covers the #[derive(BuiltinFunction)] macro, the generic-over-O rule, state for series, the value+namespace object duality, and registration.
---

# Add a built-in

Built-ins live in `crates/pine-builtins/src/<namespace>/`. A built-in is a struct
that derives `BuiltinFunction`; the macro turns its fields into the call
signature and generates the callable.

## 1. Declare the struct

```rust
/// ta.sma(source, length) - Simple Moving Average
#[derive(BuiltinFunction)]
#[builtin(name = "ta.sma", stateful)]
pub struct TaSma {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}
```

`#[builtin(...)]` options: `name = "ns.fn"` (required), `stateful` (per-call-site
memory across bars — needed for anything using history/series), `output = Trait`
(an extra capability the builtin needs from the output type, e.g. `LabelOutput`,
`PlotOutput` — only for structs that declare no generics of their own),
`type_params = N` (for `array.new<T>` style generics).

Field attributes:
- `#[arg(default = EXPR)]` — optional argument. `EXPR` can call `ctx`/helpers
  (e.g. `default = bar_source(ctx, "high")`). `Option<T>` fields are also optional.
- `#[arg(variadic)]` — collects trailing positionals into a `Vec<Value<O>>`.
- `#[arg(lazy)]` — receive the argument unevaluated as `Value::Expr` (e.g.
  `request.security`'s expression), to replay yourself.
- `#[state]` — not a call argument; the builtin's own memory, carried across bars
  by the call site. Requires `stateful`.
- `#[length_check]` — a window length that must be `> 0`; rejected before `execute`.
- `#[type_param]` — a `<T>` type argument (needs `type_params = N`).

Field types drive coercion: `f64`, `i64`/`Num` (keeps int-vs-float), `bool`,
`String`, `Color`, or `Value<O>` for a raw/any value.

## 2. Implement `execute`

The **generic-over-`O`** rule:
- A struct that holds a `Value<O>` (or otherwise declares `O`) writes
  `impl<O: PineOutput> Foo<O> { fn execute(&self, ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> }`.
- A struct with only plain fields (`f64`, `String`, …) is non-generic, so `O`
  goes on the method: `impl Foo { fn execute<O: PineOutput>(&mut self, ctx: &mut Interpreter<O>) -> ... }`.

Stateful builtins take `&mut self` (mutate `#[state]` fields). Return `Value::Na`
during warm-up (e.g. before a window fills).

## 3. Register it

Insert into the namespace map in that module's `register()`:
```rust
ta_ns.insert("sma".to_string(), TaSma::builtin_value::<O>());
```
`builtin_value::<O>()` returns the `Value` (callable + signature) to register.
`register_namespace_objects` (in `src/lib.rs`) assembles all namespaces.

### A name that is both a value and a function
Some names are a value *and* callable (`ta.vwap`, `time`, date names). Model them
as `Value::Object { type_name, fields, call: Some(Builtin{..}), value: Some(fn) }`
— `value` is the lazy scalar read for the bare name, `call` is the function. See
`ta.vwap` / `time` for the pattern. Namespace-owned series that advance each bar
register a `PerBarAdvance` (see `advance_accumulators`).

## 4. Test it

Built-ins are behavior — test with an integration fixture and cross-check PineTS.
Follow the **integration-tests-pinets** skill. Also register the builtin's name in
`pine-reference` if it should count toward reference parity.

Before finishing: `cargo build --workspace`, `cargo test --workspace`, and
`cargo clippy --workspace --tests` all clean.
