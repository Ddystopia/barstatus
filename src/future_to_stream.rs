use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use tokio_stream::Stream;

pin_project_lite::pin_project! {
    pub struct FutureToStream<F, T> {
        #[pin]
        pinned: F,
        _marker: PhantomData<T>,
    }
}

impl<F, T> FutureToStream<F, T> {
    pub fn new(pinned: F) -> Self {
        Self {
            pinned,
            _marker: PhantomData,
        }
    }
}

impl<F: Future<Output = !>, T> Stream for FutureToStream<F, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match Future::poll(self.project().pinned, cx) {
            Poll::Pending => Poll::<Option<T>>::Pending,
        }
    }
}
