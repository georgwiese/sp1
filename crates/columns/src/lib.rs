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

    // Generate code to flatten fields
    let field_list_code = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => {
                fields.named.iter().map(|field| generate_field_code(field, &generic_type)).collect()
            }
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| generate_unnamed_field_code(i, field, &generic_type))
                .collect(),
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

fn generate_field_code(
    field: &syn::Field,
    generic_type: &syn::TypeParam,
) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;

    // Check if the field type is `T`
    if let syn::Type::Path(type_path) = field_type {
        if type_path.path.is_ident(&generic_type.ident) {
            return quote! {
                fields.push(#field_name.to_string());
            };
        }
    }

    // Handle `GenericArray<ElementType, Size>`
    if let syn::Type::Path(type_path) = field_type {
        if type_path.path.segments.last().unwrap().ident == "GenericArray" {
            if let syn::PathArguments::AngleBracketed(args) =
                &type_path.path.segments.last().unwrap().arguments
            {
                let element_type =
                    match args.args.first().expect("Expected element type in GenericArray") {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => panic!("Expected a type as the first argument in GenericArray"),
                    };

                let size_type =
                    match args.args.iter().nth(1).expect("Expected size type in GenericArray") {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => panic!("Expected a type as the second argument in GenericArray"),
                    };

                // Check if the element type is `T`
                if let syn::Type::Path(type_path) = element_type {
                    if type_path.path.is_ident(&generic_type.ident) {
                        // Treat the `GenericArray` of `T` as terminal
                        return quote! {
                            for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                                fields.push(format!("{}__{}", #field_name, i));
                            }
                        };
                    }
                }

                // Recursively process the `GenericArray` elements
                return quote! {
                    if let Some(sub_fields) = <#element_type as FlattenFieldsHelper>::flatten_fields() {
                        for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                            for sub_field in &sub_fields {
                                fields.push(format!("{}__{}__{}", #field_name, i, sub_field));
                            }
                        }
                    } else {
                        for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                            fields.push(format!("{}__{}", #field_name, i));
                        }
                    }
                };
            }
        }
    }

    // Handle array types `[T; N]`
    if let syn::Type::Array(array_type) = field_type {
        let elem_type = &*array_type.elem; // Dereference the Box to get the inner type
        let array_len = &array_type.len;

        // Check if the array element type is `T`
        if let syn::Type::Path(type_path) = elem_type {
            if type_path.path.is_ident(&generic_type.ident) {
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

    // For other types
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
}

fn generate_unnamed_field_code(
    index: usize,
    field: &syn::Field,
    generic_type: &syn::TypeParam,
) -> proc_macro2::TokenStream {
    let field_type = &field.ty;

    // Check if the field type is `T`
    if let syn::Type::Path(type_path) = field_type {
        if type_path.path.is_ident(&generic_type.ident) {
            return quote! {
                fields.push(format!("{}", #index));
            };
        }
    }

    // Handle `GenericArray<ElementType, Size>`
    if let syn::Type::Path(type_path) = field_type {
        if type_path.path.segments.last().unwrap().ident == "GenericArray" {
            if let syn::PathArguments::AngleBracketed(args) =
                &type_path.path.segments.last().unwrap().arguments
            {
                let element_type = &args.args[0];
                let size_type = &args.args[1];
                let element_type =
                    match args.args.first().expect("Expected element type in GenericArray") {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => panic!("Expected a type as the first argument in GenericArray"),
                    };

                let size_type =
                    match args.args.iter().nth(1).expect("Expected size type in GenericArray") {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => panic!("Expected a type as the second argument in GenericArray"),
                    };

                // Check if the element type is `T`
                if let syn::Type::Path(type_path) = element_type {
                    if type_path.path.is_ident(&generic_type.ident) {
                        // Treat the `GenericArray` of `T` as terminal
                        return quote! {
                            for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                                fields.push(format!("{}__{}", #index, i));
                            }
                        };
                    }
                }

                return quote! {
                    if let Some(sub_fields) = <#element_type as FlattenFieldsHelper>::flatten_fields() {
                        for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                            for sub_field in &sub_fields {
                                fields.push(format!("{}__{}__{}", #index, i, sub_field));
                            }
                        }
                    } else {
                        for i in 0..<#size_type as typenum::Unsigned>::USIZE {
                            fields.push(format!("{}__{}", #index, i));
                        }
                    }
                };
            }
        }
    }

    // Handle array types `[T; N]`
    if let syn::Type::Array(array_type) = field_type {
        let elem_type = &*array_type.elem; // Dereference the Box to get the inner type
        let array_len = &array_type.len;

        // Check if the array element type is `T`
        if let syn::Type::Path(type_path) = elem_type {
            if type_path.path.is_ident(&generic_type.ident) {
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

    // For other types
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
}
