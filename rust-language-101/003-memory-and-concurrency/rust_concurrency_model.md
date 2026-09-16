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

## Phase 2: Thread Lifecycle & Structured Concurrency

*Mastering native OS threads and eliminating lifetime mismatches.*

In Phase 1, we analyzed the hardware memory hierarchy and established how `Send` and `Sync` allow the compiler to police thread boundaries.  
Now we move from type theory into execution:  

- how the operating system provisions threads,
- how Rust wraps these OS primitives, why the `'static` lifetime constraint exists, and
- how structured concurrency eliminates that constraint

### Module 2.1: Native OS Threads (`std::thread`)

Rust's standard library implements a **1:1 threading model**.  
Every time you call `std::thread::spawn`, the runtime makes a direct syscall to the host operating system:

- **Linux:** `clone(2)` with `CLONE_VM`, `CLONE_FS`, `CLONE_FILES`, `CLONE_SIGHAND`, `CLONE_THREAD`.
- **macOS / BSD:** `pthread_create(3)`.
- **Windows:** `CreateThread` (via the Win32 API).

There is no hidden virtual runtime, no green-thread scheduler, and no garbage collection intercepting execution.  
Rust threads are operating system threads, scheduled directly by the OS kernel's scheduler (such as CFS/EEVDF on modern Linux).

```text
+-------------------------------------------------------------+
|                   Your Rust Binary   nnn                    |
|   thread::spawn(f)                  thread::spawn(g)        |
+----------+----------------------------------+---------------+
           | (sys_clone / pthread_create)     |
           v                                  v
+-------------------------------------------------------------+
|                      OS Kernel Space                        |
|   [ Native OS Thread A ]            [ Native OS Thread B ]  |
|   - Dedicated Kernel Stack          - Dedicated Kernel Stack|
|   - Thread Control Block (TCB)      - Thread Control Block  |
+----------+----------------------------------+---------------+
           | Scheduled preemptively           |
           v                                  v
+-------------------------------------------------------------+
|                            CPU Hardware                     |
|                 Core 0                      Core 1          |
+-------------------------------------------------------------+
```

#### 1. Anatomy of `thread::spawn` and the `'static` Boundary

The signature of `std::thread::spawn` is one of the most informative in the standard library:

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

Every single bound serves an explicit mechanical purpose:

- `F: FnOnce() -> T`: The entry-point closure is executed once.  
  It can return a value of type `T`.
- `F: Send`: The closure environment itself (all variables captured inside it) must be safe to physically transfer across thread boundaries.  
  If you capture a type that is `!Send` (like `Rc<T>`), this bound trips.
- `T: Send`: The return value will eventually be passed back to whichever thread joins it.  
  Therefore, `T` must also be thread-transferable.
- `F: 'static` & `T: 'static`: **The fundamental barrier.** The type system cannot know when the OS scheduler will terminate the spawned thread.  
  The parent thread could finish, pop its entire stack frame off memory, and exit within $10\mu\text{s}$, while the spawned thread remains running for 10 minutes.  
  Therefore, the closure cannot borrow **any** data with a temporary lifetime from the parent thread's stack.  
  Any reference captured must live for the entire duration of the program (`'static`).

#### The Compiler Error You Will Inevitably Encounter

Consider this naive code:

```rust
fn process_locally() {
    let numbers = vec![1, 2, 3, 4, 5];

    // Attempting to borrow `numbers` into the child thread
    let handle = std::thread::spawn(|| {
        println!("Length: {}", numbers.len());
    });

    handle.join().unwrap();
}
```

The compiler rejects this immediately:

```text
error[E0373]: closure may outlive the current function, but it borrows `numbers`,
              which is owned by the current function
 --> src/main.rs:5:37
  |
5 |     let handle = std::thread::spawn(|| {
  |                                     ^^ may outlive borrowed value `numbers`
6 |         println!("Length: {}", numbers.len());
  |                                ------- `numbers` is borrowed here
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `numbers` (and any other referenced variables),
      use the `move` keyword
  |
5 |     let handle = std::thread::spawn(move || {
  |                                     ++++
```

