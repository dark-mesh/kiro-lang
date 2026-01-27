# 🥝 Kiro Language

**Kiro** is a modern, experimental programming language designed to explore the intersection of high-level expressivity and low-level performance concepts. It features a dual-execution model (Interpreter + Transpiler to Rust) and introduces unique syntax for flow control and concurrency.

## 🚀 Key Features

- **Safe by Default**: Variables are immutable unless explicitly declared with `var`.
- **Expressive Loops**: Powerful `loop` constructs with built-in filtering (`on`) and stepping (`per`).
- **Pointers Made Easy**: Simple `ref` and `deref` syntax that compiles to safe Rust concurrency primitives (`Arc<Mutex>`).
- **Async First**: Built-in `run` keyword to easily spawn asynchronous tasks.
- **Transpiled to Rust**: Code is compiled to robust Rust code, benefiting from Rust's ecosystem and performance.

---

## 📦 Installation & Usage

Value your time? Just clone and run.

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) installed.

### Running a Program

1. Create a file named `main.kiro` in the project root.
2. Run the compiler/interpreter setup:
   ```bash
   cargo run
   ```
   This will:
   - Interpret the code immediately for feedback.
   - Transpile the code to Rust in `kiro_build_cache/`.
   - Compile and execute the transpiled Rust binary.

---

## 📚 Syntax Guide

### 1. Variables & Types

Kiro uses a "sane default" approach to mutability.

- **Immutable Declaration**: Omit `var` to create a constant.
  ```kiro
  pi = 3.14      // Immutable
  name = "Kiro"  // Immutable
  ```
- **Mutable Declaration**: Use `var` to allow changes.
  ```kiro
  var count = 0
  count = count + 1 // OK
  ```

**Supported Types:**

- `num`: 64-bit Floating point numbers (e.g., `3.14`, `42`).
- `str`: Strings (e.g., `"Hello"`).
- `bool`: Booleans (`true`, `false`).
- `adr`: Addresses/Pointers.

### 2. Control Flow

#### Conditionals (`on` / `off`)

Instead of `if/else`, Kiro uses `on` (condition) and `off` (else).

```kiro
var temperature = 30
on (temperature > 25) {
    print "It's hot!"
} off {
    print "It's pleasant."
}
```

#### While Loop (`loop on`)

```kiro
var x = 0
loop on (x < 5) {
    print x
    x = x + 1
}
```

#### For Loop (`loop in`)

The `loop in` construct is extremely powerful, supporting ranges, steps (`per`), and inline filters (`on`).

```kiro
// Basic loop: 0 to 9
loop i in 0..10 {
    print i
}

// Advanced loop: 0 to 19, step by 2, only keep numbers > 10
loop x in 0..20 per 2 on (x > 10) {
    print x
} off {
    // Optional block that runs if the filter (on) is false?
    // (Note: Implementation specific, currently 'off' in loops handles filtered-out items)
}
```

### 3. Functions

Functions are declared with `fn`. Arguments must be typed.

```kiro
fn add(a: num, b: num) {
    // Implicit returns are not fully supported yet, use expressions or assignments.
    print a + b
}

add(10, 20)
```

> **Note**: A `pure` keyword exists (`pure fn`) for future strict-mode implementations (side-effect free functions).

### 4. Pointers & Memory (`ref` / `deref`)

Kiro abstracts away complex memory management while giving you pointer-like behavior. references are thread-safe by default (compiling to `Arc<Mutex<T>>`).

```kiro
var value = 100
var ptr = ref value    // Create a reference
print deref ptr        // Read value (100)
```

### 5. Concurrency (`run`)

Spawning a background thread (async task) is as simple as using `run`.

```kiro
fn worker(id: num) {
    print "Worker started"
}

run worker(1) // Runs immediately in background
print "Main thread continues"
```

---

## 🏗️ Architecture

Kiro uses a "Double Pass" system:

1.  **Interpreter (`src/interpreter.rs`)**:
    - Walks the AST (Tree Sitter) directly.
    - Provides immediate, interactive feedback.
    - Great for small scripts and debugging.
2.  **Transpiler (`src/compiler.rs`)**:
    - Converts Kiro AST into valid **Rust** code.
    - Automatically builds a Cargo project in `kiro_build_cache`.
    - Compiles the result for maximum performance and stability.

## 🛠️ Project Structure

- `main.kiro`: Your entry point source file.
- `src/lib.rs`: Grammar definition (Rust Sitter).
- `src/interpreter.rs`: Runtime logic for the interpreter.
- `src/compiler.rs`: Logic for generating Rust code.
- `src/build_manager.rs`: Handles the background `cargo` processes.

---

_Built with ❤️ in Rust._
