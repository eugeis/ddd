extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AsDslItem)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let dsl_ident = format_ident!("Dsl{ident}Impl");
    let trait_ident = format_ident!("Dsl{ident}");

    let fields = match &ast {
        syn::DeriveInput{
            data: syn::Data::Struct(
                syn::DataStruct{
                    fields: syn::Fields::Named (
                        syn::FieldsNamed{
                            named: fields,
                            ..
                        },
                    ),
                    ..
                },

            ),
            ..
        } => {
            fields
        },
        _ => unimplemented!("derive(AsDslItem) only supports structs with named fields")
    };

    let dsl_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_empty = format_ident!("{id}_empty");
        let ty = try_optional(&field.ty).or(std::option::Option::Some(field.ty));
        
        quote! { 
            #id: std::option::Option<#ty>,
            #id_empty: #ty
         }
    });

    let dsl_defaults = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_empty = format_ident!("{id}_empty");
        let ty = field.ty;
        let empty_value = empty_value(&ty);

        quote! { 
            #id: std::option::Option::None,
            #id_empty: #empty_value
        }
    });

    let getters_def = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;
        let ty_ref = type_ref(&ty);

        quote! {
            fn #id(&self) -> #ty_ref;
        }
    });

    let getters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_empty = format_ident!("{id}_empty");
        let ty = field.ty;
        let ty_ref = type_ref(&ty);

        quote! {
            fn #id(&self) -> #ty_ref {
                match &self.#id {
                    Some(v) => &v,
                    None => &self.#id_empty,
                }
            }
        }
    });

    let setters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = try_optional(&field.ty).or(std::option::Option::Some(field.ty));

        quote! {
            pub fn #id(&mut self, value: #ty) -> &mut Self {
                self.#id = std::option::Option::Some(value);
                self
            }
        }
    });

    let output = quote! {
        pub trait #trait_ident {
            #(#getters_def)*
        }

        pub struct #dsl_ident {
            #(#dsl_fields),*
        }

        impl #dsl_ident {
            #(#setters)*
        }

        impl #trait_ident for #dsl_ident {
            #(#getters)*
        }

        impl #ident {
            pub fn dsl() -> #dsl_ident {
                #dsl_ident {
                    #(#dsl_defaults),*
                }
            }
        }
    };
    proc_macro::TokenStream::from(output)
}

fn try_optional(ty: &syn::Type) -> std::option::Option<syn::Type> {
    // Pull out the first path segments (containing just the Option)
    // Verify that there's exactly one value in the path
    let segments = match ty {
        syn::Type::Path(
            syn::TypePath{
                path: syn::Path {
                    segments,
                    ..
                },
                ..
            }
        ) 
        if segments.len() == 1
        => segments.clone(),
        _ => return std::option::Option::None,
    };

    // Pull out the first arg segment in the Option
    // Verify that there's exactly one parameter
    let args = match &segments[0] {
        syn::PathSegment{
            ident,
            arguments: syn::PathArguments::AngleBracketed(
                syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }
            )
        }
        if ident == "Option" && args.len() == 1
        => args,
        _ => return std::option::Option::None,
    };

    // Extract that single type
    // Verify that there's exactly one
    // TODO: Future case should deal with things like lifetimes etc that could also be in here
    let ty = match &args[0] {
        syn::GenericArgument::Type(t) => t,
        _ => return std::option::Option::None
    };

    Some(ty.clone())
}

fn empty_value(ty: &syn::Type) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Path(path) => {
            let path_str = quote::quote! {#path}.to_string();
            match path_str.as_str() {
                "String" => quote! { String::new() },
                "bool" => quote! { false },
                "i8" => quote! { 0i8 },
                "u8" => quote! { 0u8 },
                "i16" => quote! { 0i16 },
                "u16" => quote! { 0u16 },
                "i32" => quote! { 0i32 },
                "u32" => quote! { 0u32 },
                "i64" => quote! { 0i64 },
                "u64" => quote! { 0u64 },
                "i128" => quote! { 0i128 },
                "u128" => quote! { 0u128 },
                "isize" => quote! { 0isize },
                "usize" => quote! { 0usize },
                "f32" => quote! { 0.0f32 },
                "f64" => quote! { 0.0f64 },
                _ => quote! { std::default::Default::default() },
            }
        }
        _ => quote! { std::default::Default::default() },
    }
}

fn type_ref(ty: &syn::Type) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Path(path) => {
            let path_str = quote::quote! {#path}.to_string();
            match path_str.as_str() {
                "String" => quote! { &str },
                "bool" => quote! { &bool },
                "i8" => quote! { &i8 },
                "u8" => quote! { &u8 },
                "i16" => quote! { &i16 },
                "u16" => quote! { &u16 },
                "i32" => quote! { &i13 },
                "u32" => quote! { &u32 },
                "i64" => quote! { &i64 },
                "u64" => quote! { &i64 },
                "i128" => quote! { &i128 },
                "u128" => quote! { &&u128 },
                "isize" => quote! { &isize },
                "usize" => quote! { &usize },
                "f32" => quote! { &f32 },
                "f64" => quote! { &f64 },
                _ => quote! { &std::default::Default::default() },
            }
        }
        _ => quote! { &path_str },
    }
}