Even though we called `.join()` on the very next line, the compiler evaluates safety **at the function boundary based strictly on types**, not through whole-program control-flow analysis.  
`thread::spawn` takes an `F: 'static`.  
A borrow `&numbers` has a local lifetime tied to `process_locally`'s stack frame.  
Because `'local` does not satisfy `'static`, it fails.

Adding `move` solves this by transferring full ownership of the entire `Vec` onto the new thread's stack:

```rust
let handle = std::thread::spawn(move || {
    println!("Length: {}", numbers.len());
});
```

Now, `numbers` is owned by the closure.  
If the parent thread returns early, the vector lives safely inside the child thread's independent memory space and is deallocated when the child thread finishes.

#### 2. Advanced Thread Configuration: `std::thread::Builder`

`std::thread::spawn` is actually a convenience wrapper around `std::thread::Builder`.  
Production systems frequently need explicit control over OS thread metadata:

```rust
use std::thread;

fn spawn_configured_worker(id: usize) -> thread::JoinHandle<()> {
    thread::Builder::new()
        // 1. Give the thread an explicit name for debugging/profiling (htop, gdb, perf)
        .name(format!("worker-node-{}", id))
        // 2. Control stack size allocation (default is usually 2MB on Linux)
        // Useful for deep recursion or, conversely, running thousands of low-memory threads
        .stack_size(4 * 1024 * 1024) // 4 MiB
        .spawn(move || {
            let current = thread::current();
            println!(
                "Running in thread {:?} (ID: {:?})", 
                current.name().unwrap_or("unnamed"), 
                current.id()
            );
        })
        .expect("Failed to allocate OS thread: system limits reached")
}
```

- **Thread Names:** Displayed in system profilers (`perf`), debugging tools (`gdb`, `lldb`), and OS task monitors (`htop`).  
  Unnamed threads appear generically as the executable name.
- **Stack Size:** Default OS stack sizes differ across platforms (e.g., Windows typically defaults to 1MB, macOS to 512KB, Linux musl to 128KB, Linux glibc to 2MB).  
  Explicitly setting `stack_size` guarantees uniform stack depth across platforms.
- **Failure Modes:** `thread::spawn` panics if the kernel refuses to create a thread (e.g., `EAGAIN` due to hitting `ulimit -u`).  
  `Builder::spawn` returns a `Result<JoinHandle<T>, std::io::Error>`, allowing graceful recovery or backpressure when the system is starved of OS handles.

#### 3. `JoinHandle<T>`, Synchronization, and Panic Propagation

When you spawn a thread, you receive a `JoinHandle<T>`:

```rust
pub struct JoinHandle<T>(...);
```

Calling `.join()` does two things:

1. **Blocks execution** of the calling thread until the target OS thread exits execution.
2. **Returns a `thread::Result<T>**`, which is a type alias for `Result<T, 'static + Any Box<dyn Send>>`.

```rust
use std::thread;

fn compute_parallel() {
    let handle = thread::spawn(|| {
        // Computational work
        42 * 2
    });

    match handle.join() {
        Ok(result) => println!("Computation succeeded: {result}"),
        Err(err_payload) => eprintln!("Thread failed with panic!"),
    }
}
```

#### The Panic Boundary

In Rust, panics do not automatically tear down the entire process (unless compiled with `panic = "abort"` in `Cargo.toml`).  
By default, unwinding stops at the thread boundary.  

If a spawned thread panics:

