---
name: integration-tests-pinets
description: Use whenever adding or verifying ANY language or runtime feature — a builtin, a semantic behavior, formatting, casts, series/history, anything a Pine script can observe. Test it first with a tests/testdata .pine fixture, then cross-reference it against PineTS. Always run new/changed fixtures through PineTS unless there is a verified incongruence between the two implementations.
---

# Test features with integration fixtures, cross-checked against PineTS

Every observable feature is validated first as an **integration fixture**, and
every fixture is **cross-referenced against PineTS**. Prefer this over crate
unit tests for anything a script can observe — unit tests are for pure internal
helpers.

## 1. Write the fixture

Add `tests/testdata/<area>/<name>.pine` with an expectation block:

```pine
//@version=6
indicator("area/name")
log.info(str.tostring(close))

// Expected output:
// 301
```

The `// Expected output:` lines are what **our interpreter** produces (use
`// Expected error:` for the error path). Run it:

```sh
TEST_FILE=<name>.pine cargo test -p pine-integration-tests --test integration -- --nocapture
```

Useful directives: `// Bars: N` (last N bars of the feed), `// Data: <file>`,
`// Timeframe: ...`, `// Footprint: <file>`, `// Inputs: {json}`.

## 2. Cross-reference against PineTS — always try it

PineTS is the oracle. Run the fixture through it and confirm it agrees with the
`// Expected output:`:

```sh
cd tests/pinets && node check.mjs <path-fragment>    # e.g. node check.mjs ta/vwap  (or a folder)
```

- **Agree** → done, the behavior is corroborated.
- **Diverge** → this is a signal, not a nuisance. Investigate: is it **our** bug,
  or a genuine PineTS limitation/difference? Fix ours if we're wrong.

## 3. Skipping PineTS — only with verified evidence

Only add a skip when you have actually run PineTS and confirmed the divergence is
**not** our bug:

```pine
// Skip PineTS: <what you observed>, e.g. PineTS returns na for ta.vwap on these
// synthetic bars (no intraday session), while we anchor on the UTC day.
```

Never skip "for no reason" or on a hunch — the reason must describe real,
observed PineTS behavior. If you can't explain the incongruence, it's probably
our bug. The `CHECK` list in `tests/pinets/check.mjs` marks which areas are held
to conformance; representation-heavy areas (colors, drawings) are intentionally
outside it.

Before finishing: `cargo test --workspace` green, and the PineTS run either
agrees or carries a justified skip.
