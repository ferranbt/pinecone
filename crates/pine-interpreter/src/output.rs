// Output-related types, traits, and implementations

use std::collections::HashMap;

/// Represents a color with RGBA components
#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub r: u8, // Red component (0-255)
    pub g: u8, // Green component (0-255)
    pub b: u8, // Blue component (0-255)
    pub t: u8, // Transparency (0-100)
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, t: u8) -> Self {
        Color { r, g, b, t }
    }
}

/// Represents a label drawable object
#[derive(Clone, Debug)]
pub struct Label {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub xloc: String,
    pub yloc: String,
    pub color: Option<Color>,
    pub style: String,
    pub textcolor: Option<Color>,
    pub size: String,
    pub textalign: String,
    pub tooltip: Option<String>,
    pub text_font_family: String,
}

/// Represents a box drawable object
#[derive(Clone, Debug)]
pub struct PineBox {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub border_color: Option<Color>,
    pub border_width: f64,
    pub border_style: String,
    pub extend: String,
    pub xloc: String,
    pub bgcolor: Option<Color>,
    pub text: String,
    pub text_size: f64,
    pub text_color: Option<Color>,
    pub text_halign: String,
    pub text_valign: String,
    pub text_wrap: String,
    pub text_font_family: String,
}

/// Represents a plot output
#[derive(Clone, Debug)]
pub struct Plot {
    pub series: f64,
    pub title: String,
    pub color: Option<Color>,
    pub linewidth: f64,
    pub style: String,
    pub trackprice: bool,
    pub histbase: f64,
    pub offset: f64,
    pub join: bool,
    pub editable: bool,
    pub show_last: Option<f64>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
    pub linestyle: String,
}

/// Represents a plotarrow output
#[derive(Clone, Debug)]
pub struct Plotarrow {
    pub series: f64,
    pub title: String,
    pub colorup: Option<Color>,
    pub colordown: Option<Color>,
    pub offset: f64,
    pub minheight: f64,
    pub maxheight: f64,
    pub editable: bool,
    pub show_last: Option<f64>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
}

/// Represents a plotbar output
#[derive(Clone, Debug)]
pub struct Plotbar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub title: String,
    pub color: Option<Color>,
    pub editable: bool,
    pub show_last: Option<f64>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
}

/// Represents a plotcandle output
#[derive(Clone, Debug)]
pub struct Plotcandle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub title: String,
    pub color: Option<Color>,
    pub wickcolor: Option<Color>,
    pub editable: bool,
    pub show_last: Option<f64>,
    pub bordercolor: Option<Color>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
}

/// Represents a plotchar output
#[derive(Clone, Debug)]
pub struct Plotchar {
    pub series: f64,
    pub title: String,
    pub char: String,
    pub location: String,
    pub color: Option<Color>,
    pub offset: f64,
    pub text: String,
    pub textcolor: Option<Color>,
    pub editable: bool,
    pub size: String,
    pub show_last: Option<f64>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
}

