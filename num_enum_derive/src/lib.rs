extern crate proc_macro;

use ::proc_macro::TokenStream;
use ::proc_macro2::Span;
use ::quote::{format_ident, quote};
use ::syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Data, DeriveInput, Error, Expr, Ident, LitInt, LitStr, Meta, Result,
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

struct NumEnumVariantAttributes {
    items: syn::punctuated::Punctuated<NumEnumVariantAttributeItem, syn::Token![,]>,
}

impl Parse for NumEnumVariantAttributes {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            items: input.parse_terminated(NumEnumVariantAttributeItem::parse)?,
        })
    }
}

enum NumEnumVariantAttributeItem {
    Default(VariantDefaultAttribute),
    Alternatives(VariantAlternativesAttribute),
}

impl Parse for NumEnumVariantAttributeItem {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::default) {
            input.parse().map(Self::Default)
        } else if lookahead.peek(kw::alternatives) {
            input.parse().map(Self::Alternatives)
        } else {
            Err(lookahead.error())
        }
    }
}

struct VariantDefaultAttribute {
    keyword: kw::default,
}

impl Parse for VariantDefaultAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            keyword: input.parse()?,
        })
    }
}

impl Spanned for VariantDefaultAttribute {
    fn span(&self) -> Span {
        self.keyword.span()
    }
}

struct VariantAlternativesAttribute {
    keyword: kw::alternatives,
    _eq_token: syn::Token![=],
    _bracket_token: syn::token::Bracket,
    expressions: syn::punctuated::Punctuated<Expr, syn::Token![,]>,
}

impl Parse for VariantAlternativesAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            keyword: input.parse()?,
            _eq_token: input.parse()?,
            _bracket_token: syn::bracketed!(content in input),
            expressions: content.parse_terminated(Expr::parse)?,
        })
    }
}

impl Spanned for VariantAlternativesAttribute {
    fn span(&self) -> Span {
        self.keyword.span()
    }
}

#[derive(Default)]
struct AttributeSpans {
    default: Vec<Span>,
    alternatives: Vec<Span>,
}

struct VariantInfo {
    ident: Ident,
    attr_spans: AttributeSpans,
    is_default: bool,
    canonical_value: Expr,
    alternative_values: Vec<Expr>,
}

impl VariantInfo {
    fn all_values(&self) -> impl Iterator<Item = &Expr> {
        ::core::iter::once(&self.canonical_value).chain(self.alternative_values.iter())
    }

    fn is_complex(&self) -> bool {
        !self.alternative_values.is_empty()
    }
}

struct EnumInfo {
    name: Ident,
    repr: Ident,
    variants: Vec<VariantInfo>,
}

impl EnumInfo {
    fn has_default_variant(&self) -> bool {
        self.default().is_some()
    }

    fn has_complex_variant(&self) -> bool {
        self.variants.iter().any(|info| info.is_complex())
    }

    fn default(&self) -> Option<&Ident> {
        self.variants
            .iter()
            .find(|info| info.is_default)
            .map(|info| &info.ident)
    }

    fn first_default_attr_span(&self) -> Option<&Span> {
        self.variants
            .iter()
            .find_map(|info| info.attr_spans.default.first())
    }

    fn first_alternatives_attr_span(&self) -> Option<&Span> {
        self.variants
            .iter()
            .find_map(|info| info.attr_spans.alternatives.first())
    }

    fn variant_idents(&self) -> Vec<Ident> {
        self.variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect()
    }

    fn expression_idents(&self) -> Vec<Vec<Ident>> {
        self.variants
            .iter()
            .map(|info| {
                let indices = 0..(info.alternative_values.len() + 1);
                indices
                    .map(|index| format_ident!("{}__num_enum_{}__", info.ident, index))
                    .collect()
            })
            .collect()
    }

