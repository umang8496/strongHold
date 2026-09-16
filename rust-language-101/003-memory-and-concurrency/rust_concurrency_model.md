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

## Phase 4: Shared-State Concurrency & Locks

Message passing coordinates threads through discrete values, but some architectures require sharing access to the exact same memory in place:  
large in-memory caches, database connection pools, shared state graphs, or direct hardware buffers.

Rust approaches shared state by enforcing strict encapsulation:  
> **you cannot access the data without first acquiring the lock, and you cannot forget to release the lock when you are done.**

### Module 4.1: Atomic Reference Counting (`Arc<T>`)

Ordinary pointers (`&T`, `Box<T>`) assume a single static owner.  
While `Rc<T>` enables multiple owners in single-threaded code, its internal reference counts are updated via ordinary arithmetic (`+ 1`), creating data races when invoked concurrently.

`Arc<T>` (**Atomically Reference Counted pointer**) provides thread-safe shared ownership by placing the value on the heap alongside two atomic counters.

```text
       Stack (Thread 1)             Heap Allocation
      +-----------------+         +---------------------------------------+
      |  Arc<T> Clone   | ------> | - strong_count: AtomicUsize (e.g., 2) |
      +-----------------+         | - weak_count:   AtomicUsize (e.g., 1) |
                                  | - data:         T                     |
       Stack (Thread 2)           +---------------------------------------+
      +-----------------+                         ^
      |  Arc<T> Clone   | ------------------------+
      +-----------------+
```

#### 1. Internal Heap Layout

When you allocate `Arc::new(value)`, Rust generates an internal struct on the heap roughly equivalent to:

```rust
// Conceptual layout inside std::sync::Arc
struct ArcInner<T> {
    strong: std::sync::atomic::AtomicUsize,
    weak:   std::sync::atomic::AtomicUsize,
    data:   T,
}
```

- **`strong` counter:**  
  Tracks the number of active `Arc<T>` pointers.  
  Once this hits `0`, the inner `T` is immediately dropped (`drop_in_place`).
- **`weak` counter:**  
  Tracks `Weak<T>` non-owning pointers (used to prevent circular memory leaks).  
  The heap memory allocation (`ArcInner<T>`) remains allocated until `strong == 0` **and** `weak == 0`.

#### 2. Performance Implications

Calling `Arc::clone(&ptr)` does not duplicate the underlying payload `T`; it only performs an atomic fetch-and-add on the `strong` counter:

```rust
self.inner().strong.fetch_add(1, Ordering::Relaxed);
```

While much cheaper than a deep copy, atomic instructions still invalidate hardware cache lines across CPU cores.  
**Rule of thumb:** Do not wrap tiny values in an `Arc` per-thread if a single reference via `std::thread::scope` can do the job without heap allocations or atomic overhead.

### Module 4.2: Mutual Exclusion (`Mutex<T>`) & RAII Guards

In languages like C++, Java, or Go, a mutex is an independent synchronization object sitting beside the data:

```cpp
// Traditional approach (error-prone)
std::mutex mtx;
std::vector<int> shared_data;

mtx.lock();
shared_data.push_back(1); // Nothing stops someone from accessing shared_data without locking!
mtx.unlock();
```

In Rust, **the data lives inside the mutex**. The `Mutex<T>` is a container that provides *interior mutability* across thread boundaries.

```rust
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    data:  UnsafeCell<T>, // Data accessible ONLY when the lock is held
}
```

#### 1. The `MutexGuard<'a, T>` Lifetime Protocol

To read or write the inner data, you must call `.lock()`. This call blocks the thread until the OS mutex is acquired, returning a `MutexGuard<T>`.

```rust
use std::sync::Mutex;

let counter = Mutex::new(0);

{
    // 1. Blocks until the lock is acquired
    let mut guard = counter.lock().unwrap(); 

    // 2. DerefMut allows mutating the inner data directly
    *guard += 1; 

    // 3. RAII Drop: `guard` goes out of scope here!
    // The lock is automatically and deterministically released.
}
```

The guard implements two critical traits:

- `Deref` / `DerefMut`: Transparently forwards reads and writes to the inner `T`.
- `Drop`: Intercepts the end of the guard's lexical scope and invokes the platform-specific OS unlock primitive (`pthread_mutex_unlock` or `ReleaseSRWLockExclusive`).

#### 2. Lock Contention & Guard Scope Leaks

Because releasing the lock is tied to the lifetime of the `MutexGuard`, keeping a guard in scope longer than necessary creates unnecessary lock contention:

```rust
// ANTI-PATTERN: Guard held across a blocking operation
let mut guard = shared_state.lock().unwrap();
guard.update_local_state();
expensive_network_call(); // CRITICAL BUG: Other threads are blocked waiting for this lock!
```

**Correction:** Explicitly limit scope using localized blocks or `drop(guard)`:

```rust
// PATTERN: Minimize critical sections
{
    let mut guard = shared_state.lock().unwrap();
    guard.update_local_state();
} // Guard dropped immediately here

expensive_network_call(); // Free to execute without starving other threads
```

### Module 4.3: Lock Poisoning

What happens if a thread panics while holding a `MutexGuard`?

In C or Go, a thread panicking or crashing while holding a lock either causes a permanent **deadlock** (the lock is never released) or leaves shared memory in an inconsistent, partially updated state.

Rust handles this via **Lock Poisoning**:

