<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD029 -->
<!-- markdownlint-disable MD040 -->

# Low-Level Design and Object-Oriented Architecture in Rust: A Guide for Java Engineers

## 1. The Architectural Paradigm Shift

In the Java and JVM ecosystem, the foundational building block of software design is the **class**. A class simultaneously fulfills four distinct responsibilities:

1. It defines **data layout** (memory footprint of fields).
2. It encapsulates **inherent behavior** (instance methods).
3. It declares **subtyping and polymorphism** (inheritance hierarchies).
4. It serves as the primary **compilation and privacy unit** (classes per file).

Rust decouples these four concerns into distinct primitives:

```text
                       Java `class`
            ┌──────────────────┬───────────────┐
            │                  │               │
            ▼                  ▼               ▼
      Data Layout          Behavior        Contracts
      (`struct`, `enum`)    (`impl`)       (`trait`)
```

- **Data Layout (`struct`, `enum`):**  
  Product types and algebraic sum types declare the exact memory structure of domain data.  
  They contain no methods, no hidden virtual table pointers, and no metadata headers.
- **Behavior (`impl`):**  
  Inherent logic is bound explicitly to types through `impl Type` blocks.  
  Data definitions remain clean declarations of state.
- **Contracts (`trait`):**  
  Abstract behavior interfaces are declared independently of the types that fulfill them.  
  Types can retroactively implement traits across module boundaries.
- **Sum Types (`enum`):**  
  Discriminated unions model state variations without the overhead of subtype class hierarchies.

## 2. The Canonical Domain Model

The following end-to-end implementation represents an e-commerce checkout subsystem.  
It demonstrates the interaction of `struct`, `impl`, `trait`, and `enum` with both compile-time (generics) and runtime (`dyn`) polymorphism.

```rust
use std::collections::HashMap;

// ============================================================================
// 1. DOMAIN ENTITIES & RELATIONSHIPS (Structs & Enums)
// ============================================================================

// --- THE NEWTYPE PATTERN ---
// Instead of a primitive `u64`, we wrap it in a dedicated tuple struct.
// Benefits:
// 1. Compile-time type safety: You cannot accidentally pass an `OrderId` to a function expecting `CustomerId`.
// 2. Zero-cost abstraction: In memory, this is identical to a bare 64-bit integer (no heap allocation, no pointer overhead).
// 3. Trait derivation:
//    - `Debug`: String formatting via `{:?}` (like Java's `toString()`).
//    - `Clone`, `Copy`: Enables cheap, implicit bitwise copying on assignment (primitive value semantics).
//    - `PartialEq`, `Eq`: Enables `==` and `!=` with strict mathematical reflexivity.
//    - `Hash`: Enables computing hash codes so this type can serve as a key in `HashMap` or `HashSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomerId(pub u64);

// Pure data layout definition.
// Unlike a Java class, this contains NO methods, NO vtable pointers, and NO inheritance overhead.
// Memory layout is flat, contiguous, and known at compile time.
#[derive(Debug, Clone)]
pub struct LineItem {
    pub product_sku: String,
    pub unit_price_cents: u64,
    pub quantity: u32,
}

// Inherent behavior bound explicitly to the `LineItem` data layout.
// In Rust, data (`struct`) and operations (`impl`) are declared separately.
impl LineItem {
    // Methods explicitly declare their relationship to the instance:
    // `&self` means immutable borrow (read-only access without taking ownership).
    pub fn subtotal(&self) -> u64 {
        self.unit_price_cents * (self.quantity as u64)
    }
}

// --- ALGEBRAIC DATA TYPES (Sum Types / Tagged Unions) ---
// In classical OOP, different statuses with different payloads require either:
// 1. An abstract base class with subclasses (`PendingStatus`, `PaidStatus`), or
// 2. A messy class with nullable fields (`transactionId == null`).
// Rust enums eliminate invalid states entirely at compile time:
// An order CANNOT have a `transaction_id` unless it is in the `Paid` state.
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Paid { transaction_id: String },
    Cancelled,
}

// --- COMPOSITION OVER INHERITANCE ---
// Relationships modeled without an ORM or class hierarchy:
// - `customer_id`: Relationship by identity (foreign key), avoiding deep cyclic object graphs and borrow checker deadlocks.
// - `items`: 1-to-Many relationship via direct composition (`Vec<LineItem>`). Memory is tightly packed in a contiguous heap array.
// - `status`: Embedded state machine represented by a tagged union.
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: u64,
    pub customer_id: CustomerId,
    pub items: Vec<LineItem>,
    pub status: OrderStatus,
}

