use std::io::{self, Cursor};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    ser::{Deserialize, SerializationError},
    varint::{VarInt, VarIntError},
};

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("VarIntError: {0}")]
    VarIntError(#[from] VarIntError),

    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),
}

#[derive(Debug, Clone)]
pub struct CompressedPacket {
    pub data: Vec<u8>,
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
        if len.0 < 0 {
            return Err(VarIntError::NegativeValue.into());
        }

        let mut buf = vec![0; len.0 as usize];
        reader.read_exact(&mut buf).await?;

        Ok(Self { data: buf })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> Result<(), PacketError> {
        VarInt(self.data.len() as i32).write(writer).await?;
        writer.write_all(&self.data).await?;
        Ok(())
    }

    pub fn as_uncompressed(&self) -> Result<UncompressedPacket, PacketError> {
        let mut cursor = Cursor::new(&self.data);
        let packet_id = VarInt::read_sync(&mut cursor)?;
        let pos = cursor.position() as usize;
        let payload = self.data[pos..].to_vec();

        Ok(UncompressedPacket { packet_id, payload })
    }

    pub fn try_uncompress(
        &self,
        threshold: Option<i32>,
    ) -> Result<Option<UncompressedPacket>, PacketError> {
        if let Some(_) = threshold {
            let mut cursor = Cursor::new(&self.data);
            let data_length = VarInt::read_sync(&mut cursor)?;

            if data_length.0 == 0 {
                let packet_id = VarInt::read_sync(&mut cursor)?;
                let pos = cursor.position() as usize;
                let payload = self.data[pos..].to_vec();

                Ok(Some(UncompressedPacket { packet_id, payload }))
            } else {
                Ok(None)
            }
        } else {
            self.as_uncompressed().map(Some)
        }
    }
}

impl UncompressedPacket {
    pub fn to_raw_packet(&self) -> Result<RawPacket, PacketError> {
        let mut buf = Vec::new();
        self.packet_id.write_sync(&mut buf)?;
        std::io::Write::write_all(&mut buf, &self.payload)?;
        Ok(RawPacket { data: buf })
    }

    pub fn convert<T: Deserialize>(&self) -> Result<T, SerializationError> {
        T::deserialize(&mut Cursor::new(&self.payload))
    }

    pub fn compress(&self, threshold: i32) -> Result<CompressedPacket, PacketError> {
        let raw_packet = self.to_raw_packet()?;

        if raw_packet.data.len() >= threshold as usize {
            todo!("Implement compression");
        } else {
            let mut data = Vec::new();
            // Prepend VarInt(0) indicating uncompressed data
            VarInt(0).write_sync(&mut data)?;
            data.extend_from_slice(&raw_packet.data);

            Ok(CompressedPacket { data })
        }
    }

    pub fn compress_to_raw(&self, threshold: Option<i32>) -> Result<RawPacket, PacketError> {
        match threshold {
            Some(t) => Ok(self.compress(t)?.to_raw_packet()),
            None => self.to_raw_packet(),
        }
    }
}

impl CompressedPacket {
    pub fn to_raw_packet(self) -> RawPacket {
        RawPacket { data: self.data }
    }
}