1. When a thread panics, the unwinding runtime runs the `Drop` implementation on active stack frames.
2. The `MutexGuard` drop logic marks an internal flag in the mutex: `poisoned = true`.
3. The lock is then released so other threads don't deadlock.
4. Any future call to `.lock()` by surviving threads returns `Err(PoisonError<MutexGuard<T>>)` instead of `Ok(guard)`.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let shared_data = Arc::new(Mutex::new(vec![1, 2, 3]));
let data_clone = Arc::clone(&shared_data);

// Thread 1: Panics mid-write
let _ = thread::spawn(move || {
    let mut guard = data_clone.lock().unwrap();
    guard.push(4);
    panic!("Fatal worker crash while holding the lock!");
}).join();

// Thread 2: Surviving thread inspects the poisoned lock
match shared_data.lock() {
    Ok(guard) => {
        println!("Lock acquired normally: {:?}", *guard);
    }
    Err(poisoned) => {
        eprintln!("WARNING: Mutex was poisoned by a previous panic!");
        
        // You can choose to recover the data anyway:
        let guard = poisoned.into_inner();
        println!("Recovered contaminated data: {:?}", *guard);
    }
}
```

Calling `.unwrap()` on `.lock()` is an intentional assertion: *"If another thread panicked while mutating this state, the invariants are broken, so crash this thread too."*

### Module 4.4: Reader-Writer Locks (`RwLock<T>`)

A `Mutex<T>` is pessimistic: it allows only one thread inside the critical section, even if 100 threads just want to read the data concurrently.  

`std::sync::RwLock<T>` maps directly to the borrow checker's aliasing rules at runtime:

- **Shared Access (`.read()`):** Arbitrary number of concurrent readers permitted simultaneously.
- **Exclusive Access (`.write()`):** Exactly one writer permitted; all readers and other writers are locked out.

```text
       Readers (Thread A, B, C)                 Writer (Thread D)
     +--------------------------+          +-------------------------+
     |   rwlock.read().unwrap() |          | rwlock.write().unwrap() |
     +--------------------------+          +-------------------------+
                  |                                     |
                  v                                     v
     [ Concurrent Shared Access ]          [ Exclusive Mutex Lock ]
```

```rust
use std::sync::RwLock;

let config = RwLock::new(String::from("v1.0.0"));

// Multiple readers can hold read guards at the exact same moment
{
    let r1 = config.read().unwrap();
    let r2 = config.read().unwrap();
    println!("Reader 1: {r1}, Reader 2: {r2}");
} // Both read guards dropped here

// A single writer has exclusive access
{
    let mut w = config.write().unwrap();
    w.push_str("-patch1");
}
```

#### The `RwLock` Tradeoff: Reader vs. Writer Starvation

An `RwLock` is not a free upgrade over a `Mutex`:

- **Reader Starvation / Writer Starvation:**  
  Depending on the OS-level implementation, a continuous stream of incoming read locks can prevent writers from ever acquiring the lock (or vice versa).
- **Instruction Overhead:**  
  Acquiring a read lock requires atomic increment, checking writer status flags, and branch logic.  
  If your critical section is tiny (e.g., updating a single integer), **a standard `Mutex` or an `Atomic` is significantly faster than an `RwLock**` due to lower lock acquisition overhead.

### Phase 4 Milestone: Thread-Safe In-Memory Cache with TTL Expiration

To unify `Arc`, `RwLock`, and fine-grained scoping, we will implement a high-concurrency key-value store where reads are concurrent, writes are serialized, and stale items expire gracefully.

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

pub struct ConcurrentTtlCache<K, V> {
    // Sharded storage protected by a Reader-Writer lock
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
}

impl<K: std::hash::Hash + Eq + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static> ConcurrentTtlCache<K, V> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert or update a key with a Time-To-Live (TTL)
    pub fn set(&self, key: K, value: V, ttl: Duration) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
        };

        // Acquire exclusive write access
        let mut write_guard = self.store.write().unwrap();
        write_guard.insert(key, entry);
    }

    /// Retrieve a value if it exists and has not expired
    pub fn get(&self, key: &K) -> Option<V> {
        // Step 1: Fast path - acquire read access
        let read_guard = self.store.read().unwrap();
        
        if let Some(entry) = read_guard.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Purge expired entries in a single pass
    pub fn cleanup_stale_entries(&self) -> usize {
        let mut write_guard = self.store.write().unwrap();
        let initial_len = write_guard.len();
        let now = Instant::now();

        write_guard.retain(|_, entry| entry.expires_at > now);
        initial_len - write_guard.len()
    }
}

// Clone creates a shallow pointer to the same shared inner cache
impl<K, V> Clone for ConcurrentTtlCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

fn main() {
    let cache = ConcurrentTtlCache::new();
    let mut workers = vec![];

    // Seed data
    cache.set("session_user_1", "authenticated", Duration::from_millis(300));
    cache.set("system_metric", "99.8%", Duration::from_secs(5));

    // Spawn 4 reader threads
    for id in 0..4 {
        let cache_ref = cache.clone();
        workers.push(thread::spawn(move || {
            for _ in 0..3 {
                if let Some(val) = cache_ref.get(&"session_user_1") {
                    println!("[Reader {id}] Cache hit: {val}");
                } else {
                    println!("[Reader {id}] Cache MISS (expired or non-existent)");
                }
                thread::sleep(Duration::from_millis(150));
            }
        }));
    }

    // Spawn 1 background sweeper thread
    let sweeper_cache = cache.clone();
    let sweeper = thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        let purged = sweeper_cache.cleanup_stale_entries();
        println!("\n>>> [Janitor] Sweep completed: {purged} stale keys evicted <<<\n");
    });

    for worker in workers {
        worker.join().unwrap();
    }
    sweeper.join().unwrap();
}
```

