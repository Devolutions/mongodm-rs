//! `Cursor`s are used to stream result of a query.

use crate::Model;
use core::pin::Pin;
use futures_core::{task, Stream};
use mongodb::bson::{from_bson, Bson};

/// Streams the result of a query asynchronously for the given `Model`.
#[derive(Debug)]
pub struct ModelCursor<M: Model> {
    inner: mongodb::Cursor,
    _pd: std::marker::PhantomData<M>,
}

impl<M: Model> From<mongodb::Cursor> for ModelCursor<M> {
    fn from(inner: mongodb::Cursor) -> Self {
        Self {
            inner,
            _pd: std::marker::PhantomData,
        }
    }
}

impl<M: Model + Unpin> Stream for ModelCursor<M> {
    type Item = mongodb::error::Result<M>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context) -> task::Poll<Option<Self::Item>> {
        match Pin::new(&mut self.get_mut().inner).poll_next(cx) {
            task::Poll::Ready(Some(Ok(doc))) => {
                let res = from_bson(Bson::Document(doc)).map_err(|e| {
                    mongodb::error::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e))
                });
                task::Poll::Ready(Some(res))
            }
            task::Poll::Ready(Some(Err(e))) => task::Poll::Ready(Some(Err(e))),
            task::Poll::Ready(None) => task::Poll::Ready(None),
            task::Poll::Pending => task::Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
