#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]
#![cfg_attr(not(feature = "external_doc"),
    doc = "See https://docs.rs/num_enum for more info about this crate."
)]

extern crate proc_macro; use ::proc_macro::TokenStream;
use ::proc_macro2::{
    Span,
};
use ::proc_quote::{
    quote,
};
use ::syn::{*,
    parse::{
        Parse,
        ParseStream,
    },
};
use ::std::{*,
    iter::FromIterator,
};

macro_rules! die {
    ($span:expr=>
        $msg:expr
    ) => (
        return Err(Error::new($span, $msg));
    );

    (
        $msg:expr
    ) => (
        die!(Span::call_site() => $msg)
    );
}

fn literal (i: u64) -> Expr
{
    let literal = LitInt::new(
        i,
        syn::IntSuffix::None,
        Span::call_site(),
    );
    parse_quote! {
        #literal
    }
}

struct EnumInfo {
    name: Ident,
    repr: Ident,
    value_expressions_to_enum_keys: Vec<(syn::Expr, syn::Ident)>,
}

impl Parse for EnumInfo {
    fn parse (input: ParseStream) -> Result<Self>
    {Ok({
        let input: DeriveInput = input.parse()?;
        let name = input.ident;
        let data = if let Data::Enum(data) = input.data {
            data
        } else {
            let span = match input.data {
                | Data::Union(data) => data.union_token.span,
                | Data::Struct(data) => data.struct_token.span,
                | _ => unreachable!(),
            };
            die!(span => "Expected enum");
        };

        let repr: Ident = {
            let mut attrs = input.attrs.into_iter();
            loop {
                if let Some(attr) = attrs.next() {
                    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                        if meta_list.ident == "repr" {
                            let mut nested = meta_list.nested.iter();
                            if nested.len() != 1 {
                                die!(meta_list.ident.span()=>
                                    "Expected exactly one `repr` argument"
                                );
                            }
                            let repr = nested.next().unwrap();
                            let repr: Ident = parse_quote! {
                                #repr
                            };
                            if repr == "C" {
                                die!(repr.span()=>
                                    "repr(C) doesn't have a well defined size"
                                );
                            } else {
                                break repr;
                            }
                        }
                    }
                } else {
                    die!("Missing `#[repr({Integer})]` attribute");
                }
            }
        };

        let mut next_discriminant = literal(0);
        let value_expressions_to_enum_keys = Vec::from_iter(
            data.variants
                .into_iter()
                .map(|variant| {
                    let disc = if let Some(d) = variant.discriminant {
                        d.1
                    } else {
                        next_discriminant.clone()
                    };
                    next_discriminant = parse_quote! {
                        #repr::wrapping_add(#disc, 1)
                    };
                    (disc, variant.ident)
                })
        );

        EnumInfo {
            name,
            repr,
            value_expressions_to_enum_keys,
        }
    })}
}

#[proc_macro_derive(IntoPrimitive)] pub
fn derive_into_primitive(input: TokenStream) -> TokenStream
{
    let EnumInfo { name, repr, .. } = parse_macro_input!(input as EnumInfo);

    TokenStream::from(quote! {
        impl From<#name> for #repr {
            #[inline]
            fn from (enum_value: #name) -> Self
            {
                enum_value as Self
            }
        }
    })
}

#[proc_macro_derive(TryFromPrimitive)] pub
fn derive_try_from_primitive(input: TokenStream) -> TokenStream
{
    let EnumInfo {
        name,
        repr,
        value_expressions_to_enum_keys,
    } = parse_macro_input!(input);

    let mut match_const_exprs = Vec::with_capacity(
        value_expressions_to_enum_keys.len()
    );
    let mut enum_keys = Vec::with_capacity(
        value_expressions_to_enum_keys.len()
    );
    value_expressions_to_enum_keys
        .into_iter()
        .for_each(|(enum_value_expression, enum_key)| {
            // Use an intermediate const so that enums defined like
            // `Two = ONE + 1u8` work properly.
            match_const_exprs.push(enum_value_expression.clone());
            enum_keys.push(enum_key);
        })
    ;

    let no_match_message = LitStr::new(&format!(
        "No value in enum `{name}` for value `{{}}`", name=name
    ), Span::call_site());

    let try_into_name_error = Ident::new(
        &format!("TryInto{}Error", &name),
        Span::call_site(),
    );

    let mut expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[doc(hidden)] pub
        struct #try_into_name_error {
            number: #repr,
        }

        impl ::core::fmt::Display for #try_into_name_error {
            fn fmt (
                self: &'_ Self,
                stream: &'_ mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result
            {
                write!(stream,
                    #no_match_message, self.number,
                )
            }
        }
    };
    if cfg!(feature = "std") {
        expanded.extend(quote! {
            impl ::std::error::Error for #try_into_name_error {}
        });
    }
    expanded.extend(quote! {
        impl ::core::convert::TryFrom<#repr> for #name {
            type Error = #try_into_name_error;

            fn try_from (
                number: #repr,
            ) -> ::core::result::Result<Self, #try_into_name_error>
            {
                #![allow(non_upper_case_globals)]
                #(
                    const #enum_keys: #repr =
                        #match_const_exprs
                    ;
                )*
                match number {
                    #(
                        | #enum_keys => ::core::result::Result::Ok(
                            #name::#enum_keys
                        ),
                    )*
                    | _ => ::core::result::Result::Err(
                        #try_into_name_error { number }
                    ),
                }
            }
        }
    });

    expanded.into()
}

#[proc_macro_derive(UnsafeFromPrimitive)] pub
fn derive_unsafe_from_primitive(stream: TokenStream) -> TokenStream
{
    let EnumInfo { name, repr, .. } = parse_macro_input!(stream as EnumInfo);

    let doc_string = LitStr::new(&format!(r#"
Transmutes `number: {repr}` into a [`{name}`].

# Safety

  - `number` must be a valid discriminant of [`{name}`]
"#,
        repr = repr,
        name = name,
    ), Span::call_site());

    TokenStream::from(quote! {
        impl #name {
            #[doc = #doc_string]
            #[inline]
            pub
            unsafe
            fn from_unchecked(number: #repr) -> Self {
                ::core::mem::transmute(number)
            }
        }
    })
}

mod doctest_readme {
    macro_rules! with_doc {(
        #[doc = $doc_string:expr]
        $item:item
    ) => (
        #[doc = $doc_string]
        $item
    )}

    with_doc! {
        #[doc = include_str!("../README.md")]
        extern {}
    }
}
