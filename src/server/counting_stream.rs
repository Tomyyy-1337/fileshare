use std::net::IpAddr;
use futures::{channel::mpsc::Sender, stream::Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::state::update::ServerMessage;

pub struct CountingStream<S> {
    inner: S,
    tx: Sender<ServerMessage>,
    index: usize,
    ip: IpAddr,
    counter: usize,
    last_send_time: tokio::time::Instant,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<S> CountingStream<S> {
    pub fn new(inner: S, tx: Sender<ServerMessage>, index: usize, ip: IpAddr, permit: tokio::sync::OwnedSemaphorePermit) -> CountingStream<S> {
        CountingStream { inner, tx, index, ip, counter: 0, last_send_time: tokio::time::Instant::now(), _permit: permit }
    }
}

impl<S> Stream for CountingStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin,
{
    type Item = Result<bytes::Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(None) => {
                let index = self.index;
                let ip = self.ip;
                let counter = self.counter;
                let _ = self.tx.try_send(ServerMessage::DownloadActive { ip, num_packets: counter });
                let _ = self.tx.try_send(ServerMessage::Downloaded { index, ip });
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(_))) => Poll::Ready(None),
            Poll::Ready(Some(data)) => {
                self.counter += 1;
                if self.last_send_time.elapsed().as_millis() > 250 {
                    let ip = self.ip;
                    let counter = self.counter;
                    let _ = self.tx.try_send(ServerMessage::DownloadActive { ip, num_packets: counter });
                    self.counter = 0;
                    self.last_send_time = tokio::time::Instant::now();
                }
                Poll::Ready(Some(data))
            }
            p @ Poll::Pending => p,
        }
    }
}
