use std::str::Utf8Chunk;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, ItemFn, LitStr};

/// TODO: implement the following procedural `#[derive(FieldCounter)]` macro
/// It should be usable on structure. When used on enums (or unions), it should produce a compile
/// error.
/// It should add an associated function called `field_count`, which returns the number of fields
/// in the struct.
/// The visibility of the function should be the same as of the struct.
#[proc_macro_derive(FieldCounter)]
pub fn derive_field_counter(stream: TokenStream) -> TokenStream {
    // Parse the input TokenStream as a DeriveInput
    let input = parse_macro_input!(stream as DeriveInput);

    let struct_name = input.ident;
    let visibility = input.vis;

    // Match on the data type (struct, enum, or union)
    let field_count = match input.data {
        Data::Struct(data_struct) => {
            // Get the number of fields in the struct
            match data_struct.fields {
                Fields::Named(ref fields) => fields.named.len(),
                Fields::Unnamed(ref fields) => fields.unnamed.len(),
                Fields::Unit => 0,
            }
        }
        // Generate a compile-time error if used on an enum or union
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "#[derive(FieldCounter)] is only supported for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate the output tokens for the `field_count` function
    let expanded = quote! {
        impl #struct_name {
            #visibility fn field_count() -> usize {
                #field_count
            }
        }
    };

    // Return the generated tokens
    TokenStream::from(expanded)
}

/// TODO: implement the following attribute procedural macro
/// It should go through all string literals in the given function, and uwuify them using
/// the [`uwuifier`](https://crates.io/crates/uwuify) crate.
///
/// Use the [`VisitMut`](https://docs.rs/syn/latest/syn/visit_mut/index.html) API from `syn`.
/// The [`parse_quote!`](https://docs.rs/syn/latest/syn/macro.parse_quote.html) macro might also be
/// useful.
#[proc_macro_attribute]
pub fn uwuifythis(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let mut func = parse_macro_input!(item as ItemFn);

    struct UwuifyStrings;

    impl VisitMut for UwuifyStrings {
        fn visit_lit_str_mut(&mut self, literal: &mut syn::LitStr) {
            let uwuified = uwuifier::uwuify_str_sse(&literal.value());
            *literal = parse_quote!(#uwuified);
        }
    }

    UwuifyStrings.visit_item_fn_mut(&mut func);

    func.into_token_stream().into()
}
