use tx_chainable::{Begin, Chainable};
use tx_chainable_integration::{Event, EventsRepository, User, UsersRepository};
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn test_single_repository_transaction(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let event_id = Uuid::new_v4();

    // Test that we can start a transaction and perform operations
    events_repo
        .begin(|mut events| {
            Box::pin(async move {
                // Create an event within the transaction
                let _event = events
                    .create_event(
                        event_id,
                        "single_repo_test".to_string(),
                        serde_json::json!({"message": "Single repository test"}),
                    )
                    .await?;
                Ok(events)
            })
        })
        .await?;

    // Verify the event was created
    let events = EventsRepository::new(pool).get_events(10).await?;
    assert_eq!(
        vec![Event {
            id: event_id,
            name: "single_repo_test".to_string(),
            payload: serde_json::json!({"message": "Single repository test"}),
        }],
        events
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cross_repository_chaining(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let user_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();

    // Test chaining operations across different repositories
    events_repo
        .begin(|events| {
            Box::pin(async move {
                let mut events = events
                    .chain(&users_repo, |mut users| {
                        Box::pin(async move {
                            // Create a user within the chained transaction
                            let _user = users
                                .create_user(user_id, "Cross Repository User".to_string())
                                .await?;
                            Ok(users)
                        })
                    })
                    .await?;

                let _event = events
                    .create_event(
                        event_id,
                        "cross_repo_test".to_string(),
                        serde_json::json!({"message": "Cross repository test"}),
                    )
                    .await?;
                Ok(events)
            })
        })
        .await?;

    // Verify the user was created
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert_eq!(
        vec![User {
            id: user_id,
            name: "Cross Repository User".to_string(),
        }],
        users
    );

    // Verify the event was created
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert_eq!(
        vec![Event {
            id: event_id,
            name: "cross_repo_test".to_string(),
            payload: serde_json::json!({"message": "Cross repository test"}),
        }],
        events
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_multiple_chains_in_transaction(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let user_1_id = Uuid::new_v4();
    let user_2_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();

    // Test multiple chains within the same transaction
    events_repo
        .begin(|events| {
            Box::pin(async move {
                // First chain to users
                let events = events
                    .chain(&users_repo, |mut users| {
                        Box::pin(async move {
                            let _user = users
                                .create_user(user_1_id, "Multiple Chains User 1".to_string())
                                .await?;
                            Ok(users)
                        })
                    })
                    .await?;

                // Second chain to users (reusing the same reference)
                let mut events = events
                    .chain(&users_repo, |mut users| {
                        Box::pin(async move {
                            let _user = users
                                .create_user(user_2_id, "Multiple Chains User 2".to_string())
                                .await?;
                            Ok(users)
                        })
                    })
                    .await?;

                // Final operation with events
                let _event = events
                    .create_event(
                        event_id,
                        "multiple_chains_test".to_string(),
                        serde_json::json!({"message": "Multiple chains test"}),
                    )
                    .await?;
                Ok(events)
            })
        })
        .await?;

    // Verify the users were created
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert_eq!(
        vec![
            User {
                id: user_1_id,
                name: "Multiple Chains User 1".to_string(),
            },
            User {
                id: user_2_id,
                name: "Multiple Chains User 2".to_string(),
            }
        ],
        users
    );

    // Verify the event was created
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert_eq!(
        vec![Event {
            id: event_id,
            name: "multiple_chains_test".to_string(),
            payload: serde_json::json!({"message": "Multiple chains test"}),
        }],
        events
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_users_repository_as_starter(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let event_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Test that we can start with users repo and chain to events repo
    users_repo
        .begin(|users| {
            Box::pin(async move {
                let mut users = users
                    .chain(&events_repo, |mut events| {
                        Box::pin(async move {
                            let _event = events
                                .create_event(
                                    event_id,
                                    "users_starter_test".to_string(),
                                    serde_json::json!({"message": "Users as starter test"}),
                                )
                                .await?;
                            Ok(events)
                        })
                    })
                    .await?;

                let _user = users
                    .create_user(user_id, "Users Starter User".to_string())
                    .await?;
                Ok(users)
            })
        })
        .await?;

    // Verify the event was created
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert_eq!(
        vec![Event {
            id: event_id,
            name: "users_starter_test".to_string(),
            payload: serde_json::json!({"message": "Users as starter test"}),
        }],
        events
    );

    // Verify the user was created
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert_eq!(
        vec![User {
            id: user_id,
            name: "Users Starter User".to_string(),
        }],
        users
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_transaction_rollback_on_error(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let user_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();

    // Test that transaction is rolled back when an error occurs
    let result = events_repo
        .begin(|events| {
            Box::pin(async move {
                let mut events = events
                    .chain(&users_repo, |mut users| {
                        Box::pin(async move {
                            // Create a user within the chained transaction
                            let _user = users
                                .create_user(user_id, "Rollback Test User".to_string())
                                .await?;
                            Ok(users)
                        })
                    })
                    .await?;

                // Create an event within the transaction
                let _event = events
                    .create_event(
                        event_id,
                        "rollback_test".to_string(),
                        serde_json::json!({"message": "This should be rolled back"}),
                    )
                    .await?;

                // Intentionally cause an error to trigger rollback
                Err(sqlx::Error::RowNotFound)
            })
        })
        .await;

    // Verify that the transaction failed
    assert!(result.is_err());

    // Verify that no users were created (transaction was rolled back)
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert!(
        users.is_empty(),
        "Users table should be empty after rollback"
    );

    // Verify that no events were created (transaction was rolled back)
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert!(
        events.is_empty(),
        "Events table should be empty after rollback"
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_error_during_chain_operation(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let user_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();

    // Test that error during chain operation properly rolls back everything
    let result = events_repo
        .begin(|mut events| {
            Box::pin(async move {
                // First, successfully create an event
                let _event = events
                    .create_event(
                        event_id,
                        "before_chain_error".to_string(),
                        serde_json::json!({"message": "This should be rolled back"}),
                    )
                    .await?;

                // Now chain to users repo and cause an error there
                let events = events
                    .chain(&users_repo, |mut users| {
                        Box::pin(async move {
                            // Create user successfully first
                            let _user = users
                                .create_user(user_id, "Chain Error User".to_string())
                                .await?;

                            // Then cause an error in the chained operation
                            Err(sqlx::Error::RowNotFound)
                        })
                    })
                    .await?; // This should propagate the error

                Ok(events)
            })
        })
        .await;

    // Verify that the transaction failed
    assert!(result.is_err());

    // Verify that no users were created (chain operation failed)
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert!(
        users.is_empty(),
        "Users table should be empty after chain error"
    );

    // Verify that no events were created (entire transaction rolled back)
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert!(
        events.is_empty(),
        "Events table should be empty after chain error"
    );

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_nested_chain_operations(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let users_repo = UsersRepository::new(pool.clone());
    let events_repo2 = events_repo.clone();
    let user_id = Uuid::new_v4();
    let event_1_id = Uuid::new_v4();
    let event_2_id = Uuid::new_v4();

    // Test chaining from within a chain operation (nested chains)
    events_repo
        .begin(|events| {
            Box::pin(async move {
                let events = events
                    .chain(&users_repo, |users| {
                        Box::pin(async move {
                            // Inside the users chain, chain back to events
                            let users = users
                                .chain(&events_repo2, |mut events_inner| {
                                    Box::pin(async move {
                                        let _event = events_inner
                                            .create_event(
                                                event_1_id,
                                                "nested_chain_event".to_string(),
                                                serde_json::json!({"message": "Event from nested chain"}),
                                            )
                                            .await?;
                                        Ok(events_inner)
                                    })
                                })
                                .await?;

                            // Create user after nested chain
                            let mut users = users;
                            let _user = users
                                .create_user(user_id, "Nested Chain User".to_string())
                                .await?;
                            Ok(users)
                        })
                    })
                    .await?;

                // Create another event in the main transaction
                let mut events = events;
                let _event = events
                    .create_event(
                        event_2_id,
                        "main_chain_event".to_string(),
                        serde_json::json!({"message": "Event from main chain"}),
                    )
                    .await?;
                Ok(events)
            })
        })
        .await?;

    // Verify the user was created
    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert_eq!(
        vec![User {
            id: user_id,
            name: "Nested Chain User".to_string(),
        }],
        users
    );

    // Verify both events were created
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert_eq!(2, events.len(), "Should have exactly 2 events");

    // Check both events exist (order might vary)
    let event_names: Vec<String> = events.iter().map(|e| e.name.clone()).collect();
    assert!(event_names.contains(&"nested_chain_event".to_string()));
    assert!(event_names.contains(&"main_chain_event".to_string()));

    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn test_empty_transaction(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());

    // Test that empty transaction works (no operations, just commit)
    events_repo
        .begin(|events| {
            Box::pin(async move {
                // Do nothing, just return the events repo
                Ok(events)
            })
        })
        .await?;

    // Verify no records were created
    let events = EventsRepository::new(pool.clone()).get_events(10).await?;
    assert!(events.is_empty(), "Events table should be empty");

    let users = UsersRepository::new(pool.clone()).get_users(10).await?;
    assert!(users.is_empty(), "Users table should be empty");

    Ok(())
}
