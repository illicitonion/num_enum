extern crate proc_macro;

use ::proc_macro::TokenStream;
use ::proc_macro2::Span;
use ::quote::{format_ident, quote};
use ::std::iter::FromIterator;
use ::syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Data, DeriveInput, Error, Expr, Ident, LitInt, LitStr, Meta,
    Result,
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

fn literal(i: u64) -> Expr {
    let literal = LitInt::new(&i.to_string(), Span::call_site());
    parse_quote! {
        #literal
    }
}

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(alternatives);
}

enum NumEnumVariantAttributes {
    Default,
    Alternatives(VariantAlternativesAttribute),
}

impl Parse for NumEnumVariantAttributes {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::default) {
            let _: kw::default = input.parse()?;
            Ok(Self::Default)
        } else if lookahead.peek(kw::alternatives) {
            input.parse().map(Self::Alternatives)
        } else {
            Err(lookahead.error())
        }
    }
}

struct VariantAlternativesAttribute {
    _keyword: kw::alternatives,
    _eq_token: syn::Token![=],
    _bracket_token: syn::token::Bracket,
    expressions: syn::punctuated::Punctuated<Expr, syn::Token![,]>,
}

impl Parse for VariantAlternativesAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            _keyword: input.parse()?,
            _eq_token: input.parse()?,
            _bracket_token: syn::bracketed!(content in input),
            expressions: content.parse_terminated(Expr::parse)?,
        })
    }
}

struct VariantInfo {
    ident: Ident,
    canonical_value: Expr,
    alternative_values: Vec<Expr>,
}

struct EnumInfo {
    name: Ident,
    repr: Ident,
    variant_infos: Vec<VariantInfo>,
    default_variant: Option<Ident>,
}

struct CanonicalAndAlternatives<T> {
    canonical: Vec<T>,
    alternatives: Vec<Vec<T>>,
}

impl EnumInfo {
    fn idents(&self) -> CanonicalAndAlternatives<Ident> {
        let (canonical, alternatives): (Vec<Ident>, Vec<Vec<Ident>>) = self
            .variant_infos
            .iter()
            .map(|info| {
                let canonical_ident = info.ident.clone();
                let alternative_idents = (0..info.alternative_values.len())
                    .map(|index| format_ident!("{}_{}", info.ident, index + 1))
                    .collect();
                (canonical_ident, alternative_idents)
            })
            .unzip();

        CanonicalAndAlternatives { canonical, alternatives }
    }

    fn expressions(&self) -> CanonicalAndAlternatives<Expr> {
        let (canonical, alternatives): (Vec<Expr>, Vec<Vec<Expr>>) = self
            .variant_infos
            .iter()
            .map(|info| (info.canonical_value.clone(), info.alternative_values.clone()))
            .unzip();

        CanonicalAndAlternatives { canonical, alternatives }
    }
}

