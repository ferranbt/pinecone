// Run the fixtures through PineTS and report where its output disagrees with a
// fixture's `// Expected output:` block (which is what our own interpreter
// produces). A divergence means we and PineTS compute a fixture differently —
// evidence to look into, not proof either side is right.
//
//   node check.mjs            # the conformance folders (see CHECK)
//   node check.mjs ta/        # any path containing "ta/", ignoring CHECK

import { PineTS } from 'pinets';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, relative } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const testdata = join(here, '..', 'testdata');
const barsCsv = join(here, '..', 'data', 'bars.csv');

// The folders we hold to PineTS conformance: the numeric and semantic core,
// where PineTS is a trustworthy oracle. Representation-heavy areas — constants,
// colors, drawings, types — are deliberately left out, since the two
// implementations format those differently without either being wrong. Pass a
// path fragment to check something outside this set explicitly.
const CHECK = ['ta/', 'math/', 'operators/', 'series/'];

// Divergences we are not treating as our bugs. Pass --all to check them anyway.
const SKIP = {
    'ta/tr.pine': 'PineTS ignores handle_na on the first bar; the docs side with us',
};

const args = process.argv.slice(2);
const checkAll = args.includes('--all');
const filter = args.find((a) => !a.startsWith('--')) ?? '';

const inScope = (name) => {
    if (!checkAll && name in SKIP) return false;
    return filter ? name.includes(filter) : CHECK.some((d) => name.startsWith(d));
};

/** Every `.pine` file under testdata, excluding imported libraries. */
function fixtures(dir) {
    const out = [];
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) {
            if (entry.name !== 'libraries') out.push(...fixtures(path));
        } else if (entry.name.endsWith('.pine')) {
            out.push(path);
        }
    }
    return out;
}

/** The bars every fixture runs against, oldest first. */
function loadBars() {
    const rows = readFileSync(barsCsv, 'utf8')
        .split('\n')
        .map((l) => l.trim())
        .filter((l) => l && !l.startsWith('#') && !l.startsWith('time'));

    const bars = rows.map((line) => {
        const [time, open, high, low, close, volume] = line.split(',').map(Number);
        return { openTime: time, open, high, low, close, volume };
    });
    // PineTS wants a closeTime; derive it from the bar spacing.
    const step = bars.length > 1 ? bars[1].openTime - bars[0].openTime : 60_000;
    for (const bar of bars) bar.closeTime = bar.openTime + step - 1;
    return bars;
}

/** The `// Expected output:` lines, or null for an error/no-output fixture. */
function expectedOutput(source) {
    if (source.includes('// Expected error:')) return null;
    const marker = source.indexOf('// Expected output:');
    if (marker === -1) return null;

    const lines = [];
    for (const raw of source.slice(marker).split('\n').slice(1)) {
        const line = raw.trim();
        if (line.startsWith('//')) {
            const value = line.slice(2).trim();
            if (value) lines.push(value);
        } else if (line) {
            break;
        }
    }
    return lines;
}

/** Trailing bar count from a `// Bars: N` directive; the default is one bar. */
function barCount(source) {
    const match = source.match(/^\s*\/\/ Bars:\s*(\d+)/m);
    return match ? Math.max(1, Number(match[1])) : 1;
}

/** Run `source` over `bars` and return the log lines, timestamp prefix stripped. */
async function runPineTS(source, bars) {
    const logs = [];
    const original = process.stdout.write.bind(process.stdout);
    process.stdout.write = (chunk) => {
        for (const line of String(chunk).split('\n')) {
            const stripped = line.replace(/^\[[^\]]*\]\s?/, '');
            if (stripped) logs.push(stripped);
        }
        return true;
    };
    try {
        const pine = new PineTS(bars, 'TEST', '1', bars.length);
        await pine.run(source);
    } finally {
        process.stdout.write = original;
    }
    return logs;
}

/** na is spelled NaN by PineTS; numbers match within a small relative tolerance. */
function valuesAgree(a, b) {
    if (a === b) return true;
    const x = Number(a);
    const y = Number(b);
    if (Number.isNaN(x) || Number.isNaN(y)) return false;
    const scale = Math.max(Math.abs(x), Math.abs(y), 1);
    return Math.abs(x - y) <= scale * 1e-9;
}

const all = fixtures(testdata)
    .map((path) => relative(testdata, path))
    .filter(inScope)
    .sort();

let checked = 0;
const diverged = [];
const errored = [];

for (const name of all) {
    const source = readFileSync(join(testdata, name), 'utf8');
    const expected = expectedOutput(source);
    if (expected === null) continue; // error fixture or nothing to print

    const bars = loadBars();
    const used = bars.slice(bars.length - barCount(source));

    let actual;
    try {
        actual = await runPineTS(source, used);
    } catch (error) {
        errored.push({ name, error: error.message });
        continue;
    }

    checked++;
    const rows = Math.max(expected.length, actual.length);
    for (let i = 0; i < rows; i++) {
        if (!valuesAgree(expected[i], actual[i])) {
            diverged.push({ name, line: i + 1, expected: expected[i], actual: actual[i] });
            break;
        }
    }
}

for (const d of diverged) {
    console.log(`✗ ${d.name}`);
    console.log(`    line ${d.line}: ours=${d.expected ?? '<none>'}  pinets=${d.actual ?? '<none>'}`);
}
if (errored.length) {
    console.log('\nPineTS could not run:');
    for (const e of errored) console.log(`  - ${e.name}: ${e.error.split('\n')[0]}`);
}

const skipped = Object.entries(SKIP);
if (!checkAll && skipped.length) {
    console.log('\nSkipped:');
    for (const [name, reason] of skipped) console.log(`  - ${name}: ${reason}`);
}

console.log(
    `\n${checked} checked · ${checked - diverged.length} agree · ` +
        `${diverged.length} diverge · ${errored.length} PineTS errors · ` +
        `${checkAll ? 0 : skipped.length} skipped`
);

// A divergence fails the run. A fixture PineTS cannot run is its limitation,
// not our regression, so it does not.
if (diverged.length) process.exitCode = 1;
