extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let builder_ident = format_ident!("{ident}Builder");
    let trait_ident = format_ident!("{ident}I");

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
        _ => unimplemented!("derive(Builder) only supports structs with named fields")
    };

    let builder_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = try_optional(&field.ty).or(std::option::Option::Some(field.ty));

        quote! { #id: std::option::Option<#ty> }
    });

    let builder_defaults = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        quote! { #id: std::option::Option::None }
    });

    let getters_def = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;

        quote! {
            fn #id(&self) -> #ty;
        }
    });

    let getters = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let ty = field.ty;
        
        quote! {
            fn #id(&self) -> #ty {
                self.#id.clone()
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

    let build_checkers = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();
        let err = format!("{id} was not set");

        if try_optional(&field.ty).is_none() {
            quote! {
                if self.#id.is_none() {
                    return std::result::Result::Err(#err.into());
                }
            }
        } else {
            quote! {}
        }
    });

    let build_fields = fields.iter().map(|field| {
        let field = field.clone();
        let id = field.ident.unwrap();

        if try_optional(&field.ty).is_some() {
            quote! {
               #id: self.#id.clone()
            }
        } else {
            quote! {
               #id: self.#id.clone().unwrap()
            }
        }
    });

    let output = quote! {
        trait #trait_ident {
            #(#getters_def)*
        }

        impl #trait_ident for #ident {
            #(#getters)*
        }

        pub struct #builder_ident {
            #(#builder_fields),*
        }

        impl #builder_ident {
            #(#setters)*

            pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                #(#build_checkers);*

                std::result::Result::Ok(#ident {
                    #(#build_fields),*
                })
            }
        }

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident { 
                    #(#builder_defaults),*
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