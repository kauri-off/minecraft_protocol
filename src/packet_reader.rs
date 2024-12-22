use std::io::{self, Cursor, Read};

use crate::{types::packet_io::PacketIO, UncompressedPacket};

pub struct PacketReader {
    stream: Cursor<Vec<u8>>,
}

impl PacketReader {
    pub fn new(packet: &UncompressedPacket) -> Self {
        let stream = Cursor::new(packet.data.clone());
        PacketReader { stream }
    }

    pub fn read<T: PacketIO>(&mut self) -> io::Result<T> {
        T::read_from(&mut self.stream)
    }

    pub fn read_option<T: PacketIO>(&mut self) -> io::Result<Option<T>> {
        let has_item: bool = PacketIO::read_from(&mut self.stream)?;

        match has_item {
            true => Ok(Some(T::read_from(&mut self.stream)?)),
            false => Ok(None),
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}
