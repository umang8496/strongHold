<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Ownership and Borrowing in Rust

## Table of Content

1. [Why Rust does not use a garbage collector](#1-why-rust-does-not-use-a-garbage-collector)
2. [Ownership: the rule that makes cleanup predictable](#2-ownership-the-rule-that-makes-cleanup-predictable)
3. [Moving, copying, and cloning values](#3-moving-copying-and-cloning-values)
4. [Borrowing with references](#4-borrowing-with-references)
5. [The borrowing rules](#5-the-borrowing-rules)
6. [References must always be valid](#6-references-must-always-be-valid)
7. [Slices: borrowed views into a collection](#7-slices-borrowed-views-into-a-collection)
8. [Choosing ownership or borrowing in an API](#8-choosing-ownership-or-borrowing-in-an-api)
9. [A practical mental model](#9-a-practical-mental-model)

## 1. Why Rust does not use a garbage collector

Programs need a way to release memory and other resources after they are no longer needed.  
Languages generally make one of these choices:

| Approach                            | Who releases resources?         | Trade-off                                                                                               |
| ----------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Manual management (`malloc`/`free`) | The programmer                  | Maximum control, but leaks, double frees, and use-after-free bugs are possible.                         |
| Tracing garbage collection          | A runtime collector             | Convenient reclamation, but needs runtime work and does not make the exact cleanup point deterministic. |
| Rust ownership                      | The compiler and generated code | Cleanup is automatic and deterministic, while invalid memory access is rejected before the program runs.|

Rust does **not** collect garbage with a tracing garbage collector.  
Instead, it uses ownership to determine, at compile time, which part of the program is responsible for each value.  
When that owner goes out of scope, Rust automatically runs the value's `Drop` logic.  
This is often called *deterministic resource management*.

That choice matters beyond memory.  
The same mechanism can close a file, release a lock, return a database connection to a pool, or free a socket at a known point in the program.  
There is no collector pause needed to discover whether an object is reachable.

```rust
struct Connection {
    name: String,
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!("closing {}", self.name);
    }
}

fn main() {
    let connection = Connection {
        name: String::from("primary database"),
    };

    println!("using {}", connection.name);
} // `connection` is dropped here, and its cleanup runs automatically.
```

The important point is not that every Rust program is faster than every garbage-collected program.  
It is that ownership lets Rust offer memory safety without requiring a tracing GC at runtime, while making resource lifetime visible in the program structure.

[Go to the Top](#table-of-content)

## 2. Ownership: the rule that makes cleanup predictable

Ownership is Rust's system for managing values. Its three core rules are:

1. Every value has exactly one **owner** at a time.
2. There can only be one owner for a value at a time.
3. When the owner leaves its scope, Rust drops the value.

### Scope and cleanup

A variable's scope begins when it is declared and ends at the closing brace of the enclosing block.

```rust
fn main() {
    {
        let message = String::from("hello");
        println!("{message}");
    } // `message` leaves scope; its heap allocation is released here.

    // println!("{message}"); // error: `message` no longer exists
}
```

`String` is a useful example because it owns heap-allocated text.  
A `String` value stored on the stack contains metadata (a pointer, length, and capacity); the characters themselves live in heap memory.  
When the `String` owner is dropped, Rust frees that heap allocation exactly once.

```text
`let message = String::from("hello");`

Stack:  message  ──►  Heap:  h e l l o
        pointer          owned character buffer
        length
        capacity
```

If two independent `String` values both believed they owned the same allocation, both would try to free it.  
That is a double-free error in manual-memory languages.  
Rust prevents the situation by transferring ownership instead of silently duplicating the owner.

[Go to the Top](#table-of-content)

## 3. Moving, copying, and cloning values

### Assignment can move ownership

For heap-owning values such as `String`, assignment transfers ownership. This is a **move**.

```rust
fn main() {
    let first = String::from("ownership");
    let second = first; // ownership moves from `first` to `second`

    println!("{second}");
    // println!("{first}"); // error: value borrowed here after move
}
```

After the move, `first` is no longer usable. Rust makes it invalid rather than allowing two variables to free the same allocation.  
A move is normally cheap: Rust transfers the stack metadata, not the heap characters.

### Passing a value to a function can also move it

Function parameters work like assignment: passing an owned, non-`Copy` value moves it into the parameter.

```rust
fn print_and_consume(text: String) {
    println!("{text}");
} // `text` is dropped here.

fn main() {
    let title = String::from("Rust");
    print_and_consume(title);

    // println!("{title}"); // error: `title` was moved into the function
}
```

Returning an owned value moves ownership to the caller:

```rust
fn make_label() -> String {
    String::from("ready")
}

fn main() {
    let label = make_label();
    println!("{label}"); // `label` is now its owner.
}
```

### `Copy`: types that are safe to duplicate bit-for-bit

Simple fixed-size types such as integers, booleans, and floating-point numbers implement the `Copy` trait.  
Assigning a `Copy` value duplicates it, so both bindings remain valid.

```rust
fn main() {
    let retries: u8 = 3;
    let remaining = retries;

    println!("retries: {retries}, remaining: {remaining}");
}
```

`Copy` is only available for types whose duplication cannot lead to double cleanup.  
A type that implements `Drop` cannot implement `Copy`.

### `Clone`: make an explicit independent duplicate

Call `clone()` when the program truly needs two independently owned copies.  
For a `String`, cloning allocates a new buffer and copies the characters.

```rust
fn main() {
    let original = String::from("independent data");
    let duplicate = original.clone();

    println!("original: {original}");
    println!("duplicate: {duplicate}");
}
```

`Clone` may be expensive, depending on the type. Prefer a borrow when the caller only needs temporary access rather than a separate owned value.

[Go to the Top](#table-of-content)

## 4. Borrowing with references

A **reference** gives temporary access to a value without taking ownership.  
Creating a reference is called **borrowing**. References are written with `&`.

```rust
fn length_of(text: &String) -> usize {
    text.len()
} // `text` is a reference, so this function does not own or drop the String.

fn main() {
    let headline = String::from("Borrowing avoids a move");
    let length = length_of(&headline);

    println!("{headline} has {length} characters");
}
```

For read-only text parameters, `&str` is generally more flexible than `&String`: it accepts a borrowed `String`, a string literal, or a substring.

```rust
fn announce(text: &str) {
    println!("Announcement: {text}");
}

fn main() {
    let owned = String::from("deployment complete");

    announce(&owned);               // `&String` coerces to `&str`
    announce("using a string literal");
}
```

### Mutable references

An immutable reference (`&T`) permits reading but not changing the borrowed value.  
To change it through a function, use a mutable reference (`&mut T`), and make the owner binding mutable too.

```rust
fn add_exclamation(text: &mut String) {
    text.push('!');
}

fn main() {
    let mut status = String::from("complete");
    add_exclamation(&mut status);

    println!("{status}"); // complete!
}
```

This explicitness is intentional: a function signature tells callers whether it reads a value, takes it, or may modify it.

[Go to the Top](#table-of-content)

## 5. The borrowing rules

At any particular time, a value may have:

- Any number of immutable references (`&T`), **or**
- Exactly one mutable reference (`&mut T`).

References must also always be valid.  
These rules are checked at compile time and prevent data races as well as many memory errors.

### Many readers are safe

Multiple parts of a program can observe the same value when nobody can change it.

```rust
fn main() {
    let config = String::from("production");
    let first_reader = &config;
    let second_reader = &config;

    println!("{first_reader} / {second_reader}");
}
```

### A writer needs exclusive access

Two mutable references could make conflicting changes through the same value.  
Rust therefore rejects this code:

```rust,compile_fail
let mut queue = vec!["first"];
let writer_one = &mut queue;
let writer_two = &mut queue; // error: cannot borrow `queue` as mutable more than once

writer_one.push("second");
writer_two.push("third");
```

The same exclusivity rule prevents an immutable reference from observing a value while a mutable reference might change it:

```rust,compile_fail
let mut name = String::from("Ada");
let reader = &name;
let writer = &mut name; // error: immutable borrow is still in use

println!("{reader}");
writer.push_str(" Lovelace");
```

### Non-lexical lifetimes keep borrows short

Modern Rust uses **non-lexical lifetimes**: a borrow usually lasts until its last use, not necessarily until the end of the enclosing block.  
That allows the following code:

```rust
fn main() {
    let mut name = String::from("Ada");
    let reader = &name;
    println!("{reader}"); // last use of `reader`

    let writer = &mut name;
    writer.push_str(" Lovelace");
    println!("{writer}");
}
```

The useful mental model remains: while a value is borrowed mutably, it has one exclusive editor;  
While it is borrowed immutably, it can have many readers but no editor.

### Why this also matters for threads

A data race requires unsynchronized concurrent access where at least one access writes.  
Rust applies the same aliasing rules to safe concurrent code through its type system.  
Ownership and borrowing alone do not make every program logically correct, but they remove a large category of accidental shared-mutation bugs before execution.

[Go to the Top](#table-of-content)

## 6. References must always be valid

Rust rejects a reference that could outlive the value it points to.  
That prevents dangling references and use-after-free errors.

```rust,compile_fail
fn invalid_reference() -> &'static String {
    let local = String::from("temporary");
    &local // error: `local` is dropped when this function returns
}
```

The local `String` is dropped at the end of `invalid_reference`; returning `&local` would leave the caller with a pointer to freed memory.  
Return the owned value instead:

```rust
fn valid_value() -> String {
    String::from("returned safely")
}

fn main() {
    let message = valid_value();
    println!("{message}");
}
```

When references are passed between functions or stored in types, Rust sometimes needs lifetime annotations to describe how long the references are valid.  
The related [lifetimes chapter](./lifetimes_in_rust.md) covers that contract in detail.  
Ownership and borrowing are still the foundation: annotations describe relationships between existing borrows; they never extend a value's actual lifetime.

[Go to the Top](#table-of-content)

## 7. Slices: borrowed views into a collection

A slice is a reference to a contiguous part of a collection rather than an owned copy.  
The most common slice types are `&str` for text and `&[T]` for arrays or vectors.

```rust
fn first_word(text: &str) -> &str {
    for (index, byte) in text.bytes().enumerate() {
        if byte == b' ' {
            return &text[..index];
        }
    }

    text
}

fn main() {
    let sentence = String::from("Rust protects memory");
    let word = first_word(&sentence);

    println!("first word: {word}");
}
```

`word` does not allocate or copy `"Rust"`; it points into `sentence`.  
Because the slice is a borrow, Rust will not allow `sentence` to be mutated in a way that could invalidate `word` while that slice is still used.

```rust
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn main() {
    let values = vec![3, 5, 8, 13];
    println!("{}", sum(&values));
    println!("{}", sum(&values[1..3])); // borrows only [5, 8]
}
```

Slices make APIs flexible: `sum` can accept a borrowed `Vec<i32>`, an array converted to a slice, or a range from either collection.

[Go to the Top](#table-of-content)

## 8. Choosing ownership or borrowing in an API

The parameter and return types of a function communicate its resource contract.

| Signature shape       | Meaning           | Typical use                                                        |
| --------------------- | ----------------- | ------------------------------------------------------------------ |
| `fn f(value: T)`      | Takes ownership   | The function must retain, transform, or consume `value`.           |
| `fn f(value: &T)`     | Reads a borrow    | The function only needs temporary read access.                     |
| `fn f(value: &mut T)` | Mutates a borrow  | The function changes a caller-owned value.                         |
| `fn f() -> T`         | Returns ownership | The caller becomes responsible for the new value.                  |
| `fn f() -> &T`        | Returns a borrow  | The returned reference must be tied to valid input or stored data. |

Here is a small example with all three parameter styles:

```rust
fn display_order(order: &str) {
    println!("Order: {order}");
}

fn mark_paid(order: &mut String) {
    order.push_str(" (paid)");
}

fn archive(order: String) -> usize {
    // Imagine this function writes `order` to durable storage.
    order.len()
} // The consumed String is dropped here after archival work finishes.

fn main() {
    let mut order = String::from("A-1024");

    display_order(&order);
    mark_paid(&mut order);
    let stored_bytes = archive(order);

    println!("archived {stored_bytes} bytes");
    // `order` cannot be used here: `archive` consumed it.
}
```

As a practical default, borrow inputs when a function only needs to inspect them.  
Take ownership when retaining the value or when consuming it makes the caller's responsibility unambiguous.  
Use `&mut` only when mutation is part of the function's job.

[Go to the Top](#table-of-content)

## 9. A practical mental model

When a compiler error mentions a move or a borrow, answer these questions in order:

1. **Who owns this value now?** An assignment, function call, or return may have moved ownership.
2. **Does this code need the value itself or only access to it?** If it only needs access, pass `&value` instead of `value`.
3. **Does the access need to mutate?** Use `&mut value` only when necessary.
4. **Are there other active borrows?** Many readers are allowed; a writer must be alone.
5. **Will the referenced data live long enough?** A reference cannot outlive its owner.

Ownership is not an extra cleanup task imposed on the programmer.  
It is the information Rust needs to generate automatic cleanup and to prove that safe code cannot use freed memory or mutate the same value through conflicting aliases.  
Once moves and borrows are visible in function signatures and assignments, the rules become a reliable design tool rather than a set of compiler obstacles.

[Go to the Top](#table-of-content)
