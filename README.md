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

## Response Derive (`#[derive(Response)]`)

The `#[derive(Response)]` macro defines encapsulated outbound response DTOs:

```rust
use rigix::Response;
use uuid::Uuid;

#[derive(Response)]
struct UserResponse {
    id: i64,
    uuid: Uuid,
    name: String,
}

// Emits:
// res.id() -> i64
// res.uuid() -> Uuid
// res.name() -> &str
// res.into_parts() -> (i64, Uuid, String)
```

## Development

```bash
./dev.sh check   # Run format check, clippy, and tests
./dev.sh fmt     # Format code
./dev.sh expand  # Expand macros
```
