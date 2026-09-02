<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD040 -->

# The Mechanics of Rust Lifetimes: From First Principles to Advanced Variance

## 1. Why Lifetimes Exist: The Physical Memory Problem

To understand why Rust introduced explicit lifetime annotations, we have to look at how computer software manages memory across different language paradigms.

### The Memory Management Spectrum

Programming languages traditionally choose between two extremes to prevent memory corruption:

1. **Garbage Collection (Java, Go, Python, C#):**
A dedicated runtime engine periodically pauses execution or traces object graphs to discover which heap allocations are still reachable.  
While this eliminates dangling pointers and use-after-free bugs, it introduces unpredictable latency, high memory footprints, and CPU overhead.

2. **Manual Memory Management (C, C++):**
The developer explicitly allocates and deallocates memory (`malloc`/`free`, `new`/`delete`).  
The compiler provides no static verification that a pointer remains valid.  
If an allocation is freed while a pointer still references it, a **dangling pointer** is created.  
Subsequent reads or writes produce **Use-After-Free** vulnerabilities, silent memory corruption, or segmentation faults.

```text
┌───────────────────────────────────────────┬───────────────────────────────────────────┐
│ Runtime Tracing (GC)                      │ Manual Management (C/C++)                 │
├───────────────────────────────────────────┼───────────────────────────────────────────┤
│ • Zero dangling pointers                  │ • Zero runtime overhead                   │
│ • Unpredictable GC pause spikes           │ • Undefined behavior on developer error   │
│ • Memory footprint bloat (1.5x - 3x)      │ • Use-after-free security vulnerabilities │
└───────────────────────────────────────────┴───────────────────────────────────────────┘
                                      ▲
                                      │
                         ┌────────────┴────────────┐
                         │   The Rust Objective    │
                         │ Zero-Cost Memory Safety │
                         │ (Compile-Time Proofs)   │
                         └─────────────────────────┘
```

Rust eliminates both compromises through static lifetime tracking: achieving absolute memory safety without a garbage collector.

### The Dangling Reference Hazard

Consider a CPU stack frame when execution enters and exits scopes.  
When a function executes a block, stack memory is reserved for local variables; when the block closes, that frame is reclaimed:

```rust
fn main() {
    let r: &i32;
    {
        let x: i32 = 42;
        r = &x; 
    } // <-- `x` is dropped from the stack here!

    println!("{r}"); // Hazard: `r` reads abandoned memory!
}
```

In an unverified language, `r` retains the physical 64-bit stack address of `x`.  
When `x` drops, that stack slot is marked reusable. Calling `println!` pushes new frames over that exact memory, causing `r` to read unpredictable garbage.  

Rust prevents this at compile time using an internal subsystem called the **Borrow Checker**.

### What a Lifetime Actually Is

A lifetime is **not**:

- A runtime duration, timer, or timestamp.
- An allocation directive.
- A mechanism that alters when an object is destroyed.

> **A lifetime is a static, compile-time label assigned by the compiler to a continuous region of the Control Flow Graph (CFG) over which a reference is guaranteed to point to valid, initialized memory.**

Every variable and reference in Rust possesses an inferred lifetime. In the previous snippet, the compiler maps the two scopes:

```text
'r (Scope of r): ┌──────────────────────────────────────┐
                 │  let r;                              │
'x (Scope of x): │  ┌─────────────────┐                 │
                 │  │  let x = 42;    │                 │
                 │  │  r = &x;        │                 │
                 │  └─────────────────┘ ◄── x dies here │
                 │  println!("{r}");  ◄── r used here!  │
                 └──────────────────────────────────────┘
```

The compiler observes that the lifetime of the data (`'x`) is strictly shorter than the lifetime of the reference pointing to it (`'r`).  
Because the relationship `'x: 'r` ("`'x` outlives `'r`") is violated, compilation is rejected with error `E0597`.

### The Function Boundary and Contract Isolation

Within a single function body, the compiler can trace every variable from creation to drop. It never requires you to annotate lifetimes locally.

However, Rust compiles every function in **complete isolation**.  
When the compiler checks `main()`, it does not analyze the internal implementation of helper functions called within `main()`.  
It relies strictly on the **function signature** as a binding contract.  

```rust
// The compiler rejects this function signature:
fn find_longest(s1: &str, s2: &str) -> &str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
```

The compiler rejects this because the signature does not declare whether the returned reference points to `s1` or `s2`.  
If the caller passes a long-lived `s1` and a short-lived `s2`, the caller cannot know how long the output reference remains safe to read.

Lifetime annotations (`'a`) exist to resolve this ambiguity at function boundaries.

```rust
fn find_longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
```

**The Meaning of `'a`:**
Writing `'a` does not prolong the life of `s1` or `s2`. It defines an algebraic relationship:

1. There exists some valid scope `'a`.
2. Both input references must live for *at least* `'a`.
3. The returned reference is valid for that *same* scope `'a`.

At the call site, the compiler sets `'a` to the **intersection (the shorter duration)** of the arguments provided.

## 2. The Core Rules: Elision and Variance

### The 3 Lifetime Elision Rules

To prevent syntax fatigue, Rust applies three deterministic rules to automatically infer lifetimes in function signatures.  
If a function fits these patterns, explicit `'a` syntax is omitted.

```text
                          ┌────────────────────────┐
                          │   Unannotated Function │
                          └───────────┬────────────┘
                                      │
                                      ▼
             ┌──────────────────────────────────────────────────┐
             │ RULE 1: Assign a fresh, unique lifetime parameter│
             │         to EVERY elided reference in the inputs. │
             └────────────────────────┬─────────────────────────┘
                                      │
                  ┌───────────────────┴───────────────────┐
                  ▼                                       ▼
       [ Exactly 1 input lifetime ]             [ Multiple input lifetimes ]
                  │                                       │
                  ▼                                       ▼
    ┌───────────────────────────┐           ┌───────────────────────────┐
    │ RULE 2:                   │           │ Is one input &self        │
    │ Output gets that exact    │           │ or &mut self?             │
    │ same lifetime.            │           └─────────────┬─────────────┘
    └───────────────────────────┘                         │
                                           ┌──────────────┴──────────────┐
                                           ▼                             ▼
                                        [ YES ]                       [ NO ]
                                           │                             │
                                           ▼                             ▼
                             ┌───────────────────────────┐ ┌───────────────────────────┐
                             │ RULE 3:                   │ │ ELISION FAILS!            │
                             │ Output gets lifetime      │ │ Compiler errors:          │
                             │ of `self`.                │ │ "missing lifetime         │
                             └───────────────────────────┘ │  specifier"               │
                                                           └───────────────────────────┘
```

#### Rule 1: Input Parameter Generation

Each elided lifetime in the function’s input parameter list is assigned a distinct, unique lifetime parameter.

- **Written:** `fn process(a: &str, b: &str)`
- **Desugared:** `fn process<'a, 'b>(a: &'a str, b: &'b str)`

#### Rule 2: Single Input Lifetime Propagation

If there is **exactly one** input lifetime parameter (whether explicit or inferred), that lifetime is automatically assigned to all unannotated output lifetimes.

- **Written:** `fn trim_leading(s: &str) -> &str`
- **Desugared:** `fn trim_leading<'a>(s: &'a str) -> &'a str`

#### Rule 3: Method `self` Propagation

If there are multiple input lifetime positions, but one of them is `&self` or `&mut self`, the lifetime of `self` is automatically assigned to all unannotated output lifetimes.

- **Written:** `fn extract(&self, pattern: &str) -> &str`
- **Desugared:** `fn extract<'s, 'p>(&'s self, pattern: &'p str) -> &'s str`

### Subtyping and Variance

Rust contains **no object-oriented inheritance**, but it does possess **subtyping over lifetimes**.

If lifetime `'long` outlives lifetime `'short` (written `'long: 'short`), then `'long` is a **subtype** of `'short`:

$$\text{'long outlives 'short} \implies \text{'long} \le \text{'short}$$

Wherever a reference with a shorter lifetime is demanded, a reference with a longer lifetime can be safely substituted.

**Variance** defines how subtyping between base components transfers to complex wrapper types.

```text
Given a Subtype relation: Sub ≤ Super ('long : 'short)

1. COVARIANCE:      Type<Sub>   ≤   Type<Super>     (Direction Preserved)
2. CONTRAVARIANCE:  Type<Super> ≤   Type<Sub>       (Direction Inverted)
3. INVARIANCE:      Type<Sub> and Type<Super> have NO relationship.
```

| Type Constructor             | Variance over `'a` | Variance over `T`                             | Practical Implication                                                                            |
| ---------------------------- | ------------------ | --------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| **`&'a T`**                  | **Covariant**      | **Covariant**                                 | A reference to long-lived data can always be passed to a function expecting a shorter lifetime.  |
| **`Box<T>`, `Vec<T>`**       | N/A                | **Covariant**                                 | Owned heap wrappers preserve the subtyping of their payload.                                     |
| **`&'a mut T`**              | **Covariant**      | **INVARIANT**                                 | The borrow `'a` can shrink, but the underlying type `T` **cannot change**.                       |
| **`Cell<T>`, `RefCell<T>`**  | N/A                | **INVARIANT**                                 | Types that allow interior mutation must be invariant over `T`.                                   |
| **`fn(T) -> U`**             | N/A                | **Contravariant (`T`)** / **Covariant (`U`)** | Arguments invert subtyping; return values preserve it.                                           |

#### Why `&mut T` Must Be Invariant Over `T`

If a mutable reference were covariant over its target type `T`, you could write a short-lived address into a pointer that the caller expects to point to long-lived memory:

```rust
// Hypothetical disaster IF &mut T were covariant over T:
fn overwrite_target<'a>(target: &mut &'a str) {
    let local = String::from("short-lived");
    // If covariant, target (&mut &'static str) could be coerced to (&mut &'short str)
    // and store a reference to `local` into the caller's static location:
    // *target = &local;
}

fn main() {
    let mut persistent: &'static str = "forever";
    // overwrite_target(&mut persistent);
    // When the function finishes, `local` is deallocated.
    // `persistent` now points to dead stack memory! USE-AFTER-FREE!
}
```

Because `&mut T` is **invariant over `T**`, the compiler rejects any coercion that attempts to shrink the target type's lifetime inside a mutable reference.

## 3. The Problem Catalog: 6 Real-World Challenges

### Challenge 1: Function Contract Isolation & Runtime Branching

#### The Code

```rust
fn select<'a>(flag: bool, x: &'a str, y: &'a str) -> &'a str {
    if flag {
        x
    } else {
        y
    }
}

fn main() {
    let string_a = String::from("alpha");
    let outcome;
    
    {
        let string_b = String::from("beta");
        outcome = select(true, string_a.as_str(), string_b.as_str());
    } // string_b dropped here

    println!("Selected: {outcome}");
}
```

#### Compiler Diagnostics

```text
error[E0597]: `string_b` does not live long enough
  --> src/main.rs:15:51
   |
15 |         outcome = select(true, string_a.as_str(), string_b.as_str());
   |                                                   ^^^^^^^^ borrowed value does not live long enough
16 |     }
   |     - `string_b` dropped here while still borrowed
17 | 
18 |     println!("Selected: {outcome}");
   |                          ------- borrow later used here
```

#### Under the Hood Mechanics

1. **Contract Overlap (Intersection):**

`select` declares that both `x` and `y` share lifetime `'a`, and returns a reference with lifetime `'a`. At the call site:

- `string_a` lives for the outer scope (`'main`).
- `string_b` lives for the inner block scope (`'inner`).
- The compiler sets `'a` to the **intersection**: `'a = 'inner`.

2. **The Isolation Barrier:**

Even though `flag` is hardcoded to `true` at runtime, the borrow checker ignores runtime values.  
It evaluates the function call against its declared contract: *"The returned reference can live at most as long as `'inner`."*

3. **The Invalid Fix Trap:**

Changing the signature to `fn select<'a, 'b>(flag: bool, x: &'a str, y: &'b str) -> &'a str` causes the function body to fail compilation.  
The compiler sees `return y;` in the `else` branch, which returns a `'b` reference when an `'a` reference was promised.

#### The Correct Fix

If the output must outlive the short-lived scope, the data must either be cloned into an owned `String`, or the lifetimes of the inputs must be decoupled at the call site by scoping `string_b` appropriately.

### Challenge 2: Struct Lifetime Propagation & Scope Contraction

#### The Code

```rust
struct Highlighter<'a> {
    text: &'a str,
}

impl<'a> Highlighter<'a> {
    fn update_text(&mut self, new_text: &'a str) {
        self.text = new_text;
    }
}

fn main() {
    let mut hl = Highlighter { text: "static greeting" };

    {
        let local_string = String::from("temporary dynamic text");
        hl.update_text(local_string.as_str());
    } // local_string drops here

    println!("Current text: {}", hl.text);
}
```

#### Compiler Diagnostics

```text
error[E0597]: `local_string` does not live long enough
  --> src/main.rs:16:24
   |
16 |         hl.update_text(local_string.as_str());
   |                        ^^^^^^^^^^^^ borrowed value does not live long enough
17 |     }
   |     - `local_string` dropped here while still borrowed
18 | 
19 |     println!("Current text: {}", hl.text);
   |                                  ------- borrow later used here
```

#### Under the Hood Mechanics

1. **Concrete Struct Specialization:**

`"static greeting"` is a string literal baked into the program's binary; its lifetime is `'static`.  
When `hl` is initialized, its concrete type becomes:

$$\text{Highlighter}\langle'\text{static}\rangle$$

2. **Parameter Enforcement:**

`update_text` demands `new_text: &'a str`. Because `'a = 'static`, the method requires any incoming slice to live for `'static`.

3. **The Hazard:**

`local_string` is destroyed at the end of the inner block. If Rust allowed this code to compile, `hl.text` would become a dangling pointer on line 19, reading reclaimed stack memory.

### Challenge 3: The Self-Borrow Lockout Trap (`&'a mut self`)

#### The Code

```rust
struct Counter<'a> {
    name: &'a str,
    count: usize,
}

impl<'a> Counter<'a> {
    // The Trap: Annotating &mut self with the struct's inner data lifetime 'a
    fn tick(&'a mut self) {
        self.count += 1;
    }
}

fn main() {
    let mut c = Counter {
        name: "request_counter",
        count: 0,
    };

    c.tick(); // Call 1
    c.tick(); // Call 2: Fails!
    
    println!("Total: {}", c.count); // Fails!
}
```

#### Compiler Diagnostics

```text
error[E0499]: cannot borrow `c` as mutable more than once at a time
  --> src/main.rs:18:5
   |
17 |     c.tick(); // Call 1
   |     -------- first mutable borrow occurs here
18 |     c.tick(); // Call 2
   |     ^ second mutable borrow occurs here
19 |     
20 |     println!("Total: {}", c.count);
   |                           ------- first borrow later used here
```

#### Under the Hood Mechanics

1. **Standard `&mut self` Elision:**

Normally, an unannotated `&mut self` desugars into `fn tick<'s>(&'s mut self)`.  
The mutable borrow `'s` lasts only for the duration of the method call.  
When `tick` returns, `'s` ends, releasing the exclusive borrow.

2. **The Explicit `'a` Trap:**

By writing `fn tick(&'a mut self)`, you explicitly state:
> *"Borrow this entire struct exclusively for the lifetime `'a`."*  
> Because `c.name` holds `"request_counter"` (`'static`), `'a` is valid for the rest of `main()`.

3. **The Lockout:**

The first call `c.tick()` acquires an exclusive mutable borrow of `c` that **never releases until the end of `main**`.  
Any subsequent access—whether another call to `c.tick()` or an immutable read in `println!`—is rejected as an illegal secondary borrow.

#### The Correct Fix

Never annotate `&self` or `&mut self` with the struct’s internal storage lifetime `'a`.  
Let Rule 1 provide a short, decoupled lifetime for the borrow:

```rust
impl<'a> Counter<'a> {
    fn tick(&mut self) { // Desugars to fn tick<'s>(&'s mut self)
        self.count += 1;
    }
}
```

### Challenge 4: Method Elision Binding & Multi-Parameter Returns

#### The Code

```rust
struct StringHolder<'a> {
    content: &'a str,
}

impl<'a> StringHolder<'a> {
    fn choose_longer(&self, other: &str) -> &str {
        if self.content.len() >= other.len() {
            self.content
        } else {
            other
        }
    }
}
```

#### Compiler Diagnostics

```text
error[E0623]: lifetime mismatch
  --> src/main.rs:9:13
   |
6  |     fn choose_longer(&self, other: &str) -> &str {
   |                                    ----     ----
   |                                    |
   |                                    this parameter and the return type are declared with different lifetimes...
...
10 |             other
   |             ^^^^^ ...but data from `other` is returned here
```

#### Under the Hood Mechanics

1. **Elision Rule 3 Interference:**

In `choose_longer(&self, other: &str) -> &str`, the compiler applies:

- Rule 1: `&self` gets lifetime `'s`, and `other` gets lifetime `'o`.
- Rule 3: Because `&self` is present, the unannotated return reference is automatically assigned the lifetime of `self` (`&'s str`).

2. **The Signature Mismatch:**

The compiler desugars the method into:

```rust
fn choose_longer<'s, 'o>(&'s self, other: &'o str) -> &'s str
```

When the method body returns `other` in the `else` block, it returns a reference with lifetime `'o`.  
The signature, however, promised a reference tied to `'s`. Because `'o` and `'s` are unrelated, compilation fails.

#### The Correct Fix

Explicitly declare a generic lifetime parameter `'b` for `other` and the return type, adding a bound that `'a` (the content inside `StringHolder`) outlives `'b`:

```rust
impl<'a> StringHolder<'a> {
    fn choose_longer<'b>(&self, other: &'b str) -> &'b str 
    where 
        'a: 'b 
    {
        if self.content.len() >= other.len() {
            self.content // 'a safely coerces to 'b via covariance
        } else {
            other        // Matches 'b directly
        }
    }
}
```

### Challenge 5: Iteration Lifetime Boundaries & Ownership Transfer

#### The Code

```rust
struct SimpleCache<'a> {
    last_item: &'a str,
}

impl<'a> SimpleCache<'a> {
    fn store(&mut self, item: &'a str) {
        self.last_item = item;
    }
}

fn main() {
    let mut cache = SimpleCache { last_item: "initial" };

    for i in 0..2 {
        let dynamic_string = format!("item-{i}");
        cache.store(dynamic_string.as_str()); // Line X
    }

    println!("Cached: {}", cache.last_item);
}
```

#### Compiler Diagnostics

```text
error[E0597]: `dynamic_string` does not live long enough
  --> src/main.rs:16:21
   |
16 |         cache.store(dynamic_string.as_str());
   |                     ^^^^^^^^^^^^^^ borrowed value does not live long enough
17 |     }
   |     - `dynamic_string` dropped here while still borrowed
18 | 
19 |     println!("Cached: {}", cache.last_item);
   |                            --------------- borrow later used here
```

#### Under the Hood Mechanics

1. **Loop Iteration Drop Semantics:**
`dynamic_string` is declared *inside* the `for` loop body. It is **allocated and dropped on every single iteration**.  
At the closing brace `}` of iteration 0, `dynamic_string` is deallocated.

2. **The Lifetime Contraction:**
`cache` was initialized with `"initial"` (`'static`), meaning `cache` lives for the outer scope of `main()`.
Attempting to store a reference that dies at the end of each iteration into a struct that outlives the loop causes a use-after-free hazard.

3. **The Ownership Paradigm Shift:**
If a structure needs to store dynamically generated data across iteration boundaries, **it must take ownership of the data, not borrow it**.

#### The Correct Fix

Remove the lifetime parameter and have the struct own an allocated `String`:

```rust
struct SimpleCache {
    last_item: String,
}

impl SimpleCache {
    fn store(&mut self, item: String) {
        self.last_item = item;
    }
}

fn main() {
    let mut cache = SimpleCache { last_item: String::from("initial") };

    for i in 0..2 {
        let dynamic_string = format!("item-{i}");
        cache.store(dynamic_string); // Ownership moved into cache
    }

    println!("Cached: {}", cache.last_item);
}
```

### Challenge 6: Mutable Reference Invariance & Pointer Aliasing Hazards

#### The Code

```rust
fn assign_str<'a>(target: &mut &'a str, source: &'a str) {
    *target = source;
}

fn main() {
    let mut greeting: &'static str = "hello";

    {
        let local_text = String::from("temporary");
        assign_str(&mut greeting, local_text.as_str()); // Line X
    } // local_text dropped here!

    println!("Greeting: {greeting}"); // Line Y
}
```

#### Compiler Diagnostics

```text
error[E0597]: `local_text` does not live long enough
  --> src/main.rs:12:35
   |
12 |         assign_str(&mut greeting, local_text.as_str());
   |                                   ^^^^^^^^^^^^^^^^^^^ borrowed value does not live long enough
13 |     }
   |     - `local_text` dropped here while still borrowed
14 | 
15 |     println!("Greeting: {greeting}");
   |                          -------- borrow later used here
```

#### Under the Hood Mechanics

1. **The Nature of `target`:**
The parameter `target` has type `&mut &'a str`.  
This is an exclusive reference to a reference variable that holds a memory address.  
When `*target = source;` executes, it physically overwrites the memory address stored inside `greeting`.

2. **Why Invariance Is Enforced:**
    - If this were an immutable reference `&'a &'static str`, covariance would allow `'static` to shrink to the inner block scope.
    - However, `target` is a **mutable reference (`&mut T`)**. `&mut T` is **invariant over `T**`.
    - The compiler forbids `greeting` (`&'static str`) from shrinking to match `local_text` (`&'inner str`).

3. **The Consequence of Invariance:**
Because `greeting` cannot shrink, `'a` in `assign_str` is forced to resolve to `'static`.  
The compiler then demands that `source` (`local_text.as_str()`) must also live for `'static`. Since `local_text` dies at the closing brace of the inner block, compilation fails.

4. **The Hazard Prevented:**
If the compiler allowed `greeting` to shrink, `greeting` in `main()` would hold an address pointing directly into `local_text`'s destroyed stack memory at Line Y.

## 4. Synthesis: The Complete Mental Model for Lifetimes

### The 7 Golden Invariants of Rust Lifetimes

1. **Lifetimes Are Zero-Cost:**
Annotations exist solely for static analysis.  
They compile away entirely, leaving raw pointers in the generated machine code.

2. **Annotations Describe; They Do Not Dictate:**
Writing `'a` does not extend the physical lifespan of an object.  
It only describes requirements to the borrow checker.

3. **Functions Are Isolated Contracts:**
The compiler never checks runtime control flow across function calls.  
Lifetimes are evaluated against the function's static signature.

4. **Shared Input Lifetimes Force Intersection:**
Assigning the same lifetime `'a` to multiple input parameters forces `'a` to shrink to the **shortest** input lifespan provided.

5. **Decouple Independent Lifetimes:**
If an output derives from only one of several input references, give each input reference a distinct lifetime parameter (`'a`, `'b`).

6. **Never Bind `&mut self` to the Struct's Lifetime Parameter:**
Writing `fn method(&'a mut self)` on a struct `Foo<'a>` creates an exclusive mutable borrow that locks the struct for its entire lifespan.

7. **`&mut T` Is Strictly Invariant Over `T`:**
A mutable reference cannot alter the lifetime of its underlying target type.  
This invariant prevents writing short-lived references into long-lived memory slots.

### Quick Diagnostic Matrix: When to Borrow vs. When to Own

```text
                                  ┌────────────────────────────────┐
                                  │ Does the data need to outlive  │
                                  │ the current stack frame?       │
                                  └───────────────┬────────────────┘
                                                  │
                         ┌────────────────────────┴────────────────────────┐
                         ▼                                                 ▼
                      [ YES ]                                           [ NO ]
                         │                                                 │
            ┌────────────┴────────────┐                     ┌──────────────┴──────────────┐
            ▼                         ▼                     ▼                             ▼
   [ Shared across threads ]   [ Single thread ]    [ Read-only inspect ]         [ In-place mutate ]
            │                         │                     │                             │
            ▼                         ▼                     ▼                             ▼
       Arc<T> / Box<T>            Rc<T> / Box<T>          &'a T                        &'a mut T
```

| Use Case                            | Recommended Type Pattern             | Lifetime Requirement                             |
| ----------------------------------- | ------------------------------------ | ------------------------------------------------ |
| **Short-lived read inspection**     | `&'a T` or `&'a str`                 | Reference valid for the inspection scope.        |
| **Short-lived in-place mutation**   | `&'a mut T`                          | Exclusive reference; invariant over `T`.         |
| **Zero-copy parsing / AST**         | `struct Ast<'a> { slice: &'a str }`  | Struct lifetime bound to input buffer.           |
| **Iterative caching / aggregation** | `struct Cache { item: String }`      | **Own the data.** Do not use references.         |
| **Single-threaded shared graphs**   | `Rc<RefCell<T>>`                     | Interior mutability; runtime borrow tracking.    |
| **Multi-threaded shared state**     | `Arc<Mutex<T>>`                      | `Send + Sync + 'static`; thread-safe ownership.  |

---
