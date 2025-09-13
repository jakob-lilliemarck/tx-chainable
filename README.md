# tx-chainable

A Rust library that implements the chainable repository pattern, hiding storage backend implementation details from service-layer code.

## Overview

`tx-chainable` provides traits and patterns that enable repositories to be composed while keeping transaction management within the repository layer. This maintains proper Domain-Driven Design (DDD) separation of concerns where services focus on business logic and repositories handle data persistence.

## Key Features

- **Repository Composition**: Chain operations across multiple repositories within a single transaction
- **Clean Architecture**: Keeps storage backend details (transactions, connection pools) hidden from services
- **DDD Compliance**: Maintains proper separation between domain services and data access layers
- **Type Safety**: Leverages Rust's type system to ensure transaction safety at compile time

## The Pattern

**Challenge**: When you need to coordinate operations across multiple repositories within a single transaction, you face a design dilemma:

- **Option A**: Pass transactions between repositories, coupling them to storage implementation
- **Option B**: Handle transaction management in services, leaking storage concerns upward

```rust
// Option A: Repository coupling to transaction types
let mut tx = pool.begin().await?;
user_repo.create_user_with_tx(&mut tx, user).await?;
event_repo.create_event_with_tx(&mut tx, event).await?;
tx.commit().await?;

// Option B: Service manages transactions directly
let mut tx = pool.begin().await?;
user_repo.create_user(&mut tx, user).await?;
event_repo.create_event(&mut tx, event).await?;
tx.commit().await?;
```

**Solution**: This crate provides a third option - repository composition that maintains clean boundaries:

```rust
// ✅ Clean composition - transaction details stay in repositories
user_repo.begin(|users| {
    users.chain(&event_repo, |events| {
        // Pure business logic, ACID guarantees preserved
    })
}).await?
```

## Core Traits

- **`Tx`**: Marks a repository as transaction-capable with associated transaction types
- **`Chainable`**: Enables composition of repositories within the same transaction context
- **`Begin`**: Provides transaction lifecycle management while hiding implementation details
- **`Execute`**: Abstracts over different executor types (pools vs transactions)

## Usage

```rust
use tx_chainable::{Begin, Chainable};

// Service layer - focuses purely on business logic
pub async fn transfer_user_with_audit(
    users_repo: &UsersRepository<PgPool>,
    events_repo: &EventsRepository<PgPool>,
    user_id: Uuid,
    new_name: String,
) -> Result<(), sqlx::Error> {
    // Transaction management stays in repository layer
    users_repo.begin(|users| {
        Box::pin(async move {
            // Update user
            let updated_user = users.update_user_name(user_id, new_name).await?;
            
            // Chain to events repo for audit logging
            let users = users.chain(&events_repo, |mut events| {
                Box::pin(async move {
                    events.create_audit_event("user_updated", &updated_user).await?;
                    Ok(events)
                })
            }).await?;
            
            Ok(users)
        })
    }).await
}
```

## Examples

See the `integration/` directory for complete working examples, including:

- **Repository implementations** - Shows how to implement the `Tx`, `Chainable`, and `Begin` traits
- **Service layer examples** - Demonstrates clean business logic without transaction management
- **DDD patterns** - Illustrates proper separation of concerns between services and repositories
- **Comprehensive tests** - Validates transaction semantics and rollback behavior

## Running Tests

To run the integration tests:

1. Set up a PostgreSQL database
2. Set the `DATABASE_URL` environment variable
3. Run: `cargo test --package tx_chainable_integration`

The tests use `sqlx::test` which automatically runs migrations and provides isolated test environments.

## Requirements

- Rust 2021 edition
- PostgreSQL database
- SQLx with async support

## TODOs

- **Reduce Box::pin ceremony** - Add convenience macro to eliminate boilerplate `Box::pin(async move { ... })` wrapping
- **Abstract database coupling** - Generalize from PostgreSQL-only to support multiple databases (MySQL, SQLite)
- **Custom error types** - Allow repositories to define domain-specific error types instead of being locked to `sqlx::Error`
- **Simplify Execute trait** - Consider removing `Execute` trait in favor of using SQLx's native executor traits directly
- **Add proc macro support** - Explore `#[transaction]` attribute macro for even cleaner syntax
