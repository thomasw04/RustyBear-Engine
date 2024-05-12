use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, Field, FieldMutability, Fields, Ident, ItemStruct, Type, Visibility};

#[proc_macro_attribute]
pub fn Entity(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);
    let name = input.ident.clone();

    //Add the world and handle fields to the struct
    if let Fields::Named(ref mut fields) = input.fields {
        fields.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new("_gen_handle", Span::call_site())),
            colon_token: None,
            mutability: FieldMutability::None,
            ty: Type::Verbatim(quote::quote! { hecs::Entity }),
        });
        fields.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new("_gen_world", Span::call_site())),
            colon_token: None,
            mutability: FieldMutability::None,
            ty: Type::Verbatim(quote::quote! { hecs::World }),
        });
    }

    //Create a function instantiate.
    let instantiate = quote::quote! {
        impl #name {
            pub fn instantiate(handle: hecs::Entity, world: hecs::World) -> Self {
                Self { _gen_handle: handle, _gen_world: world }
            }
        }
    };

    //Implement the entity trait for the struct
    let expanded = quote::quote! {
        impl crate::entities::entity::Entity for #name {
            fn handle(&self) -> hecs::Entity {
                self._gen_handle
            }

            fn world(&self) -> &hecs::World {
                &self._gen_world
            }

            fn world_mut(&mut self) -> &mut hecs::World {
                &mut self._gen_world
            }
        }
    };

    //Add input, instantiate and expanded to the token stream and return it.
    TokenStream::from(quote::quote! {
        #input
        #instantiate
        #expanded
    })
}
