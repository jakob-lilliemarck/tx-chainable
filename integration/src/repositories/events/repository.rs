use crate::repositories::events::models::Event;
use sqlx::{PgPool, PgTransaction};
use tx_chainable::{
    tx_chain::{Begin, End, TxChain, TxType},
    BoxFuture, Execute, GetExecutor, Tx,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct EventsRepository<E: Execute> {
    executor: E,
}

// ============================================================
// ============================================================
#[derive(Debug)]
pub enum MyError {
    SqlxError(sqlx::Error),
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::SqlxError(err) => write!(f, "SQLx error: {}", err),
        }
    }
}

impl std::error::Error for MyError {}

impl TxType for EventsRepository<PgPool> {
    type Tx<'tx> = PgTransaction<'tx>;
    type TxType<'tx> = EventsRepository<Self::Tx<'tx>>;
}

impl<'a> Begin<'a> for EventsRepository<PgPool> {
    type Error = MyError;

    fn end() -> Box<dyn FnOnce(Self::Tx<'a>) -> BoxFuture<'a, Result<(), Self::Error>>> {
        Box::new(|tx| {
            Box::pin(async move {
                tx.commit().await.map_err(|err| MyError::SqlxError(err))?;
                Ok(())
            })
        })
    }

    fn begin<F>(
        self,
        f: F,
    ) -> BoxFuture<
        'a,
        Result<
            TxChain<'a, End<'a, Self::Tx<'a>, Self::Error>, Self::Tx<'a>, Self::Error>,
            Self::Error,
        >,
    >
    where
        F: FnOnce(&Self::TxType<'a>) -> BoxFuture<'a, Result<(), Self::Error>> + Send + 'a,
    {
        Box::pin(async move {
            let tx = self
                .executor
                .begin()
                .await
                .map_err(|e| MyError::SqlxError(e))?;

            let tx_type = Self::TxType::from(tx);
            f(&tx_type).await?;
            let tx = tx_type.into();

            let chain = TxChain::new(Self::end(), tx);

            Ok(chain)
        })
    }
}

// ============================================================
// ============================================================

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
                    "SELECT id, name, payload FROM events ORDER BY name, id LIMIT $1",
                )
                .bind(limit)
                .fetch_all(e)
            })
            .await
    }

    pub async fn create_event(
        &mut self,
        id: Uuid,
        name: String,
        payload: serde_json::Value,
    ) -> Result<Event, sqlx::Error> {
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
