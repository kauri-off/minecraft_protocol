extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitInt, parse_macro_input};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    input
        .attrs
        .push(syn::parse_quote!(#[derive(minecraft_protocol::packet::PacketIO)]));

    let struct_name = &input.ident;
    let packet_id_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("packet"))
        .expect("Expected #[packet(ID)] attribute");

    let packet_id: LitInt = packet_id_attr
        .parse_args()
        .expect("Expected packet ID as integer, e.g., #[packet(0x00)]");
    let packet_id_value = packet_id.base10_parse::<i32>().unwrap();

    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>(),
        _ => panic!("Packet can only be derived for structs with named fields"),
    };

    let expanded = quote! {
        impl #struct_name {
            pub const PACKET_ID: minecraft_protocol::varint::VarInt =
                minecraft_protocol::varint::VarInt(#packet_id_value);
        }

        impl minecraft_protocol::packet::PacketIO for #struct_name {
            fn write<W: std::io::Write + Unpin>(
                &self,
                writer: &mut W,
            ) -> Result<(), minecraft_protocol::ser::SerializationError> {
                minecraft_protocol::ser::Serialize::serialize(&Self::PACKET_ID, writer)?;
                #(minecraft_protocol::ser::Serialize::serialize(&self.#fields, writer)?;)*
                Ok(())
            }


            fn read<R: std::io::Read + Unpin>(
                reader: &mut R,
            ) -> Result<Self, minecraft_protocol::ser::SerializationError> {
                Ok(Self {
                    #(#fields: minecraft_protocol::ser::Deserialize::deserialize(reader)?,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
