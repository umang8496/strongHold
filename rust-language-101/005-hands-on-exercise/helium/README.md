<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# HELIUM

This project helps get the developer familiar with the hands-on rust coding.  

- [Exercise 001 (Variables + Functions)](#exercise-001)
- [Exercise 002 (String vs &str)](#exercise-002)
- [Exercise 003 (String vs &str)](#exercise-003)
- [Exercise 004 (Vec)](#exercise-004)
- [Exercise 005 (Vec + &[i32])](#exercise-005)

---

## Exercise 001

Write a Rust program that:

- Creates two integer variables `a = 10` and `b = 20`
- Creates a function `add(a, b)` which returns their sum
- Creates another function `multiply(a, b)` which returns their product
- In `main()`, call both functions and print:

    ```text
        Sum: 30
        Product: 200
    ```

- Constraints:
  - Don't use `mut`
  - Don't use external crates
  - Don't use unnecessary abstractions

### Response

```rust
fn add(num_a: i32, num_b: i32) -> i32 {
    num_a + num_b
}

fn multiply(num_a: i32, num_b: i32) -> i32 {
    num_a * num_b
}

fn main() {
    let a: i32 = 10;
    let b: i32 = 20;
    println!("Sum: {}", add(a, b));
    println!("Product: {}", multiply(a, b));
}
```

---

## Exercise 002

Write a Rust program which:

- Creates `name = "Umang"`
- Creates a function `fn greet(name: ???) -> ???`
- It should produce `Hello, Umang!`
- Constraints:  
  - name should be passed to `greet()`
  - `greet()` should return the greeting as a `String`
  - Don't use `mut`
  - Don't use `format!` yet
  - Don't use any external crate

### Response

```rust
fn greet(name: &str) -> String {
    String::from("Hello, ") + name + "!"
}

fn greet_another_version(name: &str) -> String {
    // Explicit mutation
    let mut result = String::from("Hello, ");
    result.push_str(name);
    result.push('!');
    result
}

fn main() {
    let name: String = String::from("Umang");
    let value: String = greet(&name);  // automatic conversion from &String to &str
    println!("{}", value);
    let another_value: String = greet_another_version(&name);
    println!("{}", another_value);
}
```

### Learning

- `String` is an owned, growable, heap-allocated UTF-8 string.
- `&str` is a borrowed string slice; it does not own the underlying string data.
- If a function only needs to read a string, prefer `fn greet(name: &str) -> String` rather than taking ownership with `String`.
- `String + &str` is supported; the `+` operation consumes the left-hand `String` and borrows the `&str`.
- Common ways to construct a `String`:

    ```rust
        String::from("Hello")
        format!("Hello, {}!", name)
        result.push_str(name)
    ```

---

## Exercise 003

Write a Rust program that returns the number of characters in a string:

- Creates a function `fn length(text: ???) -> usize`
- Creates another function `multiply(a, b)` which returns their product
- In `main()`, have the following code:

    ```rust
        let text = String::from("Hello Rust");
        println!("{}", length(???));
    ```

- Decide yourself whether `length()` should take `String`, `&String`, or `&str`

### Response

```rust
fn length(text: &str) -> i32 {
    text.len() as i32
}

fn main() {
    let text = String::from("Hello Rust");
    println!("{}", length(&text));
}
```

Here is the function signature:

```rust
pub const fn len(&self) -> usize {...}
```

---

## Exercise 004

Write a function:

- `fn sum(numbers: ???) -> ???` that takes a collection of integers and returns their sum
- Given: `let numbers = vec![10, 20, 30, 40, 50];`
- The program should print: `Sum: 150`
- Constraints:
  - Don't consume the vector unnecessarily
  - Don't use `iter().sum()` yet
  - Implement the summation yourself using a loop
  - Decide the appropriate types for the function

### Response

```rust
fn sum(numbers: Vec<i32>) -> i32 {
    let mut sum: i32 = 0;
    for number in numbers {
        sum += number;
    }
    sum
}

fn sum_better_impl(numbers: &[i32]) -> i32 {
    let mut sum: i32 = 0;
    for number in numbers {
        sum += number;
    }
    sum
}


fn main() {
    let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];
    let better_result: i32 = sum_better_impl(&numbers);
    println!("Better Sum: {}", better_result);
    let result: i32 = sum(numbers);
    println!("Sum: {}", result);
}
```

### Learning

- `Vec<T>` is a growable, dynamically sized collection.
- `[T; N]` is a fixed-size array, and its size is part of its type.
- Iterating directly over an owned `Vec`, `for number in numbers` consumes/moves the vector.
- `Vec<T>` is typically used when the collection size can change at runtime.
- Arrays are useful when the size is known and fixed at compile time.

---

## Exercise 005

Write sum() again, but this time:

- `fn sum(numbers: &[i32]) -> i32`
- Requirements:
  - It must not consume the collection
  - Use a `for` loop
  - No `.iter().sum()`
  - Call it using a `Vec<i32>`
  - After calling `sum()`, print the original vector as well
- Expected output:

    ```text
        Sum: 150
        Numbers: [10, 20, 30, 40, 50]
    ```

### Response

```rust
fn sum(numbers: &[i32]) -> i32 {
    let mut sum: i32 = 0;
    for &number in numbers {
        sum += number;
    }
    sum
}

fn main() {
    let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];
    let result: i32 = sum(&numbers);
    println!("Sum: {}", result);
    println!("Numbers: {:?}", numbers);
}
```

### Learning

- `&[T]` is a borrowed view into a sequence of `T`.
- Both a `Vec<T>` and an array can be borrowed as a slice:

    ```rust
        Vec<i32>  → &[i32]
        [i32; 5]  → &[i32]
    ```

- Taking `&[T]` allows a function to work with multiple sequence types without taking ownership:

    ```rust
        fn sum(numbers: &[i32]) -> i32
    ```

- Iterating over a slice normally produces references:

    ```rust
        for number in numbers {
            // number: &i32
        }
    ```

- We can explicitly dereference `*number`.
- Or destructure the reference while binding:

    ```rust
        for &number in numbers {
            // number: i32
        }
    ```

- Rust can perform automatic dereferencing in certain operations, which is why `sum += number;` can work even when number is `&i32`.
- `usize` is the conventional Rust type for sizes, lengths, and indexes; `str::len()` returns `usize`.

---
