extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(IntoPrimitive)]
pub fn derive_into_primitive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let enum_info = parse_enum(input);

    let name = enum_info.name;
    let repr = enum_info.repr;

    let expanded = quote! {
        impl From<#name> for #repr {
            fn from(number: #name) -> Self {
                number as Self
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(TryFromPrimitive)]
pub fn derive_try_from_primitive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let enum_info = parse_enum(input);

    let TryIntoEnumInfo {
        name,
        repr,
        match_const_names,
        match_const_exprs,
        enum_keys,
        no_match_message,
        ..
    } = TryIntoEnumInfo::from(enum_info);

    let match_const_names2 = match_const_names.clone();
    let repeated_repr = std::iter::repeat(repr.clone()).take(enum_keys.len());
    let repeated_name = std::iter::repeat(name.clone()).take(enum_keys.len());

    let expanded = quote! {
        impl ::std::convert::TryFrom<#repr> for #name {
            type Error=String;

            fn try_from(number: #repr) -> Result<Self, Self::Error> {
                #( const #match_const_names: #repeated_repr = #match_const_exprs; )*

                match number {
                    #( #match_const_names2 => Ok(#repeated_name::#enum_keys), )*
                    _ => Err(format!(#no_match_message, number)),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

struct EnumInfo {
    name: syn::Ident,
    repr: proc_macro2::Ident,
    value_expressions_to_enum_keys: Vec<(syn::Expr, syn::Ident)>,
    visibility: syn::Visibility,
}

fn parse_enum(input: DeriveInput) -> EnumInfo {
    let mut repr = None;
    for attr in input.attrs {
        if attr.path.segments.len() == 1
            && format!("{}", attr.path.segments.first().unwrap().value().ident) == "repr"
        {
            let tokens: Vec<proc_macro2::TokenTree> = attr.tts.into_iter().collect();
            if tokens.len() == 1 {
                if let proc_macro2::TokenTree::Group(ref group) = tokens[0] {
                    match group.stream().into_iter().next().unwrap() {
                        proc_macro2::TokenTree::Ident(ident) => {
                            if &format!("{}", ident) == "C" {
                                panic!("Can't generate num_enum traits for repr(C) enums because they don't have a generally defined size.")
                            }
                            repr = Some(ident);
                            break;
                        }
                        val => {
                            panic!("Got unexpected repr: {}", val);
                        }
                    }
                }
            }
        }
    }

    let mut variants = vec![];

    let mut next_discriminant = literal(0);

    if let Data::Enum(data) = input.data {
        for variant in data.variants {
            let disc = if let Some(d) = variant.discriminant {
                d.1
            } else {
                next_discriminant
            };
            next_discriminant = syn::Expr::Binary(syn::ExprBinary {
                attrs: vec![],
                left: Box::new(disc.clone()),
                op: syn::BinOp::Add(syn::token::Add {
                    spans: [proc_macro2::Span::call_site()],
                }),
                right: Box::new(literal(1)),
            });
            variants.push((disc, variant.ident.clone()));
        }
    } else {
        panic!("Can only operate on enums");
    }

    EnumInfo {
        name: input.ident,
        repr: repr.expect("Couldn't find repr for enum"),
        value_expressions_to_enum_keys: variants,
        visibility: input.vis,
    }
}

fn literal(i: u64) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(syn::LitInt::new(
            i,
            syn::IntSuffix::None,
            proc_macro2::Span::call_site(),
        )),
    })
}

struct TryIntoEnumInfo {
    name: proc_macro2::Ident,
    visibility: syn::Visibility,
    repr: proc_macro2::Ident,
    match_const_names: Vec<proc_macro2::Ident>,
    match_const_exprs: Vec<syn::Expr>,
    enum_keys: Vec<proc_macro2::Ident>,
    no_match_message: String,
}

impl TryIntoEnumInfo {
    fn from(enum_info: EnumInfo) -> TryIntoEnumInfo {
        let mut match_const_names =
            Vec::with_capacity(enum_info.value_expressions_to_enum_keys.len());
        let mut match_const_exprs =
            Vec::with_capacity(enum_info.value_expressions_to_enum_keys.len());
        let mut enum_keys = Vec::with_capacity(enum_info.value_expressions_to_enum_keys.len());

        for (enum_value_expression, enum_key) in enum_info.value_expressions_to_enum_keys {
            // Use an intermediate const so that enums defined like `Two = ONE + 1u8` work properly.
            let match_const = format!("__num_enum_match_{}", enum_key);
            match_const_names.push(proc_macro2::Ident::new(
                &match_const,
                proc_macro2::Span::call_site(),
            ));
            match_const_exprs.push(enum_value_expression.clone());
            enum_keys.push(enum_key);
        }

        let no_match_message = format!("No value in enum {} for value {{}}", enum_info.name);

        TryIntoEnumInfo {
            name: enum_info.name,
            visibility: enum_info.visibility,
            repr: enum_info.repr,
            match_const_names: match_const_names,
            match_const_exprs: match_const_exprs,
            enum_keys: enum_keys,
            no_match_message: no_match_message,
        }
    }
}
