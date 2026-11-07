//! Procedural macros for the Rigix application framework.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Field, Fields, Ident, Result, Type, Visibility,
};

/// Validates that all fields in an entity struct have private (Inherited) visibility.
///
/// Accumulates compiler errors across all non-private fields so all offending fields
/// are reported simultaneously with precise spans pointing directly to the visibility token.
fn validate_private_fields(fields: &Punctuated<Field, Comma>) -> Result<()> {
    let mut combined_error: Option<Error> = None;

    for field in fields {
        if !matches!(field.vis, Visibility::Inherited) {
            let field_name = field
                .ident
                .as_ref()
                .map(|i| format!("field `{}`", i))
                .unwrap_or_else(|| "unnamed field".to_string());

            let err = Error::new_spanned(
                &field.vis,
                format!(
                    "Rigix entity violation: {} must be private. Remove `pub` visibility.",
                    field_name
                ),
            );

            match &mut combined_error {
                Some(existing) => existing.combine(err),
                None => combined_error = Some(err),
            }
        }
    }

    match combined_error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// Generates public getters and an `into_parts` deconstructor for an entity struct.
///
/// - `String` fields return `&str`.
/// - Copy primitives (`i8`..`i128`, `u8`..`u128`, `isize`, `usize`, `bool`, `f32`, `f64`, `char`) and `Uuid` return by value.
/// - All other types return an immutable reference `&T`.
/// - `into_parts(self)` decomposes the struct into a tuple containing all fields in declaration order.
fn generate_getters(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> TokenStream2 {
    let getters = fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref()?;
        let field_type = &field.ty;

        let getter = match field_type {
            Type::Path(type_path) if type_path.qself.is_none() => {
                let last_segment = type_path.path.segments.last()?;
                let ident_str = last_segment.ident.to_string();

                match ident_str.as_str() {
                    "String" => quote! {
                        pub fn #field_ident(&self) -> &str {
                            &self.#field_ident
                        }
                    },
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" | "bool" | "f32" | "f64" | "char" | "Uuid" => {
                        quote! {
                            pub fn #field_ident(&self) -> #field_type {
                                self.#field_ident
                            }
                        }
                    }
                    _ => quote! {
                        pub fn #field_ident(&self) -> &#field_type {
                            &self.#field_ident
                        }
                    },
                }
            }
            _ => quote! {
                pub fn #field_ident(&self) -> &#field_type {
                    &self.#field_ident
                }
            },
        };

        Some(getter)
    });

    let field_types = fields.iter().map(|f| &f.ty);
    let field_idents = fields.iter().filter_map(|f| f.ident.as_ref());

    quote! {
        impl #struct_name {
            #(#getters)*

            pub fn into_parts(self) -> (#(#field_types),*) {
                (#(self.#field_idents),*)
            }
        }
    }
}

/// Defines a domain entity and derives its associated models, wire DTOs, and database rows.
///
/// When applied to a struct, this macro performs the following actions:
/// 1. Checks that all fields on the entity are private.
/// 2. Generates public reference getters and an `into_parts` method for all fields.
#[proc_macro_attribute]
pub fn entity(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let input_ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &input_ast.ident;

    let fields = match &input_ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            Fields::Unnamed(fields_unnamed) => &fields_unnamed.unnamed,
            Fields::Unit => {
                return Error::new_spanned(
                    &input_ast,
                    "#[entity] cannot be applied to a unit struct",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return Error::new_spanned(&input_ast, "#[entity] can only be applied to structs")
                .to_compile_error()
                .into();
        }
    };

    if let Err(err) = validate_private_fields(fields) {
        return err.to_compile_error().into();
    }

    let getters = generate_getters(struct_name, fields);

    let expanded = quote! {
        #input_ast

        #getters
    };

    TokenStream::from(expanded)
}