- The stack of the **spawned thread** is unwound.
- Destructors (`Drop` implementations) of local variables in that thread are executed.
- The error payload is captured into an `Err(Box<dyn Any + Send>)`.
- The **parent thread remains completely unaffected** until it calls `join()`.

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("Child running...");
        panic!("Fatal internal error in worker!");
    });

    println!("Parent thread is still executing unaffected.");

    match handle.join() {
        Ok(_) => println!("Worker completed normally."),
        Err(panic_box) => {
            // Downcast the dynamic error box to extract the panic string
            if let Some(msg) = panic_box.downcast_ref::<&'static str>() {
                println!("Caught child panic string: '{msg}'");
            } else if let Some(msg) = panic_box.downcast_ref::<String>() {
                println!("Caught child panic String: '{msg}'");
            } else {
                println!("Caught unknown panic type.");
            }
        }
    }

    println!("Parent cleanly handled the worker failure and continues running.");
}
```

This isolation makes it possible to build resilient supervision patterns: supervisor threads can detect failed workers via `join()`, clear resources, and spawn replacements.  

### Module 2.2: Structured Concurrency (`std::thread::scope`)

Before Rust 1.63, the `'static` constraint of `thread::spawn` forced developers into an inefficient pattern whenever threads needed to inspect the same data:

```
Stack Variable [data] ---> Boxed into Arc [Arc::new(data)] ---> Arc Clone across Threads
(Heap Allocation + Cache line invalidation on reference count atomic ops)
```

In August 2022, Rust stabilized `std::thread::scope` (derived from the pioneering work in the `crossbeam` crate).  
It introduced **Structured Concurrency** natively to the standard library.

#### 1. The Core Philosophy of Structured Concurrency

In unstructured concurrency (`thread::spawn`), a spawned thread has an arbitrary lifetime.  
It is a "fire-and-forget" resource whose lifetime escapes the lexical scope where it was created:

```text
Function Entry
  │
  ├─ thread::spawn ──────────────> (escapes anywhere, outlives function)
  │
Function Return (Stack Destroyed)
```

In structured concurrency, thread lifetimes mirror lexical blocks in your code:

```text
Function Entry
  │
  ┌── thread::scope starts ──────────┐
  │   ├─ s.spawn (Thread A) ───┐     │
  │   ├─ s.spawn (Thread B) ───┤     │
  │   │                        v     │
  │   └─ Implicit Join Barrier ──────┤ (Execution CANNOT pass this line
  │                                  │  until all child threads exit)
  └── thread::scope ends ────────────┘
  │
Function Return (Stack Destroyed)
```

Because execution cannot exit the closure of `thread::scope` while any spawned thread is still running, **the stack variables of the caller are guaranteed to remain valid in memory for every child thread**.

#### 2. The Mechanics of `thread::scope`

Let's inspect the signature:

```rust
pub fn scope<'env, F, R>(f: F) -> R
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> R,
```

Notice the lifetime relationship:

- `'env`: The lifetime of the environment outside the scope (your current stack frame).
- `'scope`: The lifetime of the execution block itself.
- The bound guarantees `'env: 'scope` (the outer stack outlives the scoped threads).

Because of this mathematical guarantee, child threads spawned via `s.spawn` can borrow stack variables (`&'env T`) directly, with zero reference counting, zero locking, and zero heap allocation:

```rust
use std::thread;

fn analyze_dataset() {
    let large_buffer = vec![100; 1_000_000]; // 1 million elements on the stack/heap
    let config_label = String::from("PRODUCTION_RUN");

    thread::scope(|s| {
        // Thread 1 borrows large_buffer AND config_label immutably
        s.spawn(|| {
            let sum: usize = large_buffer[0..500_000].iter().sum();
            println!("[{config_label}] Chunk 1 Sum: {sum}");
        });

        // Thread 2 borrows the exact same references immutably
        s.spawn(|| {
            let sum: usize = large_buffer[500_000..].iter().sum();
            println!("[{config_label}] Chunk 2 Sum: {sum}");
        });

        // Scope closure ends here. 
        // Rust physically blocks the parent thread, calling .join() on both threads!
    });

    // We can immediately use `large_buffer` and `config_label` here.
    // They were never moved or consumed!
    println!("Analysis complete for: {config_label}, total items: {}", large_buffer.len());
}
```

#### 3. Concurrent Mutation Without Locks: Slices and `split_at_mut`

A frequent misconception among developers new to Rust is:

