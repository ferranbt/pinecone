//! The `table.*` namespace: a grid of cells overlaid on the chart.
//!
//! Mirrors the `box`/`line` namespaces — id-based create / mutate / delete over
//! the [`TableOutput`] sink. Also registers the `position.*` constants that
//! `table.new`'s first argument expects.

use pine_builtin_macro::BuiltinFunction;
use pine_interpreter::{
    Color, Interpreter, PineOutput, RuntimeError, Table, TableCell, TableOutput, Value,
};
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

/// The `position.*` constants `table.new` anchors to.
const POSITIONS: &[&str] = &[
    "top_left",
    "top_center",
    "top_right",
    "middle_left",
    "middle_center",
    "middle_right",
    "bottom_left",
    "bottom_center",
    "bottom_right",
];

/// The `table` namespace plus the `position` constants it depends on.
pub fn register<O: PineOutput + TableOutput>() -> Vec<(String, Value<O>)> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert(
        "new".to_string(),
        Value::BuiltinFunction(Rc::new(TableNew::builtin_fn::<O>)),
    );
    members.insert(
        "cell".to_string(),
        Value::BuiltinFunction(Rc::new(TableCellFn::builtin_fn::<O>)),
    );
    members.insert(
        "cell_set_text".to_string(),
        Value::BuiltinFunction(Rc::new(TableCellSetText::builtin_fn::<O>)),
    );
    members.insert(
        "cell_set_bgcolor".to_string(),
        Value::BuiltinFunction(Rc::new(TableCellSetBgcolor::builtin_fn::<O>)),
    );
    members.insert(
        "cell_set_text_color".to_string(),
        Value::BuiltinFunction(Rc::new(TableCellSetTextColor::builtin_fn::<O>)),
    );
    members.insert(
        "set_bgcolor".to_string(),
        Value::BuiltinFunction(Rc::new(TableSetBgcolor::builtin_fn::<O>)),
    );
    members.insert(
        "clear".to_string(),
        Value::BuiltinFunction(Rc::new(TableClear::builtin_fn::<O>)),
    );
    members.insert(
        "delete".to_string(),
        Value::BuiltinFunction(Rc::new(TableDelete::builtin_fn::<O>)),
    );
    let table_object = Value::Object {
        type_name: "table".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    };

    let mut position_members: HashMap<String, Value<O>> = HashMap::new();
    for pos in POSITIONS {
        position_members.insert(pos.to_string(), Value::String(pos.to_string()));
    }
    let position_object = Value::Object {
        type_name: "position".to_string(),
        fields: Rc::new(RefCell::new(position_members)),
        call: None,
    };

    vec![
        ("table".to_string(), table_object),
        ("position".to_string(), position_object),
    ]
}
