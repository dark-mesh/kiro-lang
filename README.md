# 🌀 Kiro Language

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
- `pipe`: Channels for asynchronous communication.
- **Strict Typed Collections**: `list <type>` and `map <key> <val>`.
- **Structs**: Custom named types (e.g., `User`).

#### Operators & Expressions

- **Concatenation**: Use `+` for strings (`"a" + "b"`).
- **Deep Equality**: `==` and `!=` work deeply for Structs, Lists, and Maps.
- **Size**: Use `len` to get the length of strings and collections (`len my_list`).

### 2. Structs

Kiro supports custom data structures. Struct names **must** start with a Capital letter, while fields use lowercase.

**Definition** (No commas needed):

```kiro
struct User {
    name: str
    age: num
    active: bool
}
```

**Initialization** (Commas required):

```kiro
var u = User {
    name: "Kiro",
    age: 1,
    active: true
}
print u.name
```

### 3. Collections

Kiro features strictly typed lists and maps with command-style operations.

#### Lists

**Initialization**:

```kiro
var numbers = list num { 10, 20, 30 }
```

**Commands**:

- `at`: Access element by index (`numbers at 0`).
- `push`: Append an element (`numbers push 40`).

#### Maps

**Initialization** (Comma separated key-value pairs):

```kiro
var scores = map str num { "Alice" 100, "Bob" 90 }
```

**Commands**:

- `at`: Access value by key (`scores at "Alice"`).

### 4. Control Flow

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

The `loop in` construct works with ranges, lists, and even strings. It supports steps (`per`) and inline filters (`on`).

```kiro
// Looping over a Range
loop i in 0..10 {
    print i
}

// Looping over a List
var items = list str { "A", "B", "C" }
loop item in items {
    print item
}

// Advanced loop: Range 0-19, step by 2, only keep numbers > 10
loop x in 0..20 per 2 on (x > 10) {
    print x
}
```

### 5. Functions

Functions are declared with `fn`. Arguments must be typed.

```kiro
fn add(a: num, b: num) {
    print a + b
}

add(10, 20)
```

> **Note**: A `pure` keyword exists (`pure fn`) for future strict-mode implementations (side-effect free functions).

### 6. Pointers & Memory (`ref` / `deref`)

Kiro abstracts away complex memory management while giving you pointer-like behavior. References are thread-safe by default (compiling to `Arc<Mutex<T>>`).

#### Standard Access

```kiro
var value = 100
var ptr = ref value    // Create a reference
print deref ptr        // Read value (100)
```

#### Auto-Deref (Structs)

When you have a pointer to a struct, you don't need to manually dereference it to access fields. Kiro handles this automatically!

```kiro
var u = User { name: "Kiro", ... }
var p = ref u

// Works directly! (No 'deref' needed)
print p.name
```

### 7. Concurrency & Pipes

Kiro makes concurrency easy with the `run` keyword and **Pipes** (channels) for communication.

#### Async Execution

Spawning a background thread (async task) is as simple as using `run`.

```kiro
fn worker(id: num) {
    print "Worker started"
}

run worker(1) // Runs immediately in the background
print "Main thread continues"
```

#### Pipes (Channels)

Pipes allow safe communication between threads. They compile to Rust's Multi-Producer Single-Consumer (MPSC) channels (or `async-channel`).

1.  **Create a Pipe**: `var p = pipe num`
2.  **Send Data (`give`)**: `give p 42`
3.  **Receive Data (`take`)**: `var val = take p`
4.  **Close Pipe**: `close p`

**Example:**

```kiro
// 1. Create a pipe for numbers
var p = pipe num

fn sender(ch: pipe) {
    print "Sending..."
    give ch 100
    give ch 200
    close ch
}

// 2. Start sender in background
run sender(p)

// 3. Receive in main thread
print take p  // Prints 100
print take p  // Prints 200
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
