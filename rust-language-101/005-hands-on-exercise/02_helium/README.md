<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD012 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# HELIUM

This project helps get the developer familiar with the hands-on rust coding.  

## Table of Content

- [Exercise 001 (Variables + Functions)](#exercise-001)
- [Exercise 002 (String vs &str)](#exercise-002)
- [Exercise 003 (String vs &str)](#exercise-003)
- [Exercise 004 (Vec)](#exercise-004)
- [Exercise 005 (Vec + &[i32])](#exercise-005)
- [Exercise 006 (Vec + Filtering)](#exercise-006)
- [Exercise 007 (Don't consume the input)](#exercise-007)
- [Exercise 008 (Find + Option)](#exercise-008)
- [Exercise 009 (`Option<T>`)](#exercise-009)
- [Exercise 010 (`Option` + `match`)](#exercise-010)
- [Exercise 011 (Transform an `Option`)](#exercise-011)
- [Exercise 012 (`Option<String>` + Ownership)](#exercise-012)
- [Exercise 013 (Structs + Methods)](#exercise-013)
- [Exercise 014 (Enums + Pattern Matching)](#exercise-014)
- [Exercise 015 (Traits Impl)](#exercise-015)
- [Exercise 016 (Trait Bounds + Generics)](#exercise-016)
- [Exercise 017 (Generic function with two trait bounds)](#exercise-017)
- [Exercise 018 (Generic function impl Trait)](#exercise-018)
- [Exercise 019 (Generic Type + Multiple Parameters)](#exercise-019)
- [Exercise 020 (Trait Objects / Dynamic Dispatch)](#exercise-020)
- [Exercise 021 (Closures Intro)](#exercise-021)
- [Exercise 022 (Closure Capture & `Fn` vs `FnMut` vs `FnOnce`)](#exercise-022)
- [Exercise 023 (Closure Capture + Ownership)](#exercise-023)
- [Exercise 024 (Closures + Iterators)](#exercise-024)
- [Exercise 025 (`filter()` + `map()`)](#exercise-025)
- [Exercise 026 (`fold()`)](#exercise-026)
- [Exercise 027 (`find()`, `any()`, and `position()`)](#exercise-027)
- [Exercise 028 (`any()` and `all()`)](#exercise-028)
- [Exercise 029 (`filter()` + `map()` + `collect()` with `String`)](#exercise-029)
- [Exercise 030 (`enumerate()` + `filter()`)](#exercise-030)
- [Exercise 031 (`flat_map()`)](#exercise-031)
- [Exercise 032 (`filter_map()`)](#exercise-032)
- [Exercise 033 (`zip()`)](#exercise-033)
- [Exercise 034 (`partition()`)](#exercise-034)
- [Exercise 035 (`fold()` with a custom result)](#exercise-035)

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

[Go to the Top](#table-of-content)

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

[Go to the Top](#table-of-content)

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

[Go to the Top](#table-of-content)

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

[Go to the Top](#table-of-content)

---

## Exercise 005

Write `sum()` again, but this time:

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

[Go to the Top](#table-of-content)

---

## Exercise 006

- Given: `let numbers = vec![10, 15, 20, 25, 30, 35, 40];`
- Write: `fn even_numbers(numbers: &[i32]) -> Vec<i32>`
- It should return a new `Vec<i32>` containing only the even numbers.
- Expected output: `[10, 20, 30, 40]`
- Constraints:
  - Borrow the input; don't consume it.
  - Return a new `Vec<i32>`.
  - Use a `for` loop.
  - Don't use `.filter()`, `.map()`, `.collect()`, etc. yet.

### Response

```rust
fn even_numbers(numbers: &[i32]) -> Vec<i32> {
    let mut array_of_even_number: Vec<i32> = Vec::new();
    for &number in numbers {
        if number % 2 == 0 {
            array_of_even_number.push(number);
        }
    }

    array_of_even_number
}

fn even_numbers_another_impl(numbers: &[i32]) -> Vec<i32> {
    let mut array_of_even_number: Vec<i32> = Vec::new();
    for number in numbers {
        if number % 2 == 0 {
            array_of_even_number.push(*number);
        }
    }

    array_of_even_number
}

fn main() {
    let numbers = vec![10, 15, 20, 25, 30, 35, 40];
    let array_of_even_number: Vec<i32> = even_numbers(&numbers);
    println!("{:?}", array_of_even_number);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 007

Let's make ownership slightly more explicit:

- Write: `fn double_numbers(numbers: &[i32]) -> Vec<i32>`
- Given: `let numbers = vec![1, 2, 3, 4, 5];`
- Expectation: `[2, 4, 6, 8, 10]`

- Constraints:
  - Input must be `&[i32]`
  - Output must be `Vec<i32>`
  - Use a `for` loop
  - Don't use `.map()`, `.collect()`, etc.
  - After calling `double_numbers()`, print the original numbers as well.

### Response

```rust
fn double_numbers(numbers: &[i32]) -> Vec<i32> {
    let mut doubled_numbers: Vec<i32> = Vec::new();
    for &number in numbers {
        doubled_numbers.push(2 * number);
    }

    doubled_numbers
}

fn main() {
    let numbers: Vec<i32> = vec![1, 2, 3, 4, 5];
    let doubled_numbers: Vec<i32> = double_numbers(&numbers);
    println!("{:?}", numbers);
    println!("{:?}", doubled_numbers);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 008

Now let's introduce `Option`, which is one of the most important Rust types:

- Implement: `fn find_number(numbers: &[i32], target: i32) -> ???`
- Given: `let numbers = vec![10, 20, 30, 40, 50];`
- Calling `find_number(&numbers, 30);` should produce `Found: 30`.
- While `find_number(&numbers, 99);` should produce: `Number not found`.
- Expectation: `[2, 4, 6, 8, 10]`
- Constraints:
  - Use a `for` loop
  - Don't use `.iter().find()`.
  - Don't use `unwrap()`.
  - You need to decide what the return type should be.

### Response

```rust
fn find_number(numbers: &[i32], target: i32) -> Option<i32> {
    for &number in numbers {
        if number == target {
            return Option::Some(number);
        }
    }

    Option::None
}

fn match_the_result(result: &Option<i32>) {
    match result {
        Some(number) => println!("Found: {}", number),
        None => println!("Number not found")
    }
}

fn main() {
    let numbers = vec![10, 20, 30, 40, 50];
    let result: Option<i32> = find_number(&numbers, 30);
    match_the_result(&result);
    let another_result: Option<i32> = find_number(&numbers, 99);
    match_the_result(&another_result);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 009

- Implement: `fn first_even(numbers: &[i32]) -> Option<i32>`
- Given: `[11, 13, 17, 20, 25, 30]` return `Some(20)`
- If the input is `[11, 13, 17, 25]` then return `None`
- Constraints:
  - Use a `for` loop
  - Don't use `.iter().find()`.
  - Don't use `unwrap()`.
  - You need to decide what the return type should be.

### Response

```rust
fn first_even(numbers: &[i32]) -> Option<i32> {
    for &number in numbers {
        if number % 2 == 0 {
            return Option::Some(number)
        }
    }

    Option::None
}

fn main() {
    let input_a: [i32; 6] = [11, 13, 17, 20, 25, 30];
    println!("{:?}", first_even(&input_a));
    let input_b: [i32; 4] = [11, 13, 17, 25];
    println!("{:?}", first_even(&input_b));
}
```

[Go to the Top](#table-of-content)

---

## Exercise 010

Let's now make you consume the Option through pattern matching.

- Implement: `fn first_even(numbers: &[i32]) -> Option<i32>` as before.
- But this time `main()` should produce:

    ```text
        First even number: 20
        No even number found
    ```

    for:

    ```text
        [11, 13, 17, 20, 25, 30]
        [11, 13, 17, 25]
    ```

- Constraints:
  - Use a `for` loop
  - Don't use `.iter().find()`.
  - Don't use `unwrap()`.
  - Don't use `if let` yet.

### Response

```rust
fn first_even(numbers: &[i32]) -> Option<i32> {
    for &number in numbers {
        if number % 2 == 0 {
            return Option::Some(number)
        }
    }

    Option::None
}

fn match_the_result(result: &Option<i32>) {
    match result {
        Some(number) => println!("First even number: {}", number),
        None => println!("No even not found")
    }
}

fn main() {
    let input_a: [i32; 6] = [11, 13, 17, 20, 25, 30];
    match_the_result(&first_even(&input_a));
    let input_b: [i32; 4] = [11, 13, 17, 25];
    match_the_result(&first_even(&input_b));
}
```

[Go to the Top](#table-of-content)

---

## Exercise 011

- Implement: `fn double_if_present(value: Option<i32>) -> Option<i32>`
- Behaviour:

    ```text
        Some(10) → Some(20)
        Some(25) → Some(50)
        None     → None
    ```

- Constraints:
  - Use `match`.
  - Use a `for` loop.
  - Don't use `.iter().find()`.
  - Don't use `unwrap()`.
  - Don't use `if let` yet.
  - Use `match`.
  - Don't use `map()` yet.

### Response

```rust
fn double_if_present(value: Option<i32>) -> Option<i32> {
    if value.is_some() {
        return match value {
            Some(number) => Option::Some(2 * number),
            None => None
        }
    } else {
        return Option::None;
    }
}

fn main() {
    let option_10: Option<i32> = Option::Some(10);
    println!("{:?}", double_if_present(option_10));
    let option_25: Option<i32> = Option::Some(25);
    println!("{:?}", double_if_present(option_25));
}
```

[Go to the Top](#table-of-content)

---

## Exercise 012

- Implement: `fn find_name(names: &[String], target: &str) -> Option<String>`
- Given:

    ```text
        let names = vec![
            String::from("Alice"),
            String::from("Bob"),
            String::from("Charlie"),
            String::from("David"),
        ];
    ```

- Calling `find_name(&names, "Charlie")` should return `Some("Charlie")`.
- Calling `find_name(&names, "Eve")` should return `None`.
- Constraints:
  - Use a `for` loop
  - Don't use `.iter().find()`.
  - Don't use `unwrap()`.
  - Don't use `if let` yet.

### Response

```rust
fn find_name(names: &[String], target: &str) -> Option<String> {
    for name in names {
        if name == target {
            return Option::Some(name.clone());
        }
    }
    Option::None
}

fn main() {
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
        String::from("David"),
    ];

    println!("{:?}", &find_name(&names, "Charlie"));
    println!("{:?}", &find_name(&names, "Eve"));
}
```

### Learning

- `&[String]` means the function borrows a slice of `String` values; it does not own them.
- Iterating over a borrowed slice `for name in names` gives `name` as `&String`.
- With `i32`, we could write: `for &number in numbers` because `i32` implements `Copy`; the underlying value can be copied out of `&i32`.
- `String` does not implement `Copy`, so we cannot use `for &name in names` to move `String` values out of a borrowed slice.
- `clone()` creates an owned copy: `name.clone() // &String → String`.

This exercise reinforced a fundamental Rust principle:  
> Borrowing data doesn't give you ownership of it;  
> If an owned value must escape the borrow, you need to create/obtain ownership explicitly.

[Go to the Top](#table-of-content)

---

## Exercise 013

Let's move into a genuinely new area: structs and impl blocks.

- Create a User struct with: `id, name, age`
- Then implement a method: `fn is_adult(&self) -> bool` that returns true when the user's age is 18 or greater.
- Given:

    ```rust
        let user = User {
            id: 101,
            name: String::from("Umang"),
            age: 30,
        };
    ```

- The program should print something equivalent to:

    ```text
        User: Umang
        Adult: true
    ```

- Constraints:
  - Define `User` using `struct`.
  - Implement `is_adult()` inside an `impl User` block.
  - The method must borrow the user; don't consume it.
  - Print the user's name and result from `main()`.
  - Don't derive any traits yet.
  - Don't use `mut`.

### Response

```rust
struct User {
    id: i32,
    name: String,
    age: i32,
}

impl User {
    fn new(id: i32, name: String, age: i32) -> Self {
        User { id, name, age }
    }

    fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

fn main() {
    let user = User::new(1, String::from("Umang"), 20);
    println!("User: {}", user.name);
    println!("Adult: {}", user.is_adult());
}
```

### Learning (Methods vs Associated Functions)

- Both are defined inside an `impl` block.
- Method → has a `self` parameter and operates on an instance.
- `fn is_adult(&self) -> bool` Called as: `user.is_adult()`
- `self` can be:
  - `&self` → immutable borrow
  - `&mut self` → mutable borrow
  - `self` → takes ownership

- Associated function → has no `self` parameter and belongs to the type itself.
- `fn new(...) -> Self` Called as: `User::new(...)`

- Rust has no special constructor syntax; `new()` is conventionally implemented as an associated function.

[Go to the Top](#table-of-content)

---

## Exercise 014

Let's move to another fundamental Rust feature: `enums`.  

- Define:

    ```rust
        enum Shape {
            Circle(f64),
            Rectangle(f64, f64),
        }
    ```

- Then implement a method: `fn area(shape: &Shape) -> f64`.
- For the following calculate and print the areas.

    ```rust
        let circle = Shape::Circle(5.0);
        let rectangle = Shape::Rectangle(10.0, 4.0);
    ```

- Constraints:
  - Use `match`.
  - `area()` must borrow the `Shape`; don't consume it.
  - Don't use `if let`.
  - Don't derive any traits.
  - Use `std::f64::consts::PI` for the circle calculation.

### Response

```rust
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}


fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(radius) => std::f64::consts::PI * radius * radius,
        Shape::Rectangle(width, height) => width * height,
    }
}

fn main() {
    let circle = Shape::Circle(10.0);
    let rectangle = Shape::Rectangle(10.0, 5.0);

    println!("Circle area: {}", area(&circle));
    println!("Rectangle area: {}", area(&rectangle));
}
```

[Go to the Top](#table-of-content)

---

## Exercise 015

- Create two structs:

    ```rust
        struct Circle {
            radius: f64,
        }

        struct Rectangle {
            width: f64,
            height: f64,
        }
    ```

- Define the trait:

    ```rust
        trait Area {
            fn area(&self) -> f64;
        }
    ```

- Then implement the trait for both structs.
- Expected behaviour.

    ```rust
        let circle = Circle { radius: 5.0 };
        let rectangle = Rectangle {
            width: 10.0,
            height: 4.0,
        };

        println!("Circle: {}", circle.area());
        println!("Rectangle: {}", rectangle.area());
    ```

- Constraints:
  - Use `match`.
  - `area()` must borrow the `Shape`; don't consume it.
  - Don't use `if let`.
  - Don't derive any traits.
  - Don't use `enum` for this exercise.
  - Use `std::f64::consts::PI` for the circle calculation.

### Response

```rust
struct Circle {
    radius: f64,
}

struct Rectangle {
    length: f64,
    width: f64,
}

trait Area {
    fn area(&self) -> f64;
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.width
    }
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        width: 5.0,
    };

    println!("Circle: {}", circle.area());
    println!("Rectangle: {}", rectangle.area());
}
```

[Go to the Top](#table-of-content)

---

## Exercise 016

Now let's make the Area trait useful across different types.

- Reuse the trait:

    ```rust
        trait Area {
            fn area(&self) -> f64;
        }
    ```

- Create a genric function: `fn print_area<T: Area>(shape: &T)`.
- It should print: `Area: <calculated area>`
- Given:

    ```rust
        let circle = Circle { radius: 5.0 };
        let rectangle = Rectangle {
            length: 10.0,
            width: 4.0,
        };
    ```

    both should be accepted by the same print_area() function.

- Constraints:
  - Use `match`.
  - Use a trait bound: `T: Area`
  - `print_area()` must borrow the shape
  - Don't use `dyn Area` yet.
  - Don't duplicate `print_area()` for each concrete type.

### Response

```rust
trait Area {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

fn print_area<T: Area>(shape: &T) {
    let calculated_area: f64 = shape.area();
    println!("Area: {}", calculated_area);
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };
    print_area(&circle);
    print_area(&rectangle);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 017

- Create a function `fn describe<T: Area + ???>(shape: &T)`
- It requires to print both:

    ```text
        Area: 314.159...
        Shape: Circle { radius: 10.0 }
    ```

- The `Circle` and `Rectangle` should remain as they are.
- Make `describe()` work for both.
- Constraints:
  - Use a generic `T`.
  - `T` must implement `Area`.
  - `T` must also satisfy whatever is necessary for `{:?}`.
  - Don't use `dyn`.
  - Don't manually implement formatting yet.

### Response

```rust
trait Area {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl std::fmt::Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circle {{ radius: {} }}", self.radius)
    }
}

struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

impl std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rectangle {{ length: {}, breadth: {} }}", self.length, self.breadth)
    }
}

fn describe<T: Area + std::fmt::Display>(shape: &T) {
    println!("Area: {}", &shape.area());
    println!("Shape: {}", &shape);
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };
    describe(&circle);
    describe(&rectangle);
}
```

Another implementation which uses `std::fmt::Debug` trait instead of `std::fmt::Display` one.

```rust
trait Area {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

#[derive(Debug)]
struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

fn describe<T: Area + std::fmt::Debug>(shape: &T) {
    println!("Area: {}", &shape.area());
    println!("Shape: {:?}", &shape);
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };
    describe(&circle);
    describe(&rectangle);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 018

- Write the same describe functionality, but instead of: `fn describe<T: Area + Debug>(shape: &T)` use `fn describe(shape: &impl Area + ???)`.
- Make it work for both `Circle` and `Rectangle`.
- Constraints:
  - Use `match`.
  - Use a trait bound: `T: Area`
  - `print_area()` must borrow the shape
  - Don't use `dyn Area` yet.
  - Don't duplicate `print_area()` for each concrete type.

### Response

```rust
trait Area {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

#[derive(Debug)]
struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

fn describe(shape: &(impl Area + std::fmt::Debug)) {
    println!("Area: {}", &shape.area());
    println!("Shape: {:?}", &shape);
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };
    describe(&circle);
    describe(&rectangle);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 019

- Create: `fn same_area<T: Area>(a: &T, b: &T) -> bool`
- It should return `true` if both shapes have the same area.
- For example:

    ```rust
        let circle1 = Circle { radius: 5.0 };
        let circle2 = Circle { radius: 5.0 };
        same_area(&circle1, &circle2); // true
    ```

- It should print: `Area: <calculated area>`
- Also test:

    ```rust
        let rectangle = Rectangle {
            length: 10.0,
            breadth: 4.0,
        };
    ```

    both should be accepted by the same print_area() function.

- Constraints:
  - Use <T: Area>.
  - Both parameters must use `T`.
  - Don't use `impl Trait`.
  - Don't use `dyn`.
  - Return only `true` or `false`.

### Response

```rust
trait Area {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

#[derive(Debug)]
struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

// This one works as well
// fn same_area<T: Area, V: Area>(a: &T, b: &V) -> bool {
//     a.area() == b.area()
// }

fn same_area(a: &impl Area, b: &impl Area) -> bool {
    a.area() == b.area()
}

fn main() {
    let circle1 = Circle { radius: 10.0 };
    let circle2 = Circle { radius: 10.0 };
    println!("{}", same_area(&circle1, &circle2));    // true

    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };

    println!("{}", same_area(&rectangle, &circle2));    // false
}
```

### Learning

- **Trait bound** `T: Area` means `T` must implement the `Area` trait.

- **Single generic type** `fn f<T: Area>(a: &T, b: &T)` means `a` and `b` **must be the same concrete type**.

- **Multiple generic types** `fn f<T: Area, V: Area>(a: &T, b: &V)` allows `a` and `b` to be **different concrete types**, as long as both implement `Area`.

- **`impl Trait`** `fn f(a: &impl Area, b: &impl Area)` allows each parameter to independently be a type implementing `Area`.

- **Generics vs `impl Trait`**:
  - Named generics (`T`) let us **relate types across parameters**.
  - `impl Trait` is useful when we don't need to name or relate the concrete type.

- **Static dispatch**: Generic functions such as: `fn f<T: Area>(shape: &T)` are resolved at compile time through **monomorphization**.

- **Dynamic dispatch: `dyn Trait`**:  
  - `&dyn Area` introduces **trait objects and dynamic dispatch**,
  - allowing different concrete types such as `Circle` and `Rectangle` to be handled through the same interface at runtime.

[Go to the Top](#table-of-content)

---

## Exercise 020

Now let's move away from another generic variation and introduce something fundamentally new.

- Given:

    ```rust
        trait Area {
            fn area(&self) -> f64;
        }

        struct Circle {
            radius: f64,
        }

        struct Rectangle {
            length: f64,
            breadth: f64,
        }
    ```

- Implement: `fn print_areas(shapes: &[&dyn Area]) {...}`
- Then:

    ```rust
        let circle = Circle { radius: 10.0 };
        let rectangle = Rectangle {
            length: 10.0,
            breadth: 5.0,
        };

        let shapes: Vec<&dyn Area> = vec![
            &circle,
            &rectangle,
        ];

        print_areas(&shapes);
    ```

- Constraints:
  - Don't use generics.
  - Don't use `impl Trait`.
  - Don't use `Any`.
  - Don't use downcasting.

The goal is to understand `dyn Trait` and **dynamic dispatch**.  
This is our first step from compile-time polymorphism → runtime polymorphism.

### Response

```rust
trait Area {
    fn area(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

#[derive(Debug)]
struct Rectangle {
    length: f64,
    breadth: f64,
}

impl Area for Rectangle {
    fn area(&self) -> f64 {
        self.length * self.breadth
    }
}

fn print_areas(shapes: &Vec<&dyn Area>) {
    for shape in shapes {
        println!("Area: {}", shape.area());
    } 
}

fn main() {
    let circle = Circle { radius: 10.0 };
    let rectangle = Rectangle {
        length: 10.0,
        breadth: 5.0,
    };

    let shapes: Vec<&dyn Area> = vec![
        &circle,
        &rectangle,
    ];

    print_areas(&shapes);
}
```

One idiomatic improvement: Although your code is valid, Rust conventionally prefers accepting a slice:

```rust
fn print_areas(shapes: &[&dyn Area]) {
    for shape in shapes {
        println!("Area: {}", shape.area());
    } 
}
```

This makes the function usable with both a `Vec` and an `array/slice`, rather than requiring specifically a `Vec`.

### Learning

- `dyn Area`: a trait object representing some type that implements Area
- `&`dyn Area: a reference to that trait object
- `Vec<&dyn Area>`: a vector containing references to potentially different Area implementations
- `&Vec<&dyn Area>`: borrow that vector

- With generics: `fn print_areas<T: Area>(shapes: &[T])`, `T` represents one concrete type.
- With: `fn print_areas(shapes: &Vec<&dyn Area>)` each element can refer to a different concrete type, as long as it implements Area.

[Go to the Top](#table-of-content)

---

## Exercise 021

- Given:

    ```rust
        fn apply_operation(a: i32, b: i32, operation: ???) -> i32 {
            operation(a, b)
        }
    ```

- Complete the functioon so that these work:

    ```rust
        let add = |a, b| a + b;
        let multiply = |a, b| a * b;

        println!("{}", apply_operation(10, 20, add));
        println!("{}", apply_operation(10, 20, multiply));
    ```

- Constraints:
  - Use a closure as the `operation` parameter.
  - Don't use `traits` explicitly.
  - Don't use `dyn`.
  - Don't use **generics** yet.

### Response

```rust
fn apply_operation<F>(a: i32, b: i32, operation: F) -> i32 
where 
    F: Fn(i32, i32) -> i32,
{
    operation(a, b)
}

fn main() {
    let add = |a: i32, b: i32| a + b;
    let multiply = |a: i32, b: i32| a * b;

    println!("{}", apply_operation(10, 20, add));
    println!("{}", apply_operation(10, 20, multiply));
}
```

[Go to the Top](#table-of-content)

---

## Exercise 022

Now let's make the callable traits concrete.

- Consider:

    ```rust
        fn execute<F>(operation: F)
        where
            F: ???,
        {
            operation();
        }
    ```

- Your task is to determine the correct trait bound for each of the following case.
- Case 1:

    ```rust
        let message = String::from("Hello");

        let print_message = || {
            println!("{}", message);
        };
    ```

- Case 2:

    ```rust
        let mut count = 0;

        let increment = || {
            count += 1;
        };
    ```

- Case 3:

    ```rust
        let message = String::from("Hello");

        let consume_message = || {
            drop(message);
        };
    ```

- For each closure, determine whether it implements: `Fn`, `FnMut` and `FnOnce`.
- Then write three functions:

    ```rust
        fn execute_fn<F>(operation: F)
        where
            F: ???
        {
            operation();
        }
    ```

    ```rust
        fn execute_fn_mut<F>(operation: F)
        where
            F: ???
        {
            operation();
        }
    ```

    ```rust
        fn execute_fn_once<F>(operation: F)
        where
            F: ???
        {
            operation();
        }
    ```

- Constraints:
  - Don't use `Box`.
  - Don't use `traits` explicitly.
  - Don't use `dyn`.
  - Use **generics** yet.

### Response

```rust
fn execute_fn<F>(operation: F)
where
    F: Fn()
{
    operation();
}

fn execute_fn_mut<F>(mut operation: F)
where
    F: FnMut()
{
    operation();
}

fn execute_fn_once<F>(operation: F)
where
    F: FnOnce()
{
    operation();
}

fn main() {
    // case 01:
    let message: String = String::from("Hello");

    let print_message = || {
        println!("{}", message);
    };

    execute_fn(print_message);

    // case 02:
    let mut count = 0;

    let increment = || {
        count += 1;
        println!("{}", count);
    };

    execute_fn_mut(increment);

    // case 03:
    let message = String::from("Hello");

    let consume_message = || {
        println!("Dropping: {}", message);
        drop(message);
    };

    execute_fn_once(consume_message);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 023

- Consider:

    ```rust
        fn execute<F>(operation: F)
        where
            F: Fn(),
        {
            operation();
        }
    ```

- Now complete the following three cases.
- Case 1 - Borrow:

    ```rust
        let name = String::from("Rust");

        let print_name = || {
            println!("{}", name);
        };

        execute(print_name);

        println!("Still usable: {}", name);
    ```

    Question: Why is `name` still usable after calling `execute()`?

- Case 2 - Mutable Borrow:

    ```rust
        let mut count = 0;

        let increment = || {
            count += 1;
        };

        increment();
        increment();

        println!("Count: {}", count);
    ```

    Question: What trait does `increment` implement, and why?

- Case 3 - Move:

    ```rust
        let name = String::from("Rust");

        let consume_name = move || {
            println!("{}", name);
        };

        consume_name();

        println!("Name: {}", name);
    ```

    Question: Does this compile?

The important concept here is:

> A closure doesn't always own what it captures.  
> Rust determines the capture mode based on how the captured variable is used — unless move forces ownership into the closure.

### Response

```rust
fn execute<F>(operation: F)
where
    F: Fn(),
{
    operation();
}

fn main() {
    // case 01:
    let name = String::from("Rust (borrowed)");
    let print_name = || {
        println!("{}", name);
    };
    execute(print_name);
    // "name" is still usable as it was initially borrowed by "execute()"
    // "exceute()" implements "Fn()"
    println!("Still usable: {}", name);

    // case 02:
    let mut count = 0;
    let mut increment = || {
        count += 1;
    };
    // here "increment" implements the "FnMut" trait
    increment();
    increment();
    println!("Count: {}", count);

    // case 03:
    let consumable_name = String::from("Rust (moved)");
    let consume_name = move || {
        println!("{}", consumable_name);
    };
    consume_name();
    // the following line cannot be compiled because of the "move" keyboard
    // here "consume_name" still implements "Fn" trait
    // println!("Name: {}", consumable_name);
}
```

### Learning

| Closure behavior                  | Typical trait     |
| --------------------------------- | ----------------- |
| Captures by reference, only reads | `Fn`              |
| Mutably accesses captured state   | `FnMut`           |
| Consumes captured value           | `FnOnce`          |
| `move` + only reads owned value   | Can still be `Fn` |
| `move` + mutates owned value      | Can be `FnMut`    |
| `move` + consumes owned value     | `FnOnce`          |

[Go to the Top](#table-of-content)

---

## Exercise 024

Now we'll connect closures to **iterators**, which is where you'll start seeing `Fn`/`FnMut` constantly in real Rust code.  

We'll start with a simple iterator transformation rather than throwing several iterator methods at you at once.  

- Implement: `fn double_numbers(numbers: &[i32]) -> Vec<i32> {...}`
- Given:

    ```rust
        let numbers = vec![1, 2, 3, 4, 5];
        let result = double_numbers(&numbers);
        println!("{:?}", result);
    ```

- Constraints:
  - Use: `.iter()`, `.map()`, `.collect()`
  - Do not use: a `for` loop, or a manually created result `Vec` or `.for_each()`.

### Response

```rust
fn double_numbers(numbers: &[i32]) -> Vec<i32> {
    let doubled: Vec<i32> = numbers.iter().map(|number| { number * 2 }).collect();
    // another way of writing the above line
    // let doubled = numbers.iter().map(|number| number * 2).collect::<Vec<i32>>();
    doubled
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let result = double_numbers(&numbers);
    println!("Initial Vec: {:?}", numbers);
    println!("Final Vec: {:?}", result);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 025

Now let's combine two iterator transformations.

- Implement: `fn even_squares(numbers: &[i32]) -> Vec<i32>`
- For: `vec![1, 2, 3, 4, 5, 6]` the result should be: `[4, 16, 36]`.

### Response

```rust
fn even_squares(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .filter(|num| { *num % 2 == 0 })
        .map(|num| { num * num })
        .collect::<Vec<i32>>()
}

fn main() {
    let numbers: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
    let result: Vec<i32> = even_squares(&numbers);
    println!("Initial Vec: {:?}", numbers);
    println!("Final Vec: {:?}", result);
}
```

[Go to the Top](#table-of-content)

---

## Exercise 026

- Implement: `fn sum_of_squares(numbers: &[i32]) -> i32`
- For: `let numbers = vec![1, 2, 3, 4, 5];` the result should be: `55`.

### Response

```rust
fn sum_of_squares(numbers: &[i32]) -> i32 {
    numbers.iter().fold(0, |acc, item| { acc + item * item})
}

fn main() {
    let numbers: Vec<i32> = vec![1, 2, 3, 4, 5];
    let result: i32 = sum_of_squares(&numbers);
    println!("Result: {}", result);
}
```

### `fold()` in Rust

`fold()` is an iterator operation that reduces multiple elements into a single final value.  
Its conceptual signature is:

```rust
fn fold<B, F>(init: B, f: F) -> B
where
    F: FnMut(B, Self::Item) -> B
```

Don't worry about memorizing the exact generic signature yet. The important structure is: `fold(initial_value, closure)`.  
The closure receives: `accumulator + current_item` and must return the next accumulator.
This closure is effectively `FnMut` because the closure conceptually updates the accumulator across iterations.

[Go to the Top](#table-of-content)

---

## Exercise 027

We've learned how to transform and reduce iterators.  
Now let's look at operations that search an iterator.  

- Implement: `fn first_greater_than(numbers: &[i32], target: i32) -> Option<i32>`
- Given: `let numbers = vec![10, 20, 30, 40, 50];`.
- These should produce:

    ```text
        first_greater_than(&numbers, 25) → Some(30)
        first_greater_than(&numbers, 40) → Some(50)
        first_greater_than(&numbers, 100) → None
    ```

### Response

```rust
fn first_greater_than(numbers: &[i32], target: i32) -> Option<i32> {
    let result = numbers.iter().find(|&&num| { num > target }).copied();
    return result;
}

fn main() {
    let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];
    println!("{:?}", first_greater_than(&numbers, 25));    // Some(30)
    println!("{:?}", first_greater_than(&numbers, 40));    // Some(50)
    println!("{:?}", first_greater_than(&numbers, 100));   // None
}
```

### Learning

- The `Iterator::find` method takes a single argument: `a closure that returns a boolean (true or false)`.
  - It loops through the iterator elements one by one and applies your closure to each element.
  - The moment the closure returns `true`, `find` short-circuits (stops looping immediately) and returns that element inside `Some`.
  - If the loop finishes and no elements match the condition, it returns `None`.
- Because it might not find a match, the return type of find is always an `Option<T>`.

- `find()`
  - Searches an iterator for the **first element** satisfying a predicate.
  - Signature conceptually: `find<F>(&mut self, predicate: F) -> Option<Self::Item>`.
  - Predicate returns `bool`: `|item| -> bool`.
  - Returns: `Some(item)` → first matching element or `None` → no match`.
  - With `.iter()` over `&[i32]`:

    ```text
    iter() → Item = &i32
    find() → Option<&i32>
    ```

  - The predicate receives a reference to the iterator item, hence `&&i32` in our example.

- `copied()`
  - Converts references to their **copied values** when the underlying type implements `Copy`.
  - Commonly used to convert: `Option<&T> → Option<T>`
  - Example:

    ```rust
    Some(&30).copied()
    // Some(30)
    ```

  - `i32` implements `Copy`, so this works:

    ```rust
    Option<&i32>.copied() → Option<i32>
    ```

  - It **does not clone** the object.
  - For non-`Copy` types where you want a duplicate, use `.cloned()` instead.
  - Mental model:

    ```text
    find()   → finds a reference
    copied() → copies the referenced value
    ```

[Go to the Top](#table-of-content)

---

## Exercise 028

- Implement: `fn contains_negative(numbers: &[i32]) -> bool`.
- It should return true if at least one number is negative.
- And implement: `fn all_positive(numbers: &[i32]) -> bool`.
- It should return true only if every number is positive.

### Response

```rust
// It should return true if at least one number is negative
fn contains_negative(numbers: &[i32]) -> bool {
    numbers.iter().any(|&num| {num < 0})
}

// It should return true only if every number is positive
fn all_positive(numbers: &[i32]) -> bool {
    numbers.iter().all(|&num| {num > 0})
}

fn main() {
    let a: Vec<i32> = vec![1, 2, 3, 4];
    let b: Vec<i32> = vec![1, -2, 3, 4];

    println!("{}", contains_negative(&a));
    println!("{}", contains_negative(&b));

    println!("{}", all_positive(&a));
    println!("{}", all_positive(&b));
}
```

#### `any()` in Rust

```rust
fn any<F>(&mut self, f: F) -> bool
where
    Self: Sized,
    F: FnMut(Self::Item) -> bool,
```

- `any()` takes a closure that returns `true` or `false`.  
- It applies this closure to each element of the iterator, and if any of them return true, then so does any().  
- If they all return `false`, it returns `false`.

- `any()` is short-circuiting; in other words, it will stop processing as soon as it finds a `true`, given that no matter what else happens, the result will also be `true`.

- An empty iterator returns `false`.

#### `all()` in Rust

```rust
fn all<F>(&mut self, f: F) -> bool
where
    Self: Sized,
    F: FnMut(Self::Item) -> bool,
```

- `all()` takes a closure that returns `true` or `false`.  
- It applies this closure to each element of the iterator, and if all of them return `true`, then so does `all()`.  
- If any of them return `false`, it returns `false`.

- `all()` is short-circuiting; in other words, it will stop processing as soon as it finds a `false`, given that no matter what else happens, the result will also be `false`.

- An empty iterator returns `true`.

[Go to the Top](#table-of-content)

---

## Exercise 029

- Implement: `fn long_names(names: &[String]) -> Vec<String>`.
- Given:

    ```rust
        let names = vec![
            String::from("Raj"),
            String::from("Alexander"),
            String::from("John"),
            String::from("Christopher"),
        ];
    ```

- Return names whose length is greater than 4.

### Response

```rust
fn long_names(names: &[String]) -> Vec<String> {
    let result = names
        .iter()
        .filter(|&name| { name.len() > 4 })
        .map(|name| { name.to_string() })
        .collect();
    result
}

fn main() {
    let names = vec![
        String::from("Raj"),
        String::from("Alexander"),
        String::from("John"),
        String::from("Christopher"),
    ];

    println!("{:?}", long_names(&names));
}
```

### What happened in this code

- **Input:** `&[String]` — the function only borrows the original strings.
- **`.iter()`** — produces `&String` references; nothing is moved.
- **`.filter()`** — keeps only strings whose length is greater than `4`.
- **`.map()`** — converts each remaining `&String` into a new owned `String` using `to_string()`.
- **`.collect()`** — gathers those owned `String`s into a `Vec<String>`.
- **Original vector remains usable** because we never moved anything out of it.
- Overall pipeline:

    ```text
        &[String]
            ↓
        iter()
            ↓
        &String
            ↓
        filter()
            ↓
        &String
            ↓
        map()
            ↓
        String
            ↓
        collect()
            ↓
        Vec<String>
    ```

[Go to the Top](#table-of-content)

---

## Exercise 030

- Implement: `fn find_even_index(numbers: &[i32]) -> Option<usize>`.
- It should return the index of the first even number.

### Response

```rust
fn find_even_index(numbers: &[i32]) -> Option<usize> {
    numbers.iter().enumerate().find_map(|(index, &num)| {
        if num % 2 == 0 {
            Some(index)
        } else {
            None
        }
    })
}

fn main() {
    let numbers: Vec<i32> = vec![11, 7, 9, 14, 21];
    let first_even_number_index: Option<usize> = find_even_index(&numbers);
    match first_even_number_index {
        Some(index) => println!("Position is {}", index),
        None => println!("No even number found"),
    }
}
```

Alternatively, we can have the following implementations too.

```rust
fn find_even_index(numbers: &[i32]) -> Option<usize> {
    numbers
        .iter()
        .enumerate()
        .find(|&(_, &num)| num % 2 == 0)
        .map(|(index, _)| index)
}
```

Or

```rust
fn find_even_index(numbers: &[i32]) -> Option<usize> {
    numbers.iter().position(|&num| num % 2 == 0)
}
```

### Learning

- **`find()`** → returns the **first matching element** → `Option<Item>`
- **`find_map()`** → returns the **first successful transformed result** → `Option<T>`
- **`position()`** → returns the **index of the first match** → `Option<usize>`

- `enumerate()` takes an iterator and adds an index to each item.
- `enumerate() → Iterator<(usize, Item)>`
- For: `numbers.iter()` where `Item = &i32`: `iter() returns &i32` and `enumerate() returns a tuple (usize, &i32)`.
- And the index is always a `usize`.

[Go to the Top](#table-of-content)

---

## Exercise 031

- Implement: `fn flatten_numbers(numbers: &[Vec<i32>]) -> Vec<i32>`
- Given:

    ```rust
        let numbers = vec![
            vec![1, 2],
            vec![3, 4, 5],
            vec![6],
        ];
    ```

- Expected result: `[1, 2, 3, 4, 5, 6]`

### Response

```rust
fn flatten_numbers(numbers: &[Vec<i32>]) -> Vec<i32> {
    numbers.iter().flat_map(|v| v.iter().cloned()).collect()
}

fn main() {
    let numbers: Vec<Vec<i32>> = vec![
        vec![1, 2],
        vec![3, 4, 5],
        vec![6],
    ];
    println!("Flatten Number: {:?}", flatten_numbers(&numbers));
}
```

### Learning

- `numbers.iter()` → iterates over each inner `Vec<i32>` as `&Vec<i32>`.
- `flat_map(|v| ...)` → for each inner vector, produces an iterator and flattens all those iterators into one.
- `|v| v.iter()` → iterates over the elements of that inner vector as `&i32`.
- `.cloned()` → converts `&i32` → `i32`.
- `.collect()` → gathers all resulting `i32` values into `Vec<i32>`.

    ```text
    Vec<Vec<i32>>
        ↓ iter()
    &Vec<i32>
        ↓ flat_map()
    multiple inner iterators
        ↓ cloned()
       i32
        ↓ collect()
      Vec<i32>
    ```

[Go to the Top](#table-of-content)

---

## Exercise 032

- Implement: `fn parse_positive_numbers(values: &[&str]) -> Vec<i32>`
- Given:

    ```rust
        let values = vec![
            "10",
            "-5",
            "hello",
            "20",
            "world",
            "30",
        ];
    ```

- Expected result: `[10, 20, 30]`

### Response

```rust
fn parse_positive_numbers(values: &[&str]) -> Vec<i32> {
    values
        .iter()
        .filter(|&val| val.parse::<i32>().map_or(false, |n| n > 0))
        .map(|&val| val.parse::<i32>().unwrap())
        .collect::<Vec<i32>>()
}

fn main() {
    let values = vec![
        "10",
        "-5",
        "hello",
        "20",
        "world",
        "30",
    ];

    println!("{:?}", parse_positive_numbers(&values));
}
```

There is another implementation as well.

```rust
fn parse_positive_numbers(values: &[&str]) -> Vec<i32> {
    values
        .iter()
        .filter_map(|value| {
            match value.parse::<i32>() {
                Ok(num) if num > 0 => Some(num),
                _ => None,
            }
        })
        .collect()
}
```

### `filter_map()`

- **Combines `filter()` + `map()`** into one operation.
- Takes each element and returns an **`Option<T>`**.
- `Some(value)` → value is **kept**.
- `None` → value is **discarded**.
- Useful when the transformation itself can fail or some elements shouldn't produce an output.
- Unlike `filter()`, it can **change the element's type**.

```rust
values.iter().filter_map(|value| {
    match value.parse::<i32>() {
        Ok(n) if n > 0 => Some(n),
        _ => None,
    }
})
```

[Go to the Top](#table-of-content)

---

## Exercise 033

- Implement:

    ```rust
        fn pair_names_with_scores(
            names: &[String],
            scores: &[i32],
        ) -> Vec<(String, i32)>
    ```

- Given:

    ```rust
        let names = vec![
            String::from("Alice"),
            String::from("Bob"),
            String::from("Charlie"),
        ];
        let scores = vec![85, 92, 78];
    ```

- Expected output:

    ```text
        [
            ("Alice", 85),
            ("Bob", 92),
            ("Charlie", 78)
        ]
    ```

### Response

```rust
fn pair_names_with_scores(names: &[String], scores: &[i32]) -> Vec<(String, i32)> {
    let result = names
                    .iter().cloned()
                    .zip(scores.iter().cloned())
                    .collect();
    result
}

fn main() {
    let names: Vec<String> = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];
    let scores = vec![85, 92, 78];
    
    let paired = pair_names_with_scores(&names, &scores);
    println!("{:?}", paired);
}
```

#### `cloned()`

- The `cloned()` method creates an **owned copy** of each element in an iterator.
- Useful when you need to transform references into owned values, like converting `&String` to `String`.
- In the example, `names.iter().cloned()` converts `&String` to `String`, and `scores.iter().cloned()` converts `&i32` to `i32`.

[Go to the Top](#table-of-content)

---

## Exercise 034

- Implement: `fn partition_numbers(numbers: &[i32]) -> (Vec<i32>, Vec<i32>)`
- Given: `let numbers = vec![1, 2, 3, 4, 5, 6];`.
- These should produce: `([1, 3, 5], [2, 4, 6])`
- The first Vec should contain odd numbers and the second should contain even numbers.
- Use `partition()` to separate odd and even numbers.

### Response

```rust
fn partition_numbers(numbers: &[i32]) -> (Vec<i32>, Vec<i32>) {
    let (odd, even): (Vec<i32>, Vec<i32>) = numbers
        .iter()
        .cloned()
        .partition(|&num| num % 2 != 0);
    (odd, even)
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let (odd, even) = partition_numbers(&numbers);
    println!("Odd: {:?}, Even: {:?}", odd, even);
}
```

#### `partition()`

- The `partition()` method splits an iterator into two collections based on a predicate.
- The first collection contains elements for which the predicate returns `true`, and the second contains elements for which the predicate returns `false`.
- In the example, `numbers.iter().cloned().partition(|&num| num % 2 != 0)` separates odd and even numbers.

[Go to the Top](#table-of-content)

---

## Exercise 035

- Implement: `fn word_lengths(words: &[String]) -> Vec<usize>`
- Given:

    ```rust
        let words = vec![
            String::from("Rust"),
            String::from("Java"),
            String::from("Python"),
        ];
    ```

- Expected: `[4, 4, 6]`
- Use `fold()` and `iter()` only.

### Response

```rust
fn word_lengths(words: &[String]) -> Vec<usize> {
    words.iter().fold(Vec::new(), |mut acc, word| {
        acc.push(word.len());
        acc
    })
}

fn main() {
    let words = vec![
        String::from("Rust"),
        String::from("Java"),
        String::from("Python"),
    ];
    let lengths = word_lengths(&words);
    println!("Word lengths: {:?}", lengths);
}
```

#### `fold()`

- The `fold()` method is used to accumulate values from an iterator into a single result.
- It takes an initial accumulator value and a closure that specifies how to combine each element with the accumulator.
- In the example, `words.iter().fold(Vec::new(), |mut acc, word| { acc.push(word.len()); acc })` creates a new `Vec<usize>` and pushes the length of each word into it.

[Go to the Top](#table-of-content)

---