/// Represents a plotshape output
#[derive(Clone, Debug)]
pub struct Plotshape {
    pub series: f64,
    pub title: String,
    pub style: String,
    pub location: String,
    pub color: Option<Color>,
    pub offset: f64,
    pub text: String,
    pub textcolor: Option<Color>,
    pub editable: bool,
    pub size: String,
    pub show_last: Option<f64>,
    pub display: String,
    pub format: Option<String>,
    pub precision: Option<f64>,
    pub force_overlay: bool,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

/// A log entry with level and message
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

/// Base trait for all output implementations
///
/// This trait defines the minimal contract that all output types must implement.
/// Extension traits (LogOutput, PlotOutput, etc.) add additional capabilities.
pub trait PineOutput: Default + Clone + std::fmt::Debug {
    /// Clear all output data for a new iteration
    fn clear(&mut self);
}

/// Extension trait for logging output
pub trait LogOutput: PineOutput {
    /// Add a log entry with the given level and message
    fn add_log(&mut self, level: LogLevel, message: String);
    /// Get all log entries
    fn get_logs(&self) -> &[LogEntry];
}

/// Macro to easily implement all output traits by delegating to a base field
///
/// # Example
/// ```ignore
/// impl_output_traits_delegate!(CustomOutput, base);
/// ```
#[macro_export]
macro_rules! impl_output_traits_delegate {
    ($type:ty, $field:ident) => {
        impl $crate::LogOutput for $type {
            fn add_log(&mut self, level: $crate::LogLevel, message: String) {
                self.$field.add_log(level, message)
            }
            fn get_logs(&self) -> &[$crate::LogEntry] {
                self.$field.get_logs()
            }
        }

        impl $crate::PlotOutput for $type {
            fn add_plot(&mut self, plot: $crate::Plot) {
                self.$field.add_plot(plot)
            }
            fn plots(&self) -> &[$crate::Plot] {
                self.$field.plots()
            }
            fn add_plotarrow(&mut self, arrow: $crate::Plotarrow) {
                self.$field.add_plotarrow(arrow)
            }
            fn plotarrows(&self) -> &[$crate::Plotarrow] {
                self.$field.plotarrows()
            }
            fn add_plotbar(&mut self, bar: $crate::Plotbar) {
                self.$field.add_plotbar(bar)
            }
            fn plotbars(&self) -> &[$crate::Plotbar] {
                self.$field.plotbars()
            }
            fn add_plotcandle(&mut self, candle: $crate::Plotcandle) {
                self.$field.add_plotcandle(candle)
            }
            fn plotcandles(&self) -> &[$crate::Plotcandle] {
                self.$field.plotcandles()
            }
            fn add_plotchar(&mut self, char: $crate::Plotchar) {
                self.$field.add_plotchar(char)
            }
            fn plotchars(&self) -> &[$crate::Plotchar] {
                self.$field.plotchars()
            }
            fn add_plotshape(&mut self, shape: $crate::Plotshape) {
                self.$field.add_plotshape(shape)
            }
            fn plotshapes(&self) -> &[$crate::Plotshape] {
                self.$field.plotshapes()
            }
        }

        impl $crate::LabelOutput for $type {
            fn add_label(&mut self, label: $crate::Label) -> usize {
                self.$field.add_label(label)
            }
            fn get_label(&self, id: usize) -> Option<&$crate::Label> {
                self.$field.get_label(id)
            }
            fn get_label_mut(&mut self, id: usize) -> Option<&mut $crate::Label> {
                self.$field.get_label_mut(id)
            }
            fn delete_label(&mut self, id: usize) -> bool {
                self.$field.delete_label(id)
            }
        }

        impl $crate::BoxOutput for $type {
            fn add_box(&mut self, box_obj: $crate::PineBox) -> usize {
                self.$field.add_box(box_obj)
            }
            fn get_box(&self, id: usize) -> Option<&$crate::PineBox> {
                self.$field.get_box(id)
            }
            fn get_box_mut(&mut self, id: usize) -> Option<&mut $crate::PineBox> {
                self.$field.get_box_mut(id)
            }
            fn delete_box(&mut self, id: usize) -> bool {
                self.$field.delete_box(id)
            }
        }
    };
}

/// Extension trait for plot-related output
pub trait PlotOutput: PineOutput {
    /// Add a plot output
    fn add_plot(&mut self, plot: Plot);
    /// Get all plot outputs
    fn plots(&self) -> &[Plot];

    /// Add a plotarrow output
    fn add_plotarrow(&mut self, arrow: Plotarrow);
    /// Get all plotarrow outputs
    fn plotarrows(&self) -> &[Plotarrow];

    /// Add a plotbar output
    fn add_plotbar(&mut self, bar: Plotbar);
    /// Get all plotbar outputs
    fn plotbars(&self) -> &[Plotbar];

    /// Add a plotcandle output
    fn add_plotcandle(&mut self, candle: Plotcandle);
    /// Get all plotcandle outputs
    fn plotcandles(&self) -> &[Plotcandle];

    /// Add a plotchar output
    fn add_plotchar(&mut self, char: Plotchar);
    /// Get all plotchar outputs
    fn plotchars(&self) -> &[Plotchar];

