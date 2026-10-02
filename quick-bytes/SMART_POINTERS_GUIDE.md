# Rust Smart Pointers

## Table of Content

- [Overview](#overview)
- [The problem](#the-problem)
- [The solution: smart pointers](#the-solution-smart-pointers)
- [`Box<T>`](#boxt)
- [`Rc<T>`](#rct)
- [`RefCell<T>`](#refcellt)
- [`Cell<T>`](#cellt)
- [`Arc<T>`](#arct)
- [`Mutex<T>`](#mutext)
- [`RwLock<T>`](#rwlockt)
- [Summary](#summary)

## Overview

**Rust smart pointers:**

1. **`Box<T>`** — heap allocation, single owner, enables recursive types and trait objects.
2. **`Rc<T>`** — reference-counted heap allocation, multiple owners, single-threaded only.
3. **`Arc<T>`** — atomic reference-counted, multiple owners, thread-safe (`Send` + `Sync`).
4. **`RefCell<T>`** — interior mutability, borrow rules checked at runtime instead of compile time, single-threaded.
5. **`Cell<T>`** — interior mutability via get/set (copy in, copy out), no runtime borrow checks.
6. **`Mutex<T>`** — interior mutability with mutual-exclusion locking, thread-safe.
7. **`RwLock<T>`** — interior mutability, multiple concurrent readers or one writer, thread-safe.

## The problem

Rust's ownership model is deliberately strict, and that strictness is exactly what guarantees memory safety without a garbage collector:

- **One owner.** Every value has exactly one owner; when the owner goes out of scope, the value is dropped. Ownership *moves* — it is not shared.
- **Borrowing is temporary and exclusive.** At any moment you may hand out *many* shared references (`&T`) **or** *one* mutable reference (`&mut T`), never both — and no reference may outlive the value it points to.
- **Sizes are known at compile time.** Anything placed on the stack must have a size the compiler can compute up front.

These rules cover the vast majority of code, but some shapes simply don't fit them:

- A **recursive type** (a list whose tail is another list) or a **trait object** (`dyn Trait`) has no compile-time size, so it can't live on the stack directly.
- A **graph or shared cache** needs the *same* value to have **several owners** at once, which a single-owner move cannot express.
- Sometimes you must **mutate data through a shared reference** — for instance a node that several parts of the program hold simultaneously.
- **Sharing across threads** layers data races on top of all of the above, so even the reference count has to become atomic and mutation has to be synchronized.

A plain reference (`&T` / `&mut T`) can't rescue any of these: it never owns the data, cannot outlive its owner, and the borrow checker forbids the aliasing-plus-mutation they require. We need types that *own* their data and encode a richer ownership, sharing, mutation, or synchronization policy — while still upholding Rust's safety guarantees. Those types are **smart pointers**.

## The solution: smart pointers

A *pointer* is a value that holds a memory address. The everyday pointer in Rust is the reference (`&T`): it borrows data, adds no overhead, and has no special powers.

A *smart pointer* is a struct that behaves like a pointer but layers extra behavior on top — ownership of the pointed-to data, reference counting, or runtime synchronization — all enforced through its API. Two traits are what make them "smart":

- **`Deref`** — lets the smart pointer be used like a plain reference to its inner `T`, so `*p`, method calls, and deref coercion all work transparently.
- **`Drop`** — runs cleanup when the pointer leaves scope: free the heap allocation, decrement a reference count, or release a lock.

Unlike a borrow, a smart pointer usually *owns* the data it points to. In fact `String` and `Vec<T>` already are smart pointers — each owns a heap buffer and frees it on drop. The types below are the general-purpose building blocks from `std`, and each answers one facet of the problem above:

- **`Box<T>`** — gives an unsized or large value a fixed-size home on the heap (recursive types, trait objects).
- **`Rc<T>`** — lets one value have several owners in a single-threaded program, via reference counting.
- **`RefCell<T>`** — mutates data held behind a shared reference in single-threaded code, with borrow rules checked at runtime.
- **`Cell<T>`** — the same interior mutability for small `Copy` values, moving whole values in and out instead of handing out references.
- **`Arc<T>`** — the same multi-owner sharing, made safe across threads with an atomic count.
- **`Mutex<T>`** — lets many threads mutate shared data safely by serializing access behind a lock.

Each section builds from a **simple** first-contact example, through an **intermediate** real-world shape, to an **advanced** idiomatic pattern.

## `Box<T>`

**Problem:** Rust needs a compile-time-known size for stack values. Recursive types and `dyn Trait` have no fixed size, and large structs are expensive to move around by value.

```rust
// Does NOT compile: a type that directly contains itself has no finite size.
enum List {
    Cons(i32, List), // recursive with no indirection
    Nil,
}
// error[E0072]: recursive type `List` has infinite size
```

**Solution:** `Box<T>` stores `T` on the heap and keeps a fixed-size pointer on the stack. Single owner, zero runtime cost beyond the allocation itself, and it derefs straight to `T`.

### Simple — a value on the heap

```rust
let b = Box::new(5);
println!("b = {}", *b); // deref to read the i32 -> 5
// `b` is dropped here; the heap allocation is freed automatically.
```

### Intermediate — recursive types and trait objects

A recursive type has no finite size, so the compiler rejects it until a `Box` breaks the cycle with a fixed-size pointer:

```rust
enum List {
    Cons(i32, Box<List>), // Box gives the variant a known size
    Nil,
}
use List::{Cons, Nil};

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
```

The same idea stores differently-typed values behind one interface as `Box<dyn Trait>`:

```rust
trait Shape {
    fn area(&self) -> f64;
}

struct Circle { r: f64 }
struct Square { s: f64 }

impl Shape for Circle { fn area(&self) -> f64 { std::f64::consts::PI * self.r * self.r } }
impl Shape for Square { fn area(&self) -> f64 { self.s * self.s } }

let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { r: 2.0 }),
    Box::new(Square { s: 3.0 }),
];

let total: f64 = shapes.iter().map(|s| s.area()).sum();
println!("total area = {total:.2}");
```

### Advanced — returning an unsized value (a boxed closure)

A closure has an anonymous type whose size depends on what it captures, so it can't be returned by value. Boxing it behind `dyn Fn` gives the caller one uniform, sized handle:

```rust
fn make_multiplier(factor: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x * factor) // captures `factor`; concrete size hidden from caller
}

let triple = make_multiplier(3);
println!("{}", triple(10)); // 30
```

The same pattern underpins `Box<dyn Error>` for type-erased error returns and `Box::leak` for promoting an allocation to `&'static mut T`.

## `Rc<T>`

**Problem:** `Box<T>` allows only one owner. Some structures — graphs, shared caches, trees with shared children — need *several* owners within a single-threaded program.

```rust
// Does NOT compile: `Box` has a single owner, so the tail can't be shared.
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use List::{Cons, Nil};

let shared = Box::new(Cons(10, Box::new(Nil)));
let a = Cons(1, shared); // `shared` is moved into `a` here
let b = Cons(2, shared); // error[E0382]: use of moved value: `shared`
```

**Solution:** `Rc<T>` (reference counted) heap-allocates `T` next to a count. `Rc::clone` bumps that count and hands back another handle — no deep copy. The data is freed the instant the count reaches zero. Access is read-only, and `Rc<T>` is **not** thread-safe.

### Simple — two owners of the same data

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a); // cheap: only increments the count

println!("{a} and {b}");
println!("count = {}", Rc::strong_count(&a)); // 2
```

### Intermediate — sharing a common tail

Two lists can share the same suffix without either one owning it exclusively:

```rust
use std::rc::Rc;

enum List {
    Cons(i32, Rc<List>),
    Nil,
}
use List::{Cons, Nil};

let shared = Rc::new(Cons(10, Rc::new(Nil)));
let _a = Cons(1, Rc::clone(&shared)); // 1 -> [shared]
let _b = Cons(2, Rc::clone(&shared)); // 2 -> [shared]

println!("shared is owned by {} lists", Rc::strong_count(&shared)); // 3
```

### Advanced — shared *mutable* state with `Rc<RefCell<T>>`

`Rc<T>` on its own is read-only. Wrapping the inner value in a `RefCell` (interior mutability, covered below) yields the canonical single-threaded "shared and mutable" handle:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
let other = Rc::clone(&shared);

other.borrow_mut().push(4);        // mutate through one handle
println!("{:?}", shared.borrow()); // observe through another -> [1, 2, 3, 4]
```

Watch out: a cycle of `Rc`s never reaches count zero and leaks. Break cycles with `Weak<T>` (covered below).

## `RefCell<T>`

**Problem:** Rust checks borrows at compile time, so you can never reach a `&mut T` through a shared `&T`. But `Rc` only ever hands out shared references, and plenty of designs — a shared graph node, a lazily filled cache, a mock that records calls — need to mutate exactly such shared data.

```rust
use std::rc::Rc;

// Does NOT compile: `Rc` only ever hands out shared `&T`, so its contents are read-only.
let shared = Rc::new(0);
let a = Rc::clone(&shared);
*a += 1; // error[E0594]: cannot assign to data in an `Rc<i32>`
```

**Solution:** `RefCell<T>` moves the borrow check from compile time to *runtime*. `.borrow()` hands out a `Ref<T>` (a `&T`) and `.borrow_mut()` a `RefMut<T>` (a `&mut T`), tracking how many of each are live. The rule is unchanged — many readers XOR one writer — but a violation *panics* (`already borrowed`) instead of failing to compile. Single-threaded only; this is the mutable half of the `Rc<RefCell<T>>` pattern.

### Simple — mutate through a shared handle

```rust
use std::cell::RefCell;

let cell = RefCell::new(5);

*cell.borrow_mut() += 1;       // take a RefMut, mutate, drop it
println!("{}", cell.borrow()); // take a Ref to read -> 6
```

### Intermediate — the `Rc<RefCell<T>>` shared, mutable node

```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
let a = Rc::clone(&shared);
let b = Rc::clone(&shared);

a.borrow_mut().push(4); // mutate through one owner
b.borrow_mut().push(5); // ...and through another

println!("{:?}", shared.borrow()); // [1, 2, 3, 4, 5]
```

### Advanced — interior mutability behind `&self` (memoization)

A method that takes `&self` can still update cached state — the value is logically immutable, but its cache is not:

```rust
use std::cell::RefCell;

struct Fib {
    cache: RefCell<Vec<u64>>, // mutable even though every method takes &self
}

impl Fib {
    fn new() -> Self {
        Fib { cache: RefCell::new(vec![0, 1]) }
    }

    fn get(&self, n: usize) -> u64 {
        while self.cache.borrow().len() <= n {
            let len = self.cache.borrow().len();
            let next = self.cache.borrow()[len - 1] + self.cache.borrow()[len - 2];
            self.cache.borrow_mut().push(next);
        }
        self.cache.borrow()[n]
    }
}

let fib = Fib::new();
println!("{}", fib.get(10)); // 55
```

The safety check is real, just deferred — overlapping borrows compile but blow up at runtime:

```rust
use std::cell::RefCell;

let cell = RefCell::new(0);
let _read = cell.borrow();      // a Ref is now live
let _write = cell.borrow_mut(); // panics: already borrowed: BorrowMutError
```

## `Cell<T>`

**Problem:** `RefCell`'s borrow tracking costs a little runtime bookkeeping and can panic. When all you need is to swap a small `Copy` value in and out behind a shared reference — never a reference *into* it — that machinery is overkill, yet the borrow checker still forbids the plain assignment.

```rust
// Does NOT compile: `&self` is a shared reference, so the field is read-only.
struct Toggle { on: bool }

impl Toggle {
    fn flip(&self) {
        self.on = !self.on; // error[E0594]: cannot assign to `self.on`, which is behind a `&` reference
    }
}
```

**Solution:** `Cell<T>` gives interior mutability by *value*: `get` copies the contents out (when `T: Copy`), `set` overwrites, `replace` swaps and returns the old value, `take` resets to `Default`. It never hands out a reference into its interior, so there are no borrows to track, nothing to panic, and no runtime overhead. Single-threaded only.

### Simple — get/set through a shared reference

```rust
use std::cell::Cell;

let c = Cell::new(5);
c.set(10);               // overwrite; no &mut needed
println!("{}", c.get()); // copy out -> 10
```

### Intermediate — a mutable field behind `&self`

```rust
use std::cell::Cell;

struct Toggle { on: Cell<bool> }

impl Toggle {
    fn flip(&self) {      // &self, not &mut self
        self.on.set(!self.on.get());
    }
}

let t = Toggle { on: Cell::new(false) };
t.flip();
t.flip();
println!("{}", t.on.get()); // false
```

### Advanced — a shared id counter, and non-`Copy` via `replace`/`take`

```rust
use std::cell::Cell;

struct IdGenerator { next: Cell<u64> }

impl IdGenerator {
    fn new() -> Self { IdGenerator { next: Cell::new(1) } }

    fn next_id(&self) -> u64 { // hands out unique ids without &mut
        let id = self.next.get();
        self.next.set(id + 1);
        id
    }
}

let ids = IdGenerator::new();
println!("{} {} {}", ids.next_id(), ids.next_id(), ids.next_id()); // 1 2 3
```

`get` needs `T: Copy`, but `replace` and `take` move whole values, so `Cell` works for non-`Copy` types too:

```rust
use std::cell::Cell;

let slot = Cell::new(String::from("first"));
let old = slot.replace(String::from("second")); // swap, return the old value
println!("{old}");                              // first
let taken = slot.take();                        // leave the Default (empty String)
println!("[{}] [{}]", taken, slot.take());      // [second] []
```

## `Arc<T>`

**Problem:** `Rc<T>`'s count is a plain integer — racy to update from multiple threads — so `Rc<T>` is neither `Send` nor `Sync`.

```rust
use std::rc::Rc;
use std::thread;

let data = Rc::new(vec![1, 2, 3]);
let data2 = Rc::clone(&data);

// Does NOT compile: `Rc` is not `Send`, so it can't cross a thread boundary.
thread::spawn(move || {
    println!("{:?}", data2);
}); // error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
```

**Solution:** `Arc<T>` (atomic reference counted) is `Rc<T>` with the count maintained by atomic operations, making it safe to clone and drop across threads. Those atomics cost a little more than `Rc`, so reach for `Arc` only when you actually cross a thread boundary. Access is still read-only.

### Simple — hand data to one thread

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);
let data2 = Arc::clone(&data);

let handle = thread::spawn(move || {
    println!("worker sees {:?}", data2);
});

handle.join().unwrap();
println!("main still owns {:?}", data); // original handle remains valid
```

### Intermediate — fan out read-only work

Every worker gets its own handle to the same buffer and processes a slice:

```rust
use std::sync::Arc;
use std::thread;

let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
let mut handles = vec![];

for chunk in 0..4 {
    let numbers = Arc::clone(&numbers);
    handles.push(thread::spawn(move || {
        let start = chunk * 25;
        let sum: i32 = numbers[start..start + 25].iter().sum();
        println!("chunk {chunk} sum = {sum}");
    }));
}

for h in handles { h.join().unwrap(); }
```

### Advanced — a shared trait object across threads

`Arc<dyn Trait + Send + Sync>` shares one behavior implementation with a whole pool of workers:

```rust
use std::sync::Arc;
use std::thread;

trait Logger: Send + Sync {
    fn log(&self, msg: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) { println!("[log] {msg}"); }
}

let logger: Arc<dyn Logger> = Arc::new(ConsoleLogger);
let mut handles = vec![];

for id in 0..3 {
    let logger = Arc::clone(&logger);
    handles.push(thread::spawn(move || logger.log(&format!("hello from thread {id}"))));
}

for h in handles { h.join().unwrap(); }
```

To *mutate* shared data across threads, pair `Arc` with a lock — that is exactly `Mutex<T>`, next.

## `Mutex<T>`

**Problem:** `Arc<T>` gives shared, read-only access. Concurrent *mutation* needs synchronization, or two threads could write at once and corrupt the data.

```rust
use std::sync::Arc;
use std::thread;

let counter = Arc::new(0);
let c = Arc::clone(&counter);

// Does NOT compile: `Arc` derefs to `&T` (shared), never `&mut T`.
thread::spawn(move || {
    *c += 1; // error[E0594]: cannot assign to data in an `Arc<i32>`
});
```

**Solution:** `Mutex<T>` (mutual exclusion) guards `T` behind `.lock()`, which blocks until the lock is free and returns a `MutexGuard<T>` that derefs to `&mut T`. The lock releases automatically when the guard drops. If a thread panics while holding the lock, the mutex becomes **poisoned** and later `.lock()` calls return `Err`, so the corruption can't be ignored silently.

### Simple — lock, mutate, unlock

```rust
use std::sync::Mutex;

let m = Mutex::new(5);
{
    let mut guard = m.lock().unwrap(); // guard: MutexGuard<i32>
    *guard += 1;
} // guard dropped here -> lock released

println!("{:?}", m); // Mutex { data: 6, poisoned: false, .. }
```

### Intermediate — a counter shared across threads

`Arc` shares ownership between threads; the `Mutex` makes the increment safe:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..5 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        *counter.lock().unwrap() += 1; // lock held only for this line
    }));
}

for h in handles { h.join().unwrap(); }
println!("{}", *counter.lock().unwrap()); // 5
```

### Advanced — shared collection and recovering from poisoning

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

let table = Arc::new(Mutex::new(HashMap::new()));
let mut handles = vec![];

for i in 0..5 {
    let table = Arc::clone(&table);
    handles.push(thread::spawn(move || {
        table.lock().unwrap().insert(i, i * i);
    }));
}

for h in handles { h.join().unwrap(); }

// Had a worker panicked mid-write, `.lock()` would return Err(poisoned);
// `into_inner()` deliberately recovers the (possibly inconsistent) data.
let guard = table.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
println!("{:?}", *guard); // {0: 0, 1: 1, 2: 4, 3: 9, 4: 16}
```

Keep critical sections short — hold the guard only as long as you must, since every other thread blocks meanwhile. For read-heavy workloads prefer `RwLock<T>` (covered below).

## `RwLock<T>`

**Problem:** `Mutex<T>` allows only one accessor at a time — even two threads that just want to *read* the data must take turns through `.lock()`. When reads vastly outnumber writes, this is wasteful: concurrent reads don't conflict with each other, only a write conflicts with anything else. `Mutex<T>` cannot express "many readers, OR one writer" — it can only express "one thread, period."

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let mut handles = vec![];

for _ in 0..10 {
    let data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        let d = data.lock().unwrap(); // blocks other readers too, unnecessarily
        println!("{:?}", *d);
    }));
}
```

Ten threads only reading still serialize through one lock.

**Solution:** `RwLock<T>` (Read-Write Lock) offers two acquisition methods instead of one:

- `.read()` → `RwLockReadGuard<T>`, derefs to `&T`. Multiple readers can hold this simultaneously.
- `.write()` → `RwLockWriteGuard<T>`, derefs to `&mut T`. Exclusive — blocks all readers and writers until released.

Both block the calling thread until the requested access is available, and both return a `Result` (poisoning works the same way as `Mutex`: a panic while holding either guard poisons the lock). Both guards release automatically on drop (RAII, same as `MutexGuard`).

Key properties:

- Many concurrent readers, or one exclusive writer — never both at once.
- `RwLock<T>` alone does not give shared ownership across threads; pair with `Arc<T>` for that, exactly like `Mutex<T>`: `Arc<RwLock<T>>`.
- Higher overhead than `Mutex<T>` per operation, and platform-dependent writer-starvation behavior (some implementations let readers indefinitely delay a waiting writer). Only worth it when reads clearly dominate writes. If access is roughly balanced or write-heavy, `Mutex<T>` is simpler and often faster.

### Simple — basic read/write

```rust
use std::sync::RwLock;

fn main() {
    let lock = RwLock::new(5);

    {
        let r1 = lock.read().unwrap();
        let r2 = lock.read().unwrap(); // multiple readers, fine
        println!("{} {}", *r1, *r2);
    } // both guards dropped, lock free

    {
        let mut w = lock.write().unwrap(); // exclusive
        *w += 1;
    }

    println!("{}", *lock.read().unwrap()); // 6
}
```

### Intermediate — concurrent readers, occasional writer

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    for _ in 0..5 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let d = data.read().unwrap();
            println!("{:?}", *d); // many of these can run concurrently
        }));
    }

    let writer_data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        let mut d = writer_data.write().unwrap(); // waits for readers to finish
        d.push(4);
    }));

    for h in handles {
        h.join().unwrap();
    }
}
```

**Memorization anchor:** `RwLock::new(x)` — `.read()` for shared `&T` (many at once), `.write()` for exclusive `&mut T` (one at a time). Same poisoning and RAII-unlock behavior as `Mutex`. Pair with `Arc` for cross-thread sharing: `Arc<RwLock<T>>`. Use when reads dominate writes.

## Summary

| Pointer      | Owners        | Thread-safe | Mutability                             |
| ------------ | ------------- | ----------- | -------------------------------------- |
| `Box<T>`     | Single        | N/A         | Direct                                 |
| `Rc<T>`      | Multiple      | No          | Read-only (pair with `RefCell`)        |
| `RefCell<T>` | Single        | No          | Interior, checked at runtime           |
| `Cell<T>`    | Single        | No          | Interior, by get/set                   |
| `Arc<T>`     | Multiple      | Yes         | Read-only (pair with `Mutex`/`RwLock`) |
| `Mutex<T>`   | N/A (wrapper) | Yes         | Interior, via lock                     |
| `RwLock<T>`  | N/A (wrapper) | Yes         | Interior, read/write lock              |
