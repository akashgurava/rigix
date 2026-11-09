# Rigix

Rigix is an opinionated, compile-time-enforced Rust application framework and metaprogramming DSL.

## Request Derive (`#[derive(Request)]`)

The `#[derive(Request)]` macro defines encapsulated inbound request/command DTOs:

```rust
use rigix::Request;
use uuid::Uuid;

#[derive(Request)]
struct CreateUser {
    id: i64,
    uuid: Uuid,
    name: String,
}

// Emits:
// req.id() -> i64
// req.uuid() -> Uuid
// req.name() -> &str
// req.into_parts() -> (i64, Uuid, String)
```

## Model Derive (`#[derive(Model)]`)

The `#[derive(Model)]` macro defines encapsulated domain models and response representations:

```rust
use rigix::Model;
use uuid::Uuid;

#[derive(Model)]
struct User {
    id: i64,
    uuid: Uuid,
    name: String,
}

// Emits:
// user.id() -> i64
// user.uuid() -> Uuid
// user.name() -> &str
// user.into_parts() -> (i64, Uuid, String)
```

## Development

```bash
./dev.sh check   # Run format check, clippy, and tests
./dev.sh fmt     # Format code
./dev.sh expand  # Expand macros
```