impl Order {
    // Idiomatic constructor: Rust does not have `new` as a language keyword.
    // By convention, `new` is an associated function (like a static factory method in Java)
    // that returns an instance of `Self`.
    pub fn new(order_id: u64, customer_id: CustomerId, items: Vec<LineItem>) -> Self {
        Self {
            order_id,
            customer_id,
            items,
            status: OrderStatus::Pending,
        }
    }

    // High-level functional aggregation:
    // `iter()` borrows the items immutably; `map` projects each item to its subtotal; `sum` reduces.
    pub fn total_cents(&self) -> u64 {
        self.items.iter().map(|item| item.subtotal()).sum()
    }
}

// ============================================================================
// 2. BEHAVIOR CONTRACTS (Traits)
// ============================================================================

// Trait = Interface in Java / Pure Virtual Class in C++.
// Defines a behavior contract without specifying how data is stored.
// Traits can be implemented for any struct, primitive, or external type (retroactive implementation).

// Ideal candidate for STATIC DISPATCH (Generics):
// Typically, a service uses one payment provider at a time per request.
// Resolved at compile time with zero runtime cost.
pub trait PaymentGateway {
    fn charge(&self, amount_cents: u64) -> Result<String, String>;
}

// Ideal candidate for DYNAMIC DISPATCH (`dyn` Trait Object):
// We want to hold multiple disparate listeners in a single list (heterogeneous collection).
// Dyn-compatible (object-safe) because:
// 1. It does NOT require `Self: Sized`.
// 2. Its methods do not return `Self`.
// 3. Its methods do not introduce new generic type parameters.
pub trait OrderObserver {
    fn on_order_paid(&self, order: &Order, transaction_id: &str);
}

// Storage abstraction: Equivalent to a Spring Data `CrudRepository<Order, Long>`.
// Can be implemented by in-memory mock stores, SQL drivers, or key-value caches.
pub trait OrderRepository {
    // Takes ownership of `Order` to persist it.
    fn save(&mut self, order: Order);
    // Returns an optional immutable reference (`Option<&Order>`) to avoid cloning memory.
    fn find_by_id(&self, id: u64) -> Option<&Order>;
}

// ============================================================================
// 3. CONCRETE IMPLEMENTATIONS (impl Trait for Struct)
// ============================================================================

// Concrete implementation 1: Credit Card
pub struct CreditCardGateway {
    pub merchant_id: String,
}

impl PaymentGateway for CreditCardGateway {
    fn charge(&self, amount_cents: u64) -> Result<String, String> {
        println!("[CC] Charging {} cents via merchant {}", amount_cents, self.merchant_id);
        Ok(format!("cc_tx_{}", amount_cents))
    }
}

// Concrete implementation 2: Crypto
pub struct CryptoGateway {
    pub wallet_address: String,
}

impl PaymentGateway for CryptoGateway {
    fn charge(&self, amount_cents: u64) -> Result<String, String> {
        println!("[Crypto] Deducting equivalent of {} cents to {}", amount_cents, self.wallet_address);
        Ok("crypto_tx_0x99a".to_string())
    }
}

// --- ZERO-SIZED TYPES (ZST) AS OBSERVERS ---
// Notice these structs have NO fields. In Rust, they occupy 0 bytes of memory.
// They exist purely as anchors to associate behavior via traits.
pub struct EmailNotifier;
impl OrderObserver for EmailNotifier {
    fn on_order_paid(&self, order: &Order, tx: &str) {
        println!("[Email] Order #{} confirmed. Ref: {}", order.order_id, tx);
    }
}

pub struct AuditLogger;
impl OrderObserver for AuditLogger {
    fn on_order_paid(&self, order: &Order, tx: &str) {
        println!("[Audit] Cust #{} charged {} cents. Tx ID: {}", 
            order.customer_id.0, order.total_cents(), tx);
    }
}

// --- MOCK / IN-MEMORY REPOSITORY ---
// `#[derive(Default)]` generates a default constructor initializing `storage` to an empty HashMap.
#[derive(Default)]
pub struct InMemoryOrderRepository {
    storage: HashMap<u64, Order>,
}

impl OrderRepository for InMemoryOrderRepository {
    fn save(&mut self, order: Order) {
        self.storage.insert(order.order_id, order);
    }