> *"If two threads are writing to data, I must use an `Arc<Mutex<T>>`."*

Locks are only required when **multiple threads access the exact same memory address** simultaneously.  
If you have an array or vector, you can partition it into disjoint, non-overlapping segments.

The borrow checker normally enforces that only a single mutable reference `&mut T` exists at a time.  
However, the standard library provides methods like `split_at_mut` and `chunks_mut` which use internal `unsafe` code (encapsulated within sound APIs) to split one large slice into independent mutable sub-slices:

```text
Array in Memory: [ 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 ]
                 |_______________|_______________|
                   Slice A (Thread 0)   Slice B (Thread 1)
```

Because the memory addresses of Slice A and Slice B are completely disjoint:

- No cache line updates step on the same word.
- No data races can occur.
- Both threads have exclusive, uncontested write access.

Here is an example computing the square of every number in a slice across 4 threads simultaneously:

```rust
use std::thread;

fn parallel_square(slice: &mut [u64], chunk_size: usize) {
    thread::scope(|s| {
        // chunks_mut yields non-overlapping mutable slices: &mut [u64]
        for chunk in slice.chunks_mut(chunk_size) {
            s.spawn(move || {
                for item in chunk {
                    *item = *item * *item;
                }
            });
        }
    });
}

fn main() {
    let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Split into chunks of 3 items per thread
    parallel_square(&mut data, 3);

    println!("Squared Data: {:?}", data);
    assert_eq!(data, vec![1, 4, 9, 16, 25, 36, 49, 64, 81, 100]);
}
```

Notice the performance characteristics:

1. **Zero Locks:** No thread contention, no sleep states, no context-switching due to locked mutexes.
2. **Zero Allocation:** The vector is modified directly in-place.
3. **Deterministic Completion:** When `parallel_square` returns, the work is 100% finished.

### Comprehensive Phase 2 Milestone Project: Parallel Chunked File Hasher

To cement these concepts, we will build a real-world CLI tool: an optimized **parallel chunked file checksum generator**.

#### System Design:

1. Generate an in-memory buffer simulating a massive file (e.g., 64 MiB).
2. Determine CPU core count dynamically via `std::thread::available_parallelism`.
3. Partition the memory buffer into equal-sized chunks across the available cores.
4. Use `std::thread::scope` to spawn worker threads that borrow their respective slice chunk immutably.
5. Compute a 64-bit non-cryptographic checksum (FNV-1a) on each slice independently.
6. Return `ScopedJoinHandle`s to collect results, then aggregate the chunk hashes into a final master checksum.

```text
Buffer: [==============================================================]
Partition: [ Chunk 0 ]    [ Chunk 1 ]    [ Chunk 2 ]    [ Chunk 3 ]
              │              │              │              │
            Thread 0       Thread 1       Thread 2       Thread 3
              │              │              │              │
            Hash (H0)      Hash (H1)      Hash (H2)      Hash (H3)
              │              │              │              │
              └──────────────┴───────┬──────┴──────────────┘
                                     v
                           Final Combined Hash
```

#### Production Implementation:

