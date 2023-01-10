extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Error, Expr, Fields, Ident, Lit, LitInt, LitStr, Meta, Result,
};

macro_rules! die {
    ($spanned:expr=>
        $msg:expr
    ) => {
        return Err(Error::new_spanned($spanned, $msg))
    };

    (
        $msg:expr
    ) => {
        return Err(Error::new(Span::call_site(), $msg))
    };
}

fn literal(i: i128) -> Expr {
    let literal = LitInt::new(&i.to_string(), Span::call_site());
    parse_quote! {
        #literal
    }
}

fn expr_to_int(val_exp: &Expr) -> Result<i128> {
    Ok(match val_exp {
        Expr::Lit(ref val_exp_lit) => match val_exp_lit.lit {
            Lit::Int(ref lit_int) => lit_int.base10_parse()?,
            _ => die!(val_exp => "Expected integer"),
        },
        _ => die!(val_exp => "Expected literal"),
    })
}

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(catch_all);
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
    CatchAll(VariantCatchAllAttribute),
    Alternatives(VariantAlternativesAttribute),
}

impl Parse for NumEnumVariantAttributeItem {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::default) {
            input.parse().map(Self::Default)
        } else if lookahead.peek(kw::catch_all) {
            input.parse().map(Self::CatchAll)
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

struct VariantCatchAllAttribute {
    keyword: kw::catch_all,
}

impl Parse for VariantCatchAllAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            keyword: input.parse()?,
        })
    }
}

impl Spanned for VariantCatchAllAttribute {
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
        let keyword = input.parse()?;
        let _eq_token = input.parse()?;
        let _bracket_token = syn::bracketed!(content in input);
        let expressions = content.parse_terminated(Expr::parse)?;
        Ok(Self {
            keyword,
            _eq_token,
            _bracket_token,
            expressions,
        })
    }
}

impl Spanned for VariantAlternativesAttribute {
    fn span(&self) -> Span {
        self.keyword.span()
    }
}

#[derive(::core::default::Default)]
struct AttributeSpans {
    default: Vec<Span>,
    catch_all: Vec<Span>,
    alternatives: Vec<Span>,
}

struct VariantInfo {
    ident: Ident,
    attr_spans: AttributeSpans,
    is_default: bool,
    is_catch_all: bool,
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

