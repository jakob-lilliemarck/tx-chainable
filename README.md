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

## Usage Examples

### Single Repository Transaction
```rust
use tx_chainable::{Begin, Chainable};

// Simple transaction within one repository
events_repo.begin(|mut events| {
    Box::pin(async move {
        let event = events.create_event(
            event_id,
            "user_registered".to_string(), 
            serde_json::json!({"user_id": user_id})
        ).await?;
        Ok(events)
    })
}).await?;
```

### Cross-Repository Chaining
```rust
// Coordinate operations across multiple repositories
events_repo.begin(|events| {
    Box::pin(async move {
        // First, chain to users repository
        let mut events = events
            .chain(&users_repo, |mut users| {
                Box::pin(async move {
                    let user = users.create_user(user_id, "John Doe".to_string()).await?;
                    Ok(users)
                })
            })
            .await?;
        
        // Then create an audit event
        let event = events.create_event(
            event_id,
            "user_created".to_string(),
            serde_json::json!({"user_id": user_id})
        ).await?;
        
        Ok(events)
    })
}).await?;
```

### Multiple Chains (Complex Workflows)
```rust
// Chain multiple operations in sequence
events_repo.begin(|events| {
    Box::pin(async move {
        // Create first user
        let events = events
            .chain(&users_repo, |mut users| {
                Box::pin(async move {
                    users.create_user(user1_id, "Alice".to_string()).await?;
                    Ok(users)
                })
            })
            .await?;
            
        // Create second user (reusing the same transaction)
        let mut events = events
            .chain(&users_repo, |mut users| {
                Box::pin(async move {
                    users.create_user(user2_id, "Bob".to_string()).await?;
                    Ok(users)
                })
            })
            .await?;
            
        // Log the batch creation
        events.create_event(
            event_id,
            "batch_users_created".to_string(),
            serde_json::json!({"count": 2})
        ).await?;
        
        Ok(events)
    })
}).await?;
```

## Examples

See the `integration/` directory for working examples:

- **Repository implementations** - Shows how to implement the `Tx`, `Chainable`, and `Begin` traits for real repositories
- **Integration tests** - Demonstrates various chaining patterns and validates transaction semantics
- **Error handling** - Tests rollback behavior when operations fail

## Running Integration Tests

The `integration/` directory contains comprehensive tests demonstrating all usage patterns:

```bash
# Set up database URL (using direnv or export)
export DATABASE_URL="postgresql://postgres:password@localhost:5432/tx_chainable_test"

# Run all integration tests
cargo test --package tx-chainable-integration

# Run specific test patterns
cargo test --package tx-chainable-integration test_cross_repository_chaining
cargo test --package tx-chainable-integration rollback
```

The tests use `sqlx::test` which automatically:
- Creates isolated test database instances
- Runs migrations before each test
- Cleans up after test completion

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
