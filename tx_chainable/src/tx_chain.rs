use crate::BoxFuture;
use std::error::Error;

pub trait TxType {
    type Tx<'tx>: Send
    where
        Self: 'tx;

    type TxType<'tx>: From<Self::Tx<'tx>> + Into<Self::Tx<'tx>>
    where
        Self: 'tx;
}

pub type End<'a, Tx, Err> = Box<dyn FnOnce(Tx) -> BoxFuture<'a, Result<(), Err>>>;

pub trait Begin<'a>: TxType
where
    Self: 'a,
{
    type Error: std::error::Error;

    fn end() -> End<'a, Self::Tx<'a>, Self::Error>;

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
        F: FnOnce(&Self::TxType<'a>) -> BoxFuture<'a, Result<(), Self::Error>> + Send + 'a;
}

// A "reference counted transaction", keeps tracks of the number of open closures using it
// When the counter reaches zero, the transaction is committed.
pub struct TxChain<'a, Cb, Tx, Err>
where
    Err: Error,
    Cb: FnOnce(Tx) -> BoxFuture<'a, Result<(), Err>>,
    Tx: Send + 'a,
{
    end: Cb,
    tx: Tx,
}

impl<'a, Cb, Tx, Err> TxChain<'a, Cb, Tx, Err>
where
    Err: Error,
    Cb: FnOnce(Tx) -> BoxFuture<'a, Result<(), Err>>,
    Tx: Send + 'a,
{
    pub fn new(callback: Cb, tx: Tx) -> Self {
        TxChain { end: callback, tx }
    }

    pub async fn and<R, F>(mut self, _: &R, f: F) -> Result<Self, Err>
    where
        R: TxType + 'a,
        R::TxType<'a>: From<Tx> + Into<Tx> + Send,
        F: FnOnce(&R::TxType<'a>) -> BoxFuture<'a, Result<(), Err>> + Send + 'a,
    {
        let tx_type = R::TxType::from(self.tx);
        f(&tx_type).await?;
        self.tx = tx_type.into();

        Ok(self)
    }

    pub async fn end(self) -> Result<(), Err> {
        (self.end)(self.tx).await
    }
}

// begin begins a new transaction or creates a savepoint if there is already an active transaction
// next chains stores a closure on the stack
// end calls all closures on the stack in order and commits if all closures succeed

// When
// let repository = Repository::new(pool);
// repository
//   .begin(|tx_repository| { tx_repository.insert("key", "value"); })
//   .with(|tx_repository| { tx_repository.insert("key2", "value2"); })
//   .end()

// let a = Repository::new(pool);
// let b = Repository::new(pool);
// a.begin(|txa| {
//   txa.next(&b, |txb| {
//     txb.insert("key", "value")
//   })
// }).end()

// let a = Repository::new(pool);
// let b = Repository::new(pool);
// a.begin(|txa| {
//   txa.next(&b, |txb| {
//     txb.begin(|txb| {
//       txb.insert("key", "value")
//     })
//   })
// }).end()