    fn find_by_id(&self, id: u64) -> Option<&Order> {
        self.storage.get(&id)
    }
}

// ============================================================================
// 4. COORDINATOR: COMBINING GENERICS AND `dyn`
// ============================================================================

// In Spring Boot, this would be an `@Service` class with `@Autowired` fields.
// In Rust, dependencies are injected explicitly via composition.
pub struct CheckoutService<R: OrderRepository> {
    // 1. STATIC DISPATCH VIA GENERICS (`R`):
    // The exact repository type is fixed at compile time.
    // - Memory: `R` is embedded directly inside `CheckoutService` with no heap pointer.
    // - Performance: Calls to `self.repository` are direct function calls and can be inlined by LLVM.
    repository: R,

    // 2. DYNAMIC DISPATCH VIA TRAIT OBJECTS (`dyn`):
    // We want to store both `EmailNotifier` (0 bytes) and `AuditLogger` (0 bytes, or any other observer size).
    // Because different implementors have different sizes, they cannot be stored directly in a `Vec`.
    // We put them behind `Box<dyn OrderObserver>`:
    // - `Box` allocates the observer on the heap.
    // - `dyn OrderObserver` creates a 16-byte FAT POINTER:
    //     [ 8 bytes: pointer to data on heap ] + [ 8 bytes: pointer to vtable ]
    observers: Vec<Box<dyn OrderObserver>>,
}

impl<R: OrderRepository> CheckoutService<R> {
    // Constructor acting as our dependency injector.
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            observers: Vec::new(),
        }
    }

    // Accepts any boxed heap allocation whose underlying type satisfies `OrderObserver`.
    pub fn register_observer(&mut self, observer: Box<dyn OrderObserver>) {
        self.observers.push(observer);
    }

    // --- PARAMETRIC POLYMORPHISM (Method-level Generic) ---
    // `<G: PaymentGateway>`: Monomorphization at work.
    // If called with `CreditCardGateway`, the compiler generates a dedicated, optimized copy
    // of `process_checkout` with the credit card logic inlined.
    // `mut order`: Takes ownership of the `Order` by value, allowing state mutation without locks.
    pub fn process_checkout<G: PaymentGateway>(
        &mut self,
        mut order: Order,
        gateway: &G,
    ) -> Result<u64, String> {
        let total = order.total_cents();

        // 1. Static call: Resolved at compile time. Direct branch instruction.
        // `?` operator: Automatically unwraps `Ok(tx_id)` or returns early on `Err`.
        let tx_id = gateway.charge(total)?;

        // Transition the entity's lifecycle state cleanly.
        order.status = OrderStatus::Paid {
            transaction_id: tx_id.clone(),
        };

        // 2. Dynamic call: Resolved at runtime via vtable.
        // Each loop iteration dereferences the fat pointer, reads the function pointer
        // from the type's vtable, and jumps to the code.
        for observer in &self.observers {
            observer.on_order_paid(&order, &tx_id);
        }

        let order_id = order.order_id;
        
        // Ownership transfer: `order` is moved into `self.repository.save()`.
        // After this line, `order` cannot be accidentally read or modified here.
        self.repository.save(order);

        Ok(order_id)
    }

    pub fn get_order(&self, id: u64) -> Option<&Order> {
        self.repository.find_by_id(id)
    }
}

// ============================================================================
// 5. EXECUTION & LINKING (The Composition Root)
// ============================================================================

