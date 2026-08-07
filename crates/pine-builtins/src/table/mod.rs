//! The `table.*` namespace: a grid of cells overlaid on the chart.
//!
//! Mirrors the `box`/`line` namespaces — id-based create / mutate / delete over
//! the [`TableOutput`] sink. Also registers the `position.*` constants that
//! `table.new`'s first argument expects.

use pine_builtin_macro::BuiltinFunction;
use pine_core::{Color, PineOutput, Table, TableCell, TableOutput};
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// table.new(position, columns, rows, bgcolor, ...) - Creates a table
#[derive(BuiltinFunction)]
#[builtin(name = "table.new", output = TableOutput)]
struct TableNew {
    position: String,
    columns: f64,
    rows: f64,
    #[arg(default = None)]
    bgcolor: Option<Color>,
    #[arg(default = None)]
    frame_color: Option<Color>,
    #[arg(default = 0.0)]
    frame_width: f64,
    #[arg(default = None)]
    border_color: Option<Color>,
    #[arg(default = 0.0)]
    border_width: f64,
}

impl TableNew {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (
            &self.frame_color,
            self.frame_width,
            &self.border_color,
            self.border_width,
        );
        let table = Table {
            position: self.position.clone(),
            columns: self.columns.max(0.0) as usize,
            rows: self.rows.max(0.0) as usize,
            bgcolor: self.bgcolor.clone(),
            cells: HashMap::new(),
        };
        let id = ctx.output.add_table(table);
        Ok(Value::Number(id as f64))
    }
}

/// table.cell(table_id, column, row, text, ...) - Sets a cell's content
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell", output = TableOutput)]
struct TableCellFn {
    table_id: f64,
    column: f64,
    row: f64,
    #[arg(default = "")]
    text: String,
    #[arg(default = None)]
    text_color: Option<Color>,
    #[arg(default = None)]
    bgcolor: Option<Color>,
    #[arg(default = "")]
    text_size: String,
    #[arg(default = "")]
    text_halign: String,
    #[arg(default = "")]
    text_valign: String,
}

impl TableCellFn {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let cell = TableCell {
            text: self.text.clone(),
            text_color: self.text_color.clone(),
            bgcolor: self.bgcolor.clone(),
            text_size: self.text_size.clone(),
            text_halign: self.text_halign.clone(),
            text_valign: self.text_valign.clone(),
        };
        set_cell(ctx, self.table_id, self.column, self.row, cell)?;
        Ok(Value::Na)
    }
}

/// table.cell_set_text(table_id, column, row, text)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text", output = TableOutput)]
struct TableCellSetText {
    table_id: f64,
    column: f64,
    row: f64,
    text: String,
}

impl TableCellSetText {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let text = self.text.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.text = text;
        Ok(Value::Na)
    }
}

/// table.cell_set_bgcolor(table_id, column, row, color)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_bgcolor", output = TableOutput)]
struct TableCellSetBgcolor {
    table_id: f64,
    column: f64,
    row: f64,
    color: Color,
}

impl TableCellSetBgcolor {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let color = self.color.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.bgcolor = Some(color);
        Ok(Value::Na)
    }
}

/// table.cell_set_text_color(table_id, column, row, color)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_color", output = TableOutput)]
struct TableCellSetTextColor {
    table_id: f64,
    column: f64,
    row: f64,
    color: Color,
}

impl TableCellSetTextColor {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let color = self.color.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.text_color = Some(color);
        Ok(Value::Na)
    }
}

/// table.set_bgcolor(table_id, color)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_bgcolor", output = TableOutput)]
struct TableSetBgcolor {
    table_id: f64,
    color: Color,
}

impl TableSetBgcolor {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        table_mut(ctx, self.table_id)?.bgcolor = Some(self.color.clone());
        Ok(Value::Na)
    }
}

/// table.clear(table_id) - Removes all cells
#[derive(BuiltinFunction)]
#[builtin(name = "table.clear", output = TableOutput)]
struct TableClear {
    table_id: f64,
}

impl TableClear {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        table_mut(ctx, self.table_id)?.cells.clear();
        Ok(Value::Na)
    }
}

/// table.delete(table_id)
#[derive(BuiltinFunction)]
#[builtin(name = "table.delete", output = TableOutput)]
struct TableDelete {
    table_id: f64,
}

impl TableDelete {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        ctx.output.delete_table(self.table_id as usize);
        Ok(Value::Na)
    }
}

/// A mutable table by id, or a "not found" error.
fn table_mut<O: PineOutput + TableOutput>(
    ctx: &mut Interpreter<O>,
    id: f64,
) -> Result<&mut Table, RuntimeError> {
    let id = id as usize;
    ctx.output
        .get_table_mut(id)
        .ok_or_else(|| RuntimeError::TypeError(format!("Table with id {} not found", id)))
}

