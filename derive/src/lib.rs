use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, Field, FieldMutability, Fields, Ident, ItemStruct, Type, Visibility};

#[proc_macro_attribute]
pub fn Entable(_args: TokenStream, input: TokenStream) -> TokenStream {
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
            ty: Type::Verbatim(quote::quote! { crate::entities::world::Entity }),
        });
        fields.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new("_gen_world", Span::call_site())),
            colon_token: None,
            mutability: FieldMutability::None,
            ty: Type::Verbatim(quote::quote! { crate::entities::world::World }),
        });
        fields.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new("_gen_cmds", Span::call_site())),
            colon_token: None,
            mutability: FieldMutability::None,
            ty: Type::Verbatim(quote::quote! { crate::entities::world::CommandBuffer }),
        });
    }

    //Add the default derive to the struct.
    input.attrs.push(syn::parse_quote! { #[derive(Default)] });

    //Create a function instantiate.
    let instantiate = quote::quote! {
        impl #name {
            pub fn instantiate(handle: crate::entities::world::Entity, world: crate::entities::world::World) -> Self {
                let cmds = crate::entities::world::CommandBuffer::new(world.inner().deref());
                Self { _gen_handle: handle, _gen_cmds: cmds, _gen_world: world, ..Default::default() }
            }
        }
    };

    //Implement the entity trait for the struct
    let expanded = quote::quote! {
        impl crate::entities::world::Entable for #name {
            fn handle(&self) -> crate::entities::world::Entity {
                self._gen_handle
            }

            fn world(&self) -> &crate::entities::world::World {
                &self._gen_world
            }

            fn world_mut(&mut self) -> &mut crate::entities::world::World {
                &mut self._gen_world
            }

            fn cmds(&self) -> &crate::entities::world::CommandBuffer {
                &self._gen_cmds
            }

            fn cmds_mut(&mut self) -> &mut crate::entities::world::CommandBuffer {
                &mut self._gen_cmds
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