fn main() {
    // A. Construct Domain Entities
    // All data allocations are explicit.
    let items = vec![
        LineItem {
            product_sku: "RUST-BOOK-01".into(),
            unit_price_cents: 4500,
            quantity: 1,
        },
        LineItem {
            product_sku: "FERRIS-PLUSH-02".into(),
            unit_price_cents: 2500,
            quantity: 2,
        },
    ];
    let order = Order::new(1001, CustomerId(42), items);

    // B. Instantiate Concrete Components
    // In Spring, the IoC ApplicationContext does this via reflection.
    // In Rust, you wire instances explicitly at the composition root (clean, deterministic startup).
    let repo = InMemoryOrderRepository::default();
    let cc_gateway = CreditCardGateway {
        merchant_id: "STRIPE_ACC_9921".into(),
    };

    // C. Wire Dependencies
    // Type inference deduces `CheckoutService<InMemoryOrderRepository>`.
    let mut checkout_service = CheckoutService::new(repo);

    // Dynamic Dispatch Setup: Box disparate types into trait objects.
    // Upcasting from concrete type to `Box<dyn OrderObserver>` happens automatically.
    checkout_service.register_observer(Box::new(EmailNotifier));
    checkout_service.register_observer(Box::new(AuditLogger));

    // D. Run Checkout
    // Compiler specializes `process_checkout` specifically for `CreditCardGateway`.
    let outcome = checkout_service.process_checkout(order, &cc_gateway);

    // Idiomatic Error Handling: Exhaustive pattern matching on `Result`.
    match outcome {
        Ok(id) => {
            let saved = checkout_service.get_order(id).unwrap();
            println!("\nPersisted state: {:?}", saved.status);
        }
        Err(err) => eprintln!("Checkout failed: {}", err),
    }
}
```

## 3. SOLID Principles: Java vs. Rust

### `S`: Single Responsibility Principle (SRP)

- **In Java:**  Classes can attract unrelated concerns (e.g., an `@Entity` class containing JPA mappings, Jackson JSON annotations, domain validations, and business methods).
- **In Rust:** SRP is enforced by separating state storage from domain operations and wire-format serialization.

```rust
// 1. Pure Domain State (Domain Layer)
pub struct Order {
    pub id: u64,
    pub total: u64,
}

// 2. Business Logic (Inherent implementation)
impl Order {
    pub fn apply_discount(&mut self, percentage: u32) {
        self.total = self.total * (100 - percentage as u64) / 100;
    }
}

// 3. Serialization Responsibility (Infrastructure Layer via Serde traits)
// Separated cleanly from core business rules without polluting the struct logic.
impl serde::Serialize for Order {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Custom serialization logic isolated from domain rules
        todo!()
    }
}
```

### `O`: Open/Closed Principle (OCP)

- **In Java:**  
  Openness to extension is achieved by subclassing open classes or implementing interfaces.  
  Adding behavior to third-party types requires decorator or adapter wrapper classes.
- **In Rust:**  
  Traits provide **retroactive implementation** governed by the **Orphan Rule** (you can implement any trait for any type, provided either the trait or the type is local to your crate).

```rust
// Contract defined in your crate
pub trait RenderHtml {
    fn to_html(&self) -> String;
}

// Retroactive extension of standard library types (impossible in Java without wrapper objects)
impl RenderHtml for u64 {
    fn to_html(&self) -> String {
        format!("<span class=\"numeric\">{}</span>", self)
    }
}

impl RenderHtml for Order {
    fn to_html(&self) -> String {
        format!("<div>Order: {} | Total: {}</div>", self.order_id, self.total_cents().to_html())
    }
}
```

### `L`: Liskov Substitution Principle (LSP)

- **In Java:** Subclasses must conform to the behavioral expectations of their supertypes. Compilers cannot prevent overridden methods from throwing runtime exceptions (e.g., `UnsupportedOperationException`).
- **In Rust:** Rust replaces inheritance with trait-based capabilities. If a type implements a trait, it must strictly satisfy the associated types, lifetime parameters, and method signatures without class-slicing risks.

```rust
pub trait Reader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error>;
}

// Any type implementing Reader can substitute for another at compile time or run time.
// There is no risk of partial state slicing or unexpected base-class downcasting bugs.
pub fn parse_header<R: Reader>(source: &mut R) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = [0u8; 128];
    source.read(&mut buffer)?;
    Ok(buffer.to_vec())
}
```

### `I`: Interface Segregation Principle (ISP)

- **In Java:** Broad interfaces often lead to large API contracts (e.g., `MouseListener` with empty method stubs in adapter classes).
- **In Rust:** Traits default to single-responsibility contracts, composed using **Supertraits** (`trait B: A`) and **Trait Bounds** (`T: Read + Write`).

```rust
// Standard library examples of ultimate ISP:
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

// Functions depend strictly on the minimal capability required:
pub fn save_to_disk<W: Write>(writer: &mut W, data: &[u8]) -> std::io::Result<()> {
    writer.write_all(data)?;
    writer.flush()
}
```

### `D`: Dependency Inversion Principle (DIP)

- **In Java:** High-level modules inject interface abstractions resolved via Spring's `ApplicationContext` using runtime reflection.
- **In Rust:** Abstractions are enforced at the module boundary. Dependency Injection is resolved explicitly at compile time via generics or at runtime via trait objects.

```rust
// Abstraction
pub trait Clock {
    fn now_millis(&self) -> u64;
}

