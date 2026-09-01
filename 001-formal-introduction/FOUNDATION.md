<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD040 -->

# Rust Core Language Foundations

## 1. Overview

Rust is a **statically typed**, **compiled**, **expression-oriented** systems programming language.  

Three foundational concepts to understand before moving into ownership and borrowing are:

1. **Data Types** — what kind of values Rust can represent
2. **Immutability** — why variables cannot be changed by default
3. **Control Flow** — how Rust evaluates conditions, loops, and expressions

This document focuses strictly on these three areas.

---

## 2. Data Types

Rust is statically typed.

The compiler must know the type of every value at compile time, although Rust can often **infer the type** without requiring an explicit annotation.

```rust
let age = 25;
```

Rust infers:

```rust
let age: i32 = 25;
```

You can also explicitly specify the type:

```rust
let age: i32 = 25;
```

Rust's types can broadly be divided into:

* Scalar types
* Compound types
* String-related types
* User-defined types

---

## 3. Scalar Types

Scalar types represent a single value.  
Rust has four primary scalar types:

* Integers
* Floating-point numbers
* Booleans
* Characters

---

## 3.1 Integer Types

Rust provides signed and unsigned integers of different sizes.

| Type    |                   Size | Range                          |
| ------- | ---------------------: | ------------------------------ |
| `i8`    |                  8-bit | `-2^7` to `2^7 - 1`            |
| `i16`   |                 16-bit | `-2^15` to `2^15 - 1`          |
| `i32`   |                 32-bit | `-2^31` to `2^31 - 1`          |
| `i64`   |                 64-bit | `-2^63` to `2^63 - 1`          |
| `i128`  |                128-bit | `-2^127` to `2^127 - 1`        |
| `isize` | architecture-dependent | Signed pointer-sized integer   |
| `u8`    |                  8-bit | `0` to `2^8 - 1`               |
| `u16`   |                 16-bit | `0` to `2^16 - 1`              |
| `u32`   |                 32-bit | `0` to `2^32 - 1`              |
| `u64`   |                 64-bit | `0` to `2^64 - 1`              |
| `u128`  |                128-bit | `0` to `2^128 - 1`             |
| `usize` | architecture-dependent | Unsigned pointer-sized integer |

### Signed vs Unsigned

Signed integers can represent negative and positive values:

```rust
let temperature: i32 = -10;
```

Unsigned integers can represent only non-negative values:

```rust
let count: u32 = 100;
```

---

## 3.2 Default Integer Type

When Rust cannot infer a more specific integer type, the default integer type is generally `i32`.

```rust
let x = 42;
```

Conceptually:

```rust
let x: i32 = 42;
```

You can override it:

```rust
let x: i64 = 42;
let y: u8 = 42;
```

---

## 3.3 Integer Literals

Rust supports different representations for integer literals.

```rust
let decimal = 98_222;
let hex = 0xff;
let octal = 0o77;
let binary = 0b1111_0000;
let byte = b'A';
```

Underscores can improve readability:

```rust
let million = 1_000_000;
```

They have no semantic effect.

---

## 4. Floating-Point Types

Rust provides:

* `f32`
* `f64`

Example:

```rust
let x: f32 = 3.14;
let y: f64 = 3.1415926535;
```

The default floating-point type is `f64`.

```rust
let pi = 3.14159;
```

is inferred as:

```rust
let pi: f64 = 3.14159;
```

For most general-purpose calculations, `f64` is the usual choice.

---

## 5. Boolean Type

Rust has a `bool` type with two possible values:

```rust
true
false
```

Example:

```rust
let is_authenticated: bool = true;
let is_admin = false;
```

Booleans are commonly used in conditions:

```rust
let is_authenticated = true;

if is_authenticated {
    println!("Access granted");
}
```

---

## 6. Character Type

Rust's `char` represents a **Unicode scalar value**.

```rust
let letter: char = 'A';
let emoji: char = '😀';
let symbol: char = '₹';
```

Character literals use **single quotes**:

```rust
let c = 'A';
```

String literals use **double quotes**:

```rust
let s = "A";
```

This distinction is important.

```rust
'A'   // char
"A"   // string slice
```

A Rust `char` is 4 bytes in size.

It is therefore not equivalent to Java's 16-bit `char`.

---

## 7. Compound Types

Compound types group multiple values together.  
Rust has two primitive compound types:

1. Tuple
2. Array

---

## 8. Tuple

A tuple groups multiple values of potentially different types.

```rust
let user = ("Qwerty", 25, true);
```

The inferred type is approximately:

```rust
(&str, i32, bool)
```

A tuple can contain different types:

```rust
let data: (i32, f64, bool) = (10, 3.14, true);
```

---

### 8.1 Accessing Tuple Elements

Tuple elements can be accessed using their positional index.

```rust
let user = ("Qwerty", 25, true);

println!("{}", user.0);
println!("{}", user.1);
println!("{}", user.2);
```

Output:

```text
Qwerty
25
true
```

Indexes start at `0`.

---

### 8.2 Tuple Destructuring

Rust can destructure a tuple:

```rust
let user = ("Qwerty", 25, true);

let (name, age, active) = user;

println!("{}", name);
println!("{}", age);
println!("{}", active);
```

This is particularly useful when a function returns multiple values.

```rust
fn get_user() -> (&'static str, i32) {
    ("Qwerty", 25)
}

let (name, age) = get_user();
```

---

### 8.3 Unit Tuple

Rust has a special tuple with zero elements:

```rust
()
```

It is called the **unit type**.

Example:

```rust
fn do_something() {
    println!("Done");
}
```

A function with no explicit return type returns `()`.

Conceptually:

```rust
fn do_something() -> () {
    println!("Done");
}
```

The unit type represents "no meaningful value."

---

## 9. Arrays

An array contains multiple values of the **same type** and has a **fixed length**.

```rust
let numbers = [10, 20, 30, 40];
```

The inferred type is:

```rust
[i32; 4]
```

The syntax is:

```text
[Type; Length]
```

For example:

```rust
let numbers: [i32; 4] = [10, 20, 30, 40];
```

---

### 9.1 Array Access

Array elements are accessed by index:

```rust
let numbers = [10, 20, 30];

println!("{}", numbers[0]);
println!("{}", numbers[1]);
println!("{}", numbers[2]);
```

Output:

```text
10
20
30
```

Indexes begin at `0`.

---

### 9.2 Arrays Have Fixed Length

This is valid:

```rust
let numbers = [10, 20, 30];
```

But the number of elements cannot be changed.

If you need a dynamically sized collection, use `Vec<T>`:

```rust
let numbers = vec![10, 20, 30];

numbers.push(40);
```

`Vec<T>` will become important later when we study collections and ownership.

---

## 10. Repeating Values in an Array

Rust provides convenient syntax for creating an array containing the same value.

```rust
let zeros = [0; 5];
```

This creates:

```text
[0, 0, 0, 0, 0]
```

The type is:

```rust
[i32; 5]
```

The general form is:

```rust
[value; length]
```

Example:

```rust
let flags = [false; 10];
```

---

## 11. String Literals and `&str`

String handling deserves special attention.

Consider:

```rust
let message = "hello";
```

The type of `message` is:

```rust
&str
```

`&str` is a **string slice**.

For now, think of it as a view into string data rather than an owned, growable string.

String literals are typically stored in the program's binary and have a fixed size.

```rust
let message: &str = "hello";
```

---

## 12. `String`

Rust also provides the `String` type.

```rust
let message = String::from("hello");
```

Unlike a string literal, `String` is an owned, growable string type.

For example:

```rust
let mut message = String::from("hello");

message.push_str(" world");

println!("{}", message);
```

Output:

```text
hello world
```

At this stage, remember the basic distinction:

| Type     | Characteristics        |
| -------- | ---------------------- |
| `&str`   | String slice           |
| `String` | Owned, growable string |

The deeper implications of this distinction belong to **ownership and borrowing**, which we will cover separately.

---

## 13. Type Inference

Rust often allows you to omit types.

