extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::proc_macro2::Span;
use ::quote::quote;
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

struct EnumInfo {
    name: Ident,
    repr: Ident,
    value_expressions_to_enum_keys: Vec<(Expr, Ident)>,
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

            let mut next_discriminant = literal(0);
            let value_expressions_to_enum_keys =
                Vec::from_iter(data.variants.into_iter().map(|variant| {
                    let disc = if let Some(d) = variant.discriminant {
                        d.1
                    } else {
                        next_discriminant.clone()
                    };
                    let ref variant_ident = variant.ident;
                    next_discriminant = parse_quote! {
                        #repr::wrapping_add(#variant_ident, 1)
                    };
                    (disc, variant.ident)
                }));

            EnumInfo {
                name,
                repr,
                value_expressions_to_enum_keys,
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
#[proc_macro_derive(TryFromPrimitive)]
pub fn derive_try_from_primitive(input: TokenStream) -> TokenStream {
    let EnumInfo {
        name,
        repr,
        value_expressions_to_enum_keys,
    } = parse_macro_input!(input);

    let (match_const_exprs, enum_keys): (Vec<Expr>, Vec<Ident>) =
        value_expressions_to_enum_keys.into_iter().unzip();

    let krate = Ident::new(&get_crate_name(), Span::call_site());

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
                // Use intermediate const(s) so that enums defined like
                // `Two = ONE + 1u8` work properly.
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
                        ::#krate::TryFromPrimitiveError { number }
                    ),
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
#[proc_macro_derive(UnsafeFromPrimitive)]
pub fn derive_unsafe_from_primitive(stream: TokenStream) -> TokenStream {
    let EnumInfo { name, repr, .. } = parse_macro_input!(stream as EnumInfo);

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