// High-level module depends on abstraction, NOT concrete system time
pub struct TokenService<C: Clock> {
    clock: C,
}

impl<C: Clock> TokenService<C> {
    pub fn new(clock: C) -> Self {
        Self { clock }
    }

    pub fn is_expired(&self, expiry_millis: u64) -> bool {
        self.clock.now_millis() > expiry_millis
    }
}
```

## 4. Behavioral Design Patterns in Rust

### 1. Strategy Pattern

In Java, Strategy requires an interface and multiple concrete class declarations.  
In Rust, you choose between **generics** (compile-time inlining), **closures** (ad-hoc strategies), or **enums** (closed sets).

#### Closure-Based Strategy (Micro-Strategies)

```rust
pub struct TaxCalculator;

impl TaxCalculator {
    pub fn compute_tax<F>(&self, gross_cents: u64, strategy: F) -> u64 
    where
        F: Fn(u64) -> u64,
    {
        strategy(gross_cents)
    }
}

// Usage: Strategy passed directly as an inline lambda closure:
let calc = TaxCalculator;
let ny_tax = calc.compute_tax(10_000, |gross| gross * 8 / 100);
let non_profit = calc.compute_tax(10_000, |_| 0);
```

#### Closed-Set Strategy (Algebraic Data Types)

```rust
pub enum DiscountStrategy {
    Percentage(u32),
    Flat(u64),
    BuyOneGetOneFree,
}

impl DiscountStrategy {
    pub fn apply(&self, subtotal: u64) -> u64 {
        match self {
            Self::Percentage(pct) => subtotal - (subtotal * (*pct as u64) / 100),
            Self::Flat(amount) => subtotal.saturating_sub(*amount),
            Self::BuyOneGetOneFree => subtotal / 2,
        }
    }
}
```

### 2. State Pattern vs The Typestate Pattern

In Java, the State pattern uses polymorphic pointers to an abstract `State` interface, requiring runtime checks or invalid-state exceptions.

Rust introduces the **Typestate Pattern**, transforming runtime checks into **compile-time invariants** using Zero-Sized Types (ZSTs) and ownership transitions.

```text
       Order<Draft>  ──.submit()──>  Order<Submitted>  ──.pay()──>  Order<Paid>
       (only has                     (only has                      (cannot be
        .add_item())                  .pay())                        paid again)
```

```rust
// 1. Discrete state tokens (Zero-Sized Types)
pub struct Draft;
pub struct Submitted;
pub struct Paid { pub transaction_id: String }

// 2. Context parameterized by compile-time state
pub struct OrderRecord<State> {
    pub id: u64,
    pub amount_cents: u64,
    state: State,
}

// 3. Methods permitted ONLY during Draft state
impl OrderRecord<Draft> {
    pub fn new(id: u64, amount_cents: u64) -> Self {
        Self { id, amount_cents, state: Draft }
    }

    // State transition consumes `self` by value, destroying the Draft instance
    pub fn submit(self) -> OrderRecord<Submitted> {
        OrderRecord {
            id: self.id,
            amount_cents: self.amount_cents,
            state: Submitted,
        }
    }
}

// 4. Methods permitted ONLY during Submitted state
impl OrderRecord<Submitted> {
    pub fn pay(self, tx_id: String) -> OrderRecord<Paid> {
        OrderRecord {
            id: self.id,
            amount_cents: self.amount_cents,
            state: Paid { transaction_id: tx_id },
        }
    }
}

// 5. Methods permitted in the final Paid state
impl OrderRecord<Paid> {
    pub fn receipt(&self) -> &str {
        &self.state.transaction_id
    }
}

// --- Practical Compilation Guarantee ---
fn typestate_demo() {
    let order = OrderRecord::new(500, 10_000);
    
    // COMPILE ERROR: Method `pay` does not exist on `OrderRecord<Draft>`:
    // order.pay("TX_1".into());

    let submitted = order.submit();
    
    // COMPILE ERROR: `order` was moved above and can no longer be accessed:
    // order.submit(); 

    let paid = submitted.pay("TX_VALID_123".into());
    println!("Transaction Receipt: {}", paid.receipt());
}
```

### 3. Observer Pattern

The domain model demonstrates an idiomatic dynamic observer implementation using `Vec<Box<dyn OrderObserver>>`:

- Observers decouple the core domain service from side effects (email, messaging, auditing).
- Observers hold minimal or zero local state (ZSTs).
- The service dispatches across runtime vtables without coupling to concrete listener implementations.

### 4. Command Pattern

In Java, commands wrap behaviors in command objects implementing an `execute()` method.  
In Rust, standard library primitives like `Box<dyn FnOnce()>` encapsulate self-contained deferred operations.

```rust
pub struct CommandQueue {
    tasks: Vec<Box<dyn FnOnce(&mut InMemoryOrderRepository)>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn push<F>(&mut self, task: F)
    where
        F: FnOnce(&mut InMemoryOrderRepository) + 'static,
    {
        self.tasks.push(Box::new(task));
    }

