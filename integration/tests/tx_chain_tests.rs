use tx_chainable::tx_chain::{Begin, TxType};
use tx_chainable_integration::{Event, EventsRepository};
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn test_single_repository_transaction(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let events_repo = EventsRepository::new(pool.clone());
    let event_id = Uuid::new_v4();

    // Test that we can start a transaction and perform operations
    events_repo
        .begin(|mut events| Box::pin(async move { Ok(()) }))
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
