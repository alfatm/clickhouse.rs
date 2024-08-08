use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::{
    attr::{Default as SerdeDefault, Field},
    Ctxt,
};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

fn column_names(data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let cx = Ctxt::new();
            let serde_fields = fields
                .named
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    (
                        field,
                        Field::from_ast(&cx, index, field, None, &SerdeDefault::None),
                    )
                })
                .filter(|(_syn_field, serde_params)| {
                    !serde_params.skip_serializing() && !serde_params.skip_deserializing()
                })
                .collect::<Vec<_>>();

            cx.check().unwrap();

            let any_flatten = serde_fields
                .iter()
                .any(|(_, serde_params)| serde_params.flatten());

            if !any_flatten {
                let column_names = serde_fields.iter().map(|(_syn_field, serde_params)| {
                    serde_params.name().serialize_name().to_string()
                });

                return quote! {
                    &[#( #column_names,)*]
                };
            }

            let column_names =
                serde_fields
                    .iter()
                    .map(|(syn_field, serde_params)| match serde_params.flatten() {
                        true => {
                            let ty = &syn_field.ty;
                            quote! {
                                &#ty::COLUMN_NAMES
                            }
                        }
                        false => {
                            let name = serde_params.name().serialize_name().to_string();
                            quote! {
                                &[#name]
                            }
                        }
                    });

            quote! {
                clickhouse::constcat::concat_slices!([&str]: #( #column_names, )*)
            }
        }
        Fields::Unnamed(_) => {
            quote! { &[] }
        }
        Fields::Unit => panic!("`Row` cannot be derived for unit structs"),
    }
}

// TODO: support wrappers `Wrapper(Inner)` and `Wrapper<T>(T)`.
// TODO: support the `nested` attribute.
// TODO: support the `crate` attribute.
#[proc_macro_derive(Row)]
pub fn row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let column_names = match &input.data {
        Data::Struct(data) => column_names(data),
        Data::Enum(_) | Data::Union(_) => panic!("`Row` can be derived only for structs"),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics clickhouse::Row for #name #ty_generics #where_clause {
            const COLUMN_NAMES: &'static [&'static str] = #column_names;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