    /// Add a plotshape output
    fn add_plotshape(&mut self, shape: Plotshape);
    /// Get all plotshape outputs
    fn plotshapes(&self) -> &[Plotshape];
}

/// Extension trait for label output
pub trait LabelOutput: PineOutput {
    /// Add a label and return its ID
    fn add_label(&mut self, label: Label) -> usize;
    /// Get a reference to a label by ID
    fn get_label(&self, id: usize) -> Option<&Label>;
    /// Get a mutable reference to a label by ID
    fn get_label_mut(&mut self, id: usize) -> Option<&mut Label>;
    /// Delete a label by ID and return true if it existed
    fn delete_label(&mut self, id: usize) -> bool;
}

/// Extension trait for box output
pub trait BoxOutput: PineOutput {
    /// Add a box and return its ID
    fn add_box(&mut self, box_obj: PineBox) -> usize;
    /// Get a reference to a box by ID
    fn get_box(&self, id: usize) -> Option<&PineBox>;
    /// Get a mutable reference to a box by ID
    fn get_box_mut(&mut self, id: usize) -> Option<&mut PineBox>;
    /// Delete a box by ID and return true if it existed
    fn delete_box(&mut self, id: usize) -> bool;
}

/// Default implementation of PineOutput that supports all features
#[derive(Default, Clone, Debug)]
pub struct DefaultPineOutput {
    /// Label storage for drawable objects
    labels: HashMap<usize, Label>,
    /// Next label ID
    next_label_id: usize,
    /// Box storage for drawable objects
    boxes: HashMap<usize, PineBox>,
    /// Next box ID
    next_box_id: usize,
    /// Plot outputs
    plots: Vec<Plot>,
    /// Plotarrow outputs
    plotarrows: Vec<Plotarrow>,
    /// Plotbar outputs
    plotbars: Vec<Plotbar>,
    /// Plotcandle outputs
    plotcandles: Vec<Plotcandle>,
    /// Plotchar outputs
    plotchars: Vec<Plotchar>,
    /// Plotshape outputs
    plotshapes: Vec<Plotshape>,
    /// Log entries
    logs: Vec<LogEntry>,
}

impl PineOutput for DefaultPineOutput {
    fn clear(&mut self) {
        self.labels.clear();
        self.boxes.clear();
        self.plots.clear();
        self.plotarrows.clear();
        self.plotbars.clear();
        self.plotcandles.clear();
        self.plotchars.clear();
        self.plotshapes.clear();
        self.logs.clear();
        // Reset ID counters
        self.next_label_id = 0;
        self.next_box_id = 0;
    }
}

impl LogOutput for DefaultPineOutput {
    fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push(LogEntry { level, message });
    }

    fn get_logs(&self) -> &[LogEntry] {
        &self.logs
    }
}

impl PlotOutput for DefaultPineOutput {
    fn add_plot(&mut self, plot: Plot) {
        self.plots.push(plot);
    }

    fn plots(&self) -> &[Plot] {
        &self.plots
    }

    fn add_plotarrow(&mut self, arrow: Plotarrow) {
        self.plotarrows.push(arrow);
    }

    fn plotarrows(&self) -> &[Plotarrow] {
        &self.plotarrows
    }

    fn add_plotbar(&mut self, bar: Plotbar) {
        self.plotbars.push(bar);
    }

    fn plotbars(&self) -> &[Plotbar] {
        &self.plotbars
    }

    fn add_plotcandle(&mut self, candle: Plotcandle) {
        self.plotcandles.push(candle);
    }

    fn plotcandles(&self) -> &[Plotcandle] {
        &self.plotcandles
    }

    fn add_plotchar(&mut self, char: Plotchar) {
        self.plotchars.push(char);
    }

    fn plotchars(&self) -> &[Plotchar] {
        &self.plotchars
    }

    fn add_plotshape(&mut self, shape: Plotshape) {
        self.plotshapes.push(shape);
    }

    fn plotshapes(&self) -> &[Plotshape] {
        &self.plotshapes
    }
}

impl LabelOutput for DefaultPineOutput {
    fn add_label(&mut self, label: Label) -> usize {
        let id = self.next_label_id;
        self.next_label_id += 1;
        self.labels.insert(id, label);
        id
    }

    fn get_label(&self, id: usize) -> Option<&Label> {
        self.labels.get(&id)
    }

    fn get_label_mut(&mut self, id: usize) -> Option<&mut Label> {
        self.labels.get_mut(&id)
    }

    fn delete_label(&mut self, id: usize) -> bool {
        self.labels.remove(&id).is_some()
    }
}

impl BoxOutput for DefaultPineOutput {
    fn add_box(&mut self, box_obj: PineBox) -> usize {
        let id = self.next_box_id;
        self.next_box_id += 1;
        self.boxes.insert(id, box_obj);
        id
    }

    fn get_box(&self, id: usize) -> Option<&PineBox> {
        self.boxes.get(&id)
    }

    fn get_box_mut(&mut self, id: usize) -> Option<&mut PineBox> {
        self.boxes.get_mut(&id)
    }

    fn delete_box(&mut self, id: usize) -> bool {
        self.boxes.remove(&id).is_some()
    }
}