```rust
use std::thread;
use std::time::Instant;

/// Fast, deterministic 64-bit FNV-1a hash algorithm
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Metadata holding individual thread results
#[derive(Debug)]
struct ChunkReport {
    worker_id: usize,
    byte_range: (usize, usize),
    chunk_hash: u64,
}

fn parallel_file_hasher(data: &[u8]) -> (u64, Vec<ChunkReport>) {
    // 1. Detect physical CPU execution units
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let total_len = data.len();
    let chunk_size = (total_len + num_threads - 1) / num_threads; // Ceiling division

    println!("------------------------------------------------------------");
    println!("Starting parallel hash: Total size = {} bytes", total_len);
    println!("Utilizing {} threads with ~{} bytes/chunk", num_threads, chunk_size);
    println!("------------------------------------------------------------");

    let mut reports = Vec::with_capacity(num_threads);

    // 2. Structured concurrency scope: All threads join before scope exit
    thread::scope(|s| {
        let mut handles = Vec::with_capacity(num_threads);

        // Partition data and spawn workers
        for (worker_id, (start_idx, chunk)) in data
            .chunks(chunk_size)
            .enumerate()
            .map(|(id, c)| (id, (id * chunk_size, c)))
        {
            let end_idx = start_idx + chunk.len();

            // Spawn inside scope: borrows `chunk` immutably with non-'static lifetime
            let handle = s.spawn(move || {
                let start_time = Instant::now();
                let hash = fnv1a_hash(chunk);
                
                println!(
                    " > [Thread {:?}] Processed bytes [{}..{}] (took {:?})",
                    thread::current().id(),
                    start_idx,
                    end_idx,
                    start_time.elapsed()
                );

                ChunkReport {
                    worker_id,
                    byte_range: (start_idx, end_idx),
                    chunk_hash: hash,
                }
            });

            handles.push(handle);
        }

        // 3. Collect and join handles inside the scope
        for handle in handles {
            match handle.join() {
                Ok(report) => reports.push(report),
                Err(e) => eprintln!("Worker thread panicked during hashing: {:?}", e),
            }
        }
    });

    // 4. Sort reports deterministically by worker_id to guarantee repeatable hashes
    reports.sort_by_key(|r| r.worker_id);

    // 5. Combine the individual chunk hashes into one aggregate root hash
    let mut final_combined_hash = FNV_OFFSET_BASIS;
    for report in &reports {
        final_combined_hash ^= report.chunk_hash;
        final_combined_hash = final_combined_hash.wrapping_mul(FNV_PRIME);
    }

    (final_combined_hash, reports)
}

fn main() {
    // Generate an in-memory 64 MiB binary payload to simulate a large file
    println!("Allocating 64 MiB payload in memory...");
    let payload_size = 64 * 1024 * 1024; // 67,108,864 bytes
    let file_data = vec![0xABu8; payload_size];

    let timer = Instant::now();
    let (master_hash, reports) = parallel_file_hasher(&file_data);
    let elapsed = timer.elapsed();

    println!("\n================ Execution Summary ================");
    for report in reports {
        println!(
            "Worker #{}: Bytes [{}..{}] -> Hash: {:#018x}",
            report.worker_id,
            report.byte_range.0,
            report.byte_range.1,
            report.chunk_hash
        );
    }
    println!("===================================================");
    println!("Aggregate Final Hash: {:#018x}", master_hash);
    println!("Total Execution Time: {:?}", elapsed);
    println!("Throughput: {:.2} MB/s", (payload_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
}
```

### Architectural Review of Phase 2

| Feature               | `thread::spawn`                                                         | `thread::scope`                                                  |
| --------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------- |
| **Lifetime Boundary** | `'static` (Must outlive the whole application run)                      | Local `'scope` (Bound to the lexical closure)                    |
| **Data Access**       | Must move ownership (`move`) or rely on heap pointers (`Arc`)           | Can directly borrow references (`&T` or non-overlapping `&mut T`)|
| **Safety Invariant**  | Caller can exit before child threads exit                               | Compiler guarantees all child threads exit before scope returns  |
| **Cleanup Trigger**   | Explicit: programmer must track each `JoinHandle` and call `.join()`    | Automatic: end of block acts as an implicit join barrier         |
| **Memory Overhead**   | Requires `Arc` for sharing, causing reference count increment contention| **Zero allocation overhead** for shared stack variables          |

