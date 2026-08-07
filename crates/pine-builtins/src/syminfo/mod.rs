use pine_core::{PineOutput, SymInfo};
use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Build the `syminfo` namespace object from host-supplied symbol information.
pub fn create_syminfo<O: PineOutput>(info: SymInfo) -> Value<O> {
    let mut members: HashMap<String, Value<O>> = HashMap::new();

    members.insert("ticker".to_string(), Value::String(info.ticker));
    members.insert("tickerid".to_string(), Value::String(info.tickerid));
    members.insert("description".to_string(), Value::String(info.description));
    members.insert("prefix".to_string(), Value::String(info.prefix));
    members.insert("currency".to_string(), Value::String(info.currency));
    members.insert("basecurrency".to_string(), Value::String(info.basecurrency));
    members.insert("type".to_string(), Value::String(info.type_));
    members.insert("mintick".to_string(), Value::Number(info.mintick));
    members.insert("pointvalue".to_string(), Value::Number(info.pointvalue));
    members.insert("timezone".to_string(), Value::String(info.timezone));
    members.insert("session".to_string(), Value::String(info.session));
    members.insert("root".to_string(), Value::String(info.root));
    members.insert(
        "current_contract".to_string(),
        Value::String(info.current_contract),
    );
    members.insert(
        "main_tickerid".to_string(),
        Value::String(info.main_tickerid),
    );
    members.insert("isin".to_string(), Value::String(info.isin));
    members.insert("country".to_string(), Value::String(info.country));
    members.insert("sector".to_string(), Value::String(info.sector));
    members.insert("industry".to_string(), Value::String(info.industry));
    members.insert("volumetype".to_string(), Value::String(info.volumetype));
    members.insert("minmove".to_string(), Value::Number(info.minmove));
    members.insert("pricescale".to_string(), Value::Number(info.pricescale));
    members.insert("mincontract".to_string(), Value::Number(info.mincontract));
    members.insert(
        "expiration_date".to_string(),
        Value::Number(info.expiration_date),
    );
    members.insert("employees".to_string(), Value::Number(info.employees));
    members.insert("shareholders".to_string(), Value::Number(info.shareholders));
    members.insert(
        "shares_outstanding_total".to_string(),
        Value::Number(info.shares_outstanding_total),
    );
    members.insert(
        "shares_outstanding_float".to_string(),
        Value::Number(info.shares_outstanding_float),
    );
    members.insert(
        "recommendations_buy".to_string(),
        Value::Number(info.recommendations_buy),
    );
    members.insert(
        "recommendations_buy_strong".to_string(),
        Value::Number(info.recommendations_buy_strong),
    );
    members.insert(
        "recommendations_hold".to_string(),
        Value::Number(info.recommendations_hold),
    );
    members.insert(
        "recommendations_sell".to_string(),
        Value::Number(info.recommendations_sell),
    );
    members.insert(
        "recommendations_sell_strong".to_string(),
        Value::Number(info.recommendations_sell_strong),
    );
    members.insert(
        "recommendations_total".to_string(),
        Value::Number(info.recommendations_total),
    );
    members.insert(
        "recommendations_date".to_string(),
        Value::Number(info.recommendations_date),
    );
    members.insert(
        "target_price_average".to_string(),
        Value::Number(info.target_price_average),
    );
    members.insert(
        "target_price_high".to_string(),
        Value::Number(info.target_price_high),
    );
    members.insert(
        "target_price_low".to_string(),
        Value::Number(info.target_price_low),
    );
    members.insert(
        "target_price_median".to_string(),
        Value::Number(info.target_price_median),
    );
    members.insert(
        "target_price_estimates".to_string(),
        Value::Number(info.target_price_estimates),
    );
    members.insert(
        "target_price_date".to_string(),
        Value::Number(info.target_price_date),
    );

    Value::Object {
        type_name: "syminfo".to_string(),
        fields: Rc::new(RefCell::new(members)),
        call: None,
    }
}
