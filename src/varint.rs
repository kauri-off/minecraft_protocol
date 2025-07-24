use std::io::{self, Read, Write};

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Error)]
pub enum VarIntError {
    #[error("Position >= 32")]
    Position,
    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),
    #[error("NegativeValue")]
    NegativeValue,
}

const SEGMENT_BITS: i32 = 0x7F;
const CONTINUE_BIT: i32 = 0x80;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarInt(pub i32);

impl VarInt {
    pub async fn read<R: AsyncRead + Unpin>(reader: &mut R) -> Result<VarInt, VarIntError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        loop {
            let current_byte = reader.read_u8().await? as i32;

            value |= (current_byte & SEGMENT_BITS) << position;

            if (current_byte & CONTINUE_BIT) == 0 {
                break;
            }
            position += 7;

            if position >= 32 {
                return Err(VarIntError::Position);
            }
        }

        Ok(VarInt(value))
    }

    pub fn read_sync<R: Read + Unpin>(reader: &mut R) -> Result<Self, VarIntError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        loop {
            let mut buf = [0; 1];
            if reader.read(&mut buf)? == 0 {
                return Err(VarIntError::IOError(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Incomplete VarInt",
                )));
            }

            let current_byte = buf[0] as i32;
            value |= (current_byte & SEGMENT_BITS) << position;

            if (current_byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;
            if position >= 32 {
                return Err(VarIntError::Position);
            }
        }

        Ok(VarInt(value))
    }

    pub async fn write<W: AsyncWrite + Unpin>(
        self: &Self,
        writer: &mut W,
    ) -> Result<(), VarIntError> {
        let mut value = self.0;
        loop {
            if (value & !SEGMENT_BITS) == 0 {
                writer.write_u8(value as u8).await?;
                break;
            }

            writer
                .write_u8(((value & SEGMENT_BITS) | CONTINUE_BIT) as u8)
                .await?;

            value = ((value as u32) >> 7) as i32;
        }

        Ok(())
    }

    pub fn write_sync<W: Write + Unpin>(self: &Self, writer: &mut W) -> Result<(), VarIntError> {
        let mut value = self.0;
        loop {
            if (value & !SEGMENT_BITS) == 0 {
                writer.write(&[value as u8])?;
                break;
            }

            writer.write(&[((value & SEGMENT_BITS) | CONTINUE_BIT) as u8])?;

            value = ((value as u32) >> 7) as i32;
        }

        Ok(())
    }
}