```rust
let age = 30;
let price = 99.99;
let active = true;
let name = "Rust";
```

The compiler determines their types from context.

You can explicitly annotate them:

```rust
let age: i32 = 30;
let price: f64 = 99.99;
let active: bool = true;
let name: &str = "Rust";
```

Explicit annotations are particularly useful when:

* The compiler cannot infer the intended type
* You want to make an API clearer
* You want to enforce a specific numeric type

---

## 14. Immutability

One of Rust's most important defaults is:

> Variables are immutable by default.

Consider:

```rust
let x = 10;

x = 20;
```

This does not compile.

The variable `x` was declared immutable.

---

## 15. Mutable Variables

To allow mutation, explicitly use `mut`.

```rust
let mut x = 10;

x = 20;

println!("{}", x);
```

Output:

```text
20
```

The keyword:

```rust
mut
```

means that the binding can be changed.

---

## 16. Why Immutability Is the Default

Rust makes mutation explicit.

Compare:

```rust
let x = 10;
```

with:

```rust
let mut x = 10;
```

The second declaration communicates:

> This variable is intentionally mutable.

This becomes especially important when writing concurrent systems because uncontrolled mutation is a major source of bugs.

Rust's ownership and borrowing system builds additional safety guarantees around mutation.

---

## 17. Immutable Does Not Mean "Value Can Never Exist Differently"

Consider:

```rust
let x = 10;

let x = 20;
```

This is valid.  
This is called **shadowing**.  
The second `x` is a new binding.  
It does not mutate the original `x`.  

---

## 18. Shadowing

Example:

```rust
let x = 10;

let x = x + 5;

println!("{}", x);
```

Output:

```text
15
```

The first binding:

```rust
x = 10
```

is shadowed by the second binding:

```rust
x = 15
```

---

## 19. Shadowing vs `mut`

These two mechanisms are fundamentally different.

### Mutation

```rust
let mut x = 10;
x = 20;
```

There is one binding, whose value changes.

### Shadowing

```rust
let x = 10;
let x = 20;
```

There are two bindings. The second hides the first.

---

## 20. Shadowing Can Change the Type

This is an important Rust feature.

```rust
let value = "42";

let value = value.parse::<i32>().unwrap();
```

Initially:

```text
value: &str
```

After shadowing:

```text
value: i32
```

This would not be possible with simple mutation because mutation does not change the variable's type.

For example:

```rust
let mut value = "42";

value = 42; // ERROR
```

The original variable has type `&str`, so assigning an `i32` is invalid.

---

## 21. Constants

Rust also supports constants.

```rust
const MAX_CONNECTIONS: u32 = 100;
```

Constants:

* Must have an explicit type
* Must be initialized with a compile-time constant expression
* Cannot be declared with `mut`
* Follow `SCREAMING_SNAKE_CASE` by convention

Example:

```rust
const MAX_RETRIES: u32 = 3;
```

Constants are different from ordinary immutable variables.

```rust
let max_retries = 3;
```

is an immutable variable.

```rust
const MAX_RETRIES: u32 = 3;
```

is a constant.

---

## 22. Scope

Variables exist within a scope.

```rust
fn main() {
    let x = 10;

    {
        let y = 20;

        println!("{}", x);
        println!("{}", y);
    }

    println!("{}", x);

    // println!("{}", y); // ERROR
}
```

`y` exists only inside the inner block.

The outer scope cannot access it.

This concept becomes extremely important when we study ownership and lifetimes.

---

## 23. Control Flow

Rust provides several control-flow mechanisms:

* `if`
* `else if`
* `else`
* `loop`
* `while`
* `for`
* `break`
* `continue`
* `match`

A major characteristic of Rust is that many control-flow constructs are **expressions**.

---

## 24. `if`

Basic example:

```rust
let age = 20;

if age >= 18 {
    println!("Adult");
}
```

The condition must evaluate to a `bool`.

Rust does not implicitly convert integers to booleans.

This is invalid:

```rust
let x = 10;

if x {
    println!("true");
}
```

You must explicitly provide a boolean condition:

