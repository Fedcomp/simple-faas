use tokio::io::{AsyncRead, AsyncWrite};

/// Auto trait for any type implementing
/// [AsyncRead](tokio::io::AsyncRead) + [AsyncWrite](tokio::io::AsyncWrite).
/// Sole purpose of this trait is to
/// make dyn [AsyncRead](tokio::io::AsyncRead) + [AsyncWrite](tokio::io::AsyncWrite) kind of possible.
pub trait AsyncStream: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite> AsyncStream for T {}