    pub fn flush_all(&mut self, repo: &mut InMemoryOrderRepository) {
        for task in self.tasks.drain(..) {
            task(repo);
        }
    }
}
```

### 5. Template Method Pattern

Implemented in Rust via default trait methods.  
The trait dictates the algorithm's skeleton, while implementors provide step-specific hooks.

```rust
pub trait DeploymentPipeline {
    fn run_tests(&self) -> bool;
    fn deploy_artifacts(&self);

    // Template method defining the invariant sequence
    fn execute(&self) -> Result<(), String> {
        println!("Starting deployment sequence...");
        if !self.run_tests() {
            return Err("Tests failed. Aborting deployment.".to_string());
        }
        self.deploy_artifacts();
        println!("Deployment finalized successfully.");
        Ok(())
    }
}
```

## 5. Low-Level Design (LLD) & Entity Modeling

Coming from Java and ORMs (Hibernate/JPA), engineers often bring assumptions that trigger friction with Rust's borrow checker:

```text
            Java/JPA Anti-Pattern in Rust: Cyclical Object Graph
            ┌───────────────────────────────────────────────┐
            │                                               │
            ▼                                               │
      ┌───────────┐         orders         ┌─────────────┐  │
      │  Customer │ ─────────────────────> │    Order    │  │ customer
      └───────────┘                        └─────────────┘  │
            ▲                                      │        │
            │                                      └────────┘
            └───────────────────────────────────────────────┘
```

### The Entity Relationship Mapping Guide

| Association Type       | Java/JPA Typical Approach        | Idiomatic Rust Representation                                      |
| ---------------------- | -------------------------------- | ------------------------------------------------------------------ |
| **One-to-One (1:1)**   | `@OneToOne Customer <-> Profile` | Direct struct composition: `profile: Profile` or `Option<Profile>` |
| **One-to-Many (1:N)**  | `@OneToMany List<Order>`         | Direct collection ownership: `items: Vec<LineItem>`                |
| **Many-to-One (N:1)**  | `@ManyToOne Customer customer`   | Identity Association: `customer_id: CustomerId`                    |
| **Many-to-Many (N:M)** | `@ManyToMany Set<Role>`          | Foreign key join table: `HashMap<UserId, Vec<RoleId>>`             |

### Modeling Guidelines

#### 1. Avoid Bidirectional Object Graphs

In Java, an `Order` points to its `Customer`, and the `Customer` holds a collection of its `Order` instances.  
In Rust, this cyclical ownership violates the Single Ownership Principle.  
You cannot borrow the `Customer` mutably while an `Order` holds an immutable reference to it.

- **Solution:** Model relationships using **Identity Types (Foreign Keys)**.

```rust
// WRONG: Cyclical Reference Trap
// struct Customer { orders: Vec<Order> }
// struct Order { customer: Rc<RefCell<Customer>> }

// CORRECT: Identity Association
pub struct Customer {
    pub id: CustomerId,
    pub name: String,
}

pub struct Order {
    pub order_id: u64,
    pub customer_id: CustomerId, // Reference by ID
}
```

#### 2. Deep Composition for Aggregates

If an entity has exclusive ownership of its components (e.g., `Order` -> `LineItem`), embed them directly inside the struct.

```rust
pub struct Order {
    pub id: u64,
    pub items: Vec<LineItem>, // Contiguous heap array, zero pointer chasing
}
```

- Memory layout is cache-friendly: the line items live contiguously inside a single heap allocation rather than scattered across JVM heap addresses.

#### 3. Modeling Circular Graphs via Arenas or Repositories

If domain requirements demand an interconnected graph (e.g., navigation paths or complex workflow nodes), resolve links through an index or repository:

```rust
pub struct GraphNode {
    pub id: usize,
    pub outgoing_edges: Vec<usize>, // Store indices/IDs, not raw references or pointers
}

