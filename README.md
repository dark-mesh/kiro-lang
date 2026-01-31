# 🌀 Kiro Language

**Kiro** is a modern, experimental programming language designed to explore the intersection of high-level expressivity and low-level performance concepts. It features a dual-execution model (Interpreter + Transpiler to Rust) and introduces unique syntax for flow control and concurrency.

## 🚀 Key Features

- **Safe by Default**: Variables are immutable unless explicitly declared with `var`.
- **Module System**: Organize code across multiple files with `import` and qualified access (`math.add`).
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
   - Recursively parse and compile all imported modules.
   - Transpile the project to Rust in `kiro_build_cache/`.
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
- `void`: Represents the absence of a value.
- `adr <type>`: Type-safe addresses/pointers.
- `pipe <type>`: Typed channels for asynchronous communication.
- **Strict Typed Collections**: `list <type>` and `map <key> <val>`.
- **Structs**: Custom named types (e.g., `User`).

#### Operators & Expressions

- **Concatenation**: Use `+` for strings (`"a" + "b"`).
- **Deep Equality**: `==` and `!=` work deeply for Structs, Lists, and Maps.
- **Size**: Use `len` to get the length of strings and collections (`len my_list`).

### 2. Module System (Separate Files)

Kiro supports code modularization. Any `.kiro` file in the same directory can be imported.

**`math.kiro`**:

```kiro
fn add(a: num, b: num) {
    return a + b
}
```

**`main.kiro`**:

```kiro
import math

fn main() {
    var result = math.add(10, 20)
    print result
}
main()
```

- **Qualified Access**: Use `module.member` to access exported functions or variables.
- **Recursive Loading**: The compiler and interpreter automatically resolve dependencies.

### 3. Structs & Mutation

#### Definition & Initialization

Struct names **must** start with a Capital letter. Fields use lowercase.

```kiro
struct User {
    name: str
    age: num
}

var u = User { name: "Kiro", age: 1 }
```

#### Field Mutation

Fields can be mutated if the struct instance is declared with `var`.

```kiro
var player = User { name: "Hero", age: 20 }
player.age = 21 // OK!
```

### 4. Collections

Kiro features strictly typed lists and maps with command-style operations.

#### Lists

```kiro
var numbers = list num { 10, 20, 30 }
print numbers at 0
numbers push 40
```

#### Maps

```kiro
var scores = map str num { "Alice" 100, "Bob" 90 }
print scores at "Alice"
```

### 5. Control Flow

#### Conditionals (`on` / `off`)

```kiro
on (x > 10) {
    print "High"
} off {
    print "Low"
}
```

#### Loops

- **While**: `loop on (cond) { ... }`
- **Iterator**: `loop i in 0..10 { ... }`
- **Advanced**: `loop x in list per 2 on (x > 5) { ... }`

#### Control Signals

Kiro supports standard control flow signals within functions and loops:

- `return value`: Exit function with a value.
- `break`: Exit the innermost loop.
- `continue`: Skip to the next iteration of the loop.

### 6. Pointers & Memory (`ref` / `deref`)

Kiro abstractly manages memory while giving you pointer-like behavior. References are thread-safe (`Arc<Mutex<T>>`).

#### Typed Pointers & Lazy Initialization

Pointers are declared with `adr <type>`. If initialized without a value, they are "lazy" (empty).

```kiro
var x = 10
var ptr = ref x
deref ptr = 20 // Mutate 'x' via pointer
print x // Returns 20

var lazy_ptr = adr str // Uninitialized pointer
lazy_ptr = ref "Now I exist"
```

#### Opaque Pointers (`adr void`)

Use `adr void` to store a raw memory address without type information. This is useful for passing handles or legacy pointers.

```kiro
var x = 10
var raw = adr void
raw = ref x // Address is extracted as a usize
print raw   // Prints the raw address (e.g. 5829058176)
```

**Auto-Deref**: Struct fields can be accessed directly on typed references: `ptr.name` instead of `(deref ptr).name`.

### 7. Concurrency & Pipes

#### Async Execution

Use `run` to spawn a background task.

```kiro
run worker(id)
```

#### Pipes (Channels)

Channels for safe communication between tasks. Pipes are typed: `pipe <type>`.

```kiro
var p = pipe num // Create a channel for numbers
give p 42
var x = take p
```

- `give p val`: Send value.
- `take p`: Receive value (awaits).
- `close p`: Close the channel.
- `pipe void`: A signal-only channel.

### 8. Functions

Functions are declared with `fn`. Arguments and return types are explicit.

```kiro
fn add(a: num, b: num) -> num {
    return a + b
}

fn do_nothing() -> void {
    print "Working..."
    return // Optional for void
}

add(10, 20)
do_nothing()
```

- **Void Functions**: If the return type is omitted, it defaults to `void`.
- **Explicit Return**: Use `-> type` to specify the return value.

> **Note**: A `pure` keyword exists (`pure fn`) for future strict-mode implementations (side-effect free functions).

---

## 🏗️ Architecture

Kiro uses a **Double Pass** system:

1.  **Interpreter (`src/interpreter/`)**:
    - Walks the AST and maintains a runtime environment.
    - Recursively loads and executes imported modules in isolation.
2.  **Transpiler (`src/compiler/`)**:
    - Converts Kiro to idiomatic **Rust**.
    - **Recursive Build**: The transpiler identifies dependencies and compiles them as Rust modules (`pub mod {name}`).
    - Hoists struct definitions and imports to ensure valid Rust output.

## 🛠️ Project Structure

- `src/grammar/`: Language rules and parser (Rust Sitter).
- `src/interpreter/`: Recursive execution engine and value representations.
- `src/compiler/`: Rust code generation logic.
- `src/build_manager.rs`: Cargo project lifecycle management.
- `main.kiro`: Entry point script.

---

_Built with ❤️ in Rust._
