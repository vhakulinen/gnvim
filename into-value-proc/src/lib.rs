use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(IntoValue)]
pub fn derive_into_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let st = match input.data {
        Data::Struct(st) => st,
        _ => panic!("IntoValue can only derive structs"),
    };

    let fields = st.fields.iter().map(|field| {
        let field = field.ident.as_ref().expect("fields must have names");
        let field_name = field.unraw().to_string();
        quote! {
            (rmpv::Value::from(#field_name), rmpv::Value::from(self.#field))
        }
    });

    let expanded = quote! {
        impl crate::IntoValue for #struct_name {
            fn into_value(self) -> rmpv::Value {
                rmpv::Value::from(vec![
                    #(#fields),*
                ])
            }
        }
    };

    TokenStream::from(expanded)
}