pub struct NetworkGraph {
    nodes: Vec<GraphNode>,
}

impl NetworkGraph {
    pub fn get_neighbors(&self, node_id: usize) -> impl Iterator<Item = &GraphNode> {
        self.nodes[node_id]
            .outgoing_edges
            .iter()
            .map(|&idx| &self.nodes[idx])
    }
}
```

## 6. Enterprise Project Architecture

Structuring an enterprise Rust application maps cleanly to layered architecture, replacing runtime IoC containers with compile-time dependency injection.

### Directory & Module Structure

```text
my_service/
├── Cargo.toml
├── src/
│   ├── main.rs                   # Composition root: wires concrete dependencies
│   ├── lib.rs                    # Re-exports modules
│   ├── domain/                   # Pure business logic (structs, enums, traits)
│   │   ├── mod.rs
│   │   ├── entities.rs           # Customer, Order, LineItem
│   │   ├── status.rs             # OrderStatus enum
│   │   └── repository.rs         # OrderRepository trait
│   ├── infrastructure/           # Concrete adapters (DB, external APIs)
│   │   ├── mod.rs
│   │   ├── persistence/
│   │   │   ├── mod.rs
│   │   │   └── postgres_repo.rs  # impl OrderRepository for Postgres
│   │   └── gateways/
│   │       ├── mod.rs
│   │       └── stripe.rs         # impl PaymentGateway for Stripe
│   └── application/              # Orchestration layer (services/use-cases)
│       ├── mod.rs
│       └── checkout_service.rs   # CheckoutService<R: OrderRepository>
```

### The Composition Root Pattern (Replacing Spring Boot `@Configuration`)

Spring Boot dynamically scans packages, resolves `@Autowired` dependencies, and executes lifecycle hooks using reflection.

Rust wires systems deterministically at startup inside `main.rs`:

```rust
// src/main.rs

use my_service::infrastructure::persistence::InMemoryOrderRepository;
use my_service::infrastructure::gateways::CreditCardGateway;
use my_service::infrastructure::notifications::{EmailNotifier, AuditLogger};
use my_service::application::CheckoutService;

fn main() {
    // 1. Instantiate low-level adapters (Infrastructure Layer)
    let repository = InMemoryOrderRepository::default();
    let payment_gateway = CreditCardGateway {
        merchant_id: "MERCHANT_KEY_123".into(),
    };

    // 2. Wire dependencies explicitly into the application service
    let mut checkout_service = CheckoutService::new(repository);

    // 3. Attach dynamic plugins/observers
    checkout_service.register_observer(Box::new(EmailNotifier));
    checkout_service.register_observer(Box::new(AuditLogger));

    // 4. Start HTTP Server or CLI runner passing application services
    println!("Application context initialized successfully with deterministic wiring.");
}
```

## 7. Comparative Quick-Reference Matrix

| Concept                    | Java / Spring Boot                         | Idiomatic Rust                                                   |
| -------------------------- | ------------------------------------------ | ---------------------------------------------------------------- |
| **Class**                  | `class User { ... }`                       | Split into data (`struct User`) and logic (`impl User`)          |
| **Interface**              | `public interface Worker`                  | `pub trait Worker`                                               |
| **Subclassing**            | `class Car extends Vehicle`                | Composition (`struct Car { vehicle: Vehicle }`) + Traits         |
| **Dynamic Polymorphism**   | Default virtual methods (`invokevirtual`)  | Explicit opt-in via trait objects (`dyn Trait`)                  |
| **Static Polymorphism**    | Java Generics (Type erasure)               | Monomorphization (`<T: Trait>`, zero runtime overhead)           |
| **Dependency Injection**   | `@Autowired`, Spring IoC container         | Constructor injection (`Service::new(repo)`) at composition root |
| **Nullable References**    | `User user = null;` (Risks NPE)            | `Option<User>` (Compiler forces exhaustive checking)             |
| **Lifecycle States**       | Status fields + runtime validation         | Typestate Pattern (`Order<Draft>` -> `Order<Paid>`)              |
| **Entity State Sharing**   | Base abstract class (`BaseEntity`)         | Composed child struct (`audit: AuditMetadata`)                   |
| **Cross-Cutting Concerns** | Spring AOP / Dynamic Bytecode Proxies      | Middleware traits, functional combinators, declarative macros    |

---