/// A mutable cell, inserting an empty one if it does not exist yet.
fn cell_mut<O: PineOutput + TableOutput>(
    ctx: &mut Interpreter<O>,
    id: f64,
    column: f64,
    row: f64,
) -> Result<&mut TableCell, RuntimeError> {
    let key = (column.max(0.0) as usize, row.max(0.0) as usize);
    Ok(table_mut(ctx, id)?.cells.entry(key).or_default())
}

fn set_cell<O: PineOutput + TableOutput>(
    ctx: &mut Interpreter<O>,
    id: f64,
    column: f64,
    row: f64,
    cell: TableCell,
) -> Result<(), RuntimeError> {
    let key = (column.max(0.0) as usize, row.max(0.0) as usize);
    table_mut(ctx, id)?.cells.insert(key, cell);
    Ok(())
}

/// The `table` namespace. `table.new`'s `position` argument is satisfied by the
/// `position.*` constants registered separately (see `constants::position`).
/// table.cell_set_text_size(table_id, column, row, text_size)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_size", output = TableOutput)]
struct TableCellSetTextSize {
    table_id: f64,
    column: f64,
    row: f64,
    text_size: String,
}

impl TableCellSetTextSize {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let text_size = self.text_size.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.text_size = text_size;
        Ok(Value::Na)
    }
}

/// table.cell_set_text_halign(table_id, column, row, text_halign)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_halign", output = TableOutput)]
struct TableCellSetTextHalign {
    table_id: f64,
    column: f64,
    row: f64,
    text_halign: String,
}

impl TableCellSetTextHalign {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let text_halign = self.text_halign.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.text_halign = text_halign;
        Ok(Value::Na)
    }
}

/// table.cell_set_text_valign(table_id, column, row, text_valign)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_valign", output = TableOutput)]
struct TableCellSetTextValign {
    table_id: f64,
    column: f64,
    row: f64,
    text_valign: String,
}

impl TableCellSetTextValign {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let text_valign = self.text_valign.clone();
        cell_mut(ctx, self.table_id, self.column, self.row)?.text_valign = text_valign;
        Ok(Value::Na)
    }
}

/// table.set_position(table_id, position)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_position", output = TableOutput)]
struct TableSetPosition {
    table_id: f64,
    position: String,
}

impl TableSetPosition {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let position = self.position.clone();
        table_mut(ctx, self.table_id)?.position = position;
        Ok(Value::Na)
    }
}

// The setters below adjust rendering hints the output model does not store
// (cell size, fonts, frame/border, merges). They validate the target exists and
// are otherwise no-ops, matching `table.new`, which likewise ignores those hints.

/// table.cell_set_width(table_id, column, row, width)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_width", output = TableOutput)]
struct TableCellSetWidth {
    table_id: f64,
    column: f64,
    row: f64,
    width: f64,
}

impl TableCellSetWidth {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = self.width;
        cell_mut(ctx, self.table_id, self.column, self.row)?;
        Ok(Value::Na)
    }
}

/// table.cell_set_height(table_id, column, row, height)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_height", output = TableOutput)]
struct TableCellSetHeight {
    table_id: f64,
    column: f64,
    row: f64,
    height: f64,
}

impl TableCellSetHeight {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = self.height;
        cell_mut(ctx, self.table_id, self.column, self.row)?;
        Ok(Value::Na)
    }
}

/// table.cell_set_text_font_family(table_id, column, row, text_font_family)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_font_family", output = TableOutput)]
struct TableCellSetTextFontFamily {
    table_id: f64,
    column: f64,
    row: f64,
    text_font_family: String,
}

impl TableCellSetTextFontFamily {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.text_font_family;
        cell_mut(ctx, self.table_id, self.column, self.row)?;
        Ok(Value::Na)
    }
}

/// table.cell_set_text_formatting(table_id, column, row, text_formatting)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_text_formatting", output = TableOutput)]
struct TableCellSetTextFormatting {
    table_id: f64,
    column: f64,
    row: f64,
    text_formatting: String,
}

impl TableCellSetTextFormatting {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.text_formatting;
        cell_mut(ctx, self.table_id, self.column, self.row)?;
        Ok(Value::Na)
    }
}

/// table.cell_set_tooltip(table_id, column, row, tooltip)
#[derive(BuiltinFunction)]
#[builtin(name = "table.cell_set_tooltip", output = TableOutput)]
struct TableCellSetTooltip {
    table_id: f64,
    column: f64,
    row: f64,
    tooltip: String,
}

