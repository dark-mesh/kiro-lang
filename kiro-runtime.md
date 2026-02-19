# kiro_runtime

`kiro_runtime` is a small Rust crate that defines shared runtime data structures used at the Kiro <-> Rust boundary.

It exists to keep host glue behavior consistent and avoid re-defining value conversion logic in every Rust integration point.

## What This Crate Provides

### Runtime Value Model

`RuntimeVal` represents Kiro values in Rust:

- `Num(f64)`
- `Str(String)`
- `Bool(bool)`
- `List(Vec<RuntimeVal>)`
- `Map(HashMap<String, RuntimeVal>)`
- `Void`

### Error Type

`KiroError` is a minimal error type for glue-level failures.

- `KiroError::new("TypeError")` style creation
- `Display` + `std::error::Error` implemented

### Conversions

The crate provides conversion helpers both directions:

From Rust to `RuntimeVal`:

- `f64`, `String`, `&str`, `bool`, `()`
- `Vec<T>` where `T: Into<RuntimeVal>`

From `RuntimeVal` to Rust:

- `TryFrom<RuntimeVal>` for `String`, `f64`, `bool`, `()`, `Vec<String>`
- Accessor helpers:
  - `RuntimeVal::as_str()`
  - `RuntimeVal::as_num()`
  - `RuntimeVal::as_bool()`

## Why It Exists

Without a shared runtime crate, host modules often duplicate:

- Value enum definitions
- Type conversion logic
- Basic error conventions

`kiro_runtime` gives one place to evolve those contracts.

## Basic Example

```rust
use kiro_runtime::{KiroError, RuntimeVal};
use std::convert::TryFrom;

fn roundtrip() -> Result<(), KiroError> {
    let raw = RuntimeVal::from(42.0);
    let n = f64::try_from(raw)?;

    let text = RuntimeVal::from("hello");
    let s = String::try_from(text)?;

    let _ = (n, s);
    Ok(())
}
```

## Integration Pattern (Host Glue)

Typical glue flow:

1. Receive Kiro values as `RuntimeVal`.
2. Convert to Rust types with `TryFrom` or `as_*` helpers.
3. Run host logic.
4. Convert result back into `RuntimeVal`.

This keeps glue explicit and type-checked.

## Current Limitations

This crate intentionally stays small. Depending on language evolution, you may later extend it with:

- Struct-like runtime records with typed metadata
- Richer list/map conversion helpers
- Result/error envelope helpers
- Runtime representations for function refs, pipes, and managed address handles

## Versioning Notes

Because this crate encodes host boundary contracts, changes should be treated carefully:

- Prefer additive changes.
- Keep conversion behavior stable.
- Coordinate updates with compiler/interpreter changes.

## License

MIT
