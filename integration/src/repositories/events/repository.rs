use crate::repositories::events::models::Event;
use sqlx::{PgPool, PgTransaction};
use tx_chainable::{Execute, Tx, GetExecutor};
use uuid::Uuid;

#[derive(Clone)]
pub struct EventsRepository<E: Execute> {
    executor: E,
}

impl<E: Execute> Tx for EventsRepository<E> {
    type TxRepository<'tx> = EventsRepository<PgTransaction<'tx>>;
}

impl<'tx> GetExecutor<'tx> for EventsRepository<PgPool> {
    type Executor = &'tx PgPool;
    fn get_executor(&'tx self) -> Self::Executor {
        &self.executor
    }
}

impl<'tx> Into<PgTransaction<'tx>> for EventsRepository<PgTransaction<'tx>> {
    fn into(self) -> PgTransaction<'tx> {
        self.executor
    }
}

impl<'tx> From<PgTransaction<'tx>> for EventsRepository<PgTransaction<'tx>> {
    fn from(tx: PgTransaction<'tx>) -> Self {
        Self { executor: tx }
    }
}

impl EventsRepository<PgPool> {
    pub fn new(pool: PgPool) -> Self {
        Self { executor: pool }
    }
}

impl<E: Execute> EventsRepository<E> {
    pub async fn get_events(&mut self, limit: i64) -> Result<Vec<Event>, sqlx::Error> {
        self.executor
            .execute(|e| {
                sqlx::query_as::<_, Event>(
                    "SELECT id, name, payload FROM events ORDER BY name, id LIMIT $1"
                )
                .bind(limit)
                .fetch_all(e)
            })
            .await
    }

    pub async fn create_event(&mut self, id: Uuid, name: String, payload: serde_json::Value) -> Result<Event, sqlx::Error> {
        self.executor
            .execute(|e| {
                sqlx::query_as::<_, Event>(
                    "INSERT INTO events (id, name, payload) VALUES ($1, $2, $3) RETURNING id, name, payload"
                )
                .bind(&id)
                .bind(&name)
                .bind(&payload)
                .fetch_one(e)
            })
            .await
    }
}
