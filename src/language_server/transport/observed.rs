//! Report the first I/O failure so the session can stop even when the other stream waits.

use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::sync::watch;

type Failure = Option<(io::ErrorKind, String)>;

pub(in crate::language_server) struct Observed<T> {
    inner: T,
    failure: watch::Sender<Failure>,
}

impl<T> Observed<T> {
    pub(in crate::language_server) fn new(inner: T, failure: watch::Sender<Failure>) -> Self {
        Self { inner, failure }
    }

    fn report<U>(&self, result: Poll<io::Result<U>>) -> Poll<io::Result<U>> {
        if let Poll::Ready(Err(error)) = &result {
            self.failure.send_if_modified(|failure| {
                if failure.is_some() {
                    return false;
                }
                *failure = Some((error.kind(), error.to_string()));
                true
            });
        }
        result
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Observed<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        self.report(result)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Observed<T> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        let result = match Pin::new(&mut self.inner).poll_write(cx, buf) {
            Poll::Ready(Ok(0)) if !buf.is_empty() => {
                Poll::Ready(Err(io::Error::new(io::ErrorKind::WriteZero, "language server output closed")))
            }
            result => result,
        };
        self.report(result)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_flush(cx);
        self.report(result)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_shutdown(cx);
        self.report(result)
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    use super::*;

    struct FailingIo;

    impl AsyncRead for FailingIo {
        fn poll_read(self: Pin<&mut Self>, _: &mut Context<'_>, _: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::ConnectionReset, "read failed")))
        }
    }

    impl AsyncWrite for FailingIo {
        fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, _: &[u8]) -> Poll<io::Result<usize>> {
            Poll::Ready(Ok(0))
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, "flush failed")))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::NotConnected, "shutdown failed")))
        }
    }

    #[tokio::test]
    async fn reader_and_writer_keep_the_first_failure() {
        let (failure, mut failed) = watch::channel(None);
        let mut reader = Observed::new(FailingIo, failure.clone());
        let mut writer = Observed::new(FailingIo, failure);
        let error = reader.read(&mut [0]).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        assert!(failed.has_changed().unwrap());
        assert_eq!(*failed.borrow_and_update(), Some((io::ErrorKind::ConnectionReset, "read failed".into())));
        assert_eq!(writer.flush().await.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
        assert!(!failed.has_changed().unwrap());
        assert_eq!(*failed.borrow(), Some((io::ErrorKind::ConnectionReset, "read failed".into())));
    }

    #[tokio::test]
    async fn write_zero_fails_only_for_nonempty_output() {
        let (failure, failed) = watch::channel(None);
        let mut writer = Observed::new(FailingIo, failure);
        assert_eq!(writer.write(&[]).await.unwrap(), 0);
        assert!(failed.borrow().is_none());
        assert!(!failed.has_changed().unwrap());
        assert_eq!(writer.write(b"response").await.unwrap_err().kind(), io::ErrorKind::WriteZero);
        assert_eq!(failed.borrow().as_ref().unwrap().0, io::ErrorKind::WriteZero);
        assert!(failed.has_changed().unwrap());
    }

    #[tokio::test]
    async fn flush_and_shutdown_failures_reach_the_session() {
        for shutdown in [false, true] {
            let (failure, failed) = watch::channel(None);
            let mut writer = Observed::new(FailingIo, failure);
            let error = if shutdown { writer.shutdown().await } else { writer.flush().await }.unwrap_err();
            let expected = if shutdown { io::ErrorKind::NotConnected } else { io::ErrorKind::BrokenPipe };
            assert_eq!(error.kind(), expected);
            assert_eq!(failed.borrow().as_ref().unwrap().0, expected);
            assert!(failed.has_changed().unwrap());
        }
    }
}