### Architectural Review: Choosing the Right Primitive

```text
Do multiple threads need to own the handle?
  ├─ No  ──> Stack reference with std::thread::scope
  └─ Yes ──> Arc<T>
              │
              Can T be modified?
                ├─ No  ──> Arc<T> (Immutable shared state)
                └─ Yes ──> What is the read/write distribution?
                            ├─ Many readers, few writers ──> Arc<RwLock<T>>
                            ├─ High contention writes    ──> Arc<Mutex<T>>
                            └─ Primitive numeric counter ──> Arc<AtomicUsize> (Phase 6)
```

[Go to the Top](#table-of-content)

---

## Phase 5: Advanced Coordination Primitives

In Phase 4, we used `Mutex` and `RwLock` to serialize access to memory.  
However, locks only answer the question: *"Can I safely touch this data right now?"*  

They do **not** solve the signaling problem: *"How do I wait until the data enters a specific state without burning 100% CPU in a spin loop?"*  

Phase 5 addresses explicit thread orchestration: parking threads until states change, synchronizing groups at a rendezvous point, and managing one-time lazy global initialization safely.

### Module 5.1: Condition Variables (`std::sync::Condvar`)

A naive approach to waiting for a state transition is busy-waiting:

```rust
// ANTI-PATTERN: Burns CPU cycles, starves other threads
loop {
    let guard = lock.lock().unwrap();
    if *guard == true { break; }
    // Drop lock and loop immediately
}
```

A **Condition Variable** (`Condvar`) solves this by allowing an idle thread to sleep while atomically releasing its associated mutex.

```text
Producer Thread                                     Consumer Thread
      │                                                   │
      │                                             1. lock.lock()
      │                                             2. while !ready {
      │                                                   condvar.wait(guard)
      │                                                }  └── Atomically releases lock
      │                                                       & parks thread in OS
      │
3. lock.lock()
4. *data = ready
5. condvar.notify_one()
   └── Wakes Consumer
6. drop(lock)
                                                    3. Unparks, re-acquires lock
                                                    4. Verifies state and proceeds
```

#### 1. The Anatomy of `Condvar::wait`

The signature of `wait` requires a mutable reference to a `MutexGuard`:

```rust
pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> LockResult<MutexGuard<'a, T>>
```

This API models the underlying OS invariant (`pthread_cond_wait` / `SleepConditionVariableSRW`):

1. It **releases the lock** held by `guard`.
2. It **blocks the calling thread** in the OS kernel.
3. Both actions occur **atomically**. If they weren't atomic, a producer could signal between the unlock and the sleep, causing the consumer to sleep forever (a lost wakeup bug).
4. When awakened, `wait` **re-acquires the lock** before returning, handing you back a valid `MutexGuard`.

#### 2. The Spurious Wakeup Invariant

A thread can wake up from `wait` even if nobody called `notify`!  
This can happen due to OS kernel interrupts or multi-core scheduling artifacts (**spurious wakeups**).  

> **Rule:** Never use an `if` condition when calling `Condvar::wait`. **Always use a `while` loop** (or the standard library helper `wait_while`).

```rust
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair_clone = Arc::clone(&pair);

thread::spawn(move || {
    let (lock, cvar) = &*pair_clone;
    let mut started = lock.lock().unwrap();
    *started = true;
    // Notify one sleeping thread
    cvar.notify_one(); 
});

let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();

// Idiomatic: Loop re-checks condition upon waking up
while !*started {
    started = cvar.wait(started).unwrap();
}

// Or using the standard library helper:
// cvar.wait_while(started, |started| !*started).unwrap();

println!("Worker signaled readiness!");
```

#### 3. `notify_one` vs. `notify_all`

- **`notify_one()`**: Unparks a single waiting thread. Use this when only one worker can process the state change (e.g., one item added to a queue).
- **`notify_all()`**: Unparks every thread currently blocked on the `Condvar`. Use this for broadcasting events (e.g., shutdown signals, milestone reached).

### Module 5.2: Barriers (`std::sync::Barrier`)

A `Condvar` coordinates based on arbitrary state. A `Barrier` coordinates threads based strictly on **count**.

A barrier enables multiple threads to synchronize the beginning of some computation.  
When initialized with a count $N$, calling `barrier.wait()` blocks each thread until exactly $N$ threads have called `barrier.wait()`.  
At that moment, the barrier opens, and all $N$ threads are released simultaneously.  

```text
Thread 1:  ─── Compute Phase 1 ───> barrier.wait() ──┐
Thread 2:  ─── Compute Phase 1 ─────────> barrier.wait() ──┼─> [ Barrier Releases ] ──> Next Phase
Thread 3:  ─── Compute Phase 1 ──> barrier.wait() ─────────┘
```

#### Cyclic Reusability and the Leader Thread

- **Cyclic:** A barrier automatically resets its counter to $0$ after releasing, allowing it to be used across repeated simulation steps (e.g., multi-step scientific calculations).
- **Leader Election:** `barrier.wait()` returns a `BarrierWaitResult`. Exactly one thread receives `is_leader() == true`, designating it to perform single-threaded housekeeping between steps (such as logging or saving checkpoints).

```rust
use std::sync::{Arc, Barrier};
use std::thread;

let num_threads = 3;
let barrier = Arc::new(Barrier::new(num_threads));
let mut handles = vec![];

for id in 0..num_threads {
    let b = Arc::clone(&barrier);
    handles.push(thread::spawn(move || {
        println!("Worker {id}: Running Stage 1...");
        
        // Wait for all 3 threads to reach this point
        let wait_result = b.wait();
        
        if wait_result.is_leader() {
            println!(">>> Leader thread {id}: All workers finished Stage 1! Resetting. <<<");
        }

        println!("Worker {id}: Running Stage 2...");
    }));
}

for h in handles {
    h.join().unwrap();
}
```

### Module 5.3: Global One-Time Initialization (`Once` & `OnceLock`)

Global mutable state is notoriously unsafe in concurrent programming.  
In older Rust codebases, developers relied on crates like `lazy_static` or `once_cell`.  
Today, Rust's standard library provides native, thread-safe, one-time initialization primitives in `std::sync`.

#### 1. `std::sync::Once` (Execution Initialization)

Ensures a closure is executed **exactly once**, regardless of how many threads invoke it concurrently:

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_subsystem() {
    INIT.call_once(|| {
        // Run hardware checks or legacy C-library bindings here.
        // Guaranteed to run once, and only once.
        println!("System initialized exactly once.");
    });
}
```

If multiple threads call `initialize_subsystem()` at the same time:

1. The first thread runs the closure.
2. The other threads block until the first thread completes the closure successfully.
3. Subsequent calls return immediately without executing the closure.

#### 2. `std::sync::OnceLock<T>` (Value Initialization)

Stabilized in Rust 1.70, `OnceLock<T>` is the modern standard replacement for `lazy_static`.  
It is a thread-safe cell that can be written to exactly once and read from concurrently without locks:

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

// Safe global static without unsafe blocks or third-party crates
static GLOBAL_CONFIG: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_config(key: &str) -> Option<&'static str> {
    let map = GLOBAL_CONFIG.get_or_init(|| {
        println!("--> Initializing global configuration map...");
        let mut m = HashMap::new();
        m.insert("host", "127.0.0.1");
        m.insert("port", "8080");
        m
    });

    map.get(key).copied()
}

fn main() {
    let h1 = std::thread::spawn(|| println!("Port: {:?}", get_config("port")));
    let h2 = std::thread::spawn(|| println!("Host: {:?}", get_config("host")));

    h1.join().unwrap();
    h2.join().unwrap();
}
```

- **No Overhead on Reads:** Once initialized, `get()` performs an atomic pointer load without acquiring OS locks.
- **Race Resolution:** If two threads call `get_or_init` simultaneously, only one initialization closure executes; the losing thread discards its result and adopts the winner's value.

### Phase 5 Milestone: Custom Bounded MPMC Queue

To synthesize these synchronization patterns, we will build a custom **Bounded Multi-Producer, Multi-Consumer (MPMC) Queue** from scratch using only a standard `VecDeque`, a `Mutex`, and two `Condvar`s:

- **`not_full` Condvar**: Producers sleep here when the buffer hits capacity.
- **`not_empty` Condvar**: Consumers sleep here when the buffer is dry.

```text
Producers (tx) ──> lock ──> [ Full? ] ──Yes──> not_full.wait()
                              │ No
                              └── Enqueue ──> not_empty.notify_one()

Consumers (rx) ──> lock ──> [ Empty? ] ──Yes──> not_empty.wait()
                              │ No
                              └── Dequeue ──> not_full.notify_one()
```

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub struct BoundedQueue<T> {
    capacity: usize,
    inner: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
}

impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        Self {
            capacity,
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    /// Push an item onto the queue, blocking if capacity is reached
    pub fn push(&self, item: T) {
        let mut queue = self.inner.lock().unwrap();

        // While queue is full, release lock and sleep on `not_full`
        while queue.len() >= self.capacity {
            queue = self.not_full.wait(queue).unwrap();
        }

        queue.push_back(item);

        // Notify one sleeping consumer that an item is available
        self.not_empty.notify_one();
    }

    /// Pop an item off the queue, blocking if the queue is empty
    pub fn pop(&self) -> T {
        let mut queue = self.inner.lock().unwrap();

        // While queue is empty, release lock and sleep on `not_empty`
        while queue.is_empty() {
            queue = self.not_empty.wait(queue).unwrap();
        }

        let item = queue.pop_front().unwrap();

        // Notify one sleeping producer that space has opened up
        self.not_full.notify_one();

        item
    }
}

fn main() {
    // Capacity of 2 elements creates immediate contention/coordination
    let queue = Arc::new(BoundedQueue::new(2));
    let mut handles = vec![];

    // Spawn 2 Consumer Threads
    for id in 0..2 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                let val = q.pop();
                println!("  [Consumer {id}] <<< Popped: {val}");
                thread::sleep(Duration::from_millis(150));
            }
        }));
    }

    // Spawn 2 Producer Threads
    for id in 0..2 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for i in 0..5 {
                let val = format!("msg-{id}-{i}");
                q.push(val.clone());
                println!("[Producer {id}] >>> Pushed: {val}");
                thread::sleep(Duration::from_millis(50));
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("\nMPMC Pipeline drained successfully without busy loops or deadlocks!");
}
```

### Architectural Review: Phase 5 Primitives

| Primitive          | Use Case                                                            | Sleep/Wake Mechanism                                                      |
| ------------------ | ------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| **`Condvar`**      | Coordinate threads based on dynamic conditions or state transitions | Releases associated `Mutex` and parks via OS futex; awakened via `notify` |
| **`Barrier`**      | Hold groups of threads until all $N$ participants reach a stage     | Tracks atomic arrival counter; unparks all callers simultaneously         |
| **`Once`**         | Execute side-effecting initialization code exactly once             | Blocks concurrent callers; subsequent callers skip immediately            |
| **`OnceLock<T>`**  | Safely compute and share a static global value lazily               | Lock-free reading once populated; safe concurrent init race resolution    |

[Go to the Top](#table-of-content)

---

## Phase 6: Low-Level Atomics & Memory Ordering

In Phase 4 and Phase 5, we relied on OS-assisted synchronization primitives (`Mutex`, `RwLock`, `Condvar`).  
Under the hood, those locks avoid burning CPU by putting threads to sleep via OS syscalls (like `futex` on Linux).

Atomics eliminate OS syscalls entirely.  
They map directly to **single machine-code instructions** supported natively by the CPU memory bus, enabling lock-free concurrency.  

### Module 6.1: Atomic Primitives (`std::sync::atomic`)

Standard integer types (`u32`, `usize`, `bool`) cannot be safely mutated concurrently because operations like `x += 1` are not atomic; they expand into three separate CPU instructions:

1. `MOV`: Load value from RAM/cache into a register.
2. `ADD`: Increment the register.
3. `MOV`: Store the register value back to cache.

If two threads execute this sequence simultaneously, their reads and writes interleave, leading to lost updates.

Rust provides atomic counterparts in `std::sync::atomic`: `AtomicBool`, `AtomicUsize`, `AtomicI32`, `AtomicPtr<T>`, etc.

#### Core Atomic Operations

- **`load(&self, order)`**: Reads the current value atomically.
- **`store(&self, val, order)`**: Overwrites the value atomically.
- **`swap(&self, val, order)`**: Replaces the value and returns the old value in one indivisible operation.
- **`fetch_add(&self, val, order)` / `fetch_sub**`: Atomically adds/subtracts and returns the *previous* value (wrapping on overflow).
- **Compare-And-Swap (`compare_exchange` vs `compare_exchange_weak`)**: The cornerstone of lock-free data structures.

#### `compare_exchange` vs. `compare_exchange_weak`

Compare-and-swap checks if the current value matches an expected value; if true, it writes a new value.

```rust
pub fn compare_exchange(
    &self,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering
) -> Result<T, T>
```

| Method                       | Behavior on Mismatch                 | Spurious Failures?                                 | Best Use Case                                                                                                                                                                             |
| ---------------------------- | ------------------------------------ | -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`compare_exchange`**       | Fails only if value $\neq$ expected  | **No.** Guaranteed deterministic check.            | When a failure branch is expensive or cannot be retried easily.                                                                                                                           |
| **`compare_exchange_weak`**  | Can fail even if value $==$ expected | **Yes** (on LL/SC architectures like ARM, RISC-V). | **Inside loops.** On ARM/RISC-V, `weak` compiles to a single load-linked/store-conditional pair, making the retry loop significantly faster than forcing strong CAS emulation.            |

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(10);

// CAS Loop Pattern using compare_exchange_weak
let mut current = counter.load(Ordering::Relaxed);
loop {
    let new_val = current * 2;
    match counter.compare_exchange_weak(
        current,
        new_val,
        Ordering::AcqRel,
        Ordering::Relaxed
    ) {
        Ok(_) => break,
        Err(actual) => current = actual, // Update expected value and retry
    }
}
```