[Go to the Top](#table-of-content)

---

## Phase 3: Message-Passing Architecture (`std::sync::mpsc`)

Message passing treats threads as independent isolation zones that coordinate strictly by sending discrete packets of data.  
Instead of sharing a pointer and managing lock contention, threads transfer ownership of memory across thread boundaries.  

### Module 3.1: Channel Mechanics & Internal Queue Topologies

To write reliable channel code, you need to know how the runtime moves bytes between threads without deadlocking or exhausting RAM.

#### 1. Unbounded Channel Internals (`mpsc::channel`)

An unbounded channel is backed by an intrusive, dynamically allocated linked list of memory blocks (often linked lists of small arrays to preserve cache locality).

```text
Producer Threads                    Channel State in Heap                      Consumer Thread
[ Thread A: tx ] ──┐                                                      
                   ├── atomic CAS ──> [ Head Node ] ──> [ Node ] ──> [ Tail ] ──> [ Thread C: rx ]
[ Thread B: tx2 ] ─┘                 (Enqueues items)               (Dequeues items)
```

- **Enqueue (`tx.send`)**:  
  Pushes a node onto the tail of the concurrent queue using atomic Compare-And-Swap (CAS) instructions.  
  This operation is **strictly lock-free and non-blocking**.  
  It never yields the CPU, meaning the sending thread can continue immediately.
- **Dequeue (`rx.recv`)**:  
  If the queue holds data, `recv()` updates the head pointer and returns `Ok(T)`.  
  If the queue is empty, the consumer issues an OS-level thread-parking call (e.g., `futex` on Linux) to sleep until a producer enqueues an item and wakes it up.
- **Failure Mode (OOM)**:  
  Because `tx.send` never blocks, an unthrottled producer enqueuing data faster than the consumer can process it will cause heap memory to expand indefinitely until the OS issues an Out-Of-Memory kill signal.

#### 2. Bounded Channel Internals (`mpsc::sync_channel`)

A bounded channel is fundamentally different: it is backed by a **fixed-size ring buffer array** guarded by atomic sequence counters and synchronization flags.

```text
       [Slot 0] ──> [Slot 1] ──> [Slot 2 (Full)] ──> [Slot 3 (Full)]
          ^                                               ^
          │ (Read Pointer: rx)                            │ (Write Pointer: tx)
```

- When `write_pointer == read_pointer + capacity`, the ring buffer is full.
- Any thread calling `tx.send()` registers itself in a waiting queue and puts itself to sleep (parks).
- As soon as the consumer thread calls `rx.recv()`, it frees a slot in the ring buffer, increments the read pointer, and unparks one waiting sender thread.
- **Rendezvous Channels (`sync_channel(0)`)**:  
  A buffer size of zero bypasses intermediate queue storage entirely.  
  A producer cannot complete `tx.send()` until a consumer is actively executing `rx.recv()` at that exact moment.  
  Data is copied directly from the producer's stack/registers into the consumer's variable.

### Module 3.2: Lifecycle, Panics, and Topologies

Channels in Rust follow deterministic RAII patterns. Disconnections, cancellations, and shutdowns are communicated entirely through enum returns rather than exceptions.

#### 1. The Disconnection Invariant

A channel connection is governed by atomic reference counting inside the channel's shared state:

```text
Sender Count:   AtomicUsize (tracks total live `Sender` handles)
Receiver Count: AtomicUsize (tracks live `Receiver` handles: max 1 in std::mpsc)
```

| Method Called   | Channel State                            | Result Returned                                   |
| --------------- | ---------------------------------------- | ------------------------------------------------- |
| `tx.send(val)`  | Receiver is dead (`Receiver Count == 0`) | `Err(SendError(val))` (returns the un-sent item!) |
| `rx.recv()`     | Buffer empty AND `Sender Count > 0`      | Blocks calling thread                             |
| `rx.recv()`     | Buffer empty AND `Sender Count == 0`     | `Err(RecvError)` (stream has cleanly terminated)  |
| `rx.try_recv()` | Buffer empty AND `Sender Count > 0`      | `Err(TryRecvError::Empty)`                        |
| `rx.try_recv()` | Buffer empty AND `Sender Count == 0`     | `Err(TryRecvError::Disconnected)`                 |

> **Critical Safety Feature:** When `tx.send(val)` fails because the consumer died, it returns `SendError(val)`.  
> It **returns ownership of the item back to the caller**, preventing data loss if you want to retry or serialize the message to disk.

#### 2. Channel Topologies

```text
1. Many-to-One (Fan-In):             2. Request-Response (Actor Pattern):
   [ Worker 1: tx ] ──┐                 [ Client Thread ]
   [ Worker 2: tx ] ──┼──> [ rx ]         │  ▲  tx.send(Req { reply_tx, .. })
   [ Worker 3: tx ] ──┘                   ▼  │  rx.recv()
                                        [ Server Thread ]
```

#### The Request-Response (Bidirectional) Pattern

Because `std::mpsc::Receiver` cannot be cloned, two-way communication cannot share a single channel.  
Instead, the producer creates an ephemeral, one-shot response channel and packs its `Sender` directly into the payload struct:

```rust
use std::sync::mpsc;
use std::thread;

// 1. The Request payload includes a return channel
struct WorkOrder {
    data: String,
    // Ephemeral one-shot sender for the answer
    reply_channel: mpsc::Sender<Result<usize, &'static str>>,
}

fn main() {
    let (order_tx, order_rx) = mpsc::channel::<WorkOrder>();

    // Dedicated backend worker
    thread::spawn(move || {
        while let Ok(order) = order_rx.recv() {
            let processed_len = order.data.len();
            // Send the response directly back to the specific caller
            let _ = order.reply_channel.send(Ok(processed_len));
        }
    });

    // Client thread submits job and waits for the specific response
    let (reply_tx, reply_rx) = mpsc::channel();
    order_tx
        .send(WorkOrder {
            data: String::from("Compute my byte length"),
            reply_channel: reply_tx,
        })
        .unwrap();

    // Block until our specific reply arrives
    let length = reply_rx.recv().unwrap().unwrap();
    println!("Backend worker responded with length: {length}");
}
```

### Module 3.3: Ecosystem Standard: `crossbeam-channel`

While `std::sync::mpsc` is reliable for basic tasks, real-world systems hit its limitations quickly:

1. `std::mpsc` is **Single-Consumer only**. You cannot have multiple worker threads competing for tasks off the same queue without wrapping the receiver in an `Arc<Mutex<Receiver<T>>>` (which defeats the purpose of channels).
2. `std::mpsc` lacks the ability to **select** over multiple channels simultaneously (listening for a message OR a cancellation signal OR a timeout).

The `crossbeam-channel` crate is the de facto standard replacement.

#### Key Enhancements of `crossbeam-channel`:

- **MPMC (Multi-Producer, Multi-Consumer):**  
  Both `Sender` and `Receiver` implement `Clone` and `Send`.  
  Multiple threads can pull concurrently from the same channel buffer with zero mutexes.
- **Cache-Padded Ring Buffers:**  
  Crossbeam’s internals use hardware cache-line padding (`CachePadded<T>`) to prevent false sharing between reader and writer atomic indices.
- **The `select!` Macro:**  
  Allows a thread to block on multiple channels at the same time, processing whichever event completes first.

#### The `select!` Event Loop Pattern

```rust
// Requires: crossbeam-channel = "0.5"
use crossbeam_channel::{select, tick, unbounded};
use std::time::Duration;

fn main() {
    let (data_tx, data_rx) = unbounded();
    let (stop_tx, stop_rx) = unbounded();
    
    // Ticks every 500ms
    let ticker = tick(Duration::from_millis(500));

    // Worker Event Loop
    loop {
        select! {
            recv(data_rx) -> msg => {
                match msg {
                    Ok(val) => println!("Received data: {val}"),
                    Err(_) => break, // All senders disconnected
                }
            }
            recv(ticker) -> _ => {
                println!("Heartbeat: system healthy...");
            }
            recv(stop_rx) -> _ => {
                println!("Shutdown signal received. Cleaning up...");
                break;
            }
        }
    }
}
```

### Phase 3 Milestone: Multi-Threaded Task Dispatcher with Poison-Pill Shutdown

To demonstrate production message passing, we will build a multi-threaded task runner.

**Design Requirements:**

- A central dispatcher sends `Task` variants to a pool of persistent worker threads.
- Multiple workers pull from a single, shared work queue.
- The system utilizes the **Poison Pill Pattern**: sending a terminal signal through the channel to trigger a graceful worker shutdown, verifying that zero jobs are dropped mid-flight.

```text
Dispatcher (Main)
  │
  ├── Tasks Enqueued ──> [ Shared Channel Buffer ]
  │                              │      │      │
  │                              ▼      ▼      ▼
  │                          Worker 0 Worker 1 Worker 2
  │                             │       │       │
  └── Sends Poison Pills ───────┴───────┴───────┘
```

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Types of messages our workers can process
enum Message {
    NewTask(Task),
    Shutdown, // The "Poison Pill"
}

struct Task {
    id: usize,
    work_units: u64,
}

/// A wrapper around mpsc::Receiver to share it across multiple threads (MPMC emulation)
#[derive(Clone)]
struct SharedReceiver<T> {
    inner: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T> SharedReceiver<T> {
    fn new(rx: mpsc::Receiver<T>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(rx)),
        }
    }

    fn recv(&self) -> Result<T, mpsc::RecvError> {
        let guard = self.inner.lock().unwrap();
        guard.recv()
    }
}

