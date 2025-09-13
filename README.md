# tx_chainable

A Rust library that provides composable database transaction chaining for PostgreSQL using SQLx.

## Overview

`tx_chainable` enables you to chain database operations across different repositories within a single transaction, providing a clean and composable API for complex database workflows.

## Key Features

- **Transaction Chaining**: Chain operations across multiple repositories within a single transaction
- **Repository Pattern Support**: Works with the repository pattern, allowing repositories to be transaction-aware
- **Composable Operations**: Build complex database workflows by chaining simple operations
- **Type Safety**: Leverages Rust's type system to ensure transaction safety at compile time

## Core Traits

- **`Tx`**: Marks a repository as transaction-capable
- **`Chainable`**: Enables chaining operations across different repositories
- **`Begin`**: Starts a new transaction and executes a closure within it
- **`Execute`**: Provides a generic interface for executing database operations

## Usage

```rust
use tx_chainable::{Begin, Chainable};

// Start a transaction and chain operations
repository_a.begin(|repo_a| {
    Box::pin(async move {
        // Chain operations across different repositories
        let repo_a = repo_a
            .chain(&repository_b, |mut repo_b| {
                Box::pin(async move {
                    // Perform operations with repo_b
                    repo_b.some_operation().await?;
                    Ok(repo_b)
                })
            })
            .await?;
            
        Ok(repo_a)
    })
}).await?;
```

## Examples

See the `integration/` directory for complete working examples and integration tests, including:

- Repository implementations that support transaction chaining
- Database migration for setting up test data
- Integration tests demonstrating usage patterns for composing complex database operations
- Tests using `sqlx::test` for isolated database testing

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
