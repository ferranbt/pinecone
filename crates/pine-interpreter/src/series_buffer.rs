//! A rolling window of a series' past values, for stateful builtins.
//!
//! A builtin declares one as a `#[state]` field, so the buffer is owned by the
//! call site and survives across bars. It records one value per bar (the macro's
//! once-per-bar guard makes sure of that) and keeps only as many as the caller
//! asks for, so a `ta.sma(close, 20)` call site holds 20 values rather than the
//! whole chart.

use std::collections::VecDeque;

/// The most values any single call site retains. Matches the lookback Pine
/// allows, and bounds memory when a script asks for an absurd length.
pub const MAX_LOOKBACK: usize = 5000;

/// A capped window of past values, newest first.
#[derive(Debug, Clone)]
pub struct SeriesBuffer<T> {
    /// Newest first: `values[0]` is the current bar.
    values: VecDeque<T>,
}

impl<T> Default for SeriesBuffer<T> {
    fn default() -> Self {
        Self {
            values: VecDeque::new(),
        }
    }
}

impl<T> SeriesBuffer<T> {
    /// Record this bar's value, retaining at most `capacity` of them.
    pub fn push(&mut self, value: T, capacity: usize) {
        self.values.push_front(value);
        let capacity = capacity.clamp(1, MAX_LOOKBACK);
        while self.values.len() > capacity {
            self.values.pop_back();
        }
    }

    /// The value `offset` bars back, or `None` if the buffer has not seen that
    /// many bars yet. `offset` 0 is the current bar.
    pub fn get(&self, offset: usize) -> Option<&T> {
        self.values.get(offset)
    }

    /// How many bars are retained.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The retained values, newest first.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }
}

impl SeriesBuffer<f64> {
    /// The newest `n` values, newest first. Shorter than `n` while warming up.
    pub fn window(&self, n: usize) -> Vec<f64> {
        self.values.iter().take(n).copied().collect()
    }

    /// Record this bar's value and return the `length` values to compute over,
    /// newest first — or `None` while fewer than `length` bars have been seen.
    ///
    /// Pine yields na until a series has enough history to answer, so warming
    /// up is the buffer's business rather than each builtin's.
    pub fn observe(&mut self, value: f64, length: usize) -> Option<Vec<f64>> {
        self.push(value, length);
        (self.len() >= length).then(|| self.window(length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_newest_values_up_to_capacity() {
        let mut buf = SeriesBuffer::default();
        for value in [1.0, 2.0, 3.0, 4.0] {
            buf.push(value, 3);
        }

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.window(3), vec![4.0, 3.0, 2.0]);
        assert_eq!(buf.get(0), Some(&4.0));
        assert_eq!(buf.get(2), Some(&2.0));
        assert_eq!(buf.get(3), None);
    }

    #[test]
    fn window_is_short_while_warming_up() {
        let mut buf = SeriesBuffer::default();
        buf.push(1.0, 5);
        buf.push(2.0, 5);

        assert_eq!(buf.window(5), vec![2.0, 1.0]);
    }

    #[test]
    fn observe_withholds_a_window_until_it_fills() {
        let mut buf = SeriesBuffer::default();

        assert_eq!(buf.observe(1.0, 3), None);
        assert_eq!(buf.observe(2.0, 3), None);
        assert_eq!(buf.observe(3.0, 3), Some(vec![3.0, 2.0, 1.0]));
        assert_eq!(buf.observe(4.0, 3), Some(vec![4.0, 3.0, 2.0]));
    }

    #[test]
    fn capacity_is_bounded_by_max_lookback() {
        let mut buf = SeriesBuffer::default();
        for value in 0..(MAX_LOOKBACK + 10) {
            buf.push(value as f64, usize::MAX);
        }

        assert_eq!(buf.len(), MAX_LOOKBACK);
    }
}
