use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio_stream::StreamExt;

pin_project_lite::pin_project! {
    struct FutureToStream<F> {
        #[pin]
        pinned: F,
    }
}

impl<F> FutureToStream<F> {
    fn new(future: F) -> Self {
        Self { pinned: future }
    }
}

pub trait AnyStream {
    fn to_any_stream<T>(self) -> impl tokio_stream::Stream<Item = T>;
}

impl<F: Future<Output = Infallible>> AnyStream for F {
    fn to_any_stream<T>(self) -> impl tokio_stream::Stream<Item = T> {
        FutureToStream::new(self).map(|v| match v {})
    }
}

impl<F: Future<Output = Infallible>> tokio_stream::Stream for FutureToStream<F> {
    type Item = Infallible;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Future::poll(self.project().pinned, cx) {
            Poll::Ready(v) => match v {},
            Poll::Pending => Poll::Pending,
        }
    }
}