### Module 6.2: The Hardware Memory Model & `Ordering`

Atomics do more than prevent torn reads/writes; their primary job is establishing **memory visibility and ordering guarantees** across threads and CPU cores.

The Rust memory model is inherited directly from **C++11**. It defines how operations synchronize and restricts both the compiler and CPU from reordering memory accesses.

```text
Weakest (Fastest)  <─────────────────────────────────────────> Strongest (Slowest)
   Relaxed           Acquire / Release / AcqRel              SeqCst
```

#### 1. `Ordering::Relaxed` (No Ordering Constraints)

- **Guarantee:** Atomicity only. Guarantees that loads and stores are indivisible (no torn reads/writes), and all modifications to that specific single atomic variable occur in a single, consistent modification order.
- **Non-Guarantee:** Provides **zero synchronization** with other memory locations. Instructions before or after the atomic operation can be freely reordered by the compiler or CPU.
- **Use Case:** Standalone counters, metrics, or flags where other memory does not depend on the sequence (e.g., counting total requests served by an HTTP server).

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static HIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn record_hit() {
    // Relaxed is sufficient: we don't care who sees which hit in what order,
    // only that no increments are dropped.
    HIT_COUNTER.fetch_add(1, Ordering::Relaxed);
}
```

#### 2. Acquire-Release Semantics (`Acquire`, `Release`, `AcqRel`)

This is the standard model for synchronizing user data across threads. Acquire and Release operate as a paired handoff:

- **`Release` (used on stores):**  
  No memory access (load or store) preceding this operation in the source code can be reordered **after** this store.  
  All previous writes (atomic and non-atomic!) are committed and made visible to any thread that performs an `Acquire` load on this same atomic variable.
- **`Acquire` (used on loads):**  
  No memory access following this operation in the source code can be reordered **before** this load.  
  It ensures the reading thread sees everything that happened before the corresponding `Release` store.
- **`AcqRel` (used on Read-Modify-Write):**  
  Combines both.  
  It acts as `Acquire` for the read part and `Release` for the write part (standard for CAS loops and mutex acquisition).

```text
Producer Core                               Consumer Core
[ Non-atomic Data Write: data = 42 ]        
[ Release Store: ready.store(true) ] ───┐   
                                        │ (Synchronization Edge)
                                        └──> [ Acquire Load: while !ready.load() ]
                                             [ Safe Read: assert_eq!(data, 42) ]
