# Custom Builtin Functions Example

This example demonstrates how to extend PineVM with custom builtin functions and custom output types.

## What This Example Shows

1. **Custom Output Type**: How to create a custom output type that extends the default functionality
2. **Custom Traits**: How to define new traits for custom capabilities (e.g., `AlertOutput`)
3. **Custom Builtin Functions**: How to create custom functions callable from PineScript
4. **Integration**: How to integrate custom and standard functionality together

## The Custom Output

The example creates `CustomOutput` which:
- Embeds `DefaultPineOutput` to get all standard functionality (plots, labels, boxes, logs)
- Adds custom fields for alerts and trade signals
- Implements all required traits by delegating to the base implementation

```rust
#[derive(Default, Clone, Debug)]
pub struct CustomOutput {
    base: DefaultPineOutput,
    alerts: Vec<Alert>,
    signals: Vec<TradeSignal>,
}
```

## Custom Builtin Functions

The example implements three custom functions:

### `alert(condition, message)`
Triggers an alert with a condition name and message.

```pinescript
alert("price_high", "Price crossed above 100!")
```

### `buy(price, size)`
Records a buy signal at the given price and size.

```pinescript
buy(close, 1.0)
```

### `sell(price, size)`
Records a sell signal at the given price and size.

```pinescript
sell(close, 0.5)
```

## How It Works

### 1. Define Custom Output

Create a struct that embeds `DefaultPineOutput` and adds your custom fields:

```rust
#[derive(Default, Clone, Debug)]
pub struct CustomOutput {
    base: DefaultPineOutput,
    alerts: Vec<Alert>,
}
```

### 2. Implement Required Traits

Implement `PineOutput` and extension traits (LogOutput, PlotOutput, etc.):

```rust
impl PineOutput for CustomOutput {
    fn clear(&mut self) {
        self.base.clear();
        self.alerts.clear();
    }
}

// Delegate standard functionality to base
impl LogOutput for CustomOutput {
    fn add_log(&mut self, level: LogLevel, message: String) {
        self.base.add_log(level, message);
    }
    // ...
}
```

### 3. Define Custom Traits

Create custom traits for your extended functionality:

```rust
pub trait AlertOutput: PineOutput {
    fn add_alert(&mut self, alert: Alert);
    fn get_alerts(&self) -> &[Alert];
}
```

### 4. Create Custom Builtin Functions

Write functions that use your custom output:

```rust
fn create_alert_function<O: PineOutput + AlertOutput>() -> BuiltinFn<O> {
    Rc::new(|ctx: &mut Interpreter<O>, args: FunctionCallArgs<O>| {
        // Extract arguments
        let message = /* parse from args */;

        // Use custom output
        ctx.output.add_alert(Alert { message });

        Ok(Value::Na)
    })
}
```

### 5. Register Functions in Interpreter

```rust
let mut interpreter: Interpreter<CustomOutput> = Interpreter::new();
interpreter.set_const_variable("alert", Value::BuiltinFunction(create_alert_function()));
```

## Running the Example

```bash
cd examples/custom-builtin-func
cargo run
```

## Example Output

```
=== Custom Builtin Functions Example ===

Executing script across 5 bars...

Bar 1: close = 97.0

Bar 2: close = 101.0
  🔔 ALERT [price_high]: Price crossed above 100!
  📈 BUY signal at 101 (size: 1)

Bar 3: close = 103.0
  🔔 ALERT [price_high]: Price crossed above 100!
  📈 BUY signal at 103 (size: 1)

Bar 4: close = 89.0
  🔔 ALERT [price_low]: Price crossed below 90!
  📉 SELL signal at 89 (size: 1)

Bar 5: close = 91.0

=== Example Complete ===
```

## Key Concepts

### Type Safety
The generic system ensures that builtin functions can only be used with compatible output types. For example, `create_alert_function()` requires `O: PineOutput + AlertOutput`, ensuring it's only used with outputs that support alerts.

### Composition
By embedding `DefaultPineOutput`, you get all standard functionality (plots, labels, etc.) for free, while adding your own custom fields.

### Trait-Based Design
The trait-based architecture allows you to:
- Mix and match capabilities
- Create outputs that only implement what they need
- Write builtin functions that declare their requirements clearly

## Use Cases

This pattern is useful for:

1. **Trading Systems**: Add order management, position tracking, risk management
2. **Analytics**: Custom metrics, statistics, performance tracking
3. **Alerts & Notifications**: Custom alert conditions and delivery mechanisms
4. **Backtesting**: Track trades, P&L, drawdowns, etc.
5. **Visualization**: Custom chart annotations, overlays, indicators

## Extending Further

You can extend this pattern by:

- Adding more custom traits (`OrderOutput`, `RiskOutput`, etc.)
- Creating namespaces for custom functions (e.g., `strategy.buy()`, `risk.check()`)
- Implementing custom storage backends for output data
- Adding persistence/serialization for custom output types
- Creating plugin systems that dynamically load custom builtins