fn main() {
    const WORKER_COUNT: usize = 3;
    const TOTAL_TASKS: usize = 9;

    let (tx, rx) = mpsc::channel::<Message>();
    let shared_rx = SharedReceiver::new(rx);

    let mut workers = Vec::with_capacity(WORKER_COUNT);

    // 1. Spawn Worker Pool
    for worker_id in 0..WORKER_COUNT {
        let rx_handle = shared_rx.clone();

        let handle = thread::spawn(move || {
            println!("[Worker {worker_id}] Online and waiting for jobs.");
            
            loop {
                // Each worker pulls tasks from the single shared queue
                match rx_handle.recv() {
                    Ok(Message::NewTask(task)) => {
                        println!(
                            "[Worker {worker_id}] Executing Task #{} (weight: {})",
                            task.id, task.work_units
                        );
                        // Simulate work
                        thread::sleep(Duration::from_millis(task.work_units * 40));
                    }
                    Ok(Message::Shutdown) => {
                        println!("[Worker {worker_id}] Poison pill received. Shutting down.");
                        break;
                    }
                    Err(_) => {
                        // All senders were dropped unexpectedly
                        eprintln!("[Worker {worker_id}] Channel hung up unexpectedly!");
                        break;
                    }
                }
            }
        });

        workers.push(handle);
    }

    // 2. Dispatch Work Items
    for i in 1..=TOTAL_TASKS {
        let task = Task {
            id: i,
            work_units: (i % 4) + 1,
        };
        tx.send(Message::NewTask(task)).unwrap();
    }

    println!("\n>>> Dispatcher: All {TOTAL_TASKS} tasks enqueued. Initiating graceful shutdown... <<<\n");

    // 3. Send one Poison Pill per worker
    for _ in 0..WORKER_COUNT {
        tx.send(Message::Shutdown).unwrap();
    }

    // 4. Drop the main sender so the channel can close
    drop(tx);

    // 5. Join all workers to guarantee all tasks completed
    for (id, handle) in workers.into_iter().enumerate() {
        handle.join().unwrap();
        println!("Worker thread {id} successfully joined.");
    }

    println!("\nSystem cleanly terminated: All tasks drained, zero leaks.");
}
```

| Feature           | `mpsc::channel`        | `mpsc::sync_channel`   | `crossbeam_channel`  |
| ----------------- | ---------------------- | ---------------------- | -------------------- |
| **Topology**      | MPSC                   | MPSC                   | **MPMC**             |
| **Buffer Type**   | Dynamic unbounded list | Fixed-size ring buffer | Bounded or Unbounded |
| **Backpressure**  | No (Can OOM)           | Yes (Blocks sender)    | Yes (Blocks sender)  |
| **Multiplexing**  | None                   | None                   | `select!` macro      |
| **Zero-Capacity** | No                     | Yes (Rendezvous)       | Yes (Rendezvous)     |

[Go to the Top](#table-of-content)

---

