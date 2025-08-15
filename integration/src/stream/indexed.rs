// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
// use markets::index::error::IndexError;
use derive_more::Constructor;
use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait Indexer {
    type Unindexed;
    type Indexed;
    fn index(&self, item: Self::Unindexed) -> Result<Self::Indexed, String>;
}

#[derive(Debug, Constructor)]
#[pin_project]
pub struct IndexedStream<Indexer, Stream> {
    #[pin]
    pub stream: Stream,
    pub indexer: Indexer,
}

impl<Index, St> Stream for IndexedStream<Index, St>
where
    Index: Indexer<Unindexed = St::Item>,
    St: Stream,
{
    type Item = Result<Index::Indexed, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(this.indexer.index(item))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
