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

This project helps get the developer familiar with ownership, lifetimes, error-handling in rust and collections library.  

## Table of Content

### Ownership & Lifetimes

- [Exercise 036 (Move Semantics)](#exercise-036)
- [Exercise 037 (Copy vs Move)](#exercise-037)
- [Exercise 038 (Ownership Through Function Boundaries)](#exercise-038)
- [Exercise 039 (Borrowing Rules and Non-Lexical Lifetimes)](#exercise-039)
- [Exercise 040 (Borrow Lifetimes and Scope)](#exercise-040)
- [Exercise 041 (Non-Lexical Lifetimes in Practice)](#exercise-041)
- [Exercise 042 (Lifetime Annotations and Lifetime Relationships)](#exercise-042)
- [Exercise 043 (Lifetimes in Structs)](#exercise-043)
- [Exercise 044 ('static Lifetime and static Items)](#exercise-044)
- [Exercise 045 (Returning References Safely)](#exercise-045)

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

### 1. Borrowing Through Functions

```rust
fn read(s: &String) {
    println!("{}", s);
}

fn modify(s: &mut String) {
    s.push_str(" world");
}

fn main() {
    let mut value = String::from("hello");

    read(&value);
    modify(&mut value);

    println!("{}", value);
}
```

This compiles because the immutable borrow created for `read()` ends after the call.

```text
value
  │
  ├── immutable borrow → read()
  │                         ↓
  │                    borrow ends
  │
  └── mutable borrow → modify()
```

### 2. NLL and Last Use

```rust
let r = &value;

read(r);

modify(&mut value);
```

This compiles because `read(r)` is the last use of `r`.

NLL allows the immutable borrow to end there, even though `r` remains in lexical scope.

```text
r created
   ↓
read(r)
   ↓
last use
   ↓
borrow ends
   ↓
&mut value allowed
```

### 3. Borrow Used Later

```rust
let r = &value;

read(r);

modify(&mut value);

println!("{}", r);
```

This does not compile.

The later use of `r` means its immutable borrow must remain active when `modify()` attempts to create a mutable borrow.

```text
r → immutable borrow
          │
          ├── modify(&mut value)  ← conflict
          │
          └── println!("{}", r)
```

### 4. Explicit Inner Scope

```rust
{
    let r = &value;
    println!("{}", r);
}

modify(&mut value);
```

The inner block limits the lexical scope of `r`.

When the block ends:

```text
r goes out of scope
       ↓
borrow ends
       ↓
mutable borrow allowed
```

However, in simple cases NLL can already end the borrow at its last use, so an explicit block is not always necessary.

### 5. Inner Scope Cannot Override Another Active Borrow

```rust
let r1 = &value;

{
    let r2 = &mut value;
    r2.push_str(" world");
}

println!("{}", r1);
```

This does not compile.

`r1` is still needed later, so its immutable borrow remains active while `r2` attempts a mutable borrow.

```text
r1 → immutable borrow ──────────────┐
                                   │
r2 → mutable borrow                │
      ↑                            │
      └──── conflict ──────────────┘
```

The inner scope only limits `r2`; it cannot shorten the lifetime of `r1`.

### Key Rules

1. A borrow lifetime is not necessarily the same as a variable's lexical scope.
2. NLL allows a borrow to end after its last actual use.
3. A reference variable can remain in scope after its borrow has ended.
4. An explicit block can constrain a variable's scope.
5. A block cannot make another active borrow disappear.
6. Mutable and immutable borrows cannot overlap.

### Final Mental Model

```text
Variable scope
      ↓
Where the variable exists

Borrow lifetime
      ↓
How long the reference is actually needed

NLL
      ↓
Can shorten borrow lifetime to last use
```

> **The borrow checker cares about when a reference is actually used, not merely whether the reference variable is still in scope.**


[Go to the Top](#table-of-content)

---

## Exercise 041

**NLL (Non-Lexical Lifetimes)** means a borrow can end when its **last actual use** ends, rather than necessarily at the end of its lexical scope.

```rust
let mut value = String::from("hello");

let r = &value;
println!("{}", r);      // last use of r

value.push_str(" world"); // allowed
```

Although `r` remains in scope, its borrow has ended after its last use.

### Key Distinction

- **Variable scope** — where a variable exists.
- **Reference** — the value stored in the variable.
- **Active borrow** — the period during which Rust must enforce the borrowing relationship.

These are not necessarily the same duration.

### Important Rule

Rust allows:

- Multiple immutable borrows simultaneously.
- One exclusive mutable borrow.
- A mutable borrow after immutable borrows have ended.
- NLL determines when those borrows actually end.

```text
immutable borrow
      |
      v
  last use
      |
      v
borrow ends
      |
      v
mutable borrow allowed
```

### Key Takeaway

> A borrow's lifetime is determined by how the reference is actually used, not simply by the surrounding `{}` scope.

This provides the foundation for understanding **explicit lifetime annotations** in Exercise 42.

[Go to the Top](#table-of-content)

---

## Exercise 042

### 1. Why Lifetimes Exist

A reference must never outlive the data it points to.

Consider:

```rust
let result;

{
    let a = String::from("hello");
    result = &a;
}

println!("{}", result);
```

This is invalid because `a` is dropped at the end of the inner block while `result` is still being used.

The result would be a **dangling reference**.  
Rust's borrow checker prevents this at compile time.

### 2. The Problem with Returning References

Consider:

```rust
fn longer(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

The function can return either `x` or `y`.  

The compiler therefore needs to know:

> What is the lifetime relationship between the returned reference and the input references?

This is not a runtime problem.  
It is a compile-time ownership and borrowing problem.

### 3. Introducing `'a`

We can express the relationship explicitly:

```rust
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

Conceptually:

```text
x ──────┐
        ├── 'a ──> returned reference
y ──────┘
```

`'a` is a **lifetime parameter**.

It names a lifetime relationship.  
It is not a fixed duration and it does not extend the lifetime of any value.

### 4. Lifetime Annotations Are Contracts

The function signature acts as a contract.

```rust
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str
```

Conceptually says:

> The returned reference is related to the lifetimes of `x` and `y`.

The compiler uses this contract when checking callers.

For example:

```rust
let a = String::from("hello");
let b = String::from("world!");

let result = longer(&a, &b);

println!("{}", result);
```

This is safe because both `a` and `b` remain alive while `result` is used.

### 5. The Shorter Lifetime Matters

Consider:

```rust
let a = String::from("hello");

let result;

{
    let b = String::from("world!");

    result = longer(&a, &b);
}

println!("{}", result);
```

This is rejected.

Why?

`b` is dropped before `result` is used.

Since `longer()` can return either `x` or `y`, `result` could refer to `b`.

Conceptually:

```text
a ───────────────────────────────>

b ────────────────>
                  ^
                  |
              dropped

result ──────────────────────────> X
```

The returned reference cannot safely remain usable beyond the lifetime for which its referenced data is valid.

### 6. Lifetime Does Not Mean Lexical Scope

A lifetime should not simply be thought of as:

> "The `{}` block where a variable exists."

We already saw this with NLL.

A variable can remain in lexical scope while its borrow has already ended.

```rust
let mut value = String::from("hello");

let r = &value;

println!("{}", r);

value.push_str(" world");
```

The borrow of `value` through `r` ends after the last use of `r`.

Therefore:

```text
variable scope ≠ active borrow lifetime
```

### 7. One Lifetime vs Multiple Lifetimes

Consider:

```rust
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str
```

There is one lifetime relationship:

```text
x ──┐
    ├── 'a ──> returned reference
y ──┘
```

Now consider:

```rust
fn first<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}
```

The relationships are independent:

```text
x ── 'a ──> returned reference

y ── 'b
```

This tells the compiler that the returned reference depends on `x`, not `y`.

### 8. Why Separate Lifetimes Matter

Consider:

```rust
fn first<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
    x
}
```

And:

```rust
let a = String::from("hello");

let result;

{
    let b = String::from("world!");

    result = first(&a, &b);
}

println!("{}", result);
```

This is valid.

The function always returns `x`.

Therefore:

```text
a ── 'a ───────────────> result

b ── 'b ──> dropped
```

`result` depends on `a`, so `b` does not need to remain alive.

### 9. Important Mental Model

Do not think:

```text
'a = 10 seconds
```

or:

```text
'a = this particular {}
```

Instead:

```text
'a = a named lifetime relationship
```

The actual lifetimes are determined when the function is used.  
The annotation tells Rust how those lifetimes are related.

### 10. What Lifetime Annotations Do Not Do

Lifetime annotations:

- Do not extend an object's lifetime.
- Do not prevent an object from being dropped.
- Do not allocate memory.
- Do not keep references alive.
- Do not change runtime behavior.

They provide information to the borrow checker about **relationships between references**.

### 11. Key Takeaway

The central idea is:

> **Lifetime annotations describe how the validity of one reference depends on the validity of other references.**

For example:

```rust
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str
```

means the returned reference is constrained by the lifetime relationship represented by `'a`.

Whereas:

```rust
fn first<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
```

expresses that the returned reference depends on `x` but not on `y`.

[Go to the Top](#table-of-content)

---

## Exercise 043

### 1. Structs Containing References

A struct containing a reference must specify the lifetime relationship of that reference.

```rust
struct User<'a> {
    name: &'a str,
}
```

This means:

> `User<'a>` contains a reference whose validity is associated with lifetime `'a`.

The struct does **not** own the referenced data.

```text
String
  │
  │ borrowed
  ▼
&'a str
  │
  ▼
User<'a>
```

### 2. Basic Usage

```rust
fn main() {
    let name = String::from("Alice");

    let user = User {
        name: &name,
    };

    println!("{}", user.name);
}
```

This is valid because `name` remains alive while `user.name` is being used.

### 3. Dangling Reference

This is invalid:

```rust
fn main() {
    let user;

    {
        let name = String::from("Alice");

        user = User {
            name: &name,
        };
    }

    println!("{}", user.name);
}
```

The relationship becomes:

```text
name
 │
 │ referenced by
 ▼
user.name

inner block ends
      ↓
name dropped
      ↓
user.name still exists
      ↓
dangling reference
```

Rust rejects this at compile time.

### 4. Lifetime as Part of the Type

The lifetime parameter belongs to the **reference stored inside the struct**.

```rust
struct User<'a> {
    name: &'a str,
}
```

It does not mean that the `User` itself must exist for exactly `'a`.

It means:

> The reference stored in `User<'a>` must remain valid according to the lifetime relationship represented by `'a`.

### 5. Lifetime Propagation Through Functions

Consider:

```rust
fn create_user<'a>(name: &'a str) -> User<'a> {
    User { name }
}
```

The relationship is:

```text
name: &'a str
      │
      ▼
User<'a>
```

The returned `User` contains a reference tied to the lifetime of the input reference.

Example:

```rust
fn main() {
    let name = String::from("Alice");

    let user = create_user(&name);

    println!("{}", user.name);
}
```

This is safe because `name` remains alive while `user.name` is used.

### 6. Lifetime Contract

A useful way to read:

```rust
fn create_user<'a>(name: &'a str) -> User<'a>
```

is:

> The `User` returned by this function contains a reference whose validity is tied to the lifetime of the `name` reference supplied to the function.

Lifetime annotations therefore describe **relationships between references**.

They do not:

- Extend an object's lifetime.
- Prevent an object from being dropped.
- Allocate memory.
- Keep an object alive.

### 7. Core Mental Model

```text
Owner
  │
  │ owns
  ▼
String
  │
  │ borrowed
  ▼
&'a str
  │
  │ stored inside
  ▼
User<'a>
```

The fundamental rule remains:

> A reference must never outlive the data it references.

### 8. Key Takeaway

Struct lifetime annotations are necessary because the compiler must know the lifetime relationship of references stored inside types.

The progression is:

```text
Reference
    ↓
Borrowing rules
    ↓
   NLL
    ↓
Function lifetime relationships
    ↓
Struct lifetime relationships
```

[Go to the Top](#table-of-content)

---

## Exercise 044

### 1. What Is `'static`?

`'static` is a special lifetime.

```rust
let message: &'static str = "hello";
```

It means:

> The referenced data is guaranteed to remain valid for the entire duration of the program.

It does **not** mean:

- The reference is immutable.
- The value is a compile-time constant.
- The value is never dropped.
- The value must be stored in the binary.

### 2. String Literals Have `'static` Lifetime

```rust
fn main() {
    let message: &'static str = "hello";

    println!("{}", message);
}
```

This compiles because the string literal is part of the program's static data and remains available for the entire program execution.

```text
Program starts
     │
     ▼
 "hello" exists
     │
     │
     ▼
Program ends
```

Therefore:

```rust
&'static str
```

is appropriate for a string literal.

### 3. Local Data Cannot Become `'static`

Consider:

```rust
fn main() {
    let message = String::from("hello");

    let r: &'static str = &message;

    println!("{}", r);
}
```

This does **not** compile.

`message` is local data:

```text
main starts
    │
    ▼
message created
    │
    ▼
r borrows message
    │
    ▼
main ends
    │
    ▼
message dropped
```

But `&'static str` promises that the referenced data remains valid for the entire program.  
The local `String` cannot satisfy that promise.  

#### Important distinction

The issue is not where the reference variable `r` is stored.  

The issue is the **lifetime of the data being referenced**.

### 4. `'static` Is Not the Same as "End of `main`"

This is an important distinction.

A local variable may remain alive until the end of `main`:

```rust
fn main() {
    let message = String::from("hello");
    // ...
}
```

But that does not make `message` `'static`.

`'static` means the data is valid for the **entire program lifetime**, not merely the lifetime of a particular function or scope.

Conceptually:

```text
Program lifetime
└──────────────────────────────────┘
                 'static

main scope
└────────────────────┘

message lifetime
└────────────────────┘
```

A local lifetime can happen to last a long time, but it is still not `'static`.

### 5. Returning a `'static` Reference

This is valid:

```rust
fn get_message() -> &'static str {
    "hello"
}
```

The function can safely return the reference because `"hello"` has `'static` lifetime.  
The function itself can return, while the referenced data continues to exist.

```text
get_message()
     │
     ├── returns
     ▼
"hello"
     │
     ▼
remains valid for entire program
```

### 6. Returning a Local Reference as `'static`

This is invalid:

```rust
fn get_message() -> &'static str {
    let message = String::from("hello");

    &message
}
```

The function promises:

```text
returned reference → valid for entire program
```

But the implementation creates:

```text
message
   │
   ▼
&message
   │
   ▼
function returns
   │
   ▼
message dropped
```

Therefore the returned reference would become dangling.

Rust rejects the mismatch between the function's contract and the actual lifetime.

### 7. `static` — A Different Concept

Rust also has the `static` keyword:

```rust
static MESSAGE: &str = "hello";
```

This declares a **global static item**.

A `static` item:

- Has a fixed memory location.
- Represents one global instance.
- Exists for the entire program execution.
- Can be accessed according to its visibility rules.

Conceptually:

```text
Program memory

┌─────────────────────┐
│ static MESSAGE      │
│        │            │
│        ▼            │
│     "hello"         │
└─────────────────────┘
          │
          ▼
    entire program
```

### 8. `'static` vs `static`

These are not the same concept.

#### `'static`

A lifetime:

```rust
&'static str
```

Means:

> The referenced data is valid for the entire program.

#### `static`

A declaration:

```rust
static MESSAGE: &str = "hello";
```

Means:

> Declare a global static item with a fixed memory location.

So:

```text
'a        → named lifetime relationship

'static   → entire-program lifetime

static    → global static item
```

### 9. `static` Does Not Mean "No Heap"

An important correction from the exercise:

It is tempting to think:

> `static` means the data cannot be heap allocated.

That is not the correct rule.

The important property of a `static` item is its **global/static lifetime and fixed storage location**.

The restriction we encountered was instead about **initialization**.

For example:

```rust
static MESSAGE: String = String::from("hello");
```

does not compile because `String::from("hello")` is not a valid constant expression for static initialization.

The issue is not simply:

```text
String → heap → therefore static is impossible
```

### 10. `static` vs `const`

These are also different:

```rust
static MAX_CONNECTIONS: usize = 100;

const DEFAULT_TIMEOUT: usize = 30;
```

Conceptually:

```text
static
  ↓
one specific global memory location

const
  ↓
compile-time constant value
```

A `static` represents a specific item in memory.

A `const` represents a constant value that can be evaluated at compile time and used where appropriate.

Therefore:

```text
static ≠ const
static ≠ 'static
```

### 11. `static mut`

Rust also allows mutable static items:

```rust
static mut COUNTER: i32 = 0;
```

Accessing or modifying such global mutable state requires `unsafe`:

```rust
fn main() {
    unsafe {
        COUNTER += 1;
    }
}
```

Why?

Because there is one shared memory location:

```text
             COUNTER
                │
        ┌───────┴───────┐
        ▼               ▼
    Thread A         Thread B
        │               │
        └───────┬───────┘
                ▼
          shared state
```

Uncontrolled concurrent access can cause data races and undefined behavior.  
Rust therefore cannot guarantee the necessary safety invariants automatically.  

`unsafe` means:

> The programmer is responsible for maintaining the required safety invariants.

For concurrent shared state, Rust normally provides safer mechanisms such as:

- Atomics
- `Mutex`
- `RwLock`
- Other synchronization primitives

### 12. `'static` Does Not Mean "Constant Expression"

This distinction is particularly important.

These are different concepts:

```text
'static
   ↓
lifetime of referenced data

constant expression
   ↓
value can be evaluated at compile time
```

For example:

```rust
let x: &'static str = "hello";
```

works because the string literal has `'static` lifetime.

But the reason is **not** simply "it is a constant expression."

### 13. Core Mental Model

Keep these three concepts separate:

```text
'a
 │
 └── named lifetime relationship


'static
 │
 └── reference valid for entire program


static
 │
 └── global item with fixed storage
```

And remember:

> **`'static` describes how long referenced data is valid. It does not make local data live longer.**

### 14. Key Takeaway

The most important lessons are:

1. `'static` is a lifetime.
2. `static` is a declaration.
3. `'static` means the referenced data is valid for the entire program.
4. String literals have `'static` lifetime.
5. Local variables cannot normally be borrowed as `'static`.
6. A function can safely return `&'static str` when it returns data that genuinely has `'static` lifetime.
7. `static mut` introduces shared mutable global state and therefore requires `unsafe`.
8. `'static` and `const` describe different concepts.
9. `'static` does not mean "constant expression."
10. Lifetime annotations describe relationships; they do not extend object lifetimes.

### Lifetime progression so far

```text
Borrowing
    ↓
Active borrows
    ↓
NLL
    ↓
Function lifetime relationships
    ↓
Lifetimes in structs
    ↓
'static lifetime
    ↓
static global items
```

[Go to the Top](#table-of-content)

---

## Exercise 045

### 1. Returning a Reference

Consider:

```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap()
}
```

This compiles even though no explicit lifetime is written.  
Rust can infer the lifetime relationship because there is only **one input reference**.
Conceptually, Rust understands:

```rust
fn first_word<'a>(s: &'a str) -> &'a str
```

The returned `&str` is therefore tied to the lifetime of `s`.

```text
text owns String
     │
     │ borrow
     ▼
    &str
     │
     ▼
   word
```

Therefore:

> `word` cannot outlive `text`.

### 2. Lifetime Elision

Rust has **lifetime elision rules** that allow certain lifetime annotations to be omitted when the compiler can determine the relationship unambiguously.

For:

```rust
fn first_word(s: &str) -> &str
```

there is only one input reference.

Therefore:

```text
s ───────────> returned reference
```

The relationship is unambiguous.

### 3. Multiple Input References

Consider:

```rust
fn first(x: &str, y: &str) -> &str {
    x
}
```

Rust cannot infer the lifetime relationship automatically.

There are two input references:

```text
x ──┐
    ├──> returned reference
y ──┘
```

The compiler does not simply guess which input lifetime the output should use.

The relationship must be expressed explicitly.

### 4. One Lifetime Parameter Can Be Enough

Since this function always returns `x`:

```rust
fn first<'a>(x: &'a str, y: &str) -> &'a str {
    x
}
```

The relationship is:

```text
x ── 'a ──> returned reference

y ── independent
```

Only `x` participates in the lifetime relationship of the returned reference.

#### Important lesson

> The number of input references does not determine the number of lifetime parameters.

Lifetime parameters describe **relationships**, not simply the number of references.

### 5. Returned Reference Can Outlive Another Input

Consider:

```rust
fn first<'a>(x: &'a str, y: &str) -> &'a str {
    x
}
```

And:

```rust
fn main() {
    let result;

    let x = String::from("hello");

    {
        let y = String::from("world");

        result = first(&x, &y);
    }

    println!("{}", result);
}
```

This compiles.

Why?

The returned reference is tied to `x`, not `y`.

```text
x ────────────────────────>
 │
 └── 'a ──> result


y ──────────>
             │
             └── dropped
```

`y` can be dropped because `result` does not depend on it.

### 6. Returning a Reference to Local Data

Consider:

```rust
fn get_first<'a>(text: &'a str) -> &'a str {
    text.split_whitespace().next().unwrap()
}
```

This function does not create a new `String`.

It returns a slice into the existing data:

```text
String
  │
  └───────────────┐
                  ▼
            "hello world"
                  ▲
                  │
               &str
```

Therefore the returned reference is tied to the lifetime of `text`.

### 7. Dangling Reference Through Scope

This is invalid:

```rust
fn main() {
    let result;

    {
        let text = String::from("hello world");
        result = get_first(&text);
    }

    println!("{}", result);
}
```

The relationship is:

```text
text
  │
  └── &str ──> result
                 │
                 ▼
            println!
```

But:

```text
inner block ends
       ↓
text dropped
       ↓
result still used
       ↓
dangling reference
```

Rust rejects the program.

### 8. Explicitly Dropping the Owner

The same problem can occur even without a separate inner scope:

```rust
fn main() {
    let text = String::from("hello world");

    let result = get_first(&text);

    drop(text);

    println!("{}", result);
}
```

This also fails.

`drop(text)` ends the ownership of `text` before the last use of `result`.

The lifetime relationship is:

```text
text
  │
  └── 'a ──> result
                 │
                 ▼
            println!(result)
```

But:

```text
drop(text)
    ↓
text no longer exists
    ↓
result still required
    ↓
invalid
```

This connects lifetime checking with **NLL**: Rust considers the actual use of the reference and rejects the explicit drop because the reference is still needed.

### 9. Lifetime Annotations Do Not Extend Lifetimes

A function such as:

```rust
fn get_first<'a>(text: &'a str) -> &'a str
```

does not make `text` live longer.

It only establishes:

```text
input reference
      │
      └──> returned reference
```

The returned reference can remain valid only while the referenced data remains valid.

### 10. Lifetime Elision vs Explicit Lifetimes

#### Elision

```rust
fn first_word(s: &str) -> &str
```

Rust can infer the relationship.

#### Explicit

```rust
fn first<'a>(x: &'a str, y: &str) -> &'a str
```

The relationship needs to be stated because multiple input references are involved.

### 11. Core Mental Model

When a function returns a reference, ask:

1. **What data does the returned reference point into?**
2. **Which input reference does it depend on?**
3. **How long does that underlying data remain valid?**
4. **Can the returned reference be used after that data is dropped?**

If the answer to the last question is yes, Rust rejects the program.

### 12. Key Takeaway

The central rule is:

> **A returned reference must never outlive the data it references.**

Lifetime annotations allow us to describe that relationship explicitly.

Lifetime elision allows Rust to omit annotations when the relationship is unambiguous.

The overall progression is:

```text
Ownership
    ↓
Borrowing
    ↓
   NLL
    ↓
Lifetime relationships
    ↓
Lifetime annotations
    ↓
Struct lifetimes
    ↓
'static
    ↓
Lifetime elision
    ↓
Returning references safely
```

[Go to the Top](#table-of-content)

---
