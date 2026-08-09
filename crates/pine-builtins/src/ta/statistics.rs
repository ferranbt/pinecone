use pine_builtin_macro::BuiltinFunction;
use pine_core::{PineOutput, SeriesBuffer};
use pine_interpreter::{Interpreter, RuntimeError, Value};

/// ta.stdev(source, length) - Standard Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.stdev", stateful)]
pub struct TaStdev {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaStdev {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        if values.len() == 1 {
            return Ok(Value::Number(0.0));
        }

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance: f64 = values
            .iter()
            .map(|&val| {
                let diff = val - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        // Standard deviation is square root of variance
        Ok(Value::Number(variance.sqrt()))
    }
}

/// ta.variance(source, length) - Variance
#[derive(BuiltinFunction)]
#[builtin(name = "ta.variance", stateful)]
pub struct TaVariance {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaVariance {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        if values.len() == 1 {
            return Ok(Value::Number(0.0));
        }

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance: f64 = values
            .iter()
            .map(|&val| {
                let diff = val - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        Ok(Value::Number(variance))
    }
}

/// ta.median(source, length) - Median value
#[derive(BuiltinFunction)]
#[builtin(name = "ta.median", stateful)]
pub struct TaMedian {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaMedian {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(mut values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        // Sort to find median
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = values.len() / 2;
        let median = if values.len() % 2 == 1 {
            // Odd number of elements - middle value
            values[mid]
        } else {
            // Even number of elements - average of two middle values
            (values[mid - 1] + values[mid]) / 2.0
        };

        Ok(Value::Number(median))
    }
}

/// ta.percentile_nearest_rank(source, length, percentage) - Percentile by the
/// nearest-rank method: the smallest value at or below which `percentage` of the
/// last `length` values fall.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.percentile_nearest_rank", stateful)]
pub struct TaPercentileNearestRank {
    source: f64,
    #[length_check]
    length: f64,
    percentage: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPercentileNearestRank {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(mut values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Nearest rank: n = ceil(P/100 * N), 1-based, clamped into the sample.
        let rank = (self.percentage / 100.0 * values.len() as f64).ceil() as usize;
        let index = rank.clamp(1, values.len()) - 1;

        Ok(Value::Number(values[index]))
    }
}

/// ta.cum(source) - Running total of `source` from the first bar onwards.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cum", stateful)]
pub struct TaCum {
    source: f64,
    /// The total so far, carried across bars by this call site.
    #[state]
    total: f64,
}

impl TaCum {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        // Pine's `cum` skips na (which arrives as NaN) rather than letting it
        // poison the total for good.
        if self.source.is_finite() {
            self.total += self.source;
        }

        Ok(Value::Number(self.total))
    }
}

/// ta.dev(source, length) - Mean Absolute Deviation
#[derive(BuiltinFunction)]
#[builtin(name = "ta.dev", stateful)]
pub struct TaDev {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaDev {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;

        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };

        // Calculate mean
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate mean absolute deviation
        let mad: f64 =
            values.iter().map(|&val| (val - mean).abs()).sum::<f64>() / values.len() as f64;

        Ok(Value::Number(mad))
    }
}

/// ta.correlation(source1, source2, length) - Pearson correlation over `length`
/// bars, in `-1 … 1` (`0` when either series has no variance).
#[derive(BuiltinFunction)]
#[builtin(name = "ta.correlation", stateful)]
pub struct TaCorrelation {
    source1: f64,
    source2: f64,
    #[length_check]
    length: f64,
    #[state]
    window1: SeriesBuffer<f64>,
    #[state]
    window2: SeriesBuffer<f64>,
}

impl TaCorrelation {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let xs = self.window1.observe(self.source1, length);
        let Some(ys) = self.window2.observe(self.source2, length) else {
            return Ok(Value::Na);
        };
        let xs = xs.expect("both windows fill together");
        let n = length as f64;
        let (mut sx, mut sy, mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            sx += x;
            sy += y;
            sxy += x * y;
            sxx += x * x;
            syy += y * y;
        }
        let cov = n * sxy - sx * sy;
        let vx = n * sxx - sx * sx;
        let vy = n * syy - sy * sy;
        if vx <= 0.0 || vy <= 0.0 {
            return Ok(Value::Number(0.0));
        }
        Ok(Value::Number(cov / (vx * vy).sqrt()))
    }
}

/// ta.percentrank(source, length) - The percentage of the previous `length`
/// values that are less than or equal to the current value.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.percentrank", stateful)]
pub struct TaPercentrank {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPercentrank {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        // The current value plus the `length` values that precede it.
        let Some(values) = self.window.observe(self.source, length + 1) else {
            return Ok(Value::Na);
        };
        let current = values[0];
        let count = values[1..].iter().filter(|&&v| v <= current).count();
        Ok(Value::Number(count as f64 / length as f64 * 100.0))
    }
}

/// ta.percentile_linear_interpolation(source, length, percentage) - The value at
/// `percentage` of the last `length` bars, linearly interpolating between ranks.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.percentile_linear_interpolation", stateful)]
pub struct TaPercentileLinearInterpolation {
    source: f64,
    #[length_check]
    length: f64,
    percentage: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaPercentileLinearInterpolation {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };
        let mut sorted = values;
        sorted.sort_by(f64::total_cmp);
        let rank = (self.percentage / 100.0 * length as f64 - 0.5).clamp(0.0, length as f64 - 1.0);
        let (lo, hi) = (rank.floor() as usize, rank.ceil() as usize);
        let result = if lo == hi {
            sorted[lo]
        } else {
            sorted[lo] + (rank - lo as f64) * (sorted[hi] - sorted[lo])
        };
        Ok(Value::Number(result))
    }
}

/// ta.max(source) - The all-time high of `source` up to the current bar.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.max", stateful)]
pub struct TaMax {
    source: f64,
    #[state]
    highest: Option<f64>,
}

impl TaMax {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        if !self.source.is_nan() {
            self.highest = Some(self.highest.map_or(self.source, |m| m.max(self.source)));
        }
        Ok(self.highest.map_or(Value::Na, Value::Number))
    }
}

/// ta.min(source) - The all-time low of `source` up to the current bar.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.min", stateful)]
pub struct TaMin {
    source: f64,
    #[state]
    lowest: Option<f64>,
}

impl TaMin {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        if !self.source.is_nan() {
            self.lowest = Some(self.lowest.map_or(self.source, |m| m.min(self.source)));
        }
        Ok(self.lowest.map_or(Value::Na, Value::Number))
    }
}

