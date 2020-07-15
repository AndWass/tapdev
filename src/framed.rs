use crate::dataframer;

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Source<Source: io::AsyncRead + Unpin, Framer: dataframer::Framer> {
    source: Source,
    framer: Framer,
}

impl<S: io::AsyncRead + Unpin, Framer: dataframer::Framer> Source<S, Framer> {
    pub fn new(source: S, framer: Framer) -> Self {
        Self { source, framer }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> io::Result<Vec<Vec<u8>>> {
        let read_result = self.source.read(buffer).await;
        match read_result {
            Ok(amount_read) => Ok(self.framer.add_slice(&buffer[..amount_read])),
            Err(e) => Err(e)
        }
    }
}

pub struct Destination<Dest: io::AsyncWrite + Unpin, Framer: dataframer::Framer> {
    destination: Dest,
    _phantom: std::marker::PhantomData<Framer>,
}

impl<Dest: io::AsyncWrite + Unpin, Framer: dataframer::Framer> Destination<Dest, Framer> {
    pub fn new(destination: Dest, _framer: Framer) -> Self {
        Self {
            destination,
            _phantom: std::marker::PhantomData::default(),
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        let framed = Framer::frame_data(data);
        self.destination.write_all(framed.as_slice()).await
    }
}
