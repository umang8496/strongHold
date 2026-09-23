<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD012 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Rust Error Handling — The Complete Guide

A dedicated, example-heavy reference for **how** and **why** Rust handles errors the
way it does. Built as a quick-recap companion: skim the diagrams, copy the snippets,
re-read the philosophy when a design decision feels ambiguous.

> **Mental model in one sentence:**
> In Rust, *errors are ordinary values* moved through the type system with `Result`
> and `Option`, while *bugs* — violated invariants the program can't sanely continue
> past — trigger a `panic!`.

## Table of Content

### Philosophy & Mental Model

- [1. Errors Are Values, Not Exceptions](#1-errors-are-values-not-exceptions)
- [2. The Two Categories of Failure](#2-the-two-categories-of-failure)
- [3. The Type System Forces You to Care](#3-the-type-system-forces-you-to-care)
- [4. Recoverable vs Unrecoverable — The Core Decision](#4-recoverable-vs-unrecoverable--the-core-decision)

### Recoverable Errors

- [5. `Option<T>` — Modeling Absence](#5-optiont--modeling-absence)
- [6. `Result<T, E>` — Modeling Failure](#6-resultt-e--modeling-failure)
- [7. Handling `Result` with `match`, `if let`, `let else`](#7-handling-result-with-match-if-let-let-else)
- [8. The Combinator Toolbox](#8-the-combinator-toolbox)
- [9. The `?` Operator In Depth](#9-the--operator-in-depth)
- [10. Converting Between `Option` and `Result`](#10-converting-between-option-and-result)

### Unrecoverable Errors

- [11. `panic!` — What Actually Happens](#11-panic--what-actually-happens)
- [12. `unwrap`, `expect`, and Their Cousins](#12-unwrap-expect-and-their-cousins)
- [13. The `assert!` Family](#13-the-assert-family)
- [14. `unreachable!`, `todo!`, `unimplemented!`](#14-unreachable-todo-unimplemented)
- [15. When to `panic!` and When Not To](#15-when-to-panic-and-when-not-to)
- [16. Unwinding vs Aborting, and `catch_unwind`](#16-unwinding-vs-aborting-and-catch_unwind)

### Custom Errors

- [17. Custom Error Enums](#17-custom-error-enums)
- [18. Implementing `Display` and the `Error` Trait](#18-implementing-display-and-the-error-trait)
- [19. `From`, `Into`, and Automatic Conversion](#19-from-into-and-automatic-conversion)
- [20. `Box<dyn Error>` — The Universal Error](#20-boxdyn-error--the-universal-error)
- [21. `thiserror` and `anyhow` — The Ecosystem Standard](#21-thiserror-and-anyhow--the-ecosystem-standard)

### The Standard Library Error Zoo (25+ Types)

- [22. Predefined Error Types Reference](#22-predefined-error-types-reference)

### Patterns & Cheat Sheet

- [23. Best Practices & Idioms](#23-best-practices--idioms)
- [24. Quick Reference Cheat Sheet](#24-quick-reference-cheat-sheet)

---

## 1. Errors Are Values, Not Exceptions

Most mainstream languages (Java, C#, Python, C++) handle errors with **exceptions**:
a hidden control-flow channel that unwinds the stack until some `catch` block grabs
it. The problem: nothing in a function's signature tells you what it can throw, and
it's easy to forget to handle anything at all.

Rust made a different choice:

```text
Exceptions (Java/Python/C++)          Rust
─────────────────────────────         ─────────────────────────────
throw/raise  → invisible channel      return Err(e) → an ordinary value
try { } catch { }                     match / ? on a returned value
signature hides what can fail         signature states what can fail
easy to ignore silently               compiler warns if you ignore it
```

A fallible Rust function *returns* its failure as data:

```rust
fn parse_number(input: &str) -> Result<i32, std::num::ParseIntError> {
    input.parse::<i32>()
}
```

The return type `Result<i32, ParseIntError>` is a **contract**: "I hand you back
either an `i32` or a `ParseIntError`, and you must decide what to do with each."

There is no `throw`. There is no invisible unwinding for ordinary failures. The error
is a value you can store in a variable, pass to a function, log, transform, or ignore
— all explicitly.

### Why this matters

- **Honesty.** The type signature is the documentation. You cannot be surprised by an
  error a function "forgot" to mention.
- **Composability.** Because errors are values, you compose them with normal tools:
  functions, generics, iterators, pattern matching.
- **Zero cost.** `Result` compiles down to a tagged union (an enum). There is no
  exception-table lookup, no stack-unwinding machinery on the happy path.

[Go to the Top](#table-of-content)

---

## 2. The Two Categories of Failure

Rust splits *everything that can go wrong* into exactly two buckets, and gives each
its own mechanism:

```text
                    Something went wrong
                            │
             ┌──────────────┴───────────────┐
             ▼                              ▼
     RECOVERABLE                      UNRECOVERABLE
   "expected, handleable"          "a bug / broken invariant"
             │                              │
             ▼                              ▼
     Result<T, E>                       panic!
     Option<T>                          (unwind or abort)
             │                              │
             ▼                              ▼
   caller decides what to do        program (or thread) stops
```

**Recoverable** errors are part of normal operation. A file might be missing. User
input might be malformed. A network request might time out. The program should keep
running and *do something sensible*. These use `Result<T, E>` (or `Option<T>` when
the only information is "present or absent").

**Unrecoverable** errors mean the program reached a state it was never supposed to
reach — a violated assumption, a logic bug, an impossible branch. Continuing would be
meaningless or dangerous. These trigger `panic!`, which tears down the current thread.

> The single most important design question in Rust error handling is:
> **"Is this a situation my caller can reasonably respond to, or is it a bug?"**
> The answer picks your mechanism.

[Go to the Top](#table-of-content)

---

## 3. The Type System Forces You to Care

`Result` is annotated `#[must_use]`. If you call a fallible function and drop the
result on the floor, the compiler warns you:

```rust
use std::fs::File;

fn main() {
    File::open("config.toml"); // ⚠️ warning: unused `Result` that must be used
}
```

```text
warning: unused `Result` that must be used
 = note: this `Result` may be an `Err` variant, which should be handled
```

You cannot *accidentally* ignore an error. To ignore one you must do it **on purpose**:

```rust
let _ = File::open("config.toml");        // explicitly discard
let _file = File::open("config.toml").ok(); // convert to Option, keep the Some
```

This is the enforcement mechanism behind the philosophy: the language nudges (and
often forces) you to acknowledge every failure path. Contrast with exceptions, where
silence is the default.

[Go to the Top](#table-of-content)

---

## 4. Recoverable vs Unrecoverable — The Core Decision

A side-by-side you'll want to re-read often:

| Question                                   | Recoverable (`Result`)        | Unrecoverable (`panic!`)          |
| ------------------------------------------ | ----------------------------- | --------------------------------- |
| Is it expected during normal operation?    | Yes                           | No — it's a bug                    |
| Can the caller do something useful?        | Yes                           | No                                |
| Caused by external input / I/O / user?     | Usually                       | Rarely                            |
| Caused by a broken program invariant?      | No                            | Yes                               |
| Should the program keep running?           | Yes                           | Not in a meaningful state         |
| Who decides the response?                  | The caller                    | Nobody — execution stops          |
| Typical trigger                            | `return Err(...)`             | `panic!`, `unwrap`, `assert!`     |
| Shows in the function signature?           | Yes (`-> Result<...>`)        | No (invisible)                    |

### Concrete examples of each

```rust
// RECOVERABLE — a missing file is a normal, expected possibility.
fn load_config(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path) // caller can fall back to defaults
}

// RECOVERABLE — bad user input is expected; ask again, don't crash.
fn parse_age(input: &str) -> Result<u8, std::num::ParseIntError> {
    input.trim().parse::<u8>()
}

// UNRECOVERABLE — this branch is logically impossible; reaching it is a bug.
fn suit_color(suit: &str) -> &str {
    match suit {
        "hearts" | "diamonds" => "red",
        "spades" | "clubs" => "black",
        other => panic!("invalid suit reached the color function: {other}"),
    }
}

// UNRECOVERABLE — an index we *know* is valid by construction; if it isn't,
// our surrounding logic is broken.
fn first_byte(data: &[u8]) -> u8 {
    assert!(!data.is_empty(), "first_byte called on empty slice");
    data[0]
}
```

### The decision flowchart

```text
            Can the failure happen because of
            input, I/O, environment, or state
            outside my control?
                       │
              ┌────────┴────────┐
             YES                NO
              │                  │
              ▼                  ▼
     Can the caller do      Does reaching this point
     something meaningful   mean a prior assumption
     other than crash?      was violated (a bug)?
              │                  │
        ┌─────┴─────┐            ├───────────────┐
       YES          NO          YES              NO (truly impossible)
        │            │           │                │
        ▼            ▼           ▼                ▼
   Result<T,E>   Result +     panic! /        unreachable!()
   (let caller   good error   assert!         (documents "can't happen")
    choose)      message
```

[Go to the Top](#table-of-content)

---

## 5. `Option<T>` — Modeling Absence

Before `Result`, meet its simpler sibling. `Option<T>` encodes "a value that might
not be there" — with **no error information**, just present or absent.

```rust
enum Option<T> {
    Some(T), // a value is present
    None,    // nothing is here
}
```

Use `Option` when absence is the *only* thing worth communicating (there's no "why").
Use `Result` when the failure carries a reason.

```rust
fn find_user(id: u32) -> Option<String> {
    match id {
        1 => Some(String::from("Alice")),
        2 => Some(String::from("Bob")),
        _ => None, // "not found" — no error detail needed
    }
}

fn main() {
    match find_user(1) {
        Some(name) => println!("Found: {}", name),
        None => println!("No such user"),
    }

    // Common ergonomic helpers:
    let name = find_user(99).unwrap_or_else(|| String::from("guest"));
    println!("{}", name); // "guest"

    // `if let` when you only care about the Some case:
    if let Some(name) = find_user(2) {
        println!("Hi, {}", name);
    }
}
```

### `Option` <-> `Result` bridges (preview)

```rust
let maybe: Option<i32> = Some(5);

// Attach an error when None:
let as_result: Result<i32, &str> = maybe.ok_or("value was missing");

// Discard the error when you only want present/absent:
let back_to_option: Option<i32> = as_result.ok();
```

[Go to the Top](#table-of-content)

---

## 6. `Result<T, E>` — Modeling Failure

The workhorse. `Result<T, E>` is an enum with two variants:

```rust
enum Result<T, E> {
    Ok(T),  // success, carrying the value
    Err(E), // failure, carrying the error
}
```

```text
Result<T, E>
│
├── Ok(T)   → the operation succeeded; here is the value
│
└── Err(E)  → the operation failed; here is the reason
```

- `T` is the success type.
- `E` is the error type — and it can be *anything*: a std error, your own enum, a
  `String`, a `Box<dyn Error>`.

```rust
use std::num::ParseIntError;

fn double(input: &str) -> Result<i32, ParseIntError> {
    let n = input.parse::<i32>()?; // ? unwraps Ok or returns Err early
    Ok(n * 2)
}

fn main() {
    println!("{:?}", double("21")); // Ok(42)
    println!("{:?}", double("x"));  // Err(ParseIntError { kind: InvalidDigit })
}
```

### `Result` in `main`

`main` itself can return a `Result`. On `Err`, the process exits with a non-zero code
and prints the error's `Debug` representation:

```rust
use std::num::ParseIntError;

fn main() -> Result<(), ParseIntError> {
    let n: i32 = "123".parse()?;
    println!("Parsed {}", n);
    Ok(())
}
```

[Go to the Top](#table-of-content)

---

## 7. Handling `Result` with `match`, `if let`, `let else`

### `match` — exhaustive and explicit

The most fundamental tool. You must handle both variants:

```rust
use std::num::ParseIntError;

fn parse_number(input: &str) -> Result<i32, ParseIntError> {
    input.parse::<i32>()
}

fn main() {
    match parse_number("42") {
        Ok(n)  => println!("Parsed: {}", n),
        Err(e) => println!("Failed: {}", e),
    }
}
```

You can match on the *kind* of error to react differently:

```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    let file = File::open("data.txt");

    match file {
        Ok(f) => println!("Opened {:?}", f),
        Err(e) => match e.kind() {
            ErrorKind::NotFound        => println!("Create the file first."),
            ErrorKind::PermissionDenied => println!("Check your permissions."),
            other                       => println!("Unexpected: {:?}", other),
        },
    }
}
```

### `if let` — when you care about one variant

```rust
if let Ok(n) = parse_number("7") {
    println!("Got {}", n);
}
// No else branch needed; the Err case is silently skipped.
```

### `let else` — bind or bail (Rust 1.65+)

Great for "extract the happy value or leave early", keeping the rest of the function
un-indented:

```rust
fn describe(input: &str) -> String {
    let Ok(n) = input.parse::<i32>() else {
        return String::from("not a number");
    };
    // `n` is in scope for the rest of the function
    format!("the number is {}", n)
}
```

[Go to the Top](#table-of-content)

---

## 8. The Combinator Toolbox

`match` is powerful but verbose. `Option` and `Result` come with dozens of methods
("combinators") that express common patterns concisely. These are the ones worth
memorizing.

### Transform the success value: `map`

```rust
let doubled: Result<i32, _> = "21".parse::<i32>().map(|n| n * 2);
// Ok(42)
```

### Transform the error: `map_err`

```rust
let parsed: Result<i32, String> =
    "abc".parse::<i32>().map_err(|e| format!("bad input: {e}"));
// Err("bad input: invalid digit found in string")
```

### Chain another fallible step: `and_then`

```rust
fn parse(s: &str) -> Result<i32, std::num::ParseIntError> { s.parse() }

let result = parse("10").and_then(|n| parse("20").map(|m| n + m));
// Ok(30) — only runs the second parse if the first succeeded
```

### Provide a fallback: `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`

```rust
let a = "5".parse::<i32>().unwrap_or(0);            // 5
let b = "x".parse::<i32>().unwrap_or(0);            // 0 (fallback)
let c = "x".parse::<i32>().unwrap_or_else(|_| -1);  // -1 (lazy fallback)
let d = "x".parse::<i32>().unwrap_or_default();     // 0 (i32::default())
```

### Inspect without consuming: `is_ok`, `is_err`, `ok`, `err`

```rust
let r: Result<i32, _> = "42".parse::<i32>();
println!("{}", r.is_ok());        // true
let opt: Option<i32> = r.ok();    // Some(42) — throw away the error
```

### A quick map of the most useful combinators

```text
On the value (Ok / Some):
  map          transform the inner value
  and_then     chain another operation that also returns Result/Option
  filter       (Option) keep Some only if a predicate holds

On the error (Err):
  map_err      transform the error
  or_else      supply an alternative Result on error

Extracting:
  unwrap_or           value or a default
  unwrap_or_else      value or a computed default
  unwrap_or_default   value or T::default()
  ok / err            Result -> Option (of value / of error)
  ok_or / ok_or_else  Option -> Result (attach an error)

Escape hatches (may panic — see §12):
  unwrap       value or panic
  expect       value or panic with your message
```

[Go to the Top](#table-of-content)

---

## 9. The `?` Operator In Depth

The `?` operator is Rust's ergonomic error-propagation tool. It replaces the verbose
"match, and on `Err` return early" boilerplate.

### What `?` expands to

```rust
// This:
let n = input.parse::<i32>()?;

// Is (essentially) shorthand for:
let n = match input.parse::<i32>() {
    Ok(value) => value,
    Err(e)    => return Err(From::from(e)), // note: From conversion!
};
```

Two things happen on the `Err` path:

1. The function **returns early**.
2. The error is converted via `From::from` into the function's declared error type.

That second point is the secret sauce that makes `?` compose across different error
types (see §19).

### `?` chains beautifully

```rust
use std::num::ParseIntError;

fn sum_three(a: &str, b: &str, c: &str) -> Result<i32, ParseIntError> {
    Ok(a.parse::<i32>()? + b.parse::<i32>()? + c.parse::<i32>()?)
}

fn main() {
    println!("{:?}", sum_three("1", "2", "3")); // Ok(6)
    println!("{:?}", sum_three("1", "x", "3")); // Err(...) — stops at "x"
}
```

### `?` works with `Option` too

In a function returning `Option`, `?` returns `None` early:

```rust
fn first_char_upper(s: &str) -> Option<char> {
    let c = s.chars().next()?;        // None if the string is empty
    Some(c.to_ascii_uppercase())
}

fn main() {
    println!("{:?}", first_char_upper("hello")); // Some('H')
    println!("{:?}", first_char_upper(""));       // None
}
```

### Where `?` can be used

`?` only works inside a function whose return type can absorb the early return —
`Result`, `Option`, or any type implementing the `Try` trait. You **cannot** use `?`
in a plain function returning `()` or `i32`.

```text
fn f() -> Result<T, E>   ✅  ? returns Err(...)
fn f() -> Option<T>      ✅  ? returns None
fn main() -> Result<..>  ✅  ? works in main if main returns Result
fn f() -> i32            ❌  nothing to early-return into
```

[Go to the Top](#table-of-content)

---

## 10. Converting Between `Option` and `Result`

You constantly move between "maybe absent" and "maybe failed". These are the bridges.

```rust
fn main() {
    // Option -> Result: attach an error to describe the None.
    let maybe: Option<i32> = None;
    let res: Result<i32, &str> = maybe.ok_or("no value present");
    println!("{:?}", res); // Err("no value present")

    // ok_or_else: compute the error lazily (only when None).
    let res2: Result<i32, String> =
        None.ok_or_else(|| format!("missing at {}", 42));

    // Result -> Option: keep the value, discard the error.
    let parsed: Result<i32, _> = "5".parse::<i32>();
    let opt: Option<i32> = parsed.ok(); // Some(5)

    // Result -> Option of the error side:
    let bad: Result<i32, _> = "x".parse::<i32>();
    let e: Option<std::num::ParseIntError> = bad.err(); // Some(ParseIntError)
    println!("{:?} {:?}", opt, e);
}
```

```text
        ok_or / ok_or_else
  Option ───────────────────────▶ Result
        ◀───────────────────────
             ok / err
```

[Go to the Top](#table-of-content)

---

## 11. `panic!` — What Actually Happens

`panic!` is Rust's mechanism for **unrecoverable** errors. It says: "the program is in
a state I don't know how to handle; stop this thread now."

```rust
fn main() {
    println!("before");
    panic!("something went irrecoverably wrong");
    // println!("after"); // never runs
}
```

Output (with a backtrace hint):

```text
before
thread 'main' panicked at src/main.rs:3:5:
something went irrecoverably wrong
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### What panic does, step by step

```text
panic!  ─▶  print message + location
        ─▶  (default) UNWIND the stack:
              run every Drop in reverse order,
              releasing locks, closing files, freeing memory
        ─▶  the thread terminates
        ─▶  if it was the main thread, the process exits (code 101)
```

Panicking is **not** how you report expected errors. It's a controlled crash for
"this should never happen" situations. A library that panics on bad input is
considered buggy; it should return `Result` instead.

### Panics can carry any payload, but strings are typical

```rust
panic!("plain message");
panic!("formatted: expected {}, got {}", 4, 5);
```

### Getting a backtrace

```text
# PowerShell
$env:RUST_BACKTRACE=1 ; cargo run

# bash
RUST_BACKTRACE=1 cargo run
```

A backtrace shows the chain of calls that led to the panic — invaluable for finding
the bug that put the program in the impossible state.

[Go to the Top](#table-of-content)

---

## 12. `unwrap`, `expect`, and Their Cousins

These methods **turn a recoverable error into an unrecoverable one**. They're
convenient, dangerous when misused, and perfectly fine when used deliberately.

### `unwrap` — value or panic

```rust
let n: i32 = "42".parse().unwrap();  // 42
let bad: i32 = "x".parse().unwrap(); // 💥 panics with the default message
```

```text
thread 'main' panicked at ...:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
```

### `expect` — value or panic *with your message*

Always prefer `expect` over `unwrap`: the message documents **why you believed it
couldn't fail**, which is exactly what the person debugging the panic needs.

```rust
let config = std::fs::read_to_string("Cargo.toml")
    .expect("Cargo.toml must exist at the crate root");
```

```text
thread 'main' panicked at ...:
Cargo.toml must exist at the crate root: Os { code: 2, kind: NotFound, ... }
```

> Convention: phrase the `expect` message as the *precondition that was supposed to
> hold*, not the error. "Env var X should be set by the launcher" reads better in a
> crash log than "failed to read env var".

### The `Option` equivalents

```rust
let some: Option<i32> = Some(5);
println!("{}", some.unwrap());              // 5
println!("{}", some.expect("must be set")); // 5

let none: Option<i32> = None;
// none.unwrap(); // 💥 panics: called `Option::unwrap()` on a `None` value
```

### When `unwrap`/`expect` are acceptable

- **Prototypes, examples, tests** — brevity beats ceremony.
- **You can prove it can't fail** — e.g. a hard-coded literal you just validated, or
  a regex you compiled from a constant string.
- **A poisoned lock** where you genuinely want to propagate the panic.

```rust
// Justified: this literal is valid UTF-8 by inspection.
let s = std::str::from_utf8(b"hello").expect("hard-coded bytes are valid UTF-8");
```

### When they're a code smell

- In **library** code that others call — return `Result` and let them decide.
- On **user input, I/O, network, env vars** — these fail routinely; handle them.

[Go to the Top](#table-of-content)

---

## 13. The `assert!` Family

Assertions enforce **invariants** — conditions your code assumes to be true. A failed
assertion panics, so assertions are a form of "check-or-crash".

```rust
fn withdraw(balance: u32, amount: u32) -> u32 {
    assert!(amount <= balance, "cannot withdraw {amount} from {balance}");
    balance - amount
}

fn main() {
    let remaining = withdraw(100, 30); // ok
    println!("{}", remaining);         // 70
    // withdraw(100, 200);             // 💥 panics: the invariant is violated
}
```

### The variants

```rust
assert!(cond);                       // panics if cond is false
assert_eq!(a, b);                    // panics if a != b, prints both values
assert_ne!(a, b);                    // panics if a == b
assert!(cond, "msg {}", detail);     // custom panic message
```

`assert_eq!` / `assert_ne!` are especially nice because the panic message prints the
actual left/right values:

```text
assertion `left == right` failed
  left: 4
 right: 5
```

### `debug_assert!` — checks only in debug builds

Same family, but compiled out in `--release`. Use for expensive invariant checks you
want during development but not in production hot paths:

```rust
fn binary_search(sorted: &[i32], target: i32) -> Option<usize> {
    debug_assert!(sorted.windows(2).all(|w| w[0] <= w[1]),
                  "input to binary_search must be sorted");
    // ... actual search ...
    sorted.iter().position(|&x| x == target)
}
```

```text
assert!        → always checked (debug + release)   → invariants that must always hold
debug_assert!  → checked in debug builds only        → costly checks / dev-time sanity
```

### Assertions in tests vs production

- In **tests**, `assert!`/`assert_eq!` are how you *express expectations* — a failure
  is a failed test, which is the whole point.
- In **production code**, an assertion documents a real invariant and converts its
  violation into an immediate, located crash rather than silent corruption.

[Go to the Top](#table-of-content)

---

## 14. `unreachable!`, `todo!`, `unimplemented!`

Three specialized panics that communicate intent.

### `unreachable!` — "control flow can't get here"

Tells both the compiler and the reader that a branch is logically impossible. If it
*does* execute, that's a bug worth crashing on.

```rust
fn parity(n: u32) -> &'static str {
    match n % 2 {
        0 => "even",
        1 => "odd",
        _ => unreachable!("n % 2 is always 0 or 1"),
    }
}
```

### `todo!` — "not written yet, but I intend to"

A placeholder that type-checks as any type, so your code compiles while you stub out
functions. Panics if actually run.

```rust
fn compute_tax(_income: f64) -> f64 {
    todo!("implement progressive tax brackets")
}
```

### `unimplemented!` — "not supported here, and maybe never"

Similar to `todo!` but signals "this case is deliberately not handled" rather than
"coming soon".

```rust
trait Shape { fn area(&self) -> f64; }

struct Point;
impl Shape for Point {
    fn area(&self) -> f64 {
        unimplemented!("a point has no area")
    }
}
```

```text
todo!          → placeholder you plan to fill in
unimplemented! → intentionally unsupported case
unreachable!   → a branch that logically cannot occur
```

All three panic; the difference is the *message they send to a human reader*.

[Go to the Top](#table-of-content)

---

## 15. When to `panic!` and When Not To

The rule of thumb, distilled:

```text
panic! / unwrap / assert! WHEN:
  ✔ A broken invariant means a bug in *your* code.
  ✔ Continuing would be unsafe, meaningless, or corrupt state.
  ✔ You are in tests, examples, or a quick prototype.
  ✔ You can prove the failure is truly impossible given surrounding code.
  ✔ A contract you documented was violated by the caller.

Result<T, E> WHEN:
  ✔ The failure is expected during normal operation.
  ✔ The caller can reasonably respond (retry, default, ask the user).
  ✔ The failure comes from input, I/O, network, environment, parsing.
  ✔ You are writing a library — let callers choose their own policy.
```

### Two functions, same operation, different choice

```rust
// Application-internal helper, called with a value we just validated:
// a failure here is a logic bug, so panicking is defensible.
fn to_percentage(fraction: f64) -> u8 {
    assert!((0.0..=1.0).contains(&fraction), "fraction must be in [0, 1]");
    (fraction * 100.0).round() as u8
}

// Public parsing API, called with arbitrary user text:
// failure is routine, so return Result and let the caller handle it.
pub fn parse_percentage(text: &str) -> Result<u8, std::num::ParseIntError> {
    let n: u8 = text.trim_end_matches('%').parse()?;
    Ok(n)
}
```

### The guiding principle

> `panic!` removes the caller's ability to decide. Reach for it only when there is no
> sensible decision left to make. If a reasonable caller might want to recover, that
> decision belongs to them — return a `Result`.

[Go to the Top](#table-of-content)

---

## 16. Unwinding vs Aborting, and `catch_unwind`

### Two panic strategies

By default a panic **unwinds**: it walks back up the stack running destructors
(`Drop`) so resources are released cleanly. You can instead configure panics to
**abort** — instantly terminate the process with no unwinding.

```toml
# Cargo.toml — abort on panic (smaller binaries, no unwinding machinery)
[profile.release]
panic = "abort"
```

```text
UNWIND (default)                    ABORT
────────────────                    ─────
run all Drop impls                  no destructors run
release locks / files / memory      OS reclaims memory on exit
thread can be isolated              whole process dies immediately
larger binary                       smaller binary, faster panic
catchable via catch_unwind          not catchable
```

### `catch_unwind` — trap a panic at a boundary

Occasionally you must stop a panic from crossing a boundary — most importantly at an
**FFI edge**, because unwinding into C is undefined behavior. `std::panic::catch_unwind`
turns a panic into a `Result`.

```rust
use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        println!("about to panic");
        panic!("boom");
    });

    match result {
        Ok(_)  => println!("closure completed normally"),
        Err(_) => println!("caught a panic; continuing"),
    }

    println!("main survived");
}
```

```text
about to panic
thread 'main' panicked at ...: boom
caught a panic; continuing
main survived
```

> **Do not** use `catch_unwind` as a general try/catch for control flow. It exists for
> boundary safety (FFI, thread/task isolation in runtimes), not for handling ordinary
> errors — those belong in `Result`. It also can't catch an `abort`, and won't catch
> panics if `panic = "abort"` is set.

[Go to the Top](#table-of-content)

---

## 17. Custom Error Enums

Real programs need error types richer than `ParseIntError`. The idiomatic base is a
plain enum, one variant per failure mode.

```rust
#[derive(Debug)]
enum ConfigError {
    NotFound,
    Empty,
    InvalidPort(String),
}

fn load_port(raw: Option<&str>) -> Result<u16, ConfigError> {
    let raw = raw.ok_or(ConfigError::NotFound)?;
    if raw.trim().is_empty() {
        return Err(ConfigError::Empty);
    }
    raw.trim()
        .parse::<u16>()
        .map_err(|_| ConfigError::InvalidPort(raw.to_string()))
}

fn main() {
    println!("{:?}", load_port(Some("8080"))); // Ok(8080)
    println!("{:?}", load_port(Some("abc")));   // Err(InvalidPort("abc"))
    println!("{:?}", load_port(None));          // Err(NotFound)
}
```

`#[derive(Debug)]` alone lets you print an error with `{:?}` and is the minimum for a
usable error type. For anything public-facing, add `Display` and `Error` (next).

[Go to the Top](#table-of-content)

---

## 18. Implementing `Display` and the `Error` Trait

A well-behaved error type implements three things:

1. `Debug` (usually derived) — developer-facing, `{:?}`.
2. `Display` (hand-written) — user-facing, `{}`.
3. `std::error::Error` — the marker trait that makes it interoperable with the whole
   ecosystem (and lets it be boxed as `Box<dyn Error>`).

```rust
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    NotFound,
    Empty,
    InvalidPort(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound        => write!(f, "configuration not found"),
            ConfigError::Empty           => write!(f, "configuration was empty"),
            ConfigError::InvalidPort(p)  => write!(f, "invalid port value: {p}"),
        }
    }
}

// The blanket impl requires Debug + Display, which we now have.
impl std::error::Error for ConfigError {}

fn main() {
    let e = ConfigError::InvalidPort(String::from("99999"));
    println!("{}", e);   // Display:  invalid port value: 99999
    println!("{:?}", e); // Debug:    InvalidPort("99999")
}
```

### The `Error` trait and error sources

`std::error::Error` also lets an error expose the lower-level error that *caused* it,
via `source()`. This builds an error chain you can walk:

```rust
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
struct PortError {
    input: String,
    source: ParseIntError, // the underlying cause
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not parse port from {:?}", self.input)
    }
}

impl std::error::Error for PortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source) // "the cause was this ParseIntError"
    }
}

fn main() {
    let err = "xyz".parse::<u16>().unwrap_err();
    let wrapped = PortError { input: "xyz".into(), source: err };

    println!("{}", wrapped);                     // top-level message
    if let Some(cause) = std::error::Error::source(&wrapped) {
        println!("caused by: {}", cause);        // underlying ParseIntError
    }
}
```

[Go to the Top](#table-of-content)

---

## 19. `From`, `Into`, and Automatic Conversion

Recall from §9 that `?` calls `From::from` on the error. This is what lets one
function propagate many *different* underlying errors as a single unified error type.

### The problem `From` solves

```rust
// This function can fail two different ways: reading a file (io::Error)
// and parsing its contents (ParseIntError). We want ONE error type out.
use std::fs;

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
}

// Teach `?` how to turn each source error into AppError:
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e) }
}
impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self { AppError::Parse(e) }
}

fn read_number(path: &str) -> Result<i32, AppError> {
    let text = fs::read_to_string(path)?; // io::Error -> AppError via From
    let n = text.trim().parse::<i32>()?;   // ParseIntError -> AppError via From
    Ok(n)
}

fn main() {
    match read_number("count.txt") {
        Ok(n)  => println!("count = {}", n),
        Err(AppError::Io(e))    => println!("I/O problem: {}", e),
        Err(AppError::Parse(e)) => println!("not a number: {}", e),
    }
}
```

```text
        ?  on  io::Error
                │
                ▼
   From<io::Error> for AppError
                │
                ▼
        Err(AppError::Io(..))  returned to caller
```

### Why implement `From` and not `Into`

By convention you implement `From`; Rust *automatically* gives you the matching
`Into` for free. `?` is written in terms of `From`, so implementing `From<SourceError>
for YourError` is all you need.

> This is exactly the pattern from NEON Exercise 051: `From<ParseIntError> for
> NumberError` lets `?` convert a library error into your domain error with zero
> boilerplate at each call site.

[Go to the Top](#table-of-content)

---

## 20. `Box<dyn Error>` — The Universal Error

Writing `From` impls and enums is great for libraries where callers need to match on
specific variants. But in **application** code — a `main`, a script, glue — you often
just want "any error, propagated upward, printed at the top." That's `Box<dyn Error>`.

```rust
use std::error::Error;
use std::fs;

// Return type: "a value, or any type that implements the Error trait, on the heap."
fn read_number(path: &str) -> Result<i32, Box<dyn Error>> {
    let text = fs::read_to_string(path)?; // io::Error       auto-boxes
    let n = text.trim().parse::<i32>()?;   // ParseIntError   auto-boxes
    Ok(n)
}

fn main() -> Result<(), Box<dyn Error>> {
    let n = read_number("count.txt")?;
    println!("count = {}", n);
    Ok(())
}
```

Because *every* std error implements `Error`, and `?` converts via `From` (and there's
a `From<E: Error>` impl for `Box<dyn Error>`), any error flows straight up with `?`.

```text
Box<dyn Error> trade-offs
  ✔ dead simple, zero boilerplate, mixes any error types
  ✔ perfect for main(), scripts, examples, prototypes
  ✘ callers can't easily match on the specific error kind
  ✘ erases the concrete type (dynamic dispatch)
```

Rule of thumb: **libraries** expose concrete error enums (callers may need to react
per-variant); **applications** often use `Box<dyn Error>` (they just report and exit).

[Go to the Top](#table-of-content)

---

## 21. `thiserror` and `anyhow` — The Ecosystem Standard

Hand-writing `Display` and `From` gets tedious. Two crates dominate real-world Rust and
map cleanly onto the library-vs-application split above.

### `thiserror` — ergonomic custom errors (for libraries)

Derives `Display`, `Error`, and `From` from attributes. You still get a concrete,
matchable enum — just without the boilerplate.

```rust
// Cargo.toml:  thiserror = "1"
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("file could not be read")]
    Io(#[from] std::io::Error),          // #[from] generates From<io::Error>

    #[error("expected a number, got {0:?}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("value {value} is out of range 0..={max}")]
    OutOfRange { value: i32, max: i32 },
}

pub fn load(path: &str) -> Result<i32, DataError> {
    let text = std::fs::read_to_string(path)?; // From<io::Error>, auto
    let n = text.trim().parse::<i32>()?;        // From<ParseIntError>, auto
    if n > 100 {
        return Err(DataError::OutOfRange { value: n, max: 100 });
    }
    Ok(n)
}
```

The `#[error("...")]` strings become the `Display` impl; `#[from]` generates the `From`
conversion that `?` needs. You wrote an enum; the crate wrote the plumbing.

### `anyhow` — effortless error propagation (for applications)

`anyhow::Error` is like a supercharged `Box<dyn Error>`: it holds any error, captures a
backtrace, and lets you attach human-readable **context**.

```rust
// Cargo.toml:  anyhow = "1"
use anyhow::{Context, Result};

fn load_config() -> Result<String> {
    let path = "app.toml";
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config at {path}"))?;
    Ok(text)
}

fn main() -> Result<()> {
    let cfg = load_config()?;
    println!("{}", cfg);
    Ok(())
}
```

On failure you get a layered report:

```text
Error: failed to read config at app.toml

Caused by:
    No such file or directory (os error 2)
```

```text
Choosing between them
  thiserror → you are a LIBRARY; callers match on your variants
  anyhow    → you are an APPLICATION; you just want context + propagation
  (they compose: a lib returns thiserror enums, the app collects them via anyhow)
```

[Go to the Top](#table-of-content)

---

## 22. Predefined Error Types Reference

A tour of the standard library's built-in error types. Each entry lists the operation
that produces it and a runnable snippet. Keep this section as your lookup table.

> Every type below implements `std::error::Error` (so they all box into
> `Box<dyn Error>` and interoperate with `?`), and all implement `Debug` + `Display`.

### 1. `std::num::ParseIntError` — bad integer text

Produced by `str::parse::<iN/uN>()` and `iN::from_str_radix`.

```rust
fn main() {
    match "42x".parse::<i32>() {
        Ok(n)  => println!("{}", n),
        Err(e) => println!("{}", e), // invalid digit found in string
    }
}
```

### 2. `std::num::ParseFloatError` — bad float text

Produced by `str::parse::<f32/f64>()`.

```rust
fn main() {
    match "3.1.4".parse::<f64>() {
        Ok(x)  => println!("{}", x),
        Err(e) => println!("{}", e), // invalid float literal
    }
}
```

### 3. `std::num::TryFromIntError` — integer conversion out of range

Produced by `TryFrom`/`TryInto` between integer types (e.g. `i32::try_from(i64)`).

```rust
fn main() {
    let big: i64 = 10_000_000_000;
    match i32::try_from(big) {
        Ok(n)  => println!("{}", n),
        Err(e) => println!("{}", e), // out of range integral type conversion attempted
    }
}
```

### 4. `std::str::ParseBoolError` — text that isn't `true`/`false`

Produced by `str::parse::<bool>()`.

```rust
fn main() {
    match "yes".parse::<bool>() {
        Ok(b)  => println!("{}", b),
        Err(e) => println!("{}", e), // provided string was not `true` or `false`
    }
}
```

### 5. `std::str::Utf8Error` — invalid UTF-8 in a byte slice

Produced by `str::from_utf8(&[u8])` (borrowing, no allocation).

```rust
fn main() {
    let bytes = [0x00, 0x9f, 0x92, 0x96]; // not valid UTF-8
    match std::str::from_utf8(&bytes) {
        Ok(s)  => println!("{}", s),
        Err(e) => println!("{}", e), // invalid utf-8 sequence of 1 bytes from index 1
    }
}
```

### 6. `std::string::FromUtf8Error` — invalid UTF-8 when building a `String`

Produced by `String::from_utf8(Vec<u8>)`. Unlike `Utf8Error`, it *owns* the bytes and
lets you recover them with `.into_bytes()`.

```rust
fn main() {
    let bytes = vec![0x00, 0x9f, 0x92, 0x96];
    match String::from_utf8(bytes) {
        Ok(s)  => println!("{}", s),
        Err(e) => {
            println!("{}", e);
            let recovered = e.into_bytes(); // get the Vec<u8> back
            println!("recovered {} bytes", recovered.len());
        }
    }
}
```

### 7. `std::string::FromUtf16Error` — invalid UTF-16

Produced by `String::from_utf16(&[u16])`.

```rust
fn main() {
    let units = [0xD800];  // unpaired surrogate
    match String::from_utf16(&units) {
        Ok(s)  => println!("{}", s),
        Err(e) => println!("{}", e), // invalid utf-16: lone surrogate found
    }
}
```

### 8. `std::char::CharTryFromError` — `u32` outside the Unicode range

Produced by `char::try_from(u32)`.

```rust
fn main() {
    match char::try_from(0x11_0000_u32) { // beyond U+10FFFF
        Ok(c)  => println!("{}", c),
        Err(e) => println!("{}", e), // converted integer out of range for `char`
    }
}
```

### 9. `std::char::ParseCharError` — string isn't exactly one char

Produced by `str::parse::<char>()`.

```rust
fn main() {
    match "ab".parse::<char>() {
        Ok(c)  => println!("{}", c),
        Err(e) => println!("{}", e), // too many characters in string
    }
}
```

### 10. `std::io::Error` — the big one: all I/O failures

Produced by files, sockets, stdin/stdout, and more. Carries an `ErrorKind` you can
match on. This is the most common error you'll handle in real programs.

```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    match File::open("missing.txt") {
        Ok(f)  => println!("{:?}", f),
        Err(e) => match e.kind() {
            ErrorKind::NotFound         => println!("not found"),
            ErrorKind::PermissionDenied => println!("permission denied"),
            other                       => println!("other: {:?}", other),
        },
    }
}
```

### 11. `std::fmt::Error` — a formatter failed

Produced inside `Display`/`Debug` impls and by `write!` into a `fmt::Write` target.

```rust
use std::fmt::Write;

fn build() -> Result<String, std::fmt::Error> {
    let mut s = String::new();
    write!(s, "x = {}", 42)?; // returns fmt::Error on failure
    Ok(s)
}

fn main() {
    println!("{:?}", build()); // Ok("x = 42")
}
```

### 12. `std::array::TryFromSliceError` — slice length ≠ array length

Produced by converting a slice to a fixed-size array via `TryInto`.

```rust
use std::convert::TryInto;

fn main() {
    let bytes: &[u8] = &[1, 2, 3, 4, 5];
    let arr: Result<[u8; 4], _> = bytes.try_into(); // 5 != 4
    match arr {
        Ok(a)  => println!("{:?}", a),
        Err(e) => println!("{}", e), // could not convert slice to array
    }
}
```

### 13. `std::sync::PoisonError<T>` — a lock was held during a panic

Produced by `Mutex::lock` / `RwLock::write` after another thread panicked while
holding the lock. You can still recover the data with `.into_inner()`.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let m = Arc::new(Mutex::new(0));
    let m2 = Arc::clone(&m);

    let _ = thread::spawn(move || {
        let _guard = m2.lock().unwrap();
        panic!("poisoning the mutex");
    }).join();

    match m.lock() {
        Ok(g)  => println!("value: {}", *g),
        Err(poisoned) => {
            let g = poisoned.into_inner(); // recover anyway
            println!("recovered from poison: {}", *g);
        }
    }
}
```

### 14. `std::sync::mpsc::RecvError` — receiving from a closed channel

Produced by `Receiver::recv` when all senders have been dropped.

```rust
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel::<i32>();
    drop(tx); // no senders remain
    match rx.recv() {
        Ok(v)  => println!("{}", v),
        Err(e) => println!("{}", e), // receiving on an empty and disconnected channel
    }
}
```

### 15. `std::sync::mpsc::SendError<T>` — sending to a closed channel

Produced by `Sender::send` when the receiver is gone. It carries the value you tried
to send so you can reuse it.

```rust
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    drop(rx); // receiver gone
    match tx.send(42) {
        Ok(())  => println!("sent"),
        Err(e)  => println!("{} (value {} bounced back)", e, e.0),
    }
}
```

### 16. `std::sync::mpsc::TryRecvError` — non-blocking receive

Produced by `Receiver::try_recv`. Distinguishes "nothing yet" from "channel closed".

```rust
use std::sync::mpsc::{self, TryRecvError};

fn main() {
    let (tx, rx) = mpsc::channel::<i32>();
    match rx.try_recv() {
        Ok(v)                        => println!("got {}", v),
        Err(TryRecvError::Empty)     => println!("nothing yet"),
        Err(TryRecvError::Disconnected) => println!("closed"),
    }
    drop(tx);
}
```

### 17. `std::cell::BorrowError` — `RefCell` already mutably borrowed

Produced by `RefCell::try_borrow` when a mutable borrow is active. (The panicking
`borrow` produces the same situation as a panic instead.)

```rust
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(5);
    let _mutable = cell.borrow_mut();
    match cell.try_borrow() {
        Ok(v)  => println!("{}", v),
        Err(e) => println!("{}", e), // already mutably borrowed
    }
}
```

### 18. `std::cell::BorrowMutError` — `RefCell` already borrowed

Produced by `RefCell::try_borrow_mut` when any borrow is active.

```rust
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(5);
    let _shared = cell.borrow();
    match cell.try_borrow_mut() {
        Ok(v)  => println!("{}", *v),
        Err(e) => println!("{}", e), // already borrowed
    }
}
```

### 19. `std::env::VarError` — reading an environment variable

Produced by `std::env::var`. Distinguishes "not set" from "not valid Unicode".

```rust
use std::env::{self, VarError};

fn main() {
    match env::var("DEFINITELY_NOT_SET") {
        Ok(v)                        => println!("{}", v),
        Err(VarError::NotPresent)    => println!("not set"),
        Err(VarError::NotUnicode(_)) => println!("not valid unicode"),
    }
}
```

### 20. `std::net::AddrParseError` — malformed IP / socket address

Produced by parsing into `IpAddr`, `Ipv4Addr`, `SocketAddr`, etc.

```rust
use std::net::IpAddr;

fn main() {
    match "not.an.ip".parse::<IpAddr>() {
        Ok(a)  => println!("{}", a),
        Err(e) => println!("{}", e), // invalid IP address syntax
    }
}
```

### 21. `std::time::SystemTimeError` — clock went backwards

Produced by `SystemTime::duration_since` when the earlier time is actually later.

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d)  => println!("{} seconds since epoch", d.as_secs()),
        Err(e) => println!("clock error: {}", e),
    }
}
```

### 22. `std::convert::Infallible` — the error that can never occur

The error type for conversions that *cannot* fail (e.g. `String: FromStr`). Because it
has no values, matching its `Err` arm is provably dead code. Being replaced over time
by the never type `!`.

```rust
use std::convert::Infallible;
use std::str::FromStr;

struct Name(String);

impl FromStr for Name {
    type Err = Infallible;              // parsing a Name never fails
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Name(s.to_string()))
    }
}

fn main() {
    let Name(n) = "Ada".parse().unwrap(); // safe: Err is impossible
    println!("{}", n);
}
```

### 23. `std::ffi::NulError` — interior NUL byte in a `CString`

Produced by `CString::new` when the input contains a `\0` (C strings can't).

```rust
use std::ffi::CString;

fn main() {
    match CString::new("hello\0world") {
        Ok(c)  => println!("{:?}", c),
        Err(e) => println!("{}", e), // nul byte found in provided data at position: 5
    }
}
```

### 24. `std::path::StripPrefixError` — prefix not part of a path

Produced by `Path::strip_prefix` when the given prefix doesn't match.

```rust
use std::path::Path;

fn main() {
    let p = Path::new("/usr/local/bin");
    match p.strip_prefix("/opt") {
        Ok(rest) => println!("{:?}", rest),
        Err(e)   => println!("{}", e), // prefix not found
    }
}
```

### 25. `std::alloc::LayoutError` — invalid memory layout parameters

Produced by `Layout::from_size_align` when alignment isn't a power of two or the size
overflows. Rarely seen outside allocator/`unsafe` code.

```rust
use std::alloc::Layout;

fn main() {
    match Layout::from_size_align(1024, 3) { // 3 is not a power of two
        Ok(l)  => println!("{:?}", l),
        Err(e) => println!("{}", e), // invalid parameters to Layout::from_size_align
    }
}
```

### 26. `std::ffi::IntoStringError` — `CString` bytes weren't valid UTF-8

Produced by `CString::into_string`. Lets you recover the original `CString`.

```rust
use std::ffi::CString;

fn main() {
    // Build a CString containing invalid UTF-8, then try to convert to String.
    let c = unsafe { CString::from_vec_unchecked(vec![0xff, 0xfe]) };
    match c.into_string() {
        Ok(s)  => println!("{}", s),
        Err(e) => println!("not utf-8: {}", e), // C string contained invalid utf-8
    }
}
```

### Quick index

```text
Parsing text     ParseIntError, ParseFloatError, ParseBoolError, ParseCharError,
                 AddrParseError
Numeric convert  TryFromIntError, CharTryFromError, Infallible
Text encoding    Utf8Error, FromUtf8Error, FromUtf16Error, NulError, IntoStringError
I/O & system     io::Error, VarError, SystemTimeError, fmt::Error
Collections/mem  TryFromSliceError, LayoutError, StripPrefixError
Concurrency      PoisonError, RecvError, SendError, TryRecvError
Interior mut     BorrowError, BorrowMutError
```

[Go to the Top](#table-of-content)

---

## 23. Best Practices & Idioms

### Prefer a `Result` type alias in a module

Cuts repetition when most functions share one error type:

```rust
#[derive(Debug)]
struct MyError;

struct Config;

// One alias, reused everywhere in the module.
pub type Result<T> = std::result::Result<T, MyError>;

pub fn load() -> Result<Config> { Ok(Config) }
pub fn save() -> Result<()>     { Ok(()) }
```

### Never `unwrap` in a library's public API

Return `Result`; let the application decide. `unwrap` in a library takes that choice
away and can crash someone else's program.

### Add context as errors propagate

A bare `io::Error` says "file not found" but not *which* file or *why you wanted it*.
Wrap with context (`anyhow`'s `.context(...)`, or a custom variant that stores the
path). Future-you debugging a log will thank present-you.

### Collect an iterator of `Result`s into one `Result`

A beautiful idiom: `Result` implements `FromIterator`, so a sequence of fallible
operations short-circuits on the first error.

```rust
fn main() {
    let inputs = ["1", "2", "3"];
    let parsed: Result<Vec<i32>, _> =
        inputs.iter().map(|s| s.parse::<i32>()).collect();
    println!("{:?}", parsed); // Ok([1, 2, 3])

    let mixed = ["1", "oops", "3"];
    let parsed2: Result<Vec<i32>, _> =
        mixed.iter().map(|s| s.parse::<i32>()).collect();
    println!("{:?}", parsed2); // Err(ParseIntError { .. }) — stops at "oops"
}
```

### Match on `io::ErrorKind`, not error strings

Never parse error *messages* — they're not stable. Match the structured `kind()`
instead, as in §22 entry 10.

### Don't over-model errors

For a throwaway script, `Box<dyn Error>` or `anyhow` is plenty. Reserve rich custom
enums for libraries and long-lived code where callers genuinely branch on the variant.

### Reserve panics for genuine bugs

If you find yourself writing `unwrap` on something that depends on user input, the
environment, or the network — stop, and return a `Result` instead. Panics are for
"this is impossible unless my own code is wrong."

[Go to the Top](#table-of-content)

---

## 24. Quick Reference Cheat Sheet

### Choosing a mechanism

```text
Absence, no reason needed .................... Option<T>
Failure with a reason ........................ Result<T, E>
A bug / impossible state ..................... panic! / assert! / unreachable!
Not written yet .............................. todo!
Deliberately unsupported ..................... unimplemented!
```

### Extracting a value

```text
r.unwrap()               value, else PANIC (default msg)
r.expect("why")          value, else PANIC (your msg)   ← prefer over unwrap
r.unwrap_or(x)           value, else x
r.unwrap_or_else(|e| …)  value, else compute from e
r.unwrap_or_default()    value, else T::default()
r?                       value, else RETURN Err early (with From conversion)
```

### Transforming

```text
r.map(|v| …)         change the Ok value
r.map_err(|e| …)     change the Err value
r.and_then(|v| …)    chain another Result-returning step
r.or_else(|e| …)     recover with another Result
```

### Converting

```text
option.ok_or(err)        Option → Result (attach error)
option.ok_or_else(|| …)  Option → Result (lazy error)
result.ok()              Result → Option (keep value)
result.err()             Result → Option (keep error)
```

### Propagation with `?`

```text
let v = fallible()?;   // Ok → unwrap;  Err → return Err(From::from(e))
                       // works in fn -> Result<_,_>, fn -> Option<_>, main -> Result
```

### Custom error checklist

```text
□ #[derive(Debug)]                     developer output
□ impl Display                         user output
□ impl std::error::Error {}            ecosystem interop + boxing
□ impl From<SourceErr> (or #[from])    so `?` converts automatically
□ (optional) fn source()               expose the underlying cause / chain
```

### Panic strategy

```text
panic = "unwind"  (default)   run Drop, release resources, catchable
panic = "abort"               instant stop, smaller binary, not catchable
RUST_BACKTRACE=1              show the call chain that led to the panic
catch_unwind                  trap a panic at an FFI/task boundary (not for control flow)
```

### Library vs application posture

```text
LIBRARY       concrete error enums (thiserror), never unwrap public paths,
              let callers match and decide
APPLICATION   Box<dyn Error> / anyhow, add context, report at the top, exit
```

[Go to the Top](#table-of-content)

---

## Closing Mental Model

```text
                     A failure occurs
                            │
          ┌─────────────────┴──────────────────┐
          ▼                                     ▼
   Expected & handleable                 A bug / broken invariant
          │                                     │
          ▼                                     ▼
    Result<T, E> / Option<T>                 panic! / assert!
          │                                     │
   ┌──────┼───────────┐                         ▼
   ▼      ▼           ▼                   unwind the stack
 match    ?      combinators              run Drop, release resources
   │      │           │                         │
   └──────┴─────┬─────┘                         ▼
               ▼                          thread/process stops
        caller decides:                   (fix the bug, don't
        retry / default / report           "handle" it)
```

> **Return `Result` when a caller can reasonably decide what to do.
> Reach for `panic!` only when there is no sensible decision left to make.**

The whole discipline reduces to honestly answering, at every fallible step:
*"Is this an expected condition my caller should handle, or a bug that should stop the
program?"* — and letting the type system carry that answer for you.

[Go to the Top](#table-of-content)