```

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static mut SHARED_DATA: u64 = 0;
static READY: AtomicBool = AtomicBool::new(false);

fn main() {
    thread::spawn(|| {
        // Safe only because of the Acquire-Release synchronization edge below!
        unsafe { SHARED_DATA = 42; }

        // RELEASE: Flushes previous writes; cannot be reordered before SHARED_DATA write
        READY.store(true, Ordering::Release);
    });

    thread::spawn(|| {
        // ACQUIRE: Guarantees everything after this sees what happened before the RELEASE
        while !READY.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }

        // Guarantees no data race: SHARED_DATA is guaranteed to be 42
        unsafe {
            println!("Data: {}", SHARED_DATA);
        }
    }).join().unwrap();
}
```

#### 3. `Ordering::SeqCst` (Sequential Consistency)

- **Guarantee:**  
  Imposes all the guarantees of `Acquire` (on loads) and `Release` (on stores), **plus** enforces a globally consistent total order across all threads.
- Every single thread in the system observes all `SeqCst` operations occurring in the exact same sequential order.
- **The Cost:**  
  On x86, `SeqCst` stores emit expensive `MFENCE` or locked instructions (`LOCK CMPXCHG`, `LOCK XADD`).  
  On ARM, it requires full memory barriers (`dmb ish`), which can stall the CPU pipeline while memory buses synchronize.
