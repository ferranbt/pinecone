//! Bars read from a CSV file.

use crate::{DataError, DataSource};
use pine_core::{Data, Ohlcv, SymInfo};
use std::io::Read;
use std::path::{Path, PathBuf};

/// One row of a bar file. Column order does not matter — rows are matched to
/// these fields by the header names.
#[derive(serde::Deserialize)]
struct Row {
    /// Opening time as a UNIX timestamp in milliseconds.
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Bars loaded from a CSV file with `time,open,high,low,close,volume` columns,
/// oldest first.
///
/// A header row naming the columns is required. `#` comment lines are ignored.
/// The timeframe is inferred from how far apart the bars are.
#[derive(Debug, Clone)]
pub struct CsvSource {
    path: PathBuf,
    syminfo: SymInfo,
}

impl CsvSource {
    /// Read `path`, failing if it cannot be parsed. The symbol defaults to a
    /// placeholder; set it with [`CsvSource::with_syminfo`] when a script reads
    /// `syminfo.*`.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let source = Self {
            path: path.as_ref().to_path_buf(),
            syminfo: SymInfo::default(),
        };
        // Read eagerly so a malformed file is reported here rather than later.
        source.load()?;
        Ok(source)
    }

    /// The symbol this file's bars belong to, exposed to scripts as `syminfo.*`.
    pub fn with_syminfo(mut self, syminfo: SymInfo) -> Self {
        self.syminfo = syminfo;
        self
    }

    fn read(&self, source: impl Read) -> Result<Vec<Ohlcv>, csv::Error> {
        csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .trim(csv::Trim::All)
            .from_reader(source)
            .deserialize()
            .map(|row| {
                let row: Row = row?;
                Ok(Ohlcv {
                    time: row.time,
                    open: row.open,
                    high: row.high,
                    low: row.low,
                    close: row.close,
                    volume: row.volume,
                })
            })
            .collect()
    }
}

impl DataSource for CsvSource {
    fn load(&self) -> Result<Data, DataError> {
        let file = std::fs::File::open(&self.path).map_err(|source| DataError::Read {
            path: self.path.display().to_string(),
            source: source.into(),
        })?;

        let rows = self.read(file).map_err(|source| DataError::Read {
            path: self.path.display().to_string(),
            source,
        })?;

        Ok(Data::from_ohlcv(rows).with_syminfo(self.syminfo.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> CsvSource {
        CsvSource {
            path: PathBuf::from("bars.csv"),
            syminfo: SymInfo::default(),
        }
    }

    #[test]
    fn reads_rows_skipping_comments() {
        let rows = source()
            .read(
                "time,open,high,low,close,volume\n\
                 # first bar\n\
                 0,100,105,95,102,1000\n\
                 60000,101,106,96,103,1010\n"
                    .as_bytes(),
            )
            .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].time, 0);
        assert_eq!(rows[0].close, 102.0);
        assert_eq!(rows[1].time, 60000);
        assert_eq!(rows[1].volume, 1010.0);
    }

    #[test]
    fn columns_are_matched_by_header_not_position() {
        let rows = source()
            .read("volume,close,low,high,open,time\n1000,102,95,105,100,0\n".as_bytes())
            .unwrap();

        assert_eq!(rows[0].open, 100.0);
        assert_eq!(rows[0].close, 102.0);
        assert_eq!(rows[0].volume, 1000.0);
    }

    #[test]
    fn reports_where_a_bad_row_is() {
        let error = source()
            .read(
                "time,open,high,low,close,volume\n\
                 0,100,105,95,102,1000\n\
                 60000,101,106,96,oops,1010\n"
                    .as_bytes(),
            )
            .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("line: 3"), "{message}");
    }

    #[test]
    fn reports_a_missing_column() {
        let error = source()
            .read("time,open,high,low,close\n0,100,105,95,102\n".as_bytes())
            .unwrap_err();

        assert!(error.to_string().contains("volume"), "{error}");
    }
}
