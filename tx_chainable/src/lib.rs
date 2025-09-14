use sqlx::{Acquire, PgExecutor, PgPool, PgTransaction};
use std::future::Future;
use std::pin::Pin;

pub mod tx_chain;

pub type BoxFuture<'tx, T> = Pin<Box<dyn Future<Output = T> + Send + 'tx>>;

pub trait Execute {
    type Executor<'tx>: PgExecutor<'tx> + Acquire<'tx>;

    fn execute<'tx, F, Fut, T>(&'tx mut self, f: F) -> Fut
    where
        F: FnOnce(Self::Executor<'tx>) -> Fut,
        Fut: Future<Output = T> + Send,
        T: Send;
}

impl Execute for PgPool {
    type Executor<'tx> = &'tx PgPool;

    fn execute<'tx, F, Fut, T>(&'tx mut self, f: F) -> Fut
    where
        F: FnOnce(Self::Executor<'tx>) -> Fut,
        Fut: Future<Output = T> + Send,
        T: Send,
    {
        f(self) // &PgPool implements Executor
    }
}

impl<'t> Execute for sqlx::PgTransaction<'t> {
    type Executor<'tx> = &'tx mut sqlx::PgConnection;

    fn execute<'tx, F, Fut, T>(&'tx mut self, f: F) -> Fut
    where
        F: FnOnce(Self::Executor<'tx>) -> Fut,
        Fut: Future<Output = T> + Send,
        T: Send,
    {
        f(self.as_mut())
    }
}

pub trait GetExecutor<'tx> {
    type Executor: sqlx::PgExecutor<'tx>;
    fn get_executor(&'tx self) -> Self::Executor;
}

pub trait Tx {
    type TxRepository<'tx>: From<PgTransaction<'tx>> + Into<PgTransaction<'tx>>;
}

pub trait Chainable<'tx>: Tx {
    fn chain<Other, F>(
        self,
        other: &Other,
        f: F,
    ) -> BoxFuture<'tx, Result<Self::TxRepository<'tx>, sqlx::Error>>
    where
        Other: Tx,
        Other::TxRepository<'tx>: From<PgTransaction<'tx>>,
        Other::TxRepository<'tx>: Into<PgTransaction<'tx>> + Send + 'tx,
        F: FnOnce(
            Other::TxRepository<'tx>,
        ) -> BoxFuture<'tx, Result<Other::TxRepository<'tx>, sqlx::Error>>,
        Self: Sized;
}

impl<'tx, R> Chainable<'tx> for R
where
    R: Tx,
    R: Into<PgTransaction<'tx>>,
{
    fn chain<Other, F>(
        self,
        _: &Other,
        f: F,
    ) -> BoxFuture<'tx, Result<Self::TxRepository<'tx>, sqlx::Error>>
    where
        Other: Tx,
        Other::TxRepository<'tx>: From<PgTransaction<'tx>>,
        Other::TxRepository<'tx>: Into<PgTransaction<'tx>> + Send + 'tx,
        F: FnOnce(
            Other::TxRepository<'tx>,
        ) -> BoxFuture<'tx, Result<Other::TxRepository<'tx>, sqlx::Error>>,
        Self: Sized,
    {
        let tx = self.into();
        let repo = <Other as Tx>::TxRepository::from(tx);
        let fut = f(repo);
        Box::pin(async move {
            let repo_result = fut.await?;
            let tx = repo_result.into();
            Ok(Self::TxRepository::from(tx)) // This assumes From<PgTransaction>
        })
    }
}

pub trait Begin<'tx>: Tx {
    fn begin<F>(&'tx self, f: F) -> BoxFuture<'tx, Result<(), sqlx::Error>>
    where
        F: FnOnce(
                Self::TxRepository<'tx>,
            ) -> BoxFuture<'tx, Result<Self::TxRepository<'tx>, sqlx::Error>>
            + Send
            + 'tx,
        Self: Sized;
}

impl<'tx, R> Begin<'tx> for R
where
    R: Tx + GetExecutor<'tx>,
    R::Executor: sqlx::Acquire<'tx, Database = sqlx::Postgres>,
{
    fn begin<F>(
        &'tx self, // Now we can take &mut self
        f: F,
    ) -> BoxFuture<'tx, Result<(), sqlx::Error>>
    where
        F: FnOnce(
                Self::TxRepository<'tx>,
            ) -> BoxFuture<'tx, Result<Self::TxRepository<'tx>, sqlx::Error>>
            + Send
            + 'tx,
        Self: Sized,
    {
        let executor = self.get_executor();
        let fut = executor.begin();
        Box::pin(async move {
            let tx = fut.await?;
            let ret = f(Self::TxRepository::from(tx)).await?;
            let committed_tx = ret.into();
            committed_tx.commit().await?;
            Ok(())
        })
    }
}
