KIRO RUST GLUE IMPLEMENTATION PLAN
=================================

GOAL
----
Provide zero-friction access to the Rust ecosystem from Kiro without:
- Rewriting Rust libraries
- Introducing inline Rust in .kiro files
- Breaking Kiro semantics
- Turning Kiro into a C-like FFI language

Rust code is authoritative. Kiro provides a typed contract.


PHASE 0 — CORE PRINCIPLES
------------------------
1. Kiro code must remain pure Kiro.
2. Rust code must live outside .kiro files.
3. Rust-backed functions are explicitly marked.
4. Type conversion is explicit and predictable.
5. Interpreter support is best-effort, compiler is authoritative.
6. No macros in the first iteration.


PHASE 1 — RUST FUNCTION DECLARATION IN KIRO
-------------------------------------------
1. Extend grammar to allow a `rust` keyword on function declarations.
2. Syntax mirrors normal Kiro functions:
   - Named parameters
   - Explicit return type
3. Function body is omitted or empty.
4. Return type must be a valid Kiro type (including error-marked types).

Purpose:
- Compiler knows function is externally implemented.
- Interpreter knows it cannot evaluate it normally.


PHASE 2 — RUST HEADER MODULE
----------------------------
1. Introduce a dedicated Rust file (e.g. header.rs).
2. This file contains all Rust glue functions callable from Kiro.
3. No inline Rust is allowed in .kiro files.
4. Each Rust glue function corresponds to one Kiro rust fn.

Purpose:
- Centralized Rust integration layer.
- Auditable and minimal surface area.


PHASE 3 — RUNTIME VALUE CONTRACT
--------------------------------
1. Define a single runtime value representation used by:
   - Interpreter
   - Compiler-generated Rust
   - Rust glue functions
2. Runtime value must represent:
   - All Kiro primitives
   - Collections
   - Errors
3. Rust glue functions accept and return runtime values.

Purpose:
- One universal boundary between Kiro and Rust.


PHASE 4 — EXPLICIT TYPE CONVERSION
----------------------------------
1. Define internal conversion rules:
   - Runtime value -> Rust type
   - Rust type -> Runtime value
2. Conversion failures must produce Kiro errors.
3. No implicit or lossy conversions.

Purpose:
- Prevent undefined behavior.
- Make failures explicit and debuggable.


PHASE 5 — RUST GLUE FUNCTION RULES
---------------------------------
1. Rust glue functions:
   - Receive runtime values as arguments.
   - Perform explicit conversion to Rust types.
   - Call Rust std or external crates.
   - Convert results back to runtime values.
2. Rust glue functions may return:
   - A runtime value (success)
   - A runtime error (failure)

Purpose:
- Full Rust ecosystem access.
- No compiler guesswork.


PHASE 6 — COMPILER INTEGRATION
------------------------------
1. When compiling a rust fn:
   - Do not generate a body.
   - Generate an external call stub.
2. Compiler enforces:
   - Parameter count
   - Parameter types
   - Declared return type
3. Compilation fails if:
   - Rust glue function is missing.
   - Types are incompatible at the boundary.

Purpose:
- Strong compile-time guarantees.


PHASE 7 — INTERPRETER BEHAVIOR
------------------------------
1. Interpreter does NOT attempt to emulate Rust logic.
2. Allowed behaviors:
   - Call Rust glue directly
   - Or raise a controlled "unavailable" error
3. Interpreter correctness is not required for Rust-backed functions.

Purpose:
- Keep interpreter useful for pure Kiro.
- Avoid blocking Rust integration.


PHASE 8 — ERROR INTEGRATION
---------------------------
1. Rust glue must convert Rust failures into Kiro error types.
2. Errors must integrate with existing:
   - on / error handling
3. Rust glue must never return void on error.

Purpose:
- Single unified error model.


PHASE 9 — LIBRARY AUTHOR WORKFLOW
---------------------------------
1. User adds a Rust crate dependency.
2. User writes minimal wrappers in header.rs.
3. Only required functions are exposed.
4. Kiro scripts can immediately use them.

Expected scale:
- Tens to hundreds of lines per library.
- Never thousands.

Purpose:
- Avoid rewriting Rust std or crates.

FINAL RESULT
------------
- Zero-friction Rust ecosystem access
- No C-style FFI
- No blind adr void usage
- No duplicated semantics
- Kiro stays minimal, expressive, and honest

# Rust Glue Example — File IO

This example demonstrates how Kiro accesses Rust’s standard library
with zero inline Rust, zero adapters per call, and full type safety.

---
## Rust Code (`header.rs`)
```rust
// This file is written by the Kiro runtime author or library integrator.
// It exposes Rust functionality to Kiro in a controlled, minimal way.

use std::fs;
use kiro_runtime::{RuntimeVal, KiroError};

pub fn read_file(args: Vec<RuntimeVal>) -> Result<RuntimeVal, KiroError> {
    let path = args[0].as_str()?;          // RuntimeVal -> &str

    match fs::read_to_string(path) {
        Ok(content) => Ok(RuntimeVal::Str(content)),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(KiroError::new("NotFound"))
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                Err(KiroError::new("PermissionDenied"))
            } else {
                Err(KiroError::new("Unknown"))
            }
        }
    }
}

```
## Kiro Code (`main.kiro`)

```kiro
error NotFound = "File not found"
error PermissionDenied = "Access denied"

rust fn read_file(path: str) -> str!

fn main() {
    var content = read_file("data.txt")

    on (content) {
        print "File contents:"
        print content
    } error NotFound {
        print "The file does not exist."
    } error PermissionDenied {
        print "You do not have permission to read this file."
    } error {
        print "Unknown error."
    }
}

main()
