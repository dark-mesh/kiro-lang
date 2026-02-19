const e=`# Chapter 0: Installing Kiro

Before writing your first Kiro program, set up a stable local environment so every chapter in this guide runs exactly as described. Kiro is distributed as a command-line binary, so the installation goal is simple: you should be able to run \`kiro --version\` in your terminal from any project directory.

Start by obtaining the Kiro binary for your operating system from your project's release process. Place it in a directory that is part of your shell \`PATH\`, then open a new terminal session and verify the installation:

\`\`\`bash
kiro --version
\`\`\`

If the command prints a version string, your installation is complete. If the shell says the command is not found, the binary is either not executable or not in your \`PATH\`.

On Unix-like systems (macOS/Linux), make sure the file has execute permissions:

\`\`\`bash
chmod +x /path/to/kiro
\`\`\`

Then add its folder to \`PATH\` in your shell profile (\`~/.zshrc\`, \`~/.bashrc\`, etc.), reload the shell, and test again.

Now create a small working directory for this book:

\`\`\`bash
mkdir kiro-playground
cd kiro-playground
\`\`\`

Create a first program file named \`hello.kiro\`:

\`\`\`kiro
print "Hello, Kiro!"
\`\`\`

Run it:

\`\`\`bash
kiro hello.kiro
\`\`\`

You should see the greeting printed to your terminal. This confirms that editing, running, and reading output all work correctly.

## Common Pitfalls

A frequent issue is installing multiple binaries and accidentally using an older one. The correct approach is to run \`which kiro\` (or your shell equivalent) and verify it points to the intended binary location.

Another issue is editing shell configuration but forgetting to restart or reload the shell. The correct method is to run \`source ~/.zshrc\` (or reopen terminal) before re-checking \`kiro --version\`.

Developers also often run examples from the wrong directory and then assume the language is broken when files are not found. The correct habit is to confirm your current directory with \`pwd\` and keep chapter files together in a dedicated workspace.

## Next Step

Continue with [Chapter 1: The Basics](../chapter-01/01_basics.md).
`,n=`# Chapter 1: The Basics

Kiro programs are easiest to understand when you see them as a sequence of explicit statements. You write values, combine them with operations, and print or return the result. This chapter introduces that rhythm so every later feature feels natural rather than abstract.

Begin with the smallest possible script:

\`\`\`kiro
print "Hello, Kiro!"
\`\`\`

Run it with:

\`\`\`bash
kiro 01_basics.kiro
\`\`\`

The language executes from top to bottom, so each line does one clear thing. Once this feels intuitive, variables become straightforward.

Assignment in Kiro uses \`=\`:

\`\`\`kiro
name = "Ada"
age = 21
is_active = true
\`\`\`

By default, these bindings are immutable. If you need to change a value later, declare it with \`var\`:

\`\`\`kiro
var count = 0
count = count + 1
count = count + 1
print count
\`\`\`

This design encourages stable values by default and explicit mutability only when necessary.

Kiro's core value types are simple: \`num\` for numbers, \`str\` for text, \`bool\` for true/false logic, and \`void\` for "no value" return contexts. You will use these types in every chapter.

Comments come in two forms. Use \`//\` for normal inline notes and \`///\` for documentation comments attached to the next item:

\`\`\`kiro
/// Computes area from radius.
fn area(r: num) -> num {
    return 3.14 * r * r
}
\`\`\`

## Common Pitfalls

New users often try to reassign immutable values and assume assignment is broken. The correct method is to decide upfront whether a variable will change; if it will, declare it with \`var\`.

Another common issue is mixing values in string output without clear concatenation. The correct method is to build output explicitly with \`+\` so each conversion step is intentional.

Some users skip environment checks and debug the wrong problem. The correct method is to confirm \`kiro --version\` first whenever execution behaves unexpectedly.

## Next Step

Continue with [Chapter 2: Control Flow](../chapter-02/02_control_flow.md).
`,t=`# Chapter 2: Control Flow

A useful program does not just execute lines blindly. It branches when conditions change and repeats work when input grows. In Kiro, this behavior is expressed with readable control keywords that make intent obvious.

Conditional branching uses \`on\` and \`off\`:

\`\`\`kiro
score = 85

on (score >= 90) {
    print "Grade A"
} off {
    print "Not A"
}
\`\`\`

Read this as: run the first block when the condition is true, otherwise run the second block. This direct style keeps condition logic easy to scan.

For repeated execution based on a condition, use \`loop on (...)\`:

\`\`\`kiro
var i = 0
loop on (i < 3) {
    print i
    i = i + 1
}
\`\`\`

When you already know a numeric interval, range loops are cleaner:

\`\`\`kiro
loop x in 0..5 {
    print x
}
\`\`\`

You can step through ranges with \`per\`:

\`\`\`kiro
loop n in 0..10 per 2 {
    print n
}
\`\`\`

And you can filter iterations with \`on\` inside range loops:

\`\`\`kiro
loop n in 0..10 on (n > 5) {
    print n
}
\`\`\`

Control keywords complete the model: \`break\` exits a loop, \`continue\` skips to the next iteration, and \`return\` exits a function.

## Common Pitfalls

Many loop bugs come from forgetting to update the loop variable in \`loop on (...)\`. The correct method is to place state updates near the end of each iteration so termination is guaranteed.

Another frequent issue is writing conditions without parentheses in complex expressions. The correct method is to keep conditions explicit and parenthesized for both readability and correctness.

Developers also overuse nested branches. The correct method is to keep branch blocks short and extract repeated logic into functions.

## Next Step

Continue with [Chapter 3: Functions & Modules](../chapter-03/03_functions.md).
`,r=`# Chapter 3: Functions & Modules

As programs grow, structure matters more than syntax. Functions let you name behavior once and reuse it everywhere. Modules let you group related behavior into files that can be imported where needed.

A Kiro function declares inputs and output type explicitly:

\`\`\`kiro
fn add(a: num, b: num) -> num {
    return a + b
}

print add(2, 3)
\`\`\`

That signature is a contract. Readers immediately understand what the function accepts and what it returns.

When branching inside a function, keep return types consistent:

\`\`\`kiro
fn clamp_to_zero(x: num) -> num {
    on (x < 0) {
        return 0
    }
    return x
}
\`\`\`

Kiro also supports \`pure fn\` for deterministic, side-effect-free logic:

\`\`\`kiro
pure fn square(x: num) -> num {
    return x * x
}
\`\`\`

Use pure functions for computation and transformation. Keep I/O and side effects in non-pure functions.

Function references are now supported for named pure functions. This lets you pass behavior around without closures or lambdas.

A function reference is created with \`ref\` and a function name:

\`\`\`kiro
pure fn inc(x: num) -> num { return x + 1 }

f = ref inc
print f(10)
\`\`\`

You can type function parameters with \`fn(...) -> ...\`:

\`\`\`kiro
pure fn apply(x: num, f: fn(num) -> num) -> num {
    return f(x)
}

print apply(5, ref inc)
\`\`\`

You can also return function references:

\`\`\`kiro
pure fn dec(x: num) -> num { return x - 1 }

pure fn pick(up: bool) -> fn(num) -> num {
    on (up) {
        return ref inc
    } off {
        return ref dec
    }
}
\`\`\`

Current rule: function references are for named pure functions only. This keeps the model simple and predictable.

Modules are just \`.kiro\` files. Import by module name without the extension.

\`mylib.kiro\`:

\`\`\`kiro
pure fn pi() -> num {
    return 3.14
}
\`\`\`

\`main.kiro\`:

\`\`\`kiro
import mylib
print mylib.pi()
\`\`\`

This module boundary is the foundation for larger projects.

## Common Pitfalls

A common error is drifting return types across branches. The correct method is to decide the return type first and enforce it in every path.

Another issue is writing overly large functions with mixed responsibilities. The correct method is to split logic into small, single-purpose functions and compose them.

Function references can fail if you try to reference impure functions. The correct method is to use \`ref\` with named pure functions and match parameter/return signatures exactly.

Module organization also breaks when imports become circular. The correct method is to define clear ownership boundaries so each module has one direction of dependency.

## Next Step

Continue with [Chapter 4: Data Structures](../chapter-04/04_data.md).
`,o=`# Chapter 4: Data Structures

Once logic is clear, data modeling becomes the next major design step. Kiro gives you three core tools: structs for shaped records, lists for ordered sequences, and maps for key-value lookup. Choosing the right structure makes code simpler before optimization is ever needed.

A struct represents one entity with named fields:

\`\`\`kiro
struct User {
    name: str
    age: num
}

var user = User { name: "Kiro", age: 10 }
print user.name
\`\`\`

Use structs whenever fields belong together conceptually.

A list stores ordered values of one type:

\`\`\`kiro
var nums = list num { 1, 2, 3 }
nums push 4
print (nums at 0)
print nums
\`\`\`

A map stores values by key, useful for lookups:

\`\`\`kiro
var scores = map str num {
    "Alice" 10,
    "Bob" 5
}

print (scores at "Alice")
\`\`\`

The practical rule is simple: choose \`struct\` for one thing with many fields, \`list\` for many items in order, and \`map\` for fast access by key.

## Common Pitfalls

A common data-modeling mistake is forcing everything into lists. The correct method is to promote related fields into a struct so field names communicate intent.

Another issue is mixing value types inside typed collections. The correct method is to treat collection element types as strict contracts and convert data before insertion.

Map lookups often fail because keys were never inserted or are inconsistent in format. The correct method is to normalize key creation (for example, casing and spacing) at insertion time and reuse that normalization at lookup time.

## Next Step

Continue with [Chapter 5: Error Handling](../chapter-05/05_errors.md).
`,i=`# Chapter 5: Error Handling

Reliable programs treat failure as a first-class design concern. Kiro follows this principle by making errors explicit values that must be handled deliberately. This keeps failure behavior visible at both function boundaries and call sites.

You can define domain-specific errors with clear names:

\`\`\`kiro
error TooSmall = "Value is too small"
error TooBig = "Value is too big"
\`\`\`

A function that can fail marks its return type with \`!\`:

\`\`\`kiro
fn check(val: num) -> str! {
    on (val < 10) {
        return TooSmall
    }

    on (val > 100) {
        return TooBig
    }

    return "Valid: " + val
}
\`\`\`

At the call site, handle outcomes explicitly:

\`\`\`kiro
var res = check(55)

on (res == TooSmall) {
    print "Too small"
} off {
    on (res == TooBig) {
        print "Too big"
    } off {
        print "Success: " + res
    }
}
\`\`\`

This explicit style is more verbose than implicit exceptions, but it scales better because success and failure paths are both visible in code review.

## Common Pitfalls

A frequent problem is defining vague error names that do not describe the situation. The correct method is to model errors around domain meaning, such as \`InvalidEmail\`, \`NotFound\`, or \`PermissionDenied\`.

Another issue is encoding failure as plain success-like strings. The correct method is to return declared error values from failable functions so callers can branch safely and predictably.

Developers also tend to handle only the happy path during prototyping. The correct method is to implement failure branches at the first call site and keep them in place as the program evolves.

## Next Step

Continue with [Chapter 6: Async & \`run\`](../chapter-06/06_async.md).
`,s=`# Chapter 6 (Optional): Advanced Concepts

This optional chapter connects features you already learned into one coherent design style. Instead of treating pointers, concurrency, and host functions as isolated tools, you can view them as complementary mechanisms for ownership, coordination, and capability extension.

Pointers (\`adr\`, \`ref\`, \`deref\`) give reference semantics when sharing or mutating existing values is necessary:

\`\`\`kiro
var x = 10
var p = ref x
print (deref p)
deref p = 20
print x
\`\`\`

Concurrency (\`run\`) helps independent work proceed without blocking, while pipes keep communication explicit:

\`\`\`kiro
fn worker(out: pipe str) {
    give out "done"
}

var ch = pipe str
run worker(ch)
print (take ch)
\`\`\`

Host declarations (\`rust fn\`) extend Kiro with Rust implementations where system-level integration or specialized libraries are needed:

\`\`\`kiro
rust fn read_file(path: str) -> str!
\`\`\`

In practice, robust systems usually combine these patterns: pure functions for logic, structs for data shape, pipes for concurrency boundaries, and host functions for external capabilities.

## Common Pitfalls

A common advanced-level mistake is combining every feature at once during initial implementation. The correct method is to introduce one abstraction at a time and validate behavior incrementally.

Another issue is choosing shared pointer mutation where message passing would be simpler. The correct method is to default to pipes for cross-task coordination and reserve pointer mutation for tightly scoped cases.

Host integration also fails when declarations and glue drift apart. The correct method is to keep Kiro signatures and Rust implementations synchronized as part of normal review.

## Next Step

Continue with [Chapter 7: Pipes](../chapter-07/07_pipes.md) if you have not completed it yet.
`,a=`# Chapter 6: Async & \`run\`

Concurrency in Kiro begins with one idea: start independent work without blocking the current flow. The \`run\` keyword gives you that capability with minimal syntax.

A simple example shows the model:

\`\`\`kiro
fn worker() {
    print "Working in background..."
}

run worker()
print "Main flow continues"
\`\`\`

\`run worker()\` starts background work and immediately returns control to the next statement. This means output order between worker and main flow is not guaranteed.

Arguments are passed normally:

\`\`\`kiro
fn log(msg: str) {
    print "Log: " + msg
}

run log("Async message")
\`\`\`

As your programs grow, asynchronous execution should be paired with clear communication boundaries, which is exactly what pipes provide in the next chapter.

## Common Pitfalls

A frequent mistake is assuming statements after \`run\` wait for completion. The correct method is to treat \`run\` as non-blocking and design synchronization intentionally.

Another issue is mixing shared mutable state into multiple concurrent tasks too early. The correct method is to prefer message passing and narrow ownership so task behavior stays understandable.

Developers also spawn tasks without lifecycle planning. The correct method is to define when work starts, how results are collected, and when channels close.

## Next Step

Continue with [Chapter 7: Pipes](../chapter-07/07_pipes.md).
`,c=`# Chapter 7: Pipes (Channels)

Pipes are Kiro's core primitive for communication between concurrent tasks. They let one part of the program send typed data and another part receive it safely, without relying on implicit shared state.

Create a typed pipe like this:

\`\`\`kiro
var p = pipe num
\`\`\`

Now pair a producer with a consumer:

\`\`\`kiro
fn producer(c: pipe num) {
    give c 10
    give c 20
    close c
}

var p = pipe num
run producer(p)

print (take p)
print (take p)
\`\`\`

\`give\` sends values into the channel, \`take\` receives them, and \`close\` signals completion when no more values will be sent.

Pipes work best when each channel has a clear purpose. In larger designs, use structs as message payloads so each message carries named fields rather than loosely connected primitives.

## Common Pitfalls

A frequent error is forgetting to close producer-owned pipes. The correct method is to close the channel at the point where production is known to be complete.

Another issue is sending values that do not match the pipe type. The correct method is to treat pipe type as a hard contract and perform conversion before sending.

Blocking bugs often come from taking values with no producer path. The correct method is to design sender and receiver flows together and verify each \`take\` has a corresponding send or terminal condition.

## Next Step

Continue with [Chapter 8: Pointers](../chapter-08/08_pointers.md).
`,l=`# Chapter 8: Pointers

Pointers in Kiro let you reference existing values directly when ownership or mutation patterns require it. They are explicit, typed, and intentionally visible in code, which keeps reference-heavy logic understandable. Kiro pointers are managed references, not raw memory pointers.

Create a pointer with \`ref\`:

\`\`\`kiro
var x = 10
var ptr = ref x
\`\`\`

The pointer type is \`adr num\`, meaning "managed address handle to a number".

Read and write through the pointer using \`deref\`:

\`\`\`kiro
print (deref ptr)
deref ptr = 20
\`\`\`

\`ref\` gives you a managed pointer value. In the current model, treat pointer mutation as mutation through that handle, not as an aliasing guarantee for the original variable binding.

When you need an untyped handle, use \`adr void\`. This is an opaque managed handle, not a raw numeric memory address:

\`\`\`kiro
var x = 42
var p = ref x

var h = adr void
h = p

print h        // handle display
\`\`\`

\`adr void\` is useful as an opaque transport/identity handle. Keep typed access through \`adr T\` pointers when you need \`deref\` reads/writes.

For structs, field access is ergonomic because pointer field use is auto-dereferenced:

\`\`\`kiro
struct User {
    name: str
}

var user = User { name: "Kiro" }
var up = ref user
print up.name
\`\`\`

Use pointers when reference semantics simplify your design. If plain values already express the behavior clearly, keep the simpler option.

## Common Pitfalls

A common issue is introducing pointers prematurely for simple data flow. The correct method is to start with plain values and only adopt pointers when shared mutation is genuinely needed.

Another issue is mutating shared data through pointers across concurrent tasks without coordination. The correct method is to define clear ownership boundaries or communicate changes through pipes.

Pointer logic also becomes fragile when dereference operations are scattered across large functions. The correct method is to keep pointer manipulation localized and wrapped in small helper functions.

## Next Step

Continue with [Chapter 9: Host Modules (Rust Side)](../chapter-09/09_host_rust.md).
`,u=`# Chapter 9: Host Modules (Rust Side)

Kiro is designed to be extensible through host functions implemented in Rust. This allows you to keep high-level logic in Kiro while delegating system access and ecosystem integration to Rust code.

When Kiro declares a \`rust fn\`, the runtime expects a matching Rust implementation. The Rust function typically receives runtime values, validates/converts them, performs work, and returns either a runtime value or a structured error.

\`\`\`rust
use kiro_runtime::{RuntimeVal, KiroError};

pub async fn read_file(args: Vec<RuntimeVal>) -> Result<RuntimeVal, KiroError> {
    let path = args[0].as_str()?;
    let content = std::fs::read_to_string(path)
        .map_err(|e| KiroError::new(&format!("read_file failed: {}", e)))?;

    Ok(RuntimeVal::Str(content))
}
\`\`\`

The practical workflow is consistent: decode arguments, execute Rust logic, map success to \`RuntimeVal\`, map failures to \`KiroError\`.

Keep host functions narrow in scope. Small host surfaces are easier to test, easier to review, and safer to evolve as language features grow.

## Common Pitfalls

A frequent integration failure is declaring host functions in Kiro but omitting Rust glue. The correct method is to treat declaration and implementation as one change set and verify both in the same run.

Another issue is trusting argument shape without validation. The correct method is to convert and check each runtime argument before use and return explicit errors for invalid input.

Panic-driven host code is also brittle. The correct method is to convert all expected failure paths into structured \`KiroError\` values.

## Next Step

Continue with [Chapter 10: Host Modules (Kiro Side)](../chapter-10/10_host_kiro.md).
`,h=`# Chapter 10: Host Modules (Kiro Side)

After implementing Rust glue, Kiro-side usage is intentionally simple. You declare a host function signature and call it as part of normal program flow.

A declaration looks like this:

\`\`\`kiro
rust fn read_file(path: str) -> str!
\`\`\`

This declaration is a contract between Kiro code and Rust code. Name, parameter types, and return type must stay aligned with the Rust implementation.

Calling the function is no different from calling a regular Kiro function:

\`\`\`kiro
error FileNotFound = "File not found"

var content = read_file("data.txt")

on (content == FileNotFound) {
    print "Missing file"
} off {
    print content
}
\`\`\`

In real projects, host calls are often at system boundaries: filesystem access, networking, cryptography, or integration with existing Rust crates.

## Common Pitfalls

A common issue is signature drift between \`.kiro\` declaration and Rust glue function. The correct method is to update both sides together and treat mismatches as build blockers.

Another issue is assuming host calls never fail. The correct method is to keep return types failable when appropriate and implement explicit success/failure handling at call sites.

Teams also frequently test only successful paths. The correct method is to create at least one controlled failure case for every host function and validate the produced error behavior.

## Final Step

Move to the [Final Project](../final-project/final_project.md).
`,d=`# Final Project: Async Task Manager

This project is the point where core Kiro concepts stop being isolated exercises and become a coherent application. You will model task data with structs, process work concurrently with \`run\`, move information through pipes, and handle failures explicitly.

A practical architecture begins with a \`Task\` struct and a worker function that receives tasks and sends results. Multiple workers can run concurrently while a coordinating loop in main collects outputs and prints status.

A recommended progression is to build the system in stages. First, run one worker with one task and verify end-to-end flow. Then introduce multiple tasks. Next, scale to multiple workers. Finally, add explicit failure handling and aggregate reporting.

Throughout implementation, keep channel ownership clear: producers close the channels they own, consumers terminate predictably, and every \`take\` has a known source path.

Run the project with:

\`\`\`bash
kiro final_project.kiro
\`\`\`

Once the baseline works, extend it with retries, priorities, or separate success/error output channels.

## Common Pitfalls

A common project-level issue is implementing concurrency and error handling simultaneously before baseline flow works. The correct method is to lock in a minimal single-worker path first, then add complexity incrementally.

Another frequent problem is unclear pipe lifecycle ownership. The correct method is to assign ownership explicitly so each pipe is closed exactly once by the correct producer.

Teams also often under-test non-happy paths. The correct method is to force at least one expected failure path per stage and verify that the system still drains and exits cleanly.
`;export{d as _,h as a,u as b,l as c,c as d,a as e,s as f,i as g,o as h,r as i,t as j,n as k,e as l};
