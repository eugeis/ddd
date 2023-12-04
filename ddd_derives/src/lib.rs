extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AsDslItem)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let dsl_ident = format_ident!("Dsl{ident}Impl");
    let trait_ident = format_ident!("Dsl{ident}");
    let trait_ident_get = format_ident!("Dsl{ident}Get");
    let trait_ident_set = format_ident!("Dsl{ident}Set");
    let dsl_ident_builder = format_ident!("dsl{ident}");
    let dsl_ident_builder_default = format_ident!("dsl{ident}Default");

    let fields = match &ast {
        syn::DeriveInput {
            data:
                syn::Data::Struct(syn::DataStruct {
                    fields: syn::Fields::Named(syn::FieldsNamed { named: fields, .. }),
                    ..
                }),
            ..
        } => fields,
        _ => unimplemented!("derive(AsDslItem) only supports structs with named fields"),
    };

    let dsl_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_empty = format_ident!("{id}_empty");
        let ty = std::option::Option::Some(&field.ty);

        if is_box_type(&field.ty) {
            quote! {
                #id: std::option::Option<#ty>
            }
        } else {
            quote! {
                #id: std::option::Option<#ty>,
                #id_empty: #ty
            }
        }
    });

    let dsl_defaults = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_empty = format_ident!("{id}_empty");
        let ty = &field.ty;
        let empty_value = empty_value(&ty);

        if is_box_type(&field.ty) {
            quote! {
                #id: std::option::Option::None
            }
        } else {
            quote! {
                #id: std::option::Option::None,
                #id_empty: #empty_value
            }
        }
    });

    let getters_def = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_get = format_ident!("{id}_get");
        let ty = field.ty;
        let ty_ref = dsl_type_ref(&ty);

        quote! {
            fn #id_get(&self) -> #ty_ref;
        }
    });

    let getters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let id_get = format_ident!("{id}_get");
        let id_empty = format_ident!("{id}_empty");
        let ty = &field.ty;
        let ty_ref = dsl_type_ref(&ty);

        if is_box_type(&field.ty) {
            quote! {
                fn #id_get(&self) -> #ty_ref {
                    &self.#id
                }
            }
        } else {
            quote! {
                fn #id_get(&self) -> #ty_ref {
                    match &self.#id {
                        Some(v) => &v,
                        None => &self.#id_empty,
                    }
                }
            }
        }
    });

    let setters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        if is_string(&field.ty) {
            quote! {
                fn #id(&mut self, value: &str) -> &mut Self {
                    self.#id = std::option::Option::Some(value.to_owned());
                    self
                }
            }
        } else {
            let ty = std::option::Option::Some(&field.ty);
            quote! {
                fn #id(&mut self, value: #ty) -> &mut Self {
                    self.#id = std::option::Option::Some(value);
                    self
                }
            }
        }
    });


    let setters_def = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        if is_string(&field.ty) {
            quote! {
                fn #id(&mut self, value: &str) -> &mut Self;
            }
        } else {
            let ty = std::option::Option::Some(&field.ty);
            quote! {
                fn #id(&mut self, value: #ty) -> &mut Self;
            }
        }
    });

    let output = quote! {
        pub trait #trait_ident_get {
            #(#getters_def)*
        }

        pub trait #trait_ident_set {
            #(#setters_def)*
        }

        pub trait #trait_ident : #trait_ident_set + #trait_ident_get {}

        #[derive(Default, Debug, Clone, PartialEq)]
        pub struct #dsl_ident {
            #(#dsl_fields),*
        }

        impl #trait_ident_set for #dsl_ident {
            #(#setters)*
        }

        impl #trait_ident_get for #dsl_ident {
            #(#getters)*
        }

        pub fn #dsl_ident_builder_default() -> #dsl_ident {
            #dsl_ident {
                #(#dsl_defaults),*
            }
        }

        pub fn #dsl_ident_builder(adapt: fn(o: &mut #dsl_ident)) -> #dsl_ident {
            let mut ret = #dsl_ident_builder_default();
            adapt(&mut ret);
            ret
        }
    };
    proc_macro::TokenStream::from(output)
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

fn is_box_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            if segment.ident.to_string() == "Box" {
                return true;
            }
        }
    }
    false
}

fn dsl_type_ref(ty: &syn::Type) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Path(path) => {
            let path_str = quote! {#path}.to_string();
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
                _ => quote! { &std::option::Option<#ty> },
            }
        }
        _ => quote! { &#ty },
    }
}

fn is_string(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => {
            let path_str = quote! {#path}.to_string();
            match path_str.as_str() {
                "String" => true,
                _ => false,
            }
        }
        _ => false,
    }
}