impl Parse for EnumInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok({
            let input: DeriveInput = input.parse()?;
            let name = input.ident;
            let data = if let Data::Enum(data) = input.data {
                data
            } else {
                let span = match input.data {
                    Data::Union(data) => data.union_token.span,
                    Data::Struct(data) => data.struct_token.span,
                    _ => unreachable!(),
                };
                die!(span => "Expected enum");
            };

            let repr: Ident = {
                let mut attrs = input.attrs.into_iter();
                loop {
                    if let Some(attr) = attrs.next() {
                        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                            if let Some(ident) = meta_list.path.get_ident() {
                                if ident == "repr" {
                                    let mut nested = meta_list.nested.iter();
                                    if nested.len() != 1 {
                                        die!(ident.span()=>
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
                        }
                    } else {
                        die!("Missing `#[repr({Integer})]` attribute");
                    }
                }
            };

            let mut default_variant: Option<Ident> = None;

            let mut next_discriminant = literal(0);
            let variant_infos = Vec::from_iter(data.variants.into_iter().map(|variant| {
                let ident = variant.ident;
                let variant_ident = ident.clone();
                let canonical_value = if let Some(d) = variant.discriminant {
                    d.1
                } else {
                    next_discriminant.clone()
                };

                let alternative_values: Vec<Expr> = variant
                    .attrs
                    .iter()
                    .filter_map(|attribute| {
                        match attribute.parse_args_with(NumEnumVariantAttributes::parse) {
                            Ok(NumEnumVariantAttributes::Default) => {
                                if default_variant.is_some() {
                                    panic!("Multiple variants marked `#[num_enum(default)]` found");
                                }
                                default_variant = Some(ident.clone());
                                None
                            }
                            Ok(NumEnumVariantAttributes::Alternatives(alternatives)) => {
                                Some(alternatives.expressions.into_iter())
                            }
                            Err(_) => None,
                        }
                    })
                    .flatten()
                    .collect();

                let info = VariantInfo {
                    ident,
                    canonical_value,
                    alternative_values,
                };
                next_discriminant = parse_quote! {
                    #repr::wrapping_add(#variant_ident, 1)
                };
                info
            }));

            EnumInfo {
                name,
                repr,
                variant_infos,
                default_variant,
            }
        })
    }
}

/// Implements `Into<Primitive>` for a `#[repr(Primitive)] enum`.
///
/// (It actually implements `From<Enum> for Primitive`)
///
/// ## Allows turning an enum into a primitive.
///
/// ```rust
/// use num_enum::IntoPrimitive;
///
/// #[derive(IntoPrimitive)]
/// #[repr(u8)]
/// enum Number {
///     Zero,
///     One,
/// }
///
/// fn main() {
///     let zero: u8 = Number::Zero.into();
///     assert_eq!(zero, 0u8);
/// }
/// ```
#[proc_macro_derive(IntoPrimitive)]
pub fn derive_into_primitive(input: TokenStream) -> TokenStream {
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

/// Implements `TryFrom<Primitive>` for a `#[repr(Primitive)] enum`.
///
/// Attempting to turn a primitive into an enum with try_from.
/// ----------------------------------------------
///
/// ```rust
/// use num_enum::TryFromPrimitive;
/// use std::convert::TryFrom;
///
/// #[derive(Debug, Eq, PartialEq, FromPrimitive)]
/// #[repr(u8)]
/// enum Number {
///     Zero,
///     #[num_enum(default)]
///     NonZero,
/// }
///
/// fn main() {
///     let zero = Number::try_from(0u8);
///     assert_eq!(zero, Ok(Number::Zero));
///
///     let one = Number::try_from(1u8);
///     assert_eq!(one, Ok(Number::NonZero));
///
///     let two = Number::try_from(2u8);
///     assert_eq!(two, Ok(Number::NonZero));
/// }
/// ```
#[proc_macro_derive(FromPrimitive, attributes(num_enum))]
pub fn derive_from_primitive(input: TokenStream) -> TokenStream {
    let enum_info: EnumInfo = parse_macro_input!(input);
    let krate = Ident::new(&get_crate_name(), Span::call_site());

    // panic!("{:#?}", enum_info);

    let CanonicalAndAlternatives {
        canonical: canonical_idents,
        alternatives: alternative_idents,
    } = enum_info.idents();
    let CanonicalAndAlternatives {
        canonical: canonical_expressions,
        alternatives: alternative_expressions,
    } = enum_info.expressions();

    debug_assert_eq!(canonical_idents.len(), canonical_expressions.len());
    debug_assert_eq!(alternative_idents.len(), alternative_expressions.len());

    let EnumInfo {
        name,
        repr,
        default_variant,
        ..
    } = enum_info;

    let default_ident = default_variant
        .expect("#[derive(FromPrimitive)] requires a variant marked with `#[num_enum(default)]`");

    TokenStream::from(quote! {
        impl ::#krate::FromPrimitive for #name {
            type Primitive = #repr;

            fn from_primitive(number: Self::Primitive) -> Self {
                #![deny(unreachable_patterns)]

                // Use intermediate const(s) so that enums defined like
                // `Two = ONE + 1u8` work properly.
                #![allow(non_upper_case_globals)]
                #(
                    const #canonical_idents: #repr =
                        #canonical_expressions
                    ;
                    #(
                        const #alternative_idents: #repr =
                            #alternative_expressions
                        ;
                    )*
                )*
                match number {
                    #(
                        | #canonical_idents #(| #alternative_idents )* => Self::#canonical_idents,
                    )*
                    | _ => Self::#default_ident,
                }
            }
        }

        impl ::core::convert::From<#repr> for #name {
            #[inline]
            fn from (
                number: #repr,
            ) -> Self {
                ::#krate::FromPrimitive::from_primitive(number)
            }
        }

        // The Rust stdlib will implement `#name: From<#repr>` for us for free!

        impl ::#krate::TryFromPrimitive for #name {
            type Primitive = #repr;

            const NAME: &'static str = stringify!(#name);

            #[inline]
            fn try_from_primitive (
                number: Self::Primitive,
            ) -> ::core::result::Result<
                Self,
                ::#krate::TryFromPrimitiveError<Self>,
            >
            {
                Ok(::#krate::FromPrimitive::from_primitive(number))
            }
        }
    })
}

