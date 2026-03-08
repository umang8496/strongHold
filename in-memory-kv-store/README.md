# VelocityCache – A Rust-Based In-Memory Cache with Durability and Observability

## Overview

As part of my exploration of backend infrastructure and storage systems, I implemented **VelocityCache**, a lightweight in-memory caching service written in **Rust**.  

The objective of this project was to design and build a **high-performance, concurrent cache service** that exposes REST APIs, supports metadata tracking, and provides durability through a **Write-Ahead Log (WAL)** mechanism.

The implementation emphasizes **correctness, concurrency, observability, and crash resilience**, drawing inspiration from architectural patterns used in systems such as **Redis, RocksDB, and PostgreSQL**.

## Key Features

**VelocityCache** provides the following capabilities:

- **In-memory key-value storage**
- **Concurrent reads with controlled writes**
- **Sliding TTL (time-to-live) expiration**
- **Per-key metadata tracking**
- **Global cache statistics**
- **Background eviction of expired keys**
- **Write-Ahead Logging for durability**
- **Crash recovery via WAL replay**
- **Structured request logging**

These features collectively demonstrate core backend infrastructure concepts such as **durability, concurrency control, and system observability**.

## Build and Run

```sh
>>> cargo build
>>> RUST_LOG=info cargo run
```

## Architecture

The system follows a layered architecture separating API handling, storage, and background processing.

```text
Client Services
│
▼
REST API Layer (Actix Web)
│
▼
In-Memory Cache Engine
│
├── Value Store
├── Metadata Store
└── Statistics Counters
│
▼
Background Workers
├── TTL Eviction Worker
└── WAL Writer
│
▼
Persistent WAL Log
```

### Components

#### API Layer

Implemented using the **Actix Web** framework.  
Provides endpoints for:

- Setting cache values
- Fetching values
- Deleting keys
- Accessing metadata
- Retrieving system statistics
- Health checks

#### Value Store

The primary cache data structure:

```sh
HashMap<String, CacheEntry>
```

Stored inside a `RwLock` to support:

- **Concurrent reads**
- **Exclusive writes**

This design reflects typical cache workloads where **reads significantly outnumber writes**.

#### Metadata Store

A separate store tracks metadata associated with each key.  
Metadata fields include:

- `created_at`
- `updated_at`
- `last_accessed_at`
- `frequency`
- `size`
- `ttl`

This separation allows metadata to evolve independently without affecting the main value store.  

#### Sliding TTL Expiration

VelocityCache implements **sliding expiration**, where keys expire only if they remain inactive.  

```sh
expiry_time = last_accessed_at + ttl
```

Every `GET` request updates the access timestamp, effectively extending the lifetime of frequently accessed keys.  

#### Background Cleanup Worker

A periodic background task scans metadata and removes keys that have exceeded their TTL.  
This ensures that:

- Expired keys are cleaned automatically
- No cleanup is required during read operations

#### Write-Ahead Logging (WAL)

All mutations are written to a **Write-Ahead Log** before being applied to memory.  
Example WAL entries:

```sh
PUT user123 Alice 60
DELETE user456
```

The WAL guarantees that state changes can be reconstructed after a crash.  

#### Asynchronous WAL Writer

To avoid blocking request threads with disk operations, WAL writes are performed asynchronously.  

Architecture:

```sh
API Threads
│
▼
WAL Channel Queue
│
▼
WAL Writer Task
│
▼
Disk Log File
```

This approach significantly improves throughput by **batching disk writes**.  

#### Crash Recovery

Upon startup, the system replays the WAL file to reconstruct the in-memory state.  
Replay logic processes operations sequentially:

```sh
PUT operations → insert/update keys
DELETE operations → remove keys
```

This ensures the cache can recover from unexpected process termination.

#### Global Cache Statistics

`VelocityCache` maintains global counters using atomic primitives.  

Metrics tracked include:

- Total requests
- Cache hits
- Cache misses
- Set operations
- Delete operations
- Total active keys

These statistics are accessible through a dedicated API endpoint.

### API Endpoints

#### Health Check

> GET /velocitycache/health

Used for service monitoring and orchestration systems.

#### Set Key

> PUT /velocitycache/cache/{key}

```sh
Request body:

```json
{
  "value": "example",
  "ttl": 60
}
```

#### Get Key

> GET /velocitycache/cache/{key}

```sh
Response:

{
  "key": "example",
  "value": "data"
}
```

#### Delete Key

> DELETE /velocitycache/cache/{key}

#### Metadata

> GET /velocitycache/cache/{key}/metadata

```sh
Example response:
{
  "created_at": "2026-03-07T16:11:56Z",
  "updated_at": "2026-03-07T16:11:56Z",
  "last_accessed_at": "2026-03-07T16:12:02Z",
  "frequency": 1,
  "size": 6,
  "ttl": 60
}
```

#### Cache Statistics

> GET /velocitycache/stats

```sh
Example response:
{
  "total_requests": 120,
  "hits": 80,
  "misses": 40,
  "sets": 25,
  "deletes": 10,
  "total_keys": 15
}
```

### Concurrency Strategy

Concurrency was addressed using:

- RwLock for read-heavy access patterns
- atomic counters for metrics
- asynchronous channels for WAL batching
- background workers for cleanup

This design reduces contention while keeping the implementation understandable.  

### Design Considerations

Several trade-offs were evaluated during development:

#### Durability vs Performance

Strict durability (fsync per write) significantly reduces throughput.  
The chosen approach uses asynchronous WAL batching for improved performance.  

#### Simplicity vs Feature Completeness

The goal was to demonstrate core storage system concepts without introducing unnecessary complexity such as distributed replication.  

### Potential Future Enhancements

The current implementation can be extended in several directions:

- Snapshotting and WAL compaction
- LRU-based eviction policies
- Consistent hashing for distributed sharding
- gRPC interface for lower latency
- Replication across multiple cache nodes
- Metrics integration with Prometheus

## Conclusion

**VelocityCache** demonstrates how fundamental backend infrastructure concepts can be implemented using Rust's strong concurrency guarantees and memory safety.

---