    fn catch_all(&self) -> Option<&Ident> {
        self.variants
            .iter()
            .find(|info| info.is_catch_all)
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
            .filter(|variant| !variant.is_catch_all)
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
            let data = match input.data {
                Data::Enum(data) => data,
                Data::Union(data) => die!(data.union_token => "Expected enum but found union"),
                Data::Struct(data) => die!(data.struct_token => "Expected enum but found struct"),
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
                                        die!(attr =>
                                            "Expected exactly one `repr` argument"
                                        );
                                    }
                                    let repr = nested.next().unwrap();
                                    let repr: Ident = parse_quote! {
                                        #repr
                                    };
                                    if repr == "C" {
                                        die!(repr =>
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
            let mut has_catch_all_variant: bool = false;

            // Vec to keep track of the used discriminants and alt values.
            let mut val_set: BTreeSet<i128> = BTreeSet::new();

            let mut next_discriminant = literal(0);
            for variant in data.variants.into_iter() {
                let ident = variant.ident.clone();

                let discriminant = match &variant.discriminant {
                    Some(d) => d.1.clone(),
                    None => next_discriminant.clone(),
                };

                let mut attr_spans: AttributeSpans = Default::default();
                let mut alternative_values: Vec<Expr> = vec![];
                // Keep the attribute around for better error reporting.
                let mut alt_attr_ref: Vec<&Attribute> = vec![];

                // `#[num_enum(default)]` is required by `#[derive(FromPrimitive)]`
                // and forbidden by `#[derive(UnsafeFromPrimitive)]`, so we need to
                // keep track of whether we encountered such an attribute:
                let mut is_default: bool = false;
                let mut is_catch_all: bool = false;

                for attribute in &variant.attrs {
                    if attribute.path.is_ident("default") {
                        if has_default_variant {
                            die!(attribute =>
                                "Multiple variants marked `#[default]` or `#[num_enum(default)]` found"
                            );
                        } else if has_catch_all_variant {
                            die!(attribute =>
                                "Attribute `default` is mutually exclusive with `catch_all`"
                            );
                        }
                        attr_spans.default.push(attribute.span());
                        is_default = true;
                        has_default_variant = true;
                    }

                    if attribute.path.is_ident("num_enum") {
                        match attribute.parse_args_with(NumEnumVariantAttributes::parse) {
                            Ok(variant_attributes) => {
                                for variant_attribute in variant_attributes.items {
                                    match variant_attribute {
                                        NumEnumVariantAttributeItem::Default(default) => {
                                            if has_default_variant {
                                                die!(default.keyword =>
                                                    "Multiple variants marked `#[default]` or `#[num_enum(default)]` found"
                                                );
                                            } else if has_catch_all_variant {
                                                die!(default.keyword =>
                                                    "Attribute `default` is mutually exclusive with `catch_all`"
                                                );
                                            }
                                            attr_spans.default.push(default.span());
                                            is_default = true;
                                            has_default_variant = true;
                                        }
                                        NumEnumVariantAttributeItem::CatchAll(catch_all) => {
                                            if has_catch_all_variant {
                                                die!(catch_all.keyword =>
                                                    "Multiple variants marked with `#[num_enum(catch_all)]`"
                                                );
                                            } else if has_default_variant {
                                                die!(catch_all.keyword =>
                                                    "Attribute `catch_all` is mutually exclusive with `default`"
                                                );
                                            }

                                            match variant
                                                .fields
                                                .iter()
                                                .collect::<Vec<_>>()
                                                .as_slice()
                                            {
                                                [syn::Field {
                                                    ty: syn::Type::Path(syn::TypePath { path, .. }),
                                                    ..
                                                }] if path.is_ident(&repr) => {
                                                    attr_spans.catch_all.push(catch_all.span());
                                                    is_catch_all = true;
                                                    has_catch_all_variant = true;
                                                }
                                                _ => {
                                                    die!(catch_all.keyword =>
                                                        "Variant with `catch_all` must be a tuple with exactly 1 field matching the repr type"
                                                    );
                                                }
                                            }
                                        }
                                        NumEnumVariantAttributeItem::Alternatives(alternatives) => {
                                            attr_spans.alternatives.push(alternatives.span());
                                            alternative_values.extend(alternatives.expressions);
                                            alt_attr_ref.push(attribute);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                die!(attribute =>
                                    format!("Invalid attribute: {}", err)
                                );
                            }
                        }
                    }
                }

                if !is_catch_all {
                    match &variant.fields {
                        Fields::Named(_) | Fields::Unnamed(_) => {
                            die!(variant => format!("`{}` only supports unit variants (with no associated data), but `{}::{}` was not a unit variant.", get_crate_name(), name, ident));
                        }
                        Fields::Unit => {}
                    }
                }

                let canonical_value = discriminant;
                let canonical_value_int = expr_to_int(&canonical_value)?;

                // Check for collision.
                if val_set.contains(&canonical_value_int) {
                    die!(ident => format!("The discriminant '{}' collides with a value attributed to a previous variant", canonical_value_int))
                }

                // Deal with the alternative values.
                let alt_val = alternative_values
                    .iter()
                    .map(expr_to_int)
                    .collect::<Result<Vec<_>>>()?;

                debug_assert_eq!(alt_val.len(), alternative_values.len());

                if !alt_val.is_empty() {
                    let mut alt_val_sorted = alt_val.clone();
                    alt_val_sorted.sort_unstable();
                    let alt_val_sorted = alt_val_sorted;

                    // check if the current discriminant is not in the alternative values.
                    if let Some(i) = alt_val.iter().position(|&x| x == canonical_value_int) {
                        die!(&alternative_values[i] => format!("'{}' in the alternative values is already attributed as the discriminant of this variant", canonical_value_int));
                    }

                    // Search for duplicates, the vec is sorted. Warn about them.
                    if (1..alt_val_sorted.len()).any(|i| alt_val_sorted[i] == alt_val_sorted[i - 1])
                    {
                        let attr = *alt_attr_ref.last().unwrap();
                        die!(attr => "There is duplication in the alternative values");
                    }
                    // Search if those alt_val where already attributed.
                    // (The val_set is BTreeSet, and iter().next_back() is the is the maximum in the set.)
                    if let Some(last_upper_val) = val_set.iter().next_back() {
                        if alt_val_sorted.first().unwrap() <= last_upper_val {
                            for (i, val) in alt_val_sorted.iter().enumerate() {
                                if val_set.contains(val) {
                                    die!(&alternative_values[i] => format!("'{}' in the alternative values is already attributed to a previous variant", val));
                                }
                            }
                        }
                    }

                    // Reconstruct the alternative_values vec of Expr but sorted.
                    alternative_values = alt_val_sorted
                        .iter()
                        .map(|val| literal(val.to_owned()))
                        .collect();

                    // Add the alternative values to the the set to keep track.
                    val_set.extend(alt_val_sorted);
                }

                // Add the current discriminant to the the set to keep track.
                let newly_inserted = val_set.insert(canonical_value_int);
                debug_assert!(newly_inserted);

                variants.push(VariantInfo {
                    ident,
                    attr_spans,
                    is_default,
                    is_catch_all,
                    canonical_value,
                    alternative_values,
                });

                // Get the next value for the discriminant.
                next_discriminant = literal(canonical_value_int + 1);
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
/// let zero: u8 = Number::Zero.into();
/// assert_eq!(zero, 0u8);
/// ```
#[proc_macro_derive(IntoPrimitive, attributes(num_enum, catch_all))]
pub fn derive_into_primitive(input: TokenStream) -> TokenStream {
    let enum_info = parse_macro_input!(input as EnumInfo);
    let catch_all = enum_info.catch_all();
    let name = &enum_info.name;
    let repr = &enum_info.repr;

    let body = if let Some(catch_all_ident) = catch_all {
        quote! {
            match enum_value {
                #name::#catch_all_ident(raw) => raw,
                rest => unsafe { *(&rest as *const #name as *const Self) }
            }
        }
    } else {
        quote! { enum_value as Self }
    };

    TokenStream::from(quote! {
        impl From<#name> for #repr {
            #[inline]
            fn from (enum_value: #name) -> Self
            {
                #body
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
/// let zero = Number::from(0u8);
/// assert_eq!(zero, Number::Zero);
///
/// let one = Number::from(1u8);
/// assert_eq!(one, Number::NonZero);
///
/// let two = Number::from(2u8);
/// assert_eq!(two, Number::NonZero);
/// ```
#[proc_macro_derive(FromPrimitive, attributes(num_enum, default, catch_all))]
pub fn derive_from_primitive(input: TokenStream) -> TokenStream {
    let enum_info: EnumInfo = parse_macro_input!(input);
    let krate = Ident::new(&get_crate_name(), Span::call_site());

    let catch_all_body = if let Some(default_ident) = enum_info.default() {
        quote! { Self::#default_ident }
    } else if let Some(catch_all_ident) = enum_info.catch_all() {
        quote! { Self::#catch_all_ident(number) }
    } else {
        let span = Span::call_site();
        let message =
            "#[derive(FromPrimitive)] requires a variant marked with `#[default]`, `#[num_enum(default)]`, or `#[num_enum(catch_all)`";
        return syn::Error::new(span, message).to_compile_error().into();
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
                    _ => #catch_all_body,
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
/// let zero = Number::try_from(0u8);
/// assert_eq!(zero, Ok(Number::Zero));
///
/// let three = Number::try_from(3u8);
/// assert_eq!(
///     three.unwrap_err().to_string(),
///     "No discriminant in enum `Number` matches the value `3`",
/// );
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
    let found_crate = proc_macro_crate::crate_name("num_enum").unwrap_or_else(|err| {
        eprintln!("Warning: {}\n    => defaulting to `num_enum`", err,);
        proc_macro_crate::FoundCrate::Itself
    });

    match found_crate {
        proc_macro_crate::FoundCrate::Itself => String::from("num_enum"),
        proc_macro_crate::FoundCrate::Name(name) => name,
    }
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

/// Implements `core::default::Default` for a `#[repr(Primitive)] enum`.
///
/// Whichever variant has the `#[default]` or `#[num_enum(default)]` attribute will be returned.
/// ----------------------------------------------
///
/// ```rust
/// #[derive(Debug, Eq, PartialEq, num_enum::Default)]
/// #[repr(u8)]
/// enum Number {
///     Zero,
///     #[default]
///     One,
/// }
///
/// assert_eq!(Number::One, Number::default());
/// assert_eq!(Number::One, <Number as ::core::default::Default>::default());
/// ```
#[proc_macro_derive(Default, attributes(num_enum, default))]
pub fn derive_default(stream: TokenStream) -> TokenStream {
    let enum_info = parse_macro_input!(stream as EnumInfo);

    let default_ident = match enum_info.default() {
        Some(ident) => ident,
        None => {
            let span = Span::call_site();
            let message =
                "#[derive(num_enum::Default)] requires a variant marked with `#[default]` or `#[num_enum(default)]`";
            return syn::Error::new(span, message).to_compile_error().into();
        }
    };

    let EnumInfo { ref name, .. } = enum_info;

    TokenStream::from(quote! {
        impl ::core::default::Default for #name {
            #[inline]
            fn default() -> Self {
                Self::#default_ident
            }
        }
    })
}
