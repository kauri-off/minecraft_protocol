use crate::{
    types::{packet_io::PacketIO, var_int::VarInt},
    UncompressedPacket,
};

pub struct PacketBuilder {
    pub packet_id: VarInt,
    pub data: Vec<u8>,
}

impl PacketBuilder {
    pub fn new(packet_id: VarInt) -> PacketBuilder {
        PacketBuilder {
            packet_id,
            data: vec![],
        }
    }

    pub fn build(self) -> UncompressedPacket {
        UncompressedPacket {
            packet_id: self.packet_id,
            data: self.data,
        }
    }

    pub fn write<T: PacketIO>(mut self, data: T) -> Self {
        let _ = data.write_to(&mut self.data);
        self
    }

    pub fn write_option<T: PacketIO>(mut self, data: Option<T>) -> Self {
        let _ = match data {
            Some(t) => {
                let _ = true.write_to(&mut self.data);
                t.write_to(&mut self.data)
            }
            None => false.write_to(&mut self.data),
        };
        self
    }

    pub fn write_buffer(mut self, buf: &[u8]) -> Self {
        self.data.extend(buf);
        self
    }
}
