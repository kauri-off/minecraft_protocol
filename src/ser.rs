use std::{
    io::{self, Read, Write},
    string::FromUtf8Error,
};

use thiserror::Error;

use crate::{
    num::Integer,
    varint::{VarInt, VarIntError},
};

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("VarInt error")]
    VarIntError(#[from] VarIntError),
    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),
    #[error("String serialization error")]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub trait Serialize {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError>;
}

pub trait Deserialize: Sized {
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

impl Serialize for VarInt {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        Ok(self.write_sync(writer)?)
    }
}

impl Deserialize for VarInt {
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<VarInt, SerializationError> {
        Ok(VarInt::read_sync(reader)?)
    }
}

impl Serialize for String {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        VarInt(self.len() as i32).write_sync(writer)?;

        Ok(writer.write_all(self.as_bytes())?)
    }
}

impl Deserialize for String {
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError> {
        let len = VarInt::read_sync(reader)?.0 as usize;
        let mut stream = vec![0; len];
        reader.read_exact(&mut stream)?;

        Ok(String::from_utf8(stream)?)
    }
}

impl<T> Serialize for T
where
    T: Integer,
{
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_bytes())?)
    }
}

impl<T> Deserialize for T
where
    T: Integer,
{
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let mut buf = vec![0; T::byte_len()];
        reader.read_exact(&mut buf)?;

        Ok(T::from_bytes(&buf))
    }
}

impl Serialize for bool {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        match self {
            true => writer.write_all(&[1])?,
            false => writer.write_all(&[0])?,
        };

        Ok(())
    }
}

impl Deserialize for bool {
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}

impl Serialize for Vec<u8> {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        VarInt(self.len() as i32).write_sync(writer)?;
        Ok(writer.write_all(&self)?)
    }
}

impl Deserialize for Vec<u8> {
    fn deserialize<R: Read + Unpin>(reader: &mut R) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        let len = VarInt::read_sync(reader)?;
        let mut buf: Vec<u8> = vec![0; len.0 as usize];
        reader.read_exact(&mut buf)?;

        Ok(buf)
    }
}

impl Serialize for [u8] {
    fn serialize<W: Write + Unpin>(&self, writer: &mut W) -> Result<(), SerializationError> {
        VarInt(self.len() as i32).write_sync(writer)?;
        writer.write_all(self)?;
        Ok(())
    }
}
