extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use sp1_columns_core::FlattenFieldsHelper;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// Author: ChatGPT

#[proc_macro_derive(FlattenFields)]
pub fn flatten_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Get the generic type `T` if it exists
    let generic_type = generics.type_params().next().expect("Expected a generic parameter");

    let field_list_code = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => {
                fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let field_type = &field.ty;

                    // Check if the field type is the same as the generic type `T`
                    if let syn::Type::Path(type_path) = field_type {
                        if type_path.path.is_ident(&generic_type.ident) {
                            // This field is of type `T`, treat it as a terminal field
                            return quote! {
                                fields.push(#field_name.to_string());
                            };
                        }
                    }

                    // Handle array types `[T; N]`
                    if let syn::Type::Array(array_type) = field_type {
                        let elem_type = &*array_type.elem; // Dereference the Box to get the inner type
                        let array_len = &array_type.len;

                        // Check if the array element type is `T`
                        if let syn::Type::Path(type_path) = elem_type {
                            if type_path.path.is_ident(&generic_type.ident) {
                                // Array of `T`, treat as terminal
                                return quote! {
                                    for i in 0..#array_len {
                                        fields.push(format!("{}__{}", #field_name, i));
                                    }
                                };
                            }
                        }

                        // Otherwise, recursively process the array elements
                        return quote! {
                            if let Some(sub_fields) = <#elem_type as FlattenFieldsHelper>::flatten_fields() {
                                for i in 0..#array_len {
                                    for sub_field in &sub_fields {
                                        fields.push(format!("{}__{}__{}", #field_name, i, sub_field));
                                    }
                                }
                            } else {
                                for i in 0..#array_len {
                                    fields.push(format!("{}__{}", #field_name, i));
                                }
                            }
                        };
                    }

                    // For non-generic, non-array types, use the existing logic
                    quote! {
                        if <#field_type as FlattenFieldsHelper>::flatten_fields().is_some() {
                            if let Some(sub_fields) = <#field_type as FlattenFieldsHelper>::flatten_fields() {
                                for sub_field in sub_fields {
                                    fields.push(format!("{}__{}", #field_name, sub_field));
                                }
                            }
                        } else {
                            fields.push(#field_name.to_string());
                        }
                    }
                }).collect::<Vec<_>>()
            }
            Fields::Unnamed(fields) => {
                fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let index = syn::Index::from(i);
                    let field_type = &field.ty;

                    // Check if the field type is the same as the generic type `T`
                    if let syn::Type::Path(type_path) = field_type {
                        if type_path.path.is_ident(&generic_type.ident) {
                            // This field is of type `T`, treat it as a terminal field
                            return quote! {
                                fields.push(format!("{}", #index));
                            };
                        }
                    }

                    // Handle array types `[T; N]`
                    if let syn::Type::Array(array_type) = field_type {
                        let elem_type = &*array_type.elem; // Dereference the Box to get the inner type
                        let array_len = &array_type.len;

                        // Check if the array element type is `T`
                        if let syn::Type::Path(type_path) = elem_type {
                            if type_path.path.is_ident(&generic_type.ident) {
                                // Array of `T`, treat as terminal
                                return quote! {
                                    for i in 0..#array_len {
                                        fields.push(format!("{}__{}", #index, i));
                                    }
                                };
                            }
                        }

                        // Otherwise, recursively process the array elements
                        return quote! {
                            if let Some(sub_fields) = <#elem_type as FlattenFieldsHelper>::flatten_fields() {
                                for i in 0..#array_len {
                                    for sub_field in &sub_fields {
                                        fields.push(format!("{}__{}__{}", #index, i, sub_field));
                                    }
                                }
                            } else {
                                for i in 0..#array_len {
                                    fields.push(format!("{}__{}", #index, i));
                                }
                            }
                        };
                    }

                    // For non-generic, non-array types, use the existing logic
                    quote! {
                        if <#field_type as FlattenFieldsHelper>::flatten_fields().is_some() {
                            if let Some(sub_fields) = <#field_type as FlattenFieldsHelper>::flatten_fields() {
                                for sub_field in sub_fields {
                                    fields.push(format!("{}__{}", #index, sub_field));
                                }
                            }
                        } else {
                            fields.push(format!("{}", #index));
                        }
                    }
                }).collect::<Vec<_>>()
            }
            Fields::Unit => vec![],
        },
        _ => panic!("FlattenFields can only be used on structs."),
    };

    let expanded = quote! {
        impl #impl_generics FlattenFieldsHelper for #struct_name #ty_generics #where_clause {
            fn flatten_fields() -> Option<Vec<String>> {
                let mut fields = Vec::new();
                #(#field_list_code)*
                Some(fields)
            }
        }
    };

    TokenStream::from(expanded)
}
