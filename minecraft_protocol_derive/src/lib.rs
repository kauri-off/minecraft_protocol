extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Поиск атрибута #[packet]
    let packet_id_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("packet"))
        .expect("Expected #[packet(ID)] attribute");

    // Парсинг значения ID напрямую в токены
    let packet_id_value = packet_id_attr
        .parse_args::<syn::Expr>()
        .expect("Expected packet ID as integer expression, e.g., #[packet(0x00)]");

    // Обработка полей структуры
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("Packet can only be derived for structs with named fields"),
    };

    // Генерация идентификаторов и типов полей
    let field_idents: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        impl #struct_name {
            pub const PACKET_ID: minecraft_protocol::varint::VarInt =
                minecraft_protocol::varint::VarInt(#packet_id_value as i32);
        }

        impl #struct_name {
            pub fn as_uncompressed(
                &self,
            ) -> Result<minecraft_protocol::packet::UncompressedPacket, minecraft_protocol::ser::SerializationError> {
                let mut payload = Vec::new();
                #(minecraft_protocol::ser::Serialize::serialize(&self.#field_idents, &mut payload)?;)*
                Ok(minecraft_protocol::packet::UncompressedPacket {
                    packet_id: Self::PACKET_ID.clone(),
                    payload
                })
            }
        }

        impl minecraft_protocol::ser::Deserialize for #struct_name {
            fn deserialize<R: std::io::Read + Unpin>(reader: &mut R) -> Result<Self, minecraft_protocol::ser::SerializationError> {
                Ok(Self {
                    #(#field_idents: <#field_types as minecraft_protocol::ser::Deserialize>::deserialize(reader)?,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