```rust
if x > 0 {
    println!("positive");
}
```

---

## 25. `if / else`

```rust
let age = 20;

if age >= 18 {
    println!("Adult");
} else {
    println!("Minor");
}
```

---

## 26. `else if`

Multiple conditions can be chained:

```rust
let score = 85;

if score >= 90 {
    println!("A");
} else if score >= 75 {
    println!("B");
} else if score >= 60 {
    println!("C");
} else {
    println!("D");
}
```

---

## 27. `if` Is an Expression

This is one of the most important differences from languages such as Java.

Rust allows:

```rust
let result = if score >= 50 {
    "pass"
} else {
    "fail"
};
```

The `if` expression produces a value.

Therefore:

```rust
result
```

contains either:

```text
"pass"
```

or:

```text
"fail"
```

---

## 28. Branches Must Produce Compatible Types

Consider:

```rust
let result = if true {
    10
} else {
    "hello"
};
```

This fails because the two branches produce different types:

```text
i32
```

and:

```text
&str
```

Rust requires the expression to have a well-defined type.

Correct:

```rust
let result = if true {
    10
} else {
    20
};
```

Now both branches produce `i32`.

---

## 29. Blocks Are Expressions

A block can produce a value.

```rust
let result = {
    let x = 10;
    let y = 20;

    x + y
};
```

`result` becomes:

```text
30
```

The final expression in the block becomes its value.

---

## 30. Expression vs Statement

Consider:

```rust
let x = 10;
```

This is a statement.

But:

```rust
10 + 20
```

is an expression because it produces a value.

A block:

```rust
{
    let x = 10;
    x + 20
}
```

is an expression whose value is:

```text
30
```

---

## 31. Semicolons Matter

Consider:

```rust
let result = {
    let x = 10;
    x + 20
};
```

The final expression:

```rust
x + 20
```

does not have a semicolon.

Therefore it becomes the value of the block.

Now consider:

```rust
let result = {
    let x = 10;
    x + 20;
};
```

The semicolon turns the final expression into a statement.

The block therefore evaluates to:

```rust
()
```

This distinction is fundamental in Rust.

---

## 32. `loop`

`loop` creates an infinite loop unless explicitly terminated.

```rust
loop {
    println!("Running");
}
```

The loop continues indefinitely.

To stop it:

```rust
let mut counter = 0;

loop {
    counter += 1;

    if counter == 5 {
        break;
    }
}
```

---

## 33. `break`

`break` terminates the nearest enclosing loop.

```rust
let mut counter = 0;

loop {
    counter += 1;

    if counter >= 3 {
        break;
    }
}
```

Execution stops when `counter` reaches `3`.

---

## 34. `break` Can Return a Value

Rust allows a `loop` to produce a value.

```rust
let mut counter = 0;

let result = loop {
    counter += 1;

    if counter == 5 {
        break counter * 2;
    }
};

println!("{}", result);
```

Output:

```text
10
```

The value passed to `break` becomes the value of the `loop` expression.

---

## 35. `while`

`while` repeats a block while a condition remains true.

```rust
let mut counter = 0;

while counter < 5 {
    println!("{}", counter);
    counter += 1;
}
```

Output:

```text
0
1
2
3
4
```

The condition is checked before every iteration.

---

## 36. `continue`

`continue` skips the remainder of the current iteration and starts the next one.

```rust
let mut counter = 0;

while counter < 5 {
    counter += 1;

    if counter == 3 {
        continue;
    }

    println!("{}", counter);
}
```

Output:

```text
1
2
4
5
```

When `counter == 3`, execution jumps to the next iteration.

---

## 37. `for`

The `for` loop is commonly used for iterating over a collection or range.

```rust
for i in 0..5 {
    println!("{}", i);
}
```

Output:

```text
0
1
2
3
4
```

The range:

```rust
0..5
```

means:

```text
0, 1, 2, 3, 4
```

The upper bound `5` is excluded.

---

## 38. Inclusive Ranges

Rust also supports inclusive ranges:

```rust
for i in 0..=5 {
    println!("{}", i);
}
```

Output:

