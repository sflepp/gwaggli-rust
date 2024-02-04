use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamExt;

pub struct SlidingWindow<T: Send> {
    pub buffer: Vec<T>,
    pub window_size: usize,
    pub frame_size: usize,
    pub tail: usize,
}

impl<T: Clone + Unpin + Send> SlidingWindow<T> {
    pub fn new(window_size: usize, frame_size: usize) -> Self {
        SlidingWindow {
            buffer: vec![],
            window_size,
            frame_size,
            tail: 0,
        }
    }
    pub fn push(&mut self, data: Vec<T>) {
        self.tail += data.len();
        self.buffer.extend(data);
    }
}

impl<T: Clone + Unpin + Send> Stream for SlidingWindow<T> {
    type Item = Vec<T>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut(); // safely obtain a mutable reference to SlidingWindow

        if this.tail < this.window_size {
            return Poll::Pending;
        }

        let data = this.buffer[0..this.window_size].to_vec();

        this.buffer.drain(0..this.frame_size);
        this.tail = this.tail.saturating_sub(this.frame_size);

        Poll::Ready(Some(data))
    }
}
#[cfg(test)]
mod tests {
    use crate::utils::sliding_window::SlidingWindow;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_sliding_window() {
        let mut sliding_window = SlidingWindow::new(10, 5);

        for i in 0..10 {
            sliding_window.push(vec![i, i, i, i, i, i, i, i, i, i]);
        }

        assert_eq!(
            sliding_window.next().await.unwrap(),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            sliding_window.next().await.unwrap(),
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1]
        );
        assert_eq!(
            sliding_window.next().await.unwrap(),
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        );
        assert_eq!(
            sliding_window.next().await.unwrap(),
            vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2]
        );
    }
}
