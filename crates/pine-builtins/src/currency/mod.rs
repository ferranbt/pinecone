use pine_interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register the currency namespace with all currency constants
pub fn register() -> Value {
    let mut members = HashMap::new();

    // All currency constants as string values
    let currencies = [
        "AED", "ARS", "AUD", "BDT", "BHD", "BRL", "BTC", "CAD", "CHF", "CLP",
        "CNY", "COP", "CZK", "DKK", "EGP", "ETH", "EUR", "GBP", "HKD", "HUF",
        "IDR", "ILS", "INR", "ISK", "JPY", "KES", "KRW", "KWD", "LKR", "MAD",
        "MXN", "MYR", "NGN", "NOK", "NONE", "NZD", "PEN", "PHP", "PKR", "PLN",
        "QAR", "RON", "RSD", "RUB", "SAR", "SEK", "SGD", "THB", "TND", "TRY",
        "TWD", "USD", "USDT", "VES", "VND", "ZAR",
    ];

    for currency in currencies {
        members.insert(currency.to_string(), Value::String(currency.to_string()));
    }

    Value::Object {
        type_name: "currency".to_string(),
        fields: Rc::new(RefCell::new(members)),
    }
}