/// Implements `TryFrom<Primitive>` for a `#[repr(Primitive)] enum`.
///
/// Attempting to turn a primitive into an enum with try_from.
/// ----------------------------------------------
///
/// ```rust
/// use num_enum::TryFromPrimitive;
/// use std::convert::TryFrom;
///
/// #[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
/// #[repr(u8)]
/// enum Number {
///     Zero,
///     One,
/// }
///
/// fn main() {
///     let zero = Number::try_from(0u8);
///     assert_eq!(zero, Ok(Number::Zero));
///
///     let three = Number::try_from(3u8);
///     assert_eq!(
///         three.unwrap_err().to_string(),
///         "No discriminant in enum `Number` matches the value `3`",
///     );
/// }
/// ```
#[proc_macro_derive(TryFromPrimitive, attributes(num_enum))]
pub fn derive_try_from_primitive(input: TokenStream) -> TokenStream {
    let enum_info: EnumInfo = parse_macro_input!(input);
    let krate = Ident::new(&get_crate_name(), Span::call_site());

    let CanonicalAndAlternatives {
        canonical: canonical_idents,
        alternatives: alternative_idents,
    } = enum_info.idents();
    let CanonicalAndAlternatives {
        canonical: canonical_expressions,
        alternatives: alternative_expressions,
    } = enum_info.expressions();

    debug_assert_eq!(canonical_idents.len(), canonical_expressions.len());
    debug_assert_eq!(alternative_idents.len(), alternative_expressions.len());

    let EnumInfo {
        name,
        repr,
        default_variant,
        ..
    } = enum_info;

    let default_arm = match default_variant {
        Some(ident) => {
            quote! {
                ::core::result::Result::Ok(
                    #name::#ident
                )
            }
        }
        None => {
            quote! {
                ::core::result::Result::Err(
                    ::#krate::TryFromPrimitiveError { number }
                )
            }
        }
    };

    TokenStream::from(quote! {
        impl ::#krate::TryFromPrimitive for #name {
            type Primitive = #repr;

            const NAME: &'static str = stringify!(#name);

            fn try_from_primitive (
                number: Self::Primitive,
            ) -> ::core::result::Result<
                Self,
                ::#krate::TryFromPrimitiveError<Self>,
            >
            {
                #![deny(unreachable_patterns)]

                // Use intermediate const(s) so that enums defined like
                // `Two = ONE + 1u8` work properly.
                #![allow(non_upper_case_globals)]
                #(
                    const #canonical_idents: #repr =
                        #canonical_expressions
                    ;
                    #(
                        const #alternative_idents: #repr =
                            #alternative_expressions
                        ;
                    )*
                )*
                match number {
                    #(
                        | #canonical_idents #(| #alternative_idents )* => ::core::result::Result::Ok(
                            Self::#canonical_idents
                        ),
                    )*
                    | _ => #default_arm,
                }
            }
        }

        impl ::core::convert::TryFrom<#repr> for #name {
            type Error = ::#krate::TryFromPrimitiveError<Self>;

            #[inline]
            fn try_from (
                number: #repr,
            ) -> ::core::result::Result<
                    Self,
                    ::#krate::TryFromPrimitiveError<Self>,
                >
            {
                ::#krate::TryFromPrimitive::try_from_primitive(number)
            }
        }
    })
}

#[cfg(feature = "proc-macro-crate")]
fn get_crate_name() -> String {
    ::proc_macro_crate::crate_name("num_enum").unwrap_or_else(|err| {
        eprintln!("Warning: {}\n    => defaulting to `num_enum`", err,);
        String::from("num_enum")
    })
}

// Don't depend on proc-macro-crate in no_std environments because it causes an awkward depndency
// on serde with std.
//
// no_std dependees on num_enum cannot rename the num_enum crate when they depend on it. Sorry.
//
// See https://github.com/illicitonion/num_enum/issues/18
#[cfg(not(feature = "proc-macro-crate"))]
fn get_crate_name() -> String {
    String::from("num_enum")
}

/// Generates a `unsafe fn from_unchecked (number: Primitive) -> Self`
/// associated function.
///
/// Allows unsafely turning a primitive into an enum with from_unchecked.
/// -------------------------------------------------------------
///
/// If you're really certain a conversion will succeed, and want to avoid a small amount of overhead, you can use unsafe
/// code to do this conversion. Unless you have data showing that the match statement generated in the `try_from` above is a
/// bottleneck for you, you should avoid doing this, as the unsafe code has potential to cause serious memory issues in
/// your program.
///
/// ```rust
/// use num_enum::UnsafeFromPrimitive;
///
/// #[derive(Debug, Eq, PartialEq, UnsafeFromPrimitive)]
/// #[repr(u8)]
/// enum Number {
///     Zero,
///     One,
/// }
///
/// fn main() {
///     assert_eq!(
///         Number::Zero,
///         unsafe { Number::from_unchecked(0_u8) },
///     );
///     assert_eq!(
///         Number::One,
///         unsafe { Number::from_unchecked(1_u8) },
///     );
/// }
///
/// unsafe fn undefined_behavior() {
///     let _ = Number::from_unchecked(2); // 2 is not a valid discriminant!
/// }
/// ```
#[proc_macro_derive(UnsafeFromPrimitive, attributes(num_enum))]
pub fn derive_unsafe_from_primitive(stream: TokenStream) -> TokenStream {
    let EnumInfo {
        name,
        repr,
        variant_infos,
        default_variant,
        ..
    } = parse_macro_input!(stream as EnumInfo);

    if default_variant.is_some() {
        panic!("#[derive(UnsafeFromPrimitive)] does not support `#[num_enum(default)]`");
    }

    if variant_infos
        .iter()
        .any(|info| !info.alternative_values.is_empty())
    {
        panic!("#[derive(UnsafeFromPrimitive)] does not support `#[num_enum(alternatives = [..])]`");
    }

    let doc_string = LitStr::new(
        &format!(
            r#"
Transmutes `number: {repr}` into a [`{name}`].

# Safety

  - `number` must represent a valid discriminant of [`{name}`]
"#,
            repr = repr,
            name = name,
        ),
        Span::call_site(),
    );

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
