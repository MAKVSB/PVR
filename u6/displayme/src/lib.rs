extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// TODO: implement the following procedural `#[derive(DisplayMe)]` macro
/// It should be usable only on structs. When used on enums (or unions), it should produce a compile
/// error.
///
/// The macro should generate code that will implement the `Display` trait for the struct. The
/// specific format of the display implementation is defined by tests in the `assignments` crate.
#[proc_macro_derive(DisplayMe)]
pub fn display_me_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let name = input.ident;

    // Generate the appropriate `Display` implementation based on the struct's fields
    let display_impl = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            // Handle named fields
            Fields::Named(fields_named) => {
                let field_names = fields_named.named.iter().map(|f| &f.ident);
                let field_count = fields_named.named.len();

                let print_statements = field_names.enumerate().map(|(index, field)| {
                    if index == field_count - 1 {
                        quote! {
                            write!(f, "\n    {}: {}", stringify!(#field), self.#field)?;
                        }
                    } else {
                        quote! {
                            write!(f, "\n    {}: {},", stringify!(#field), self.#field)?;
                        }
                    }
                });

                let is_empty = if fields_named.named.len() != 0 {"\n"} else {""};

                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "struct {} {{", stringify!(#name))?;
                            #(
                                #print_statements;
                            )*
                            write!(f, #is_empty);
                            write!(f, "}}")
                        }
                    }
                }
            }
            // Handle unnamed (tuple) fields
            Fields::Unnamed(fields_unnamed) => {
                let field_indices = 0..fields_unnamed.unnamed.len();

                let print_statements = field_indices.map(|field_indices| {
                    if field_indices == fields_unnamed.unnamed.len() - 1 {
                        quote! {
                            write!(f, "\n    {}: {}", #field_indices, &self.#field_indices)?;
                        }
                    } else {
                        quote! {
                            write!(f, "\n    {}: {},", #field_indices, &self.#field_indices)?;
                        }
                    }
                });

                let is_empty = if fields_unnamed.unnamed.len() != 0 {"\n"} else {""};

                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "struct {} (", stringify!(#name))?;
                            #(
                                #print_statements;
                            )*
                            write!(f, #is_empty);
                            write!(f, ")")
                        }
                    }
                }
            }
            // Handle unit structs
            Fields::Unit => {
                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "struct {};", stringify!(#name))
                        }
                    }
                }
            }
        },
        // Handle unsupported cases (enum, union)
        _ => unimplemented!("DisplayMe only supports structs"),
    };

    TokenStream::from(display_impl)
}
