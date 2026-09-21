<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD012 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# NEON

This project helps get the developer familiar with ownership, lifetimes, and error-handling in rust.  

## Table of Content

- [Exercise 036 (Move Semantics)](#exercise-036)
- [Exercise 037 (Copy vs Move)](#exercise-037)
- [Exercise 038 (Ownership Through Function Boundaries)](#exercise-038)
- [Exercise 039 (Borrowing Rules and Non-Lexical Lifetimes)](#exercise-039)

---

## Exercise 036

Rust uses **ownership transfer** instead of implicitly copying owned values.  
When ownership of a value moves from one binding to another, the original binding becomes unusable.  

### 1. Binding → Binding

```rust
let s1 = String::from("hello");
let s2 = s1;

println!("{}", s2);
```

Ownership moves:

```text
s1 ──move──> s2
```

`s1` can no longer be used.  
No deep copy of the heap data occurs

### 2. Ownership and Function Calls

```rust
fn consume(s: String) {
    println!("{}", s);
}

let s1 = String::from("hello");
consume(s1);
```

Ownership moves into the function:

```text
s1 ──move──> s
```

`s` becomes the owner inside `consume()`.  
When `consume()` returns, `s` goes out of scope and the `String` is dropped.

Therefore:

```rust
println!("{}", s1); // ERROR
```

is invalid because `s1` no longer owns the value.

### 3. Moving Ownership Back Through a Return Value

```rust
fn consume(s: String) -> String {
    println!("{}", s);
    s
}

let s1 = String::from("hello");
let s2 = consume(s1);
```

Ownership moves multiple times:

```text
s1 ──move──> s ──move──> s2
```

The value is not copied.  
When `s` goes out of scope, it does not drop the `String`, because ownership has already moved out of `s`.  
`s2` is now the owner.

### 4. Returning a Local Value

```rust
fn create() -> String {
    let s = String::from("hello");
    s
}

fn main() {
    let s1 = create();
}
```

Ownership moves:

```text
create()
    s ──move──> s1
```

Although `s` is a local variable, the `String` is **not dropped** when `create()` ends because ownership has already moved out of `s`.  
The binding `s` becomes unusable, while `s1` becomes the owner.  

### 5. Drop and Scope

The owner is responsible for the eventual destruction of the value.

```text
Owner goes out of scope
        ↓
Value is dropped
        ↓
Owned resources are released
```

For `String`, dropping it eventually releases its heap allocation.

### Key Rules

#### Rule 1 — Move transfers ownership

```text
old owner ──move──> new owner
```

#### Rule 2 — The old binding becomes unusable

Rust statically prevents using a value after ownership has moved from it.

#### Rule 3 — Move does not mean deep copy

For a `String`, the heap-allocated contents are not copied during a move.

#### Rule 4 — Values can move through multiple owners

```text
s1 → function parameter → return value → s2
```

#### Rule 5 — A value can move out of a scope

A local variable can transfer ownership to its caller.

```text
local variable → caller
```

### Final Mental Model

> **A value has one owner at a time.**  
> **Moving transfers ownership without copying the underlying owned data.**  
> **The previous binding becomes unusable, and the new owner becomes responsible for eventually dropping the value.**

[Go to the Top](#table-of-content)

---

## Exercise 037

Rust distinguishes between **Move**, **Copy**, and **Clone**.  

The key question is:

> When a value is assigned to another variable, does ownership move, or is the value duplicated?

### 1. Move

For types that do not implement `Copy`, assignment moves ownership.

```rust
let s1 = String::from("hello");
let s2 = s1;

println!("{}", s1); // ERROR
```

Ownership:

```text
s1 ──move──> s2
```

After the move, `s1` is no longer usable.  
No deep copy of the heap data occurs.

### 2. `Copy` Trait

Some types implement the `Copy` trait.

For example:

```rust
let x = 10;
let y = x;

println!("{}", x);
println!("{}", y);
```

Since `i32: Copy`, the assignment performs an implicit copy.

```text
x = 10
│
├──copy──> y = 10
```

Both remain valid.

`Copy` is a **marker trait**.  
It does not define a `copy()` method that the programmer calls.  

Conceptually:

```rust
trait Copy {}
```

The compiler knows that types implementing `Copy` can be duplicated implicitly.
Common `Copy` types include: `i32, i64, u32, bool, char, f32, f64`

### 3. `Clone` Trait

`Clone` represents **explicit duplication**.  
Its important method is conceptually:

```rust
fn clone(&self) -> Self;
```

Example:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();
```

This creates an independent `String`.  
Conceptually:

```text
s1 ──→ heap A: "hello"

s2 ──→ heap B: "hello"
```

Both values are independently owned.
Unlike `Copy`, `Clone` can perform arbitrary work, including heap allocation.  

### 4. `Copy` vs `Clone`

| Property                   | `Copy` | `Clone`    |
| -------------------------- | ------ | ---------- |
| Implicit                   | Yes    | No         |
| Explicit method            | No     | `.clone()` |
| Can perform arbitrary work | No     | Yes        |
| Can involve allocation     | No     | Yes        |
| `i32`                      | Yes    | Yes        |
| `String`                   | No     | Yes        |

Every `Copy` type must also implement `Clone`.  

Therefore:

```rust
let x = 10;

let y = x;         // implicit Copy
let z = x.clone(); // explicit Clone
```

are both valid.

### 5. Custom Types and `Copy`

A custom type can implement `Copy` only when all of its fields are also `Copy`.  

This works:

```rust
#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}
```

Because:

```text
Point
 ├── x: i32 → Copy
 └── y: i32 → Copy
```

Therefore:

```rust
let p1 = Point { x: 10, y: 20 };
let p2 = p1;

println!("{}", p1.x); // OK
println!("{}", p2.x); // OK
```

`p1` is copied rather than moved.

### 6. Why `String` Cannot Be `Copy`

This does not work:

```rust
#[derive(Copy, Clone)]
struct User {
    name: String,
}
```

because `String` does not implement `Copy`.  

A `String` owns heap memory.  
A simple bitwise copy would result in two `String` values referring to the same allocation, which would violate Rust's ownership rules.

Instead, use `Clone`:

```rust
let u2 = u1.clone();
```

This creates an independent allocation.

### 7. `Clone` Does Not Mean `Copy`

Having `Clone` does not automatically make a type `Copy`.

```rust
#[derive(Clone)]
struct User {
    name: String,
}
```

This is valid:

```rust
let u2 = u1.clone();
```

But:

```rust
let u2 = u1;
```

is a **move**, not a clone.

After the move:

```text
u1 ──move──> u2
```

`u1` is no longer usable.

### 8. Custom `Clone`

`Clone` can be implemented manually for custom types.

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point {
            x: self.x + 100,
            y: self.y + 100,
        }
    }
}
```

Therefore:

```rust
let p1 = Point { x: 10, y: 20 };
let p2 = p1.clone();
```

can have custom cloning semantics.

This demonstrates that:

> `Clone` is an explicit operation whose behavior is defined by the type's implementation.

### 9. Multiple Copies

For a `Copy` type:

```rust
let x = 10;

let y = x;
let z = x;
```

`x` is copied twice:

```text
x = 10
│
├──copy──> y = 10
│
└──copy──> z = 10
```

`x` remains usable because no ownership transfer occurred.

### 10. Move After Clone

Consider:

```rust
let s1 = String::from("hello");

let s2 = s1.clone();
let s3 = s1;
```

After `clone()`:

```text
s1 → heap A
s2 → heap B
```

Then:

```rust
let s3 = s1;
```

moves the original value:

```text
s1 → moved
s2 → heap B
s3 → heap A
```

Therefore:

```rust
println!("{}", s1); // 
println!("{}", s2); // 
println!("{}", s3); // 
```

`clone()` does not make `s1` permanently immune to moves. It simply creates another independent value while leaving `s1` usable.

### Final Mental Model

```text
MOVE
  ownership transfers
  original binding becomes unusable


COPY
  implicit duplication
  original remains usable
  type implements Copy


CLONE
  explicit duplication via .clone()
  original remains usable
  can perform expensive/custom work
```

The fundamental distinction is:

> **Move transfers ownership.**  
> **Copy duplicates a value implicitly. Clone explicitly creates another value.**

[Go to the Top](#table-of-content)

---

## Exercise 038

Ownership moves not only between variables, but also **across function boundaries**.  
When an owned value is passed to a function, ownership is transferred to the function parameter.  
If that function passes the value to another function, ownership can move again.

### 1. Ownership Moving Through Functions

Consider:

```rust
fn take(s: String) {
    println!("take: {}", s);
}

fn process(s: String) {
    take(s);

    println!("process: {}", s);
}

fn main() {
    let value = String::from("hello");

    process(value);

    println!("main: {}", value);
}
```

This does not compile.

The ownership chain is:

```text
main::value
      |
      | move
      v
process::s
      |
      | move
      v
take::s
```

After:

```rust
process(value);
```

`value` has been moved into `process()`.  

Therefore:

```rust
println!("main: {}", value);
```

is invalid.

Inside `process()`:

```rust
take(s);
```

moves ownership from `process::s` into `take::s`.

Therefore:

```rust
println!("process: {}", s);
```

is also invalid.

### 2. Ownership Can Be Returned

Ownership can be transferred back through a function's return value.

```rust
fn take(s: String) -> String {
    println!("take: {}", s);
    s
}

fn process(s: String) -> String {
    let s = take(s);

    println!("process: {}", s);

    s
}

fn main() {
    let value = String::from("hello");

    let value = process(value);

    println!("main: {}", value);
}
```

Ownership moves through the functions:

```text
main::value
      |
      | move
      v
process::s
      |
      | move
      v
take::s
      |
      | return ownership
      v
process::s
      |
      | return ownership
      v
main::value
```

No deep copy is required.

The same `String` ownership is transferred between bindings.

### 3. Borrowing Instead of Moving

If a function only needs to read a value, it does not necessarily need to take ownership.

Instead, it can borrow the value:

```rust
fn take(s: &str) {
    println!("take: {}", s);
}

fn process(s: &str) {
    take(s);
    println!("process: {}", s);
}

fn main() {
    let value = String::from("hello");

    process(&value);

    println!("main: {}", value);
}
```

Here, `value` remains the owner throughout the entire operation.

Conceptually:

```text
    main
    │
    │ owns
    v
    value ────> String
    │
    │ borrow
    v
    process(&value)
    │
    │ borrow
    v
    take(s)
```

Neither `process()` nor `take()` becomes the owner.

### 4. Move vs Borrow

#### Move

```rust
fn process(s: String) {
    // s owns the String
}

process(value);
```

Ownership:

```text
value ──move──> s
```

After the call, `value` cannot be used.

### Borrow

```rust
fn process(s: &str) {
    // s temporarily accesses the String data
}

process(&value);
```

Ownership:

```text
value ──owns──> String
   |
   └── borrow ──> process
```

After the call, `value` remains usable.

### 5. Important Observation

A function parameter determines what happens to ownership.

```rust
fn process(s: String)
```

means:

> `process` receives ownership.

Whereas:

```rust
fn process(s: &String)
```

or:

```rust
fn process(s: &str)
```

means:

> `process` receives a reference and does not take ownership.

For read-only string processing, `&str` is often preferable because it can accept both:

```rust
String
```

and string literals:

```rust
&str
```

### 6. Reference Level Matters

If:

```rust
fn process(s: &str)
```

then `s` is already a reference.

Therefore:

```rust
take(s);
```

is normally correct.

But:

```rust
take(&s);
```

creates another level of reference:

```text
s:    &str
&s:  &&str
```

This is an important distinction when working with references.

### Key Rules

1. Passing an owned value to a function can transfer ownership.
2. A function parameter of type `String` becomes the owner of the passed `String`.
3. Ownership can be transferred again to another function.
4. Ownership can be returned through a function's return value.
5. A reference allows a function to access a value without taking ownership.
6. Passing `&value` borrows the value rather than moving it.
7. A function receiving `&str` does not own the underlying `String`.
8. A reference can itself be borrowed, producing another reference level such as `&&str`.

### Final Mental Model

```text
Ownership transfer:

main
  |
  | String
  v
process
  |
  | String
  v
take


Borrowing:

main
  |
  | owns String
  v
value
  |
  | &str
  v
process
  |
  | &str
  v
take
```

The fundamental distinction is:

> **Passing an owned value transfers ownership. Passing a reference transfers access, not ownership.**


[Go to the Top](#table-of-content)

---

## Exercise 039

Rust allows values to be borrowed through references without transferring ownership.  
There are two fundamental forms:

```text
Immutable borrow
&value

Mutable borrow
&mut value
```

Rust enforces strict rules around these references to prevent invalid memory access and data races.

### 1. Immutable References

Multiple immutable references can exist simultaneously.

```rust
let s = String::from("hello");

let r1 = &s;
let r2 = &s;

println!("{}", r1);
println!("{}", r2);
println!("{}", s);
```

This is valid.

Ownership remains with `s`:

```text
       ┌── r1 ──→
s ─────┤
       └── r2 ──→
```

`r1` and `r2` only provide read access.

They do not own the `String`.

### 2. The Fundamental Borrowing Rule

Rust follows this rule:

```text
Either:

0 or more active immutable references

OR

exactly 1 active mutable reference

Never both simultaneously.
```

Conceptually:

```text
Immutable:

s ──→ r1
  ├─→ r2
  └─→ r3

Allowed.
```

But:

```text
s ──→ r1
  ├─→ r2
  └─→ r3 (&mut)

Not allowed.
```

### 3. Mutable References

A mutable reference provides exclusive mutable access.

```rust
let mut s = String::from("hello");

let r1 = &mut s;

r1.push_str(" world");
```

This is valid.

However:

```rust
let r1 = &mut s;
let r2 = &mut s;
```

is invalid because two active mutable references exist simultaneously.

```text
s
├── r1 (&mut)
└── r2 (&mut)   ← conflict
```

### 4. Why Multiple Mutable References Are Prohibited

Multiple mutable references could allow conflicting access to the same memory.

This is especially important for preventing data races in concurrent programs.

Rust therefore enforces:

> **At most one active mutable reference to a value at any given point.**

This rule also applies to single-threaded code.

### 5. Mutable and Immutable References Cannot Overlap

This is invalid:

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &mut s;

println!("{}", r1);
println!("{}", r2);
```

`r1` is still needed when `r2` is created.

Therefore the immutable borrow and mutable borrow overlap.

```text
r1: immutable borrow ──────────────┐
                                   │
r2: mutable borrow                 │
      ↑                            │
      └──── conflict ──────────────┘
```

### 6. Non-Lexical Lifetimes

Rust uses **Non-Lexical Lifetimes (NLL)** to determine when a borrow is actually needed.

Consider:

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;

println!("{}", r1);
println!("{}", r2);

let r3 = &mut s;

r3.push_str(" world");
```

This compiles.

Why?

The immutable references are no longer used after:

```rust
println!("{}", r2);
```

Therefore their borrows can end before:

```rust
let r3 = &mut s;
```

Conceptually:

```text
r1 created
r2 created
    │
    ├── r1 used
    └── r2 used
          ↓
    immutable borrows end
          ↓
    r3 = &mut s
```

The borrow lifetime is determined by actual usage rather than simply by the enclosing `{}` scope.

### 7. Sequential Mutable Borrows

Multiple mutable references can exist **sequentially**.

```rust
let mut s = String::from("hello");

let r1 = &mut s;
r1.push_str(" world");

let r2 = &mut s;
r2.push_str("!");
```

This is valid.

The sequence is:

```text
r1 created
   ↓
r1 used
   ↓
r1 borrow ends
   ↓
r2 created
   ↓
r2 used
```

The important distinction is:

> Multiple mutable references cannot be **active simultaneously**, but they can be created sequentially.

### 8. NLL Does Not Make Every Borrow Safe

Consider:

```rust
let mut s = String::from("hello");

let r1 = &mut s;

r1.push_str(" world");

let r2 = &mut s;

r2.push_str("!");

println!("{}", r1);
println!("{}", r2);
```

This does not compile.

Although `r1` was used before `r2` was created, `r1` is used again afterward.

Therefore Rust must keep the borrow associated with `r1` active until that later use.

Conceptually:

```text
r1 created
   ↓
r1 used
   ↓
r2 created
   ↓
r2 used
   ↓
r1 used again
   ↑
   └── conflict
```

### 9. Borrow Lifetime vs Variable Scope

A reference variable may exist in a lexical scope while its **borrow is no longer active**.

For example:

```rust
let mut s = String::from("hello");

let r1 = &s;

println!("{}", r1);

let r2 = &mut s;
```

The variable `r1` is still technically in scope, but its borrow can end after its last use.

This is one of the important effects of NLL.

Therefore:

> **Variable scope and borrow lifetime are related but are not necessarily identical.**

### Key Rules

1. A value can have multiple active immutable references.
2. A value can have at most one active mutable reference.
3. Immutable and mutable references cannot be active simultaneously.
4. A reference does not own the underlying value.
5. Mutable references provide exclusive mutable access.
6. Multiple mutable references can exist sequentially.
7. NLL allows a borrow to end after its last actual use.
8. A borrow may remain active longer if the reference is used later.
9. Rust's borrow checker determines whether references overlap in an invalid way.

### Final Mental Model

```text
                    Borrowing
                       │
             ┌─────────┴─────────┐
             ↓                   ↓
        Immutable             Mutable
          &T                   &mut T
             │                   │
             ↓                   ↓
      Multiple allowed      Only one active
             │                   │
             └─────────┬─────────┘
                       ↓
                Cannot overlap
                       │
                       ↓
                Borrow checker
                       │
                       ↓
                      NLL
                       │
                       ↓
          Borrow ends after last use
```

The fundamental rule:

> **At any given point, a value can have either multiple active immutable borrows or one active mutable borrow, but never both.**

And the key NLL principle:

> **A borrow generally remains active only for as long as the reference is actually needed.**

[Go to the Top](#table-of-content)

---

## Exercise 040

[Go to the Top](#table-of-content)

---

## Exercise 041

[Go to the Top](#table-of-content)

---

## Exercise 042

[Go to the Top](#table-of-content)

---

## Exercise 043

[Go to the Top](#table-of-content)

---

## Exercise 044

[Go to the Top](#table-of-content)

---

## Exercise 045

[Go to the Top](#table-of-content)

---