    fn variant_expressions(&self) -> Vec<Vec<Expr>> {
        self.variants
            .iter()
            .map(|variant| variant.all_values().cloned().collect())
            .collect()
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

            let mut variants: Vec<VariantInfo> = vec![];
            let mut has_default_variant: bool = false;

            let mut next_discriminant = literal(0);
            for variant in data.variants.into_iter() {
                let ident = variant.ident;

                let discriminant = match variant.discriminant {
                    Some(d) => d.1,
                    None => next_discriminant.clone(),
                };

                let mut attr_spans: AttributeSpans = Default::default();
                let mut alternative_values: Vec<Expr> = vec![];

                // `#[num_enum(default)]` is required by `#[derive(FromPrimitive)]`
                // and forbidden by `#[derive(UnsafeFromPrimitive)]`, so we need to
                // keep track of whether we encountered such an attribute:
                let mut is_default: bool = false;

                for attribute in variant.attrs {
                    if !attribute.path.is_ident("num_enum") {
                        continue;
                    }
                    match attribute.parse_args_with(NumEnumVariantAttributes::parse) {
                        Ok(variant_attributes) => {
                            for variant_attribute in variant_attributes.items.iter() {
                                match variant_attribute {
                                    NumEnumVariantAttributeItem::Default(default) => {
                                        if has_default_variant {
                                            die!(default.span()=>
                                                "Multiple variants marked `#[num_enum(default)]` found"
                                            );
                                        }
                                        attr_spans.default.push(default.span());
                                        is_default = true;
                                    }
                                    NumEnumVariantAttributeItem::Alternatives(alternatives) => {
                                        attr_spans.alternatives.push(alternatives.span());
                                        alternative_values
                                            .extend(alternatives.expressions.iter().cloned());
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            die!(attribute.span()=>
                                format!("Invalid attribute: {}", err)
                            );
                        }
                    }

                    has_default_variant |= is_default;
                }

                let canonical_value = discriminant.clone();

                variants.push(VariantInfo {
                    ident,
                    attr_spans,
                    is_default,
                    canonical_value,
                    alternative_values,
                });

                next_discriminant = parse_quote! {
                    #repr::wrapping_add(#discriminant, 1)
                };
            }

            EnumInfo {
                name,
                repr,
                variants,
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

/// Implements `From<Primitive>` for a `#[repr(Primitive)] enum`.
///
/// Turning a primitive into an enum with `from`.
/// ----------------------------------------------
///
/// ```rust
/// use num_enum::FromPrimitive;
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
///     let zero = Number::from(0u8);
///     assert_eq!(zero, Number::Zero);
///
///     let one = Number::from(1u8);
///     assert_eq!(one, Number::NonZero);
///
///     let two = Number::from(2u8);
///     assert_eq!(two, Number::NonZero);
/// }
/// ```
#[proc_macro_derive(FromPrimitive, attributes(num_enum))]
pub fn derive_from_primitive(input: TokenStream) -> TokenStream {
    let enum_info: EnumInfo = parse_macro_input!(input);
    let krate = Ident::new(&get_crate_name(), Span::call_site());

    let default_ident: Ident = match enum_info.default() {
        Some(ident) => ident.clone(),
        None => {
            let span = Span::call_site();
            let message =
                "#[derive(FromPrimitive)] requires a variant marked with `#[num_enum(default)]`";
            return syn::Error::new(span, message).to_compile_error().into();
        }
    };

    let EnumInfo {
        ref name, ref repr, ..
    } = enum_info;

    let variant_idents: Vec<Ident> = enum_info.variant_idents();
    let expression_idents: Vec<Vec<Ident>> = enum_info.expression_idents();
    let variant_expressions: Vec<Vec<Expr>> = enum_info.variant_expressions();

    debug_assert_eq!(variant_idents.len(), variant_expressions.len());

    TokenStream::from(quote! {
        impl ::#krate::FromPrimitive for #name {
            type Primitive = #repr;

            fn from_primitive(number: Self::Primitive) -> Self {
                // Use intermediate const(s) so that enums defined like
                // `Two = ONE + 1u8` work properly.
                #![allow(non_upper_case_globals)]
                #(
                    #(
                        const #expression_idents: #repr = #variant_expressions;
                    )*
                )*
                #[deny(unreachable_patterns)]
                match number {
                    #(
                        #( #expression_idents )|*
                        => Self::#variant_idents,
                    )*
                    #[allow(unreachable_patterns)]
                    _ => Self::#default_ident,
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
/// Attempting to turn a primitive into an enum with `try_from`.
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

    let EnumInfo {
        ref name, ref repr, ..
    } = enum_info;

    let variant_idents: Vec<Ident> = enum_info.variant_idents();
    let expression_idents: Vec<Vec<Ident>> = enum_info.expression_idents();
    let variant_expressions: Vec<Vec<Expr>> = enum_info.variant_expressions();

    debug_assert_eq!(variant_idents.len(), variant_expressions.len());

    let default_arm = match enum_info.default() {
        Some(ident) => {
            quote! {
                _ => ::core::result::Result::Ok(
                    #name::#ident
                )
            }
        }
        None => {
            quote! {
                _ => ::core::result::Result::Err(
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
                ::#krate::TryFromPrimitiveError<Self>
            > {
                // Use intermediate const(s) so that enums defined like
                // `Two = ONE + 1u8` work properly.
                #![allow(non_upper_case_globals)]
                #(
                    #(
                        const #expression_idents: #repr = #variant_expressions;
                    )*
                )*
                #[deny(unreachable_patterns)]
                match number {
                    #(
                        #( #expression_idents )|*
                        => ::core::result::Result::Ok(Self::#variant_idents),
                    )*
                    #[allow(unreachable_patterns)]
                    #default_arm,
                }
            }
        }

        impl ::core::convert::TryFrom<#repr> for #name {
            type Error = ::#krate::TryFromPrimitiveError<Self>;

            #[inline]
            fn try_from (
                number: #repr,
            ) -> ::core::result::Result<Self, ::#krate::TryFromPrimitiveError<Self>>
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

// Don't depend on proc-macro-crate in no_std environments because it causes an awkward dependency
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
    let enum_info = parse_macro_input!(stream as EnumInfo);

    if enum_info.has_default_variant() {
        let span = enum_info
            .first_default_attr_span()
            .cloned()
            .expect("Expected span");
        let message = "#[derive(UnsafeFromPrimitive)] does not support `#[num_enum(default)]`";
        return syn::Error::new(span, message).to_compile_error().into();
    }

    if enum_info.has_complex_variant() {
        let span = enum_info
            .first_alternatives_attr_span()
            .cloned()
            .expect("Expected span");
        let message =
            "#[derive(UnsafeFromPrimitive)] does not support `#[num_enum(alternatives = [..])]`";
        return syn::Error::new(span, message).to_compile_error().into();
    }

    let EnumInfo {
        ref name, ref repr, ..
    } = enum_info;

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
            pub unsafe fn from_unchecked(number: #repr) -> Self {
                ::core::mem::transmute(number)
            }
        }
    })
}