- **When to use:**  
  It is the default choice when you are unsure of the memory model implications, or for complex algorithms where multiple atomic variables interact and all threads must observe their transitions in identical order.

### Module 6.3: Hardware Memory Fences (`std::sync::atomic::fence`)

Sometimes you want to synchronize memory without coupling the barrier directly to an atomic load or store.  
A **fence** separates memory operations without needing an atomic operation on every variable:

```rust
use std::sync::atomic::{fence, Ordering};

// Establishes release semantics for preceding writes
fence(Ordering::Release);

// Establishes acquire semantics for subsequent reads
fence(Ordering::Acquire);
```

Fences emit hardware-level synchronization instructions (e.g., `dmb` on ARM, or acting as compiler barriers on x86, which is already strongly ordered for most reads/writes).

### Phase 6 Milestone: Custom Lock-Free Spinlock

To see how `Acquire` and `Release` work together in hardware, we will build a working, starvation-resistant **Spinlock**.

Instead of putting threads to sleep via the OS kernel, a spinlock busy-loops using atomic instructions.  
It is designed for low-contention scenarios where critical sections execute in nanoseconds.

```rust
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::hint;

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// Safety: As long as T can be sent across threads, SpinLock can be shared (&SpinLock: Sync)
unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        // TTAS (Test and Test-And-Set) optimization:
        // First check with a relaxed load to avoid invalidating CPU cache lines in a loop.
        while self.locked.load(Ordering::Relaxed)
            || self.locked.swap(true, Ordering::Acquire)
        {
            // Emits a CPU-level pause instruction (PAUSE on x86, YIELD on ARM).
            // Prevents pipeline stalls and saves power during busy-waiting.
            hint::spin_loop();
        }

        SpinLockGuard { lock: self }
    }
}

// RAII Guard: Releases the lock on drop using Release ordering
impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        // RELEASE: Ensures all mutations through DerefMut are flushed 
        // before the lock flag flips back to false!
        self.lock.locked.store(false, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: We hold the spinlock exclusively
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: We hold exclusive write access
        unsafe { &mut *self.lock.data.get() }
    }
}

fn main() {
    use std::sync::Arc;
    use std::thread;

    let shared_counter = Arc::new(SpinLock::new(0));
    let mut handles = vec![];

    for _ in 0..4 {
        let counter = Arc::clone(&shared_counter);
        handles.push(thread::spawn(move || {
            for _ in 0..10_000 {
                let mut guard = counter.lock();
                *guard += 1; // Mutating inner data through lock
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_val = *shared_counter.lock();
    println!("Final Counter Value: {final_val}");
    assert_eq!(final_val, 40_000);
}
```

### Architectural Review: Memory Ordering Selection Matrix

| Ordering        | Cost (x86)                                   | Cost (ARM/RISC-V)                        | Use Case                                                 |
| --------------- | -------------------------------------------- | ---------------------------------------- | -------------------------------------------------------- |
| **`Relaxed`**   | Zero overhead (identical to plain MOV)       | Zero overhead                            | Counters, statistics, flags that do not guard data       |
| **`Release`**   | Free (x86 hardware does not reorder stores)  | Memory barrier instruction (`dmb.ish`)   | Publishing shared data, releasing locks                  |
| **`Acquire`**   | Free (x86 hardware does not reorder loads)   | Memory barrier instruction (`dmb.ishld`) | Consuming published data, acquiring locks                |
| **`AcqRel`**    | Free (for Read-Modify-Write)                 | Full barrier                             | Read-Modify-Write operations, swap, CAS                  |
| **`SeqCst`**    | Emits `LOCK` prefix / `MFENCE`               | Full memory barrier                      | Complex multi-variable invariants across multiple cores  |

