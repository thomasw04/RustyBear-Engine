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
            ty: Type::Verbatim(quote::quote! { crate::entities::world::Entity }),
        });
        fields.named.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: Some(Ident::new("_gen_world", Span::call_site())),
            colon_token: None,
            mutability: FieldMutability::None,
            ty: Type::Verbatim(quote::quote! { Option<&'a crate::entities::world::World> }),
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
    input.generics.params.push(syn::parse_quote! { 'a });

    //Create a function instantiate.
    let instantiate = quote::quote! {
        impl<'a> crate::entities::world::Instantiable for #name<'a> {
            fn instantiate<T: hecs::DynamicBundle>(world: &crate::entities::world::World, components: T) {
                let handle = world.reserve_entity();
                let cmds = crate::entities::world::CommandBuffer::new();

                world.insert_one(handle, std::boxed::Box::new(Self { _gen_handle: handle.into(), _gen_cmds: cmds, _gen_world: Some(world), ..Default::default() }));
                world.insert(handle, components);

                if let Some(mut component) = world.get::<&mut Script>(handle) {
                    component.on_spawn(handle.into());
                }
            }
        }
    };

    //Implement the entity trait for the struct
    let expanded = quote::quote! {
        impl<'a> crate::entities::world::Entable for #name<'a> {
            fn handle(&self) -> crate::entities::world::Entity {
                self._gen_handle
            }

            fn world(&self) -> &'a crate::entities::world::World {
                &self._gen_world.expect("This is a bug. Please report it.")
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