/// ta.range(source, length) - `highest(source, length) - lowest(source, length)`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.range", stateful)]
pub struct TaRange {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaRange {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };
        let highest = values.iter().copied().fold(f64::MIN, f64::max);
        let lowest = values.iter().copied().fold(f64::MAX, f64::min);
        Ok(Value::Number(highest - lowest))
    }
}

/// ta.mode(source, length) - The most frequent value over the last `length`
/// bars; ties are broken by the smallest value.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.mode", stateful)]
pub struct TaMode {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaMode {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };
        let mut counts: Vec<(f64, usize)> = Vec::new();
        for &v in values.iter().filter(|v| !v.is_nan()) {
            match counts.iter_mut().find(|(x, _)| *x == v) {
                Some((_, c)) => *c += 1,
                None => counts.push((v, 1)),
            }
        }
        let mode = counts
            .into_iter()
            .reduce(|best, cur| match cur.1.cmp(&best.1) {
                std::cmp::Ordering::Greater => cur,
                std::cmp::Ordering::Equal if cur.0 < best.0 => cur,
                _ => best,
            });
        Ok(mode.map_or(Value::Na, |(v, _)| Value::Number(v)))
    }
}

/// ta.cog(source, length) - Center Of Gravity.
///
/// `-Σ(source[u] * (u + 1)) / Σ(source[u])` over the last `length` bars (`u = 0`
/// is the current bar); `na` when the window is not full or the sum is `0`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.cog", stateful)]
pub struct TaCog {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaCog {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };
        let sum: f64 = values.iter().sum();
        if sum == 0.0 {
            return Ok(Value::Na);
        }
        let weighted: f64 = values
            .iter()
            .enumerate()
            .map(|(u, v)| v * (u as f64 + 1.0))
            .sum();
        Ok(Value::Number(-weighted / sum))
    }
}

/// Average (tie-corrected) ascending ranks of `values`, 1-based.
fn average_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && values[order[j + 1]] == values[order[i]] {
            j += 1;
        }
        let rank = (i + j) as f64 / 2.0 + 1.0;
        for &idx in &order[i..=j] {
            ranks[idx] = rank;
        }
        i = j + 1;
    }
    ranks
}

/// ta.rci(source, length) - Rank Correlation Index.
///
/// Spearman's rank correlation between `source` and the bar index over `length`
/// bars, scaled to `-100 … 100`.
#[derive(BuiltinFunction)]
#[builtin(name = "ta.rci", stateful)]
pub struct TaRci {
    source: f64,
    #[length_check]
    length: f64,
    #[state]
    window: SeriesBuffer<f64>,
}

impl TaRci {
    fn execute<O: PineOutput>(
        &mut self,
        _ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let length = self.length as usize;
        let Some(values) = self.window.observe(self.source, length) else {
            return Ok(Value::Na);
        };
        let n = values.len();
        if n < 2 {
            return Ok(Value::Na);
        }
        // `values` is newest-first, so the newest bar carries the highest time rank.
        let ranks = average_ranks(&values);
        let sum_d2: f64 = ranks
            .iter()
            .enumerate()
            .map(|(i, &rank)| {
                let time_rank = (n - i) as f64;
                (rank - time_rank).powi(2)
            })
            .sum();
        let rho = 1.0 - 6.0 * sum_d2 / (n as f64 * ((n * n) as f64 - 1.0));
        Ok(Value::Number(rho * 100.0))
    }
}
