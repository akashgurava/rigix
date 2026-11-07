# Rigix

Rigix is an opinionated, compile-time-enforced Rust application framework and metaprogramming DSL.

## Entity Macro (`#[entity]`)

The `#[entity]` macro defines domain entities and derives their associated structures:

```rust
use rigix::entity;
use uuid::Uuid;

#[entity]
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
