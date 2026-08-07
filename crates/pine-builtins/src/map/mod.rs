//! The `map.*` namespace: an insertion-ordered key/value collection.

use pine_builtin_macro::BuiltinFunction;
use pine_core::PineOutput;
use pine_interpreter::{Interpreter, RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type MapData<O> = Rc<RefCell<Vec<(Value<O>, Value<O>)>>>;

/// The pairs of a `Value::Map`, or a type error.
fn as_map<O: PineOutput>(value: &Value<O>) -> Result<&MapData<O>, RuntimeError> {
    match value {
        Value::Map { data, .. } => Ok(data),
        _ => Err(RuntimeError::TypeError("expected a map".to_string())),
    }
}

/// map.new<keyType, valueType>() - An empty map.
#[derive(BuiltinFunction)]
#[builtin(name = "map.new", type_params = 2)]
struct MapNew {
    #[type_param]
    key_type: String,
    #[type_param]
    value_type: String,
}

impl MapNew {
    fn execute<O: PineOutput>(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Map {
            key_type: self.key_type.clone(),
            value_type: self.value_type.clone(),
            data: Rc::new(RefCell::new(Vec::new())),
        })
    }
}

/// map.get(id, key) - The value for `key`, or `na`.
#[derive(BuiltinFunction)]
#[builtin(name = "map.get")]
struct MapGet<O: PineOutput> {
    id: Value<O>,
    key: Value<O>,
}

impl<O: PineOutput> MapGet<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let data = as_map(&self.id)?.borrow();
        Ok(data
            .iter()
            .find(|(k, _)| *k == self.key)
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Na))
    }
}

/// map.put(id, key, value) - Insert or overwrite `key`, returning the previous
/// value (`na` if the key was new).
#[derive(BuiltinFunction)]
#[builtin(name = "map.put")]
struct MapPut<O: PineOutput> {
    id: Value<O>,
    key: Value<O>,
    value: Value<O>,
}

impl<O: PineOutput> MapPut<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut data = as_map(&self.id)?.borrow_mut();
        if let Some(pair) = data.iter_mut().find(|(k, _)| *k == self.key) {
            Ok(std::mem::replace(&mut pair.1, self.value.clone()))
        } else {
            data.push((self.key.clone(), self.value.clone()));
            Ok(Value::Na)
        }
    }
}

/// map.contains(id, key) - Whether `key` is present.
#[derive(BuiltinFunction)]
#[builtin(name = "map.contains")]
struct MapContains<O: PineOutput> {
    id: Value<O>,
    key: Value<O>,
}

impl<O: PineOutput> MapContains<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let data = as_map(&self.id)?.borrow();
        Ok(Value::Bool(data.iter().any(|(k, _)| *k == self.key)))
    }
}

/// map.remove(id, key) - Remove `key`, returning its value (`na` if absent).
#[derive(BuiltinFunction)]
#[builtin(name = "map.remove")]
struct MapRemove<O: PineOutput> {
    id: Value<O>,
    key: Value<O>,
}

impl<O: PineOutput> MapRemove<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let mut data = as_map(&self.id)?.borrow_mut();
        match data.iter().position(|(k, _)| *k == self.key) {
            Some(index) => Ok(data.remove(index).1),
            None => Ok(Value::Na),
        }
    }
}

/// map.keys(id) - An array of the keys, in insertion order.
#[derive(BuiltinFunction)]
#[builtin(name = "map.keys")]
struct MapKeys<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MapKeys<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let keys = as_map(&self.id)?
            .borrow()
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(keys))))
    }
}

/// map.values(id) - An array of the values, in insertion order.
#[derive(BuiltinFunction)]
#[builtin(name = "map.values")]
struct MapValues<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MapValues<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let values = as_map(&self.id)?
            .borrow()
            .iter()
            .map(|(_, v)| v.clone())
            .collect();
        Ok(Value::Array(Rc::new(RefCell::new(values))))
    }
}

/// map.size(id) - The number of key/value pairs.
#[derive(BuiltinFunction)]
#[builtin(name = "map.size")]
struct MapSize<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MapSize<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        Ok(Value::Int(as_map(&self.id)?.borrow().len() as i64))
    }
}

/// map.clear(id) - Remove all pairs.
#[derive(BuiltinFunction)]
#[builtin(name = "map.clear")]
struct MapClear<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MapClear<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        as_map(&self.id)?.borrow_mut().clear();
        Ok(Value::Na)
    }
}

/// map.copy(id) - A shallow copy of the map.
#[derive(BuiltinFunction)]
#[builtin(name = "map.copy")]
struct MapCopy<O: PineOutput> {
    id: Value<O>,
}

impl<O: PineOutput> MapCopy<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let (key_type, value_type) = match &self.id {
            Value::Map {
                key_type,
                value_type,
                ..
            } => (key_type.clone(), value_type.clone()),
            _ => return Err(RuntimeError::TypeError("expected a map".to_string())),
        };
        let data = as_map(&self.id)?.borrow().clone();
        Ok(Value::Map {
            key_type,
            value_type,
            data: Rc::new(RefCell::new(data)),
        })
    }
}

/// map.put_all(id, id2) - Copy every pair from `id2` into `id`.
#[derive(BuiltinFunction)]
#[builtin(name = "map.put_all")]
struct MapPutAll<O: PineOutput> {
    id: Value<O>,
    id2: Value<O>,
}

impl<O: PineOutput> MapPutAll<O> {
    fn execute(&self, _ctx: &mut Interpreter<O>) -> Result<Value<O>, RuntimeError> {
        let source = as_map(&self.id2)?.borrow().clone();
        let mut target = as_map(&self.id)?.borrow_mut();
        for (key, value) in source {
            if let Some(pair) = target.iter_mut().find(|(k, _)| *k == key) {
                pair.1 = value;
            } else {
                target.push((key, value));
            }
        }
        Ok(Value::Na)
    }
}

/// Register the `map.*` namespace object.
pub fn register<O: PineOutput>() -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();
    members.insert("new".to_string(), MapNew::builtin_value::<O>());
    members.insert("get".to_string(), MapGet::<O>::builtin_value());
    members.insert("put".to_string(), MapPut::<O>::builtin_value());
    members.insert("contains".to_string(), MapContains::<O>::builtin_value());
    members.insert("remove".to_string(), MapRemove::<O>::builtin_value());
    members.insert("keys".to_string(), MapKeys::<O>::builtin_value());
    members.insert("values".to_string(), MapValues::<O>::builtin_value());
    members.insert("size".to_string(), MapSize::<O>::builtin_value());
    members.insert("clear".to_string(), MapClear::<O>::builtin_value());
    members.insert("copy".to_string(), MapCopy::<O>::builtin_value());
    members.insert("put_all".to_string(), MapPutAll::<O>::builtin_value());
    Value::Object {
        type_name: "map".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
        value: None,
    }
}
