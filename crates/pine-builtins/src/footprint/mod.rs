//! The `footprint.*` / `volume_row.*` namespaces: accessors over the volume
//! footprint objects `request.footprint` produces. Without an order-flow feed
//! there are no footprints, so every accessor reads `na`.

use pine_core::{FootprintRow, PineOutput};
use pine_interpreter::{Builtin, EvaluatedArg, FunctionCallArgs, Interpreter, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn object<O: PineOutput>(type_name: &str, fields: HashMap<String, Value<O>>) -> Value<O> {
    Value::Object {
        type_name: type_name.to_string(),
        fields: Rc::new(RefCell::new(fields)),
        call: None,
        value: None,
    }
}

fn arg<O: PineOutput>(args: &FunctionCallArgs<O>, i: usize) -> Option<&Value<O>> {
    args.args.get(i).map(|a| match a {
        EvaluatedArg::Positional(v) => v,
        EvaluatedArg::Named { value, .. } => value,
    })
}

/// A `name(id)` accessor returning the `field` of the object passed as `id`
/// (`na` when `id` is not an object with that field).
fn field_reader<O: PineOutput>(field: &'static str) -> Value<O> {
    Value::BuiltinFunction(Builtin::untyped(Rc::new(
        move |_ctx: &mut Interpreter<O>, args: FunctionCallArgs<O>| {
            Ok(match arg(&args, 0) {
                Some(Value::Object { fields, .. }) => {
                    fields.borrow().get(field).cloned().unwrap_or(Value::Na)
                }
                _ => Value::Na,
            })
        },
    )))
}

fn volume_row_object<O: PineOutput>(row: &FootprintRow, has_buy: bool, has_sell: bool) -> Value<O> {
    let mut f: HashMap<String, Value<O>> = HashMap::new();
    f.insert("down_price".into(), Value::Number(row.down_price));
    f.insert("up_price".into(), Value::Number(row.up_price));
    f.insert("buy_volume".into(), Value::Number(row.buy_volume));
    f.insert("sell_volume".into(), Value::Number(row.sell_volume));
    f.insert(
        "total_volume".into(),
        Value::Number(row.buy_volume + row.sell_volume),
    );
    f.insert(
        "delta".into(),
        Value::Number(row.buy_volume - row.sell_volume),
    );
    f.insert("has_buy_imbalance".into(), Value::Bool(has_buy));
    f.insert("has_sell_imbalance".into(), Value::Bool(has_sell));
    object("volume_row", f)
}

/// The value-area bounds `(val, vah)`: expand from the POC row, always taking the
/// higher-volume neighbour, until `target` volume is covered.
fn value_area(rows: &[FootprintRow], poc: usize, target: f64) -> (usize, usize) {
    let vol = |i: usize| rows[i].buy_volume + rows[i].sell_volume;
    let mut covered = vol(poc);
    let (mut lo, mut hi) = (poc, poc);
    while covered < target && (lo > 0 || hi + 1 < rows.len()) {
        let below = (lo > 0).then(|| vol(lo - 1));
        let above = (hi + 1 < rows.len()).then(|| vol(hi + 1));
        match (below, above) {
            (Some(b), Some(a)) if b >= a => {
                lo -= 1;
                covered += b;
            }
            (Some(_), Some(a)) => {
                hi += 1;
                covered += a;
            }
            (Some(b), None) => {
                lo -= 1;
                covered += b;
            }
            (None, Some(a)) => {
                hi += 1;
                covered += a;
            }
            (None, None) => break,
        }
    }
    (lo, hi)
}

/// Build a `footprint` object from its rows (lowest price first): the aggregate
/// sums, the per-row `volume_row` objects, and the POC / value-area rows.
pub fn build_footprint<O: PineOutput>(
    rows: Vec<FootprintRow>,
    va_percent: f64,
    imbalance_percent: f64,
) -> Value<O> {
    let n = rows.len();
    let ratio = imbalance_percent / 100.0;
    let row_objs: Vec<Value<O>> = (0..n)
        .map(|i| {
            let r = &rows[i];
            // A buy imbalance stacks against the sell volume of the row below;
            // a sell imbalance against the buy volume of the row above.
            let has_buy = i > 0
                && rows[i - 1].sell_volume > 0.0
                && r.buy_volume >= rows[i - 1].sell_volume * ratio;
            let has_sell = i + 1 < n
                && rows[i + 1].buy_volume > 0.0
                && r.sell_volume >= rows[i + 1].buy_volume * ratio;
            volume_row_object(r, has_buy, has_sell)
        })
        .collect();

    let total_buy: f64 = rows.iter().map(|r| r.buy_volume).sum();
    let total_sell: f64 = rows.iter().map(|r| r.sell_volume).sum();
    let total = total_buy + total_sell;

    let poc = (0..n).max_by(|&a, &b| {
        let (va, vb) = (
            rows[a].buy_volume + rows[a].sell_volume,
            rows[b].buy_volume + rows[b].sell_volume,
        );
        va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let (val_idx, vah_idx) = match poc {
        Some(poc) => {
            let (lo, hi) = value_area(&rows, poc, total * va_percent / 100.0);
            (Some(lo), Some(hi))
        }
        None => (None, None),
    };
    let pick = |idx: Option<usize>| idx.map(|i| row_objs[i].clone()).unwrap_or(Value::Na);

    let mut f: HashMap<String, Value<O>> = HashMap::new();
    f.insert("buy_volume".into(), Value::Number(total_buy));
    f.insert("sell_volume".into(), Value::Number(total_sell));
    f.insert("total_volume".into(), Value::Number(total));
    f.insert("delta".into(), Value::Number(total_buy - total_sell));
    f.insert("poc".into(), pick(poc));
    f.insert("vah".into(), pick(vah_idx));
    f.insert("val".into(), pick(val_idx));
    f.insert("rows".into(), Value::Array(Rc::new(RefCell::new(row_objs))));
    object("footprint", f)
}

/// footprint.get_row_by_price(id, price) - The row whose price range contains
/// `price`, or `na`.
fn get_row_by_price<O: PineOutput>() -> Value<O> {
    Value::BuiltinFunction(Builtin::untyped(Rc::new(
        move |_ctx: &mut Interpreter<O>, args: FunctionCallArgs<O>| {
            let Some(price) = arg(&args, 1).and_then(|v| v.as_number().ok()) else {
                return Ok(Value::Na);
            };
            let Some(Value::Object { fields, .. }) = arg(&args, 0) else {
                return Ok(Value::Na);
            };
            let Some(Value::Array(rows)) = fields.borrow().get("rows").cloned() else {
                return Ok(Value::Na);
            };
            for row in rows.borrow().iter() {
                if let Value::Object { fields, .. } = row {
                    let fields = fields.borrow();
                    let read = |name| fields.get(name).and_then(|v| v.as_number().ok());
                    if let (Some(down), Some(up)) = (read("down_price"), read("up_price")) {
                        if price >= down && price <= up {
                            return Ok(row.clone());
                        }
                    }
                }
            }
            Ok(Value::Na)
        },
    )))
}

/// Register the `footprint.*` namespace object.
pub fn register_footprint<O: PineOutput>() -> Value<O> {
    let mut m: HashMap<String, Value<O>> = HashMap::new();
    for field in [
        "buy_volume",
        "sell_volume",
        "total_volume",
        "delta",
        "poc",
        "vah",
        "val",
        "rows",
    ] {
        m.insert(field.to_string(), field_reader(field));
    }
    m.insert("get_row_by_price".to_string(), get_row_by_price());
    object("footprint", m)
}

/// Register the `volume_row.*` namespace object.
pub fn register_volume_row<O: PineOutput>() -> Value<O> {
    let mut m: HashMap<String, Value<O>> = HashMap::new();
    for field in [
        "buy_volume",
        "sell_volume",
        "total_volume",
        "delta",
        "up_price",
        "down_price",
        "has_buy_imbalance",
        "has_sell_imbalance",
    ] {
        m.insert(field.to_string(), field_reader(field));
    }
    object("volume_row", m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_core::DefaultPineOutput;

    fn row(down: f64, up: f64, buy: f64, sell: f64) -> FootprintRow {
        FootprintRow {
            down_price: down,
            up_price: up,
            buy_volume: buy,
            sell_volume: sell,
        }
    }

    fn field(obj: &Value<DefaultPineOutput>, name: &str) -> Value<DefaultPineOutput> {
        match obj {
            Value::Object { fields, .. } => fields.borrow().get(name).cloned().unwrap(),
            _ => panic!("expected an object"),
        }
    }

    #[test]
    fn builds_aggregates_poc_and_imbalance() {
        let rows = vec![
            row(100.0, 101.0, 10.0, 50.0),
            row(101.0, 102.0, 60.0, 20.0),
            row(102.0, 103.0, 5.0, 5.0),
        ];
        let fp = build_footprint::<DefaultPineOutput>(rows, 70.0, 200.0);

        assert_eq!(field(&fp, "buy_volume").as_number().unwrap(), 75.0);
        assert_eq!(field(&fp, "delta").as_number().unwrap(), 0.0);

        // POC is the highest-volume row (101..102, total 80).
        let poc = field(&fp, "poc");
        assert_eq!(field(&poc, "up_price").as_number().unwrap(), 102.0);
        assert_eq!(field(&poc, "total_volume").as_number().unwrap(), 80.0);
        // Its sell (20) is at least 2× the buy (5) of the row above.
        assert!(matches!(
            field(&poc, "has_sell_imbalance"),
            Value::Bool(true)
        ));
    }
}
