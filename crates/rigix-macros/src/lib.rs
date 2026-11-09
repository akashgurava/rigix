//! Procedural macros for the Rigix application framework.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Field, Fields, Ident, Result, Type, Visibility,
};

/// Validates that all fields in a struct have private visibility.
///
/// Accumulates compiler errors across all non-private fields so all offending fields
/// are reported simultaneously with precise spans pointing directly to the visibility token.
fn validate_private_fields(macro_kind: &str, fields: &Punctuated<Field, Comma>) -> Result<()> {
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
                    "Rigix {} violation: {} must be private. Remove `pub` visibility.",
                    macro_kind, field_name
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

/// Generates public getters and an `into_parts(self)` tuple deconstructor for all fields on a struct.
///
/// Type mapping rules:
/// - `String` fields return string slices (`&str`).
/// - Primitive Copy types (`i8`..`i128`, `u8`..`u128`, `bool`, `f32`, `f64`, `usize`, `isize`, `char`) and `Uuid` return by value.
/// - All other types return an immutable reference `&T`.
/// - `into_parts(self)` consumes the entity and returns all fields in declaration order as a tuple.
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

fn derive_encapsulated_struct_tokens(
    macro_kind: &str,
    input_ast: DeriveInput,
) -> Result<TokenStream2> {
    let struct_name = &input_ast.ident;

    let fields = match &input_ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            Fields::Unnamed(fields_unnamed) => &fields_unnamed.unnamed,
            Fields::Unit => {
                return Err(Error::new_spanned(
                    &input_ast,
                    format!(
                        "#[derive({})] cannot be applied to a unit struct",
                        macro_kind
                    ),
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input_ast,
                format!("#[derive({})] can only be applied to structs", macro_kind),
            ));
        }
    };

    validate_private_fields(macro_kind, fields)?;

    Ok(generate_getters(struct_name, fields))
}

fn derive_encapsulated_struct(macro_kind: &str, input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    match derive_encapsulated_struct_tokens(macro_kind, input_ast) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Defines an inbound request DTO with compile-time encapsulated fields.
///
/// When derived on a struct, this macro performs the following actions:
/// 1. Checks that all fields on the struct are private (inherited visibility).
/// 2. Generates public reference getters (`&field` or Copy for primitives) and an `into_parts(self)` method.
#[proc_macro_derive(Request)]
pub fn request(input: TokenStream) -> TokenStream {
    derive_encapsulated_struct("Request", input)
}

/// Defines a domain model / response DTO with compile-time encapsulated fields.
///
/// When derived on a struct, this macro performs the following actions:
/// 1. Checks that all fields on the struct are private (inherited visibility).
/// 2. Generates public reference getters (`&field` or Copy for primitives) and an `into_parts(self)` method.
#[proc_macro_derive(Model)]
pub fn model(input: TokenStream) -> TokenStream {
    derive_encapsulated_struct("Model", input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, FieldsNamed, ItemStruct};

    #[test]
    fn test_validate_private_fields_passes_on_inherited_visibility() {
        let item: ItemStruct = parse_quote! {
            struct Sample {
                id: i64,
                name: String,
            }
        };
        if let Fields::Named(FieldsNamed { named, .. }) = item.fields {
            assert!(validate_private_fields("Request", &named).is_ok());
            assert!(validate_private_fields("Model", &named).is_ok());
        } else {
            panic!("expected named fields");
        }
    }

    #[test]
    fn test_validate_private_fields_fails_on_pub_visibility() {
        let item: ItemStruct = parse_quote! {
            struct Sample {
                pub id: i64,
                name: String,
            }
        };
        if let Fields::Named(FieldsNamed { named, .. }) = item.fields {
            let req_err = validate_private_fields("Request", &named).unwrap_err();
            assert_eq!(
                req_err.to_string(),
                "Rigix Request violation: field `id` must be private. Remove `pub` visibility."
            );

            let model_err = validate_private_fields("Model", &named).unwrap_err();
            assert_eq!(
                model_err.to_string(),
                "Rigix Model violation: field `id` must be private. Remove `pub` visibility."
            );
        } else {
            panic!("expected named fields");
        }
    }

    #[test]
    fn test_validate_private_fields_accumulates_all_restricted_visibilities() {
        let item: ItemStruct = parse_quote! {
            struct Sample {
                pub a: i64,
                pub(crate) b: String,
                pub(super) c: bool,
            }
        };
        if let Fields::Named(FieldsNamed { named, .. }) = item.fields {
            let err = validate_private_fields("Request", &named).unwrap_err();
            let messages: Vec<String> = err.into_iter().map(|e| e.to_string()).collect();
            assert_eq!(messages.len(), 3);
            assert!(messages[0].contains("field `a`"));
            assert!(messages[1].contains("field `b`"));
            assert!(messages[2].contains("field `c`"));
        } else {
            panic!("expected named fields");
        }
    }

    #[test]
    fn test_generate_getters_emits_expected_return_types() {
        let item: ItemStruct = parse_quote! {
            struct Account {
                id: i64,
                uuid: Uuid,
                name: String,
                active: bool,
                tags: Vec<String>,
            }
        };
        if let Fields::Named(FieldsNamed { named, .. }) = item.fields {
            let tokens = generate_getters(&item.ident, &named);
            let code = tokens.to_string();

            // Primitive / Uuid Copy return by value
            assert!(code.contains("pub fn id (& self) -> i64 { self . id }"));
            assert!(code.contains("pub fn uuid (& self) -> Uuid { self . uuid }"));
            assert!(code.contains("pub fn active (& self) -> bool { self . active }"));

            // String returns string slice
            assert!(code.contains("pub fn name (& self) -> & str { & self . name }"));

            // Compound returns reference
            assert!(code.contains("pub fn tags (& self) -> & Vec < String > { & self . tags }"));

            // into_parts consumes self and returns tuple
            assert!(code.contains(
                "pub fn into_parts (self) -> (i64 , Uuid , String , bool , Vec < String >)"
            ));
            assert!(code
                .contains("(self . id , self . uuid , self . name , self . active , self . tags)"));
        } else {
            panic!("expected named fields");
        }
    }

    #[test]
    fn test_derive_rejects_unit_and_enum() {
        let unit_struct: DeriveInput = parse_quote! {
            struct Empty;
        };
        let err = derive_encapsulated_struct_tokens("Request", unit_struct).unwrap_err();
        assert_eq!(
            err.to_string(),
            "#[derive(Request)] cannot be applied to a unit struct"
        );

        let enum_item: DeriveInput = parse_quote! {
            enum Action {
                Foo,
                Bar,
            }
        };
        let err = derive_encapsulated_struct_tokens("Model", enum_item).unwrap_err();
        assert_eq!(
            err.to_string(),
            "#[derive(Model)] can only be applied to structs"
        );
    }
}