[Go to the Top](#table-of-content)

---

## Phase 7: The Bridge: OS Threads vs. Asynchronous Concurrency

We have covered low-level concurrency down to CPU instruction caches and memory ordering fences.  
At the application layer, however, another architectural dividing line emerges: **Preemptive OS Threads** vs. **Cooperative Asynchronous Tasks**.

### Module 7.1: Preemptive vs. Cooperative Multitasking

The distinction between OS multi-threading and async programming is not stylistic;  
It is an engineering tradeoff between **memory density**, **context-switch latency**, and **workload characteristics**.

```text
Preemptive OS Threads (1:1 Model):
  Thread 1 ────[ Slice A ]──────(OS Timer Interrupt)───> [ Wait in Runqueue ]
  Thread 2 ────────────────────[ Interleaved Exec ]─────> [ Kernel Context Switch ]

Cooperative Async Tasks (M:N Task-on-Thread Model):
  Worker Thread (OS) ─────────────────────────────────────────────────────────>
     Task A  ──[ Runs to .await ]──> (Yields back to Runtime)
     Task B                         └──[ Picks up & Runs to .await ]──> (Yields)
```

| Dimension            | Native OS Threads (`std::thread`)                                                                               | Async Tasks (`tokio::task`)                                                                   |
| -------------------- | --------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| **Scheduling**       | **Preemptive:** The OS kernel interrupts execution at arbitrary points via timer interrupts.                    | **Cooperative:** Tasks yield execution voluntarily at explicit `.await` points.               |
| **Stack Allocation** | Fixed stack (typically 2MB virtual, allocated upfront per thread).                                              | Zero dedicated stack. Tasks are compile-time state machines packed into compact heap structs. |
| **Memory Footprint** | ~10,000 threads can exhaust GBs of virtual memory and kernel control structures.                                | 100,000+ tasks can comfortably live in several hundred megabytes.                             |
| **Switch Cost**      | Full kernel context switch: CPU register save, cache pollution, TLB invalidation ($1\text{--}2\,\mu\text{s}$).  | Function call / state-machine jump in user-space ($\approx 10\text{--}50\text{ ns}$).         |
| **Primary Domain**   | **CPU-bound workloads** (crypto, encoding, scientific matrix transforms).                                       | **I/O-bound workloads** (HTTP servers, WebSocket brokers, network proxies).                   |

### Module 7.2: Tokio Runtime Architecture Under the Hood

The standard library provides the `Future` trait, but **it ships with no async runtime out of the box**.  
Production systems use `tokio`, which acts as an operating system within user space.

#### 1. The Multi-Thread Work-Stealing Engine

Tokio’s multi-threaded runtime maintains a pool of worker OS threads (typically 1 worker thread per physical CPU core).

```text
Worker Thread 0                     Worker Thread 1
+---------------------------+       +---------------------------+
| Local Run Queue (max 256) |       | Local Run Queue (max 256) |
| [Task 1] [Task 2] [Task 3]|       | [Task 4] (empty...)       |
+---------------------------+       +---------------------------+
             │                                    │
             ▼                                    ▼
       Executes Task 1                 Steals Task 3 from Worker 0!
             │                                    ▲
             └──────── Work-Stealing Edge ────────┘
```

1. **Local Run Queues:**  
  Each worker thread has a fixed-capacity ring buffer (256 tasks) designed with lock-free atomic operations.  
  Workers pull from their own queue without taking shared locks.
2. **Work-Stealing:** If Worker 1's local queue is empty, it attempts to steal half the tasks from Worker 0's local queue via atomic CAS instructions.
3. **Global Queue:** If all local queues overflow, tasks spill to a shared mutex-protected fallback queue.
4. **OS Reactor (`mio`):**  
  While tasks wait on network sockets, Tokio registers their file descriptors with OS event-notification APIs (`epoll` on Linux, `kqueue` on macOS, `IOCP` on Windows).  
  When the kernel signals socket readiness, the reactor unparks the corresponding task and pushes it back into a worker's run queue.

### Module 7.3: The Bridge: Bridging Sync and Async Safely

The single most common bug in Rust async applications is running blocking, compute-heavy, or non-async I/O code inside an async worker thread.

#### The Golden Rule of Async

> **Never block an async worker thread.**

If a task invokes `std::thread::sleep`, queries a synchronous database driver, or computes a heavy image transform inside a plain async function, **that entire worker thread is frozen**.  
None of the hundreds or thousands of other tasks assigned to that worker can progress.

```rust
// CRITICAL BUG: Freezes the entire Tokio worker thread
async fn bad_handler() {
    // This blocks the OS thread running the async event loop!
    std::thread::sleep(std::time::Duration::from_secs(5)); 
}

//  CORRECT ASYNC SLEEP: Yields control back to the scheduler
async fn good_handler() {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
}
```

#### Offloading Heavy Sync Work: `spawn_blocking`

When you must run synchronous or CPU-heavy workloads (e.g., password hashing with Argon2, image decoding, synchronous file access), you must offload them using `tokio::task::spawn_blocking`.

Tokio manages an entirely separate, dynamic pool of dedicated OS threads specifically for blocking work:

```rust
use tokio::task;

async fn handle_user_login(password: String, hash: String) -> bool {
    // Moves execution completely off the async worker thread pool 
    // onto a dedicated blocking thread pool
    let is_valid = task::spawn_blocking(move || {
        // CPU-bound password verification (Argon2 / PBKDF2)
        verify_password_cpu_intensive(&password, &hash)
    })
    .await
    .expect("Blocking task panicked or runtime shut down");

    is_valid
}

fn verify_password_cpu_intensive(_pass: &str, _hash: &str) -> bool {
    // Simulated 200ms CPU-heavy computation
    true
}
```

#### `Sync` vs. `Async` Primitives: Choose Wisely

Never use `tokio::sync::Mutex` as a default replacement for `std::sync::Mutex`. They serve distinct purposes:

```text
Should I hold the lock across an `.await` boundary?
  ├─ No  ──> Use std::sync::Mutex (faster, zero runtime allocation)
  └─ Yes ──> Use tokio::sync::Mutex (preserves task-yielding across locks)
```

```rust
// Holding an std::sync::Mutex across .await is a compile-time bug or deadlock hazard!
use std::sync::Arc;

async fn lock_selection_example() {
    // Standard Mutex is fine if the critical section is brief and does NOT cross .await:
    let std_locked = Arc::new(std::sync::Mutex::new(0));
    {
        let mut guard = std_locked.lock().unwrap();
        *guard += 1;
    } // Guard dropped before any .await

    // Tokio Mutex is ONLY required if you MUST yield while holding the lock:
    let tokio_locked = Arc::new(tokio::sync::Mutex::new(0));
    {
        let mut guard = tokio_locked.lock().await;
        some_async_network_call().await; // Guard remains held across yield!
        *guard += 1;
    }
}

async fn some_async_network_call() {}
```

### Phase 7 Milestone: Hybrid Sync-Async Parallel Pipeline

To synthesize the bridge between raw OS multi-threading and cooperative async tasks, we will build a production-style **hybrid parallel ingestion engine**:

- An async Tokio network coordinator ingests mock incoming telemetry jobs.
- Heavy CPU computational work is dynamically offloaded to dedicated OS threads via `spawn_blocking`.
- Cross-thread communication uses bounded Tokio channels (`tokio::sync::mpsc`) to enforce backpressure.

```rust
// Cargo.toml dependencies:
// tokio = { version = "1", features = ["full"] }

use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct TelemetryPacket {
    sensor_id: u32,
    raw_payload: Vec<u8>,
}

#[derive(Debug)]
struct ProcessedResult {
    sensor_id: u32,
    computed_metric: u64,
    processing_time_ms: u128,
}

/// Simulated CPU-bound numerical analytics (strictly synchronous)
fn heavy_cpu_crunch(packet: TelemetryPacket) -> ProcessedResult {
    let start = Instant::now();
    
    // Simulate intensive cryptographic/mathematical processing
    let mut accumulator: u64 = 0;
    for (idx, byte) in packet.raw_payload.iter().enumerate() {
        accumulator = accumulator.wrapping_add((*byte as u64) * (idx as u64 + 1));
        for _ in 0..10_000 {
            accumulator = accumulator.rotate_left(1) ^ 0x5555_5555;
        }
    }

    ProcessedResult {
        sensor_id: packet.sensor_id,
        computed_metric: accumulator,
        processing_time_ms: start.elapsed().as_millis(),
    }
}

#[tokio::main]
async fn main() {
    println!("=== Starting Hybrid Sync/Async Processing Pipeline ===\n");

    // Bounded async channel for backpressure (capacity: 4 items)
    let (tx, mut rx) = mpsc::channel::<TelemetryPacket>(4);

    // 1. Producer Task (Asynchronous I/O simulation)
    let producer = tokio::spawn(async move {
        for id in 1..=6 {
            // Non-blocking async sleep simulating network arrival
            sleep(Duration::from_millis(100)).await;

            let packet = TelemetryPacket {
                sensor_id: id,
                raw_payload: vec![(id as u8) * 7; 1024],
            };

            println!("[Ingest Worker] Received packet from sensor #{id} over network");
            
            // Backpressure: sends wait if the consumer channel is full
            tx.send(packet).await.unwrap();
        }
        println!("[Ingest Worker] Ingestion complete. Disconnecting pipe.");
    });

    // 2. Consumer Loop: Bridges async reception to sync OS threads
    let mut processing_handles = vec![];

    while let Some(packet) = rx.recv().await {
        println!("[Pipeline Dispatcher] Offloading sensor #{} to OS thread pool...", packet.sensor_id);

        // Bridge to OS thread pool: does not block the Tokio reactor!
        let handle = task::spawn_blocking(move || heavy_cpu_crunch(packet));
        processing_handles.push(handle);
    }

    // Await all producers and offloaded compute tasks
    producer.await.unwrap();

    println!("\n=== Aggregating Results ===");
    for handle in processing_handles {
        let result = handle.await.expect("Worker thread panicked!");
        println!(
            "-> Sensor #{}: Metric = {:#018x} (Computed in {}ms)",
            result.sensor_id, result.computed_metric, result.processing_time_ms
        );
    }

    println!("\nPipeline cleanly drained with zero blocked event loops.");
}
```

### The Complete Concurrency Decision Matrix

With all 7 phases completed, you now have the complete Rust concurrency decision tree:

```text
What is the core nature of your task?
│
├── CPU-Bound (Heavy Math, Compression, Image Processing, Matrix Ops)
│   ├── Do workloads share disjoint memory in-place?
│   │   ├── Yes ──> std::thread::scope + split_at_mut (Zero Allocation, Zero Locks)
│   │   └── No  ──> Rayon parallel iterators / std::thread::spawn
│   └── Is it triggered from an async application?
│       └── Yes ──> tokio::task::spawn_blocking
│
└── I/O-Bound (Web Servers, Sockets, High-Fanout Microservices)
    ├── Needs millions of connections with low idle overhead?
    │   └── Yes ──> Async Rust (Tokio runtime + Futures)
    └── State Synchronization Needs:
        ├── Passing discrete messages? ──> crossbeam-channel (Sync) / tokio::sync::mpsc (Async)
        ├── Concurrent reads, occasional writes? ──> std::sync::RwLock
        ├── Short critical sections? ──> std::sync::Mutex
        ├── Lock-free counters or flags? ──> std::sync::atomic (Relaxed / Acquire-Release)
        └── Complex signaling / condition testing? ──> std::sync::Condvar
```

[Go to the Top](#table-of-content)

---

