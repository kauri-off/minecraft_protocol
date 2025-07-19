use openssl::symm::{Cipher, Crypter, Mode};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct CFB8Stream<R, W> {
    pub read_stream: CFB8ReadHalf<R>,
    pub write_stream: CFB8WriteHalf<W>,
}

pub struct CFB8ReadHalf<R> {
    read_half: R,
    decrypter: Crypter,
}

pub struct CFB8WriteHalf<W> {
    write_half: W,
    encrypter: Crypter,
}

impl<W> CFB8WriteHalf<W> {
    pub fn new(write_half: W, key: &[u8; 16]) -> io::Result<Self> {
        let encrypter = Self::get_encrypter(key)?;

        Ok(Self {
            write_half,
            encrypter,
        })
    }

    fn get_encrypter(key: &[u8; 16]) -> io::Result<Crypter> {
        let cipher = Cipher::aes_128_cfb8();

        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(key))?;
        encrypter.pad(false);
        Ok(encrypter)
    }

    pub fn into_inner(self) -> W {
        self.write_half
    }
}

impl<R> CFB8ReadHalf<R> {
    pub fn new(read_half: R, key: &[u8; 16]) -> io::Result<Self> {
        let decrypter = Self::get_decrypter(key)?;

        Ok(Self {
            read_half,
            decrypter,
        })
    }

    fn get_decrypter(key: &[u8; 16]) -> io::Result<Crypter> {
        let cipher = Cipher::aes_128_cfb8();

        let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(key))?;
        decrypter.pad(false);
        Ok(decrypter)
    }

    pub fn into_inner(self) -> R {
        self.read_half
    }
}

impl<R, W> CFB8Stream<R, W> {
    pub fn new(read_half: R, write_half: W, key: &[u8; 16]) -> io::Result<Self> {
        let read_stream = CFB8ReadHalf::new(read_half, key)?;

        let write_stream = CFB8WriteHalf::new(write_half, key)?;

        Ok(Self {
            read_stream,
            write_stream,
        })
    }

    pub fn split(self) -> (CFB8ReadHalf<R>, CFB8WriteHalf<W>) {
        (self.read_stream, self.write_stream)
    }

    pub fn split_inner(self) -> (R, W) {
        (self.read_stream.read_half, self.write_stream.write_half)
    }
}

impl CFB8Stream<OwnedReadHalf, OwnedWriteHalf> {
    pub fn new_from_tcp(
        stream: TcpStream,
        key: &[u8; 16],
    ) -> io::Result<CFB8Stream<OwnedReadHalf, OwnedWriteHalf>> {
        let (read_half, write_half) = stream.into_split();

        CFB8Stream::new(read_half, write_half, key)
    }
}

impl<R> AsyncRead for CFB8ReadHalf<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let pre_len = buf.filled().len();
        let poll = Pin::new(&mut self.read_half).poll_read(cx, buf);

        if let Poll::Ready(Ok(())) = poll {
            let new_data = &mut buf.filled_mut()[pre_len..];
            let mut output = vec![0; new_data.len()];
            self.decrypter
                .update(new_data, &mut output)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            new_data.copy_from_slice(&output);
        }

        poll
    }
}

impl<W> AsyncWrite for CFB8WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let mut encrypted = vec![0; buf.len() + 16];
        let count = self
            .encrypter
            .update(buf, &mut encrypted)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        encrypted.truncate(count);

        Pin::new(&mut self.write_half).poll_write(cx, &encrypted)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.write_half).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.write_half).poll_shutdown(cx)
    }
}

impl<R, W> AsyncRead for CFB8Stream<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        AsyncRead::poll_read(Pin::new(&mut self.read_stream), cx, buf)
    }
}

impl<R, W> AsyncWrite for CFB8Stream<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.write_stream), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.write_stream), cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.write_stream), cx)
    }
}
