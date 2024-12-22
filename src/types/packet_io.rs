use std::io::{self, Error, ErrorKind, Read, Write};

use super::{num::Integer, var_int::VarInt};

pub trait PacketIO {
    fn write_to<W: Write + Unpin>(self, writer: &mut W) -> io::Result<()>;
    fn read_from<R: Read + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized;
}

impl PacketIO for VarInt {
    fn write_to<W: Write + Unpin>(self, writer: &mut W) -> io::Result<()> {
        self.write_sync(writer)
    }

    fn read_from<R: Read + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        VarInt::read_sync(reader)
    }
}

impl PacketIO for String {
    fn write_to<W: Write + Unpin>(self, writer: &mut W) -> io::Result<()> {
        VarInt(self.len() as i32).write_to(writer)?;

        writer.write_all(self.as_bytes())
    }

    fn read_from<R: Read + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let len = VarInt::read_sync(reader)?.0 as usize;
        let mut stream = vec![0; len];
        reader.read_exact(&mut stream)?;

        String::from_utf8(stream).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

impl<T> PacketIO for T
where
    T: Integer,
{
    fn write_to<W: Write + Unpin>(self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_bytes())
    }

    fn read_from<R: Read + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = vec![0; T::byte_len()];
        reader.read_exact(&mut buf)?;

        Ok(T::from_bytes(&buf))
    }
}

impl PacketIO for bool {
    fn write_to<W: Write + Unpin>(self, writer: &mut W) -> io::Result<()> {
        match self {
            true => writer.write_all(&[1]),
            false => writer.write_all(&[0]),
        }
    }

    fn read_from<R: Read + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(ErrorKind::Other, "Not a bool")),
        }
    }
}
