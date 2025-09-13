use crate::users::models::User;
use sqlx::{PgPool, PgTransaction};
use tx_chainable::{Execute, GetExecutor, Tx};
use uuid::Uuid;

#[derive(Clone)]
pub struct UsersRepository<E: Execute> {
    executor: E,
}

impl<E: Execute> Tx for UsersRepository<E> {
    type TxRepository<'tx> = UsersRepository<PgTransaction<'tx>>;
}

impl<'tx> GetExecutor<'tx> for UsersRepository<PgPool> {
    type Executor = &'tx PgPool;
    fn get_executor(&'tx self) -> Self::Executor {
        &self.executor
    }
}

impl<'tx> Into<PgTransaction<'tx>> for UsersRepository<PgTransaction<'tx>> {
    fn into(self) -> PgTransaction<'tx> {
        self.executor
    }
}

impl<'tx> From<PgTransaction<'tx>> for UsersRepository<PgTransaction<'tx>> {
    fn from(tx: PgTransaction<'tx>) -> Self {
        Self { executor: tx }
    }
}

impl UsersRepository<PgPool> {
    pub fn new(pool: PgPool) -> Self {
        Self { executor: pool }
    }
}

impl<E: Execute> UsersRepository<E> {
    pub async fn get_users(&mut self, limit: i64) -> Result<Vec<User>, sqlx::Error> {
        self.executor
            .execute(|e| {
                sqlx::query_as::<_, User>("SELECT id, name FROM users ORDER BY name, id LIMIT $1")
                    .bind(limit)
                    .fetch_all(e)
            })
            .await
    }

    pub async fn create_user(&mut self, id: Uuid, name: String) -> Result<User, sqlx::Error> {
        self.executor
            .execute(|e| {
                sqlx::query_as::<_, User>(
                    "INSERT INTO users (id, name) VALUES ($1, $2) RETURNING id, name",
                )
                .bind(&id)
                .bind(&name)
                .fetch_one(e)
            })
            .await
    }
}
