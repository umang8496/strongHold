<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->

# Comprehensive Guide for Multi-Threading in Rust

## Table of Content

- [Phase 1: Hardware, OS & Rust’s Safety Foundations](#phase-1-hardware-os--rusts-safety-foundations)
- [Phase 2: Thread Lifecycle & Structured Concurrency](#phase-2-thread-lifecycle--structured-concurrency)
- [Phase 3: Message-Passing Architecture (`std::sync::mpsc`)](#phase-3-message-passing-architecture-stdsyncmpsc)
- [Phase 4: Shared-State Concurrency & Locks](#phase-4-shared-state-concurrency--locks)
- [Phase 5: Advanced Coordination Primitives](#phase-5-advanced-coordination-primitives)
- [Phase 6: Low-Level Atomics & Memory Ordering](#phase-6-low-level-atomics--memory-ordering)
- [Phase 7: The Bridge: OS Threads vs. Asynchronous Concurrency](#phase-7-the-bridge-os-threads-vs-asynchronous-concurrency)

---

## Phase 1: Hardware, OS & Rust’s Safety Foundations

*Build an intuition for what threads actually are at the hardware/OS boundary and why Rust’s type system can prevent concurrency bugs before code ever runs.*  

To understand why Rust treats concurrency the way it does, we have to start with the silicon.  
Concurrency bugs are not language quirks;  
They are the direct by-product of modern CPU hardware architecture and aggressive compiler optimizations.  

### Part 1: The Hardware Reality

#### 1. Why "Just Write to Memory" Doesn't Work

CPUs are blindingly fast compared to system RAM.  
Accessing a CPU register takes a fraction of a nanosecond, while a trip to main memory takes roughly 50 to 100 nanoseconds.  
To hide this latency, CPU designers introduced multi-tiered caches (L1, L2, L3) and speculative instruction pipelines.  

- **L1 and L2 Caches are Local:** Each physical CPU core has its own private L1 and L2 caches. L3 is typically shared across cores.
- **Store Buffers:** When a CPU core executes a write instruction, it doesn't write to RAM or even immediately to L3.  
  It writes into a local hardware queue called a **store buffer** and moves on to the next instruction before the write finishes propagating.

```text
+------------------------+      +------------------------+
|        Core 0          |      |        Core 1          |
|------------------------|      |------------------------|
|  [Registers]           |      |  [Registers]           |
|  [Store Buffer]        |      |  [Store Buffer]        |
|  [L1 Cache]            |      |  [L1 Cache]            |
|  [L2 Cache]            |      |  [L2 Cache]            |
+-----------+------------+      +-----------+------------+
            |                               |
            +---------------+---------------+
                            |
                    [Shared L3 Cache]
                            |
                    [Main Memory / RAM]
```

#### 2. Instruction Reordering: Two Layers of Deception

Your source code is rarely executed in the exact order you wrote it.

1. **Compiler Reordering:**  
The compiler analyzes your code under the *as-if rule* (the program must behave as intended within a single thread).  
If swapping line 1 and line 2 produces faster assembly without changing single-threaded output, the compiler reorders them.  

2. **CPU Out-of-Order Execution:**  
Even if the compiler emits instructions in order, modern superscalar CPUs execute instructions speculatively out of order based on available execution units and cache hits.

**Why this breaks multi-threading:**
Core 0 might write `data = 42` and then `ready = true`.  
Due to local store buffers or CPU out-of-order execution, Core 1 might see `ready == true` *before* the update to `data` has flushed out of Core 0's cache!

#### 3. Cache Lines and False Sharing

CPUs do not read or write single bytes to memory. They operate in chunks called **cache lines** (typically 64 bytes).

When Core 0 reads a 4-byte integer, it pulls the surrounding 64 bytes into its L1 cache.

If Core 0 and Core 1 frequently write to *different* variables that happen to sit within the *same* 64-byte boundary,  
the hardware cache-coherence protocol (e.g., MESI) bounces the entire cache line back and forth between the two cores.  
This is called **false sharing**, and it can degrade performance by an order of magnitude without any explicit locking errors.  

#### 4. Data Race vs. Race Condition

These two terms are frequently confused, but the distinction is critical:

| Term               | What It Means                                                                                                                                                                             | Handled By Rust?                                                |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| **Data Race**      | Two threads access the same memory location concurrently, **at least one is a write**, and there is **no synchronization**. Leads to undefined behavior (torn reads, corrupted pointers). | **Eliminated at compile time** by the type system.              |
| **Race Condition** | A high-level logic flaw where the correctness of a program depends on the timing/interleaving of independent events (e.g., check-then-act sequences).                                     | **Not eliminated.** The compiler cannot read programmer intent. |

### Part 2: How Rust Uses `Send` and `Sync`

Rust solves data races not with an expensive runtime check, but at compile time through two fundamental marker traits: `Send` and `Sync`.

They are defined in `std::marker` and have **no methods**:

```rust
pub unsafe auto trait Send {}
pub unsafe auto trait Sync {}
```

Because they are **auto traits**, the compiler automatically implements them for your custom types if all of their internal fields implement them.

#### 1. The Definitions

- **`Send`:** A type `T` is `Send` if it is safe to **transfer ownership** of `T` across a thread boundary.
- **`Sync`:** A type `T` is `Sync` if it is safe to **share references** (`&T`) between multiple threads concurrently.

#### 2. The Golden Rule of Concurrency

The relationship between the two traits can be summed up in a single sentence:

$$\text{A type } T \text{ is } Sync \iff \&T \text{ is } Send$$

If you can safely pass a shared reference (`&T`) to another thread, it means multiple threads can hold `&T` at the same time.  
Therefore, `T` must be thread-safe for concurrent inspection (`Sync`).

#### 3. How the Borrow Checker Integrates with `Send` and `Sync`

Rust's fundamental aliasing rule is:

> **Aliasing XOR Mutability:** You may have any number of immutable references (`&T`), OR exactly one mutable reference (`&mut T`), but never both at the same time.

Notice how this maps directly onto the definition of a data race:

- A data race requires **at least one write** (`&mut T`) occurring alongside other reads or writes (aliasing).
- Because Rust forbids aliased mutable references by default, ordinary single-threaded safety rules *already* make data races illegal!

`Send` and `Sync` simply tell the compiler whether these guarantees hold true when crossing OS thread boundaries.

### Part 3: Types That Break `Send` and `Sync` (Negative Examples)

To understand why these traits matter, examine the types where they are disabled:

#### Case 1: `Rc<T>` (Neither `Send` nor `Sync`)

`Rc<T>` is a reference-counted pointer for single-threaded code. When you call `.clone()`, it increments an internal counter:

```rust
// Internal logic of Rc::clone (simplified)
self.inner().strong_count.set(self.inner().strong_count.get() + 1);
```

This increment uses plain non-atomic instructions (`load`, `add`, `store`). If two threads cloned the same `Rc` concurrently:

1. Both read count = 1.
2. Both increment to 2.
3. Both write back 2.
4. Total count is 2, but 3 handles exist! The memory will be freed prematurely, causing a **use-after-free**.

Therefore, Rust implements:

```rust
impl<T: ?Sized> !Send for Rc<T> {}
impl<T: ?Sized> !Sync for Rc<T> {}
```

Passing an `Rc` into `thread::spawn` fails with a compile-time error: `Rc<...> cannot be sent between threads safely`.

#### Case 2: `RefCell<T>` (`Send`, but NOT `Sync`)

`RefCell<T>` provides interior mutability via runtime borrow checking. It allows mutating data through a shared reference (`&RefCell<T>`).

- Can you move a whole `RefCell` to another thread?  
  **Yes.**  
  If only one thread owns it at a time, it is safe. Thus, `RefCell<T>: Send`.
- Can two threads share a reference `&RefCell<T>`?  
  **No!**  
  Its internal borrow flag is an ordinary integer.  
  Two threads calling `.borrow_mut()` at the same time would create a data race on the flag itself.  
  Thus, `RefCell<T>` is **`!Sync`**.

#### Case 3: `Mutex<T>` (Turns non-`Sync` data into `Sync`)

A `Mutex<T>` wraps arbitrary data `T`.  
It allows multiple threads to access `T` through shared references (`&Mutex<T>`) because its `.lock()` method guarantees exclusive, serialized access.

In Rust's standard library:

```rust
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
```

Notice the condition: as long as `T` can be sent between threads (`Send`), wrapping it in a `Mutex` makes it safe to share across threads (`Sync`).

1. **Hardware Caches & Out-of-Order Execution** mean threads don't read the same memory state in the same order without explicit synchronization.
2. **Data races** cause undefined behavior at the memory level; **race conditions** are application-level logic flaws.
3. **`Send`** = Safe to move to another thread.
4. **`Sync`** = Safe to share via `&T` across threads ($T: Sync \iff \&T: Send$).
5. Rust eliminates data races at compile time because the borrow checker enforces exclusive mutability, and `Send`/`Sync` forbid types with unsynchronized internals from crossing thread boundaries.

[Go to the Top](#table-of-content)

---
