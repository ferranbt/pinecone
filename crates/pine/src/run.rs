//! The result of replaying a script over a whole series of bars.

use pine_interpreter::{
    AlertCondition, AlertConditionOutput, Indicator, IndicatorOutput, Input, InputOutput, LogEntry,
    LogOutput, Plot, PlotOutput,
};
use std::collections::BTreeMap;

/// A run's per-bar outputs turned into columns.
///
/// Drawings are missing because the output traits expose labels, lines and
/// boxes only by id, so there is no way to enumerate what a bar created.
#[derive(Debug, Clone, Default)]
pub struct RunResult {
    pub bars: usize,
    /// Plotted values by title, one slot per bar; `None` where the plot was na.
    pub plots: BTreeMap<String, Vec<Option<f64>>>,
    pub logs: Vec<LogEntry>,
    pub alerts: Vec<AlertCondition>,
    pub indicator: Option<Indicator>,
    pub inputs: Vec<Input>,
}

impl RunResult {
    /// Transpose the per-bar outputs [`crate::Script::run`] returns.
    pub fn collect<O>(outputs: &[O]) -> Self
    where
        O: PlotOutput + LogOutput + AlertConditionOutput + IndicatorOutput + InputOutput,
    {
        let mut result = Self::default();

        for output in outputs {
            result.push_bar(output.plots());
            result.logs.extend(output.get_logs().iter().cloned());
        }

        // These describe the script, not a bar, so the last word wins.
        if let Some(last) = outputs.last() {
            result.alerts = last.alertconditions().to_vec();
            result.inputs = last.inputs().to_vec();
            result.indicator = last.indicator().cloned();
        }

        result
    }

    /// Append one bar, padding every column so titles stay aligned whether a
    /// plot starts late or stops early.
    fn push_bar(&mut self, plots: &[Plot]) {
        for plot in plots {
            let column = self.plots.entry(plot.title.clone()).or_default();
            column.resize(self.bars, None);
            column.push((!plot.series.is_nan()).then_some(plot.series));
        }

        self.bars += 1;

        for column in self.plots.values_mut() {
            column.resize(self.bars, None);
        }
    }

    /// The values plotted under `title`, or `None` if nothing plotted it.
    pub fn plot(&self, title: &str) -> Option<&[Option<f64>]> {
        self.plots.get(title).map(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plot(title: &str, series: f64) -> Plot {
        Plot {
            series,
            title: title.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn columns_line_up_with_bars() {
        let mut run = RunResult::default();
        run.push_bar(&[plot("a", 1.0)]);
        run.push_bar(&[plot("a", 2.0)]);

        assert_eq!(run.bars, 2);
        assert_eq!(run.plot("a"), Some([Some(1.0), Some(2.0)].as_slice()));
    }

    #[test]
    fn na_becomes_a_gap() {
        let mut run = RunResult::default();
        run.push_bar(&[plot("a", f64::NAN)]);
        run.push_bar(&[plot("a", 2.0)]);

        assert_eq!(run.plot("a"), Some([None, Some(2.0)].as_slice()));
    }

    #[test]
    fn a_plot_appearing_late_is_padded_at_the_front() {
        let mut run = RunResult::default();
        run.push_bar(&[plot("a", 1.0)]);
        run.push_bar(&[plot("a", 2.0), plot("b", 9.0)]);

        assert_eq!(run.plot("a"), Some([Some(1.0), Some(2.0)].as_slice()));
        assert_eq!(run.plot("b"), Some([None, Some(9.0)].as_slice()));
    }

    #[test]
    fn a_plot_that_stops_is_padded_at_the_end() {
        let mut run = RunResult::default();
        run.push_bar(&[plot("a", 1.0)]);
        run.push_bar(&[]);

        assert_eq!(run.plot("a"), Some([Some(1.0), None].as_slice()));
        assert_eq!(run.bars, 2);
    }

    #[test]
    fn an_unplotted_title_is_absent() {
        let run = RunResult::default();
        assert!(run.plot("nope").is_none());
    }
}