```text
0
1
2
3
4
5
```

Difference:

```text
0..5     -> 0, 1, 2, 3, 4
0..=5    -> 0, 1, 2, 3, 4, 5
```

---

## 39. Iterating Over an Array

```rust
let numbers = [10, 20, 30];

for number in numbers {
    println!("{}", number);
}
```

Output:

```text
10
20
30
```

Rust's `for` loop works with values that implement the appropriate iteration behavior.

We will study iterators in more depth later.

---

## 40. `match`

`match` is Rust's powerful pattern-matching construct.

Basic example:

```rust
let number = 2;

match number {
    1 => println!("One"),
    2 => println!("Two"),
    3 => println!("Three"),
    _ => println!("Something else"),
}
```

The `_` pattern acts as a catch-all.

---

## 41. `match` Is an Expression

Like `if`, `match` produces a value.

```rust
let number = 2;

let result = match number {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "other",
};

println!("{}", result);
```

Output:

```text
two
```

---

## 42. `match` Must Be Exhaustive

Rust requires every possible value to be handled.

This is incomplete:

```rust
let number = 10;

match number {
    1 => println!("one"),
    2 => println!("two"),
}
```

The compiler rejects it because values other than `1` and `2` are not handled.

A catch-all pattern fixes it:

```rust
match number {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}
```

This requirement is one of Rust's major safety features.

---

## 43. Summary of Core Types

| Category         | Types                                      |
| ---------------- | ------------------------------------------ |
| Integer          | `i8`, `i16`, `i32`, `i64`, `i128`, `isize` |
| Unsigned integer | `u8`, `u16`, `u32`, `u64`, `u128`, `usize` |
| Floating point   | `f32`, `f64`                               |
| Boolean          | `bool`                                     |
| Character        | `char`                                     |
| Tuple            | `(T1, T2, ...)`                            |
| Array            | `[T; N]`                                   |
| String slice     | `&str`                                     |
| Owned string     | `String`                                   |
| Unit             | `()`                                       |

---

## 44. Summary of Immutability

### Immutable variable

```rust
let x = 10;
```

Cannot be reassigned.

### Mutable variable

```rust
let mut x = 10;
x = 20;
```

Can be reassigned.

### Shadowing

```rust
let x = 10;
let x = 20;
```

Creates a new binding.

### Shadowing can change type

```rust
let value = "42";
let value = value.parse::<i32>().unwrap();
```

### Constant

```rust
const MAX_RETRIES: u32 = 3;
```

Constants are compile-time constants and cannot be mutable.

---

## 45. Summary of Control Flow

| Construct  | Purpose                |
| ---------- | ---------------------- |
| `if`       | Conditional execution  |
| `else if`  | Additional conditions  |
| `else`     | Fallback branch        |
| `loop`     | Repeated execution     |
| `while`    | Conditional loop       |
| `for`      | Iteration              |
| `break`    | Exit loop              |
| `continue` | Skip current iteration |
| `match`    | Pattern matching       |

A key Rust principle:

> Many control-flow constructs are expressions and therefore produce values.

For example:

```rust
let result = if condition {
    10
} else {
    20
};
```

and:

```rust
let result = match value {
    1 => "one",
    _ => "other",
};
```

---

## 46. Mental Model Before Ownership

At this point, you should be comfortable with these distinctions:

```text
Type
 ├── Scalar
 │    ├── Integer
 │    ├── Float
 │    ├── Boolean
 │    └── Character
 │
 └── Compound
      ├── Tuple
      └── Array
```

And:

```text
Variable
 ├── immutable: let
 └── mutable:   let mut
```

And:

```text
Binding
 ├── mutation
 │    └── let mut x
 │
 └── shadowing
      └── let x = ...
```

And:

```text
Control Flow
 ├── if / else
 ├── loop
 ├── while
 ├── for
 └── match
```

The next major step is **ownership**.

That is where concepts such as:

* `String`
* `&str`
* mutable variables
* scopes
* function calls
* moves
* `Copy`
* borrowing
* lifetimes

start coming together into one coherent model.

---