impl TableCellSetTooltip {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.tooltip;
        cell_mut(ctx, self.table_id, self.column, self.row)?;
        Ok(Value::Na)
    }
}

/// table.merge_cells(table_id, start_column, start_row, end_column, end_row)
#[derive(BuiltinFunction)]
#[builtin(name = "table.merge_cells", output = TableOutput)]
struct TableMergeCells {
    table_id: f64,
    start_column: f64,
    start_row: f64,
    end_column: f64,
    end_row: f64,
}

impl TableMergeCells {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = (self.start_column, self.start_row, self.end_column, self.end_row);
        table_mut(ctx, self.table_id)?;
        Ok(Value::Na)
    }
}

/// table.set_border_color(table_id, border_color)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_border_color", output = TableOutput)]
struct TableSetBorderColor {
    table_id: f64,
    border_color: Color,
}

impl TableSetBorderColor {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.border_color;
        table_mut(ctx, self.table_id)?;
        Ok(Value::Na)
    }
}

/// table.set_border_width(table_id, border_width)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_border_width", output = TableOutput)]
struct TableSetBorderWidth {
    table_id: f64,
    border_width: f64,
}

impl TableSetBorderWidth {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = self.border_width;
        table_mut(ctx, self.table_id)?;
        Ok(Value::Na)
    }
}

/// table.set_frame_color(table_id, frame_color)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_frame_color", output = TableOutput)]
struct TableSetFrameColor {
    table_id: f64,
    frame_color: Color,
}

impl TableSetFrameColor {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = &self.frame_color;
        table_mut(ctx, self.table_id)?;
        Ok(Value::Na)
    }
}

/// table.set_frame_width(table_id, frame_width)
#[derive(BuiltinFunction)]
#[builtin(name = "table.set_frame_width", output = TableOutput)]
struct TableSetFrameWidth {
    table_id: f64,
    frame_width: f64,
}

impl TableSetFrameWidth {
    fn execute<O: PineOutput + TableOutput>(
        &self,
        ctx: &mut Interpreter<O>,
    ) -> Result<Value<O>, RuntimeError> {
        let _ = self.frame_width;
        table_mut(ctx, self.table_id)?;
        Ok(Value::Na)
    }
}

pub fn register<O: PineOutput + TableOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("new".to_string(), TableNew::builtin_value::<O>());
    members.insert("cell".to_string(), TableCellFn::builtin_value::<O>());
    members.insert(
        "cell_set_text".to_string(),
        TableCellSetText::builtin_value::<O>(),
    );
    members.insert(
        "cell_set_bgcolor".to_string(),
        TableCellSetBgcolor::builtin_value::<O>(),
    );
    members.insert(
        "cell_set_text_color".to_string(),
        TableCellSetTextColor::builtin_value::<O>(),
    );
    members.insert(
        "set_bgcolor".to_string(),
        TableSetBgcolor::builtin_value::<O>(),
    );
    members.insert("clear".to_string(), TableClear::builtin_value::<O>());
    members.insert("delete".to_string(), TableDelete::builtin_value::<O>());
    members.insert("cell_set_width".to_string(), TableCellSetWidth::builtin_value::<O>());
    members.insert("cell_set_height".to_string(), TableCellSetHeight::builtin_value::<O>());
    members.insert("cell_set_text_size".to_string(), TableCellSetTextSize::builtin_value::<O>());
    members.insert(
        "cell_set_text_halign".to_string(),
        TableCellSetTextHalign::builtin_value::<O>(),
    );
    members.insert(
        "cell_set_text_valign".to_string(),
        TableCellSetTextValign::builtin_value::<O>(),
    );
    members.insert(
        "cell_set_text_font_family".to_string(),
        TableCellSetTextFontFamily::builtin_value::<O>(),
    );
    members.insert(
        "cell_set_text_formatting".to_string(),
        TableCellSetTextFormatting::builtin_value::<O>(),
    );
    members.insert("cell_set_tooltip".to_string(), TableCellSetTooltip::builtin_value::<O>());
    members.insert("merge_cells".to_string(), TableMergeCells::builtin_value::<O>());
    members.insert("set_position".to_string(), TableSetPosition::builtin_value::<O>());
    members.insert("set_border_color".to_string(), TableSetBorderColor::builtin_value::<O>());
    members.insert("set_border_width".to_string(), TableSetBorderWidth::builtin_value::<O>());
    members.insert("set_frame_color".to_string(), TableSetFrameColor::builtin_value::<O>());
    members.insert("set_frame_width".to_string(), TableSetFrameWidth::builtin_value::<O>());
    members.insert("all".to_string(), Value::Array(Rc::new(RefCell::new(Vec::new()))));
    Value::Object {
        type_name: "table".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
