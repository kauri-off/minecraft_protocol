use std::io::{self, Cursor, Read, Write};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    ser::SerializationError,
    varint::{VarInt, VarIntError},
};

pub trait PacketIO {
    fn write<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError>;

    fn read<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("DecryptionError")]
    DecryptionError,

    #[error("VarIntError: {0}")]
    VarIntError(#[from] VarIntError),

    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),
}

#[derive(Debug, Clone)]
pub struct UncompressedPacket {
    pub packet_id: VarInt,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub data: Vec<u8>,
}

impl RawPacket {
    pub async fn read<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Self, PacketError> {
        let len = VarInt::read(reader).await?;
        let mut buf = vec![0; len.0 as usize];

        reader.read_exact(&mut buf).await?;

        Ok(Self { data: buf })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> Result<(), PacketError> {
        VarInt(self.data.len() as i32).write(writer).await?;

        writer.write_all(&self.data).await?;

        Ok(())
    }

    pub fn from_packetio<T: PacketIO>(packet: &T) -> Result<Self, SerializationError> {
        let mut buf = Vec::new();

        packet.write(&mut buf)?;
        Ok(Self { data: buf })
    }

    pub fn as_uncompressed(self) -> Result<UncompressedPacket, PacketError> {
        let mut cursor = Cursor::new(self.data);
        let packet_id = VarInt::read_sync(&mut cursor)?;
        let mut payload = Vec::new();
        std::io::Read::read_to_end(&mut cursor, &mut payload)?;

        Ok(UncompressedPacket { packet_id, payload })
    }
}

impl UncompressedPacket {
    pub fn to_raw_packet(&self) -> Result<RawPacket, PacketError> {
        let mut buf = Vec::new();
        self.packet_id.write_sync(&mut buf)?;
        buf.extend(&self.payload);
        Ok(RawPacket { data: buf })
    }

    pub fn convert<T: PacketIO>(&self) -> Result<T, SerializationError> {
        T::read(&mut Cursor::new(&self.payload))
    }
}
