extern crate proc_macro;

use proc_macro::{Group, Ident, TokenStream, TokenTree};
use std::collections::BTreeMap;

#[proc_macro_derive(IntoPrimitive)]
pub fn derive_into_primitive(stream: TokenStream) -> TokenStream {
    let enum_info = parse_enum(stream);

    format!(r#"
impl From<{name}> for {discriminant} {{
  fn from(number: {name}) -> Self {{
    number as Self
  }}
}}
"#, name=enum_info.name, discriminant=enum_info.discriminant).parse().unwrap()
}

// Requires try_from which isn't yet stable.
#[proc_macro_derive(TryFromPrimitive)]
pub fn derive_try_from_primitive(stream: TokenStream) -> TokenStream {
    let enum_info = parse_enum(stream);

    let mut match_consts = String::new();
    let mut match_body = String::new();
    for (enum_value_expression, enum_key) in enum_info.value_expressions_to_enum_keys {
        // Use an intermediate const so that enums defined like `Two = ONE + 1u8` work properly.
        let match_const = format!("__num_enum_match_{}", enum_key);
        match_consts.push_str(&format!("const {match_const}: {discriminant} = {enum_value_expression};\n", match_const=match_const, discriminant=enum_info.discriminant, enum_value_expression=enum_value_expression));
        match_body.push_str(&format!("{match_const} => Ok({name}::{enum_key}),\n", match_const=match_const, name=enum_info.name, enum_key=enum_key));
    }

    format!(r#"

impl ::std::convert::TryFrom<{discriminant}> for {name} {{
  type Error=String;

  fn try_from(number: {discriminant}) -> Result<Self, Self::Error> {{
    {match_consts}

    match number {{
      {match_body}
      _ => Err(format!("No value in enum {name} for value {{}}", number)),
    }}
  }}
}}
"#, name=enum_info.name, discriminant=enum_info.discriminant, match_consts=match_consts, match_body=match_body).parse().unwrap()
}

// Glue until try_from is stable.
#[proc_macro_derive(CustomTryInto)]
pub fn derive_option_from_custom_try_into(stream: TokenStream) -> TokenStream {
    let enum_info = parse_enum(stream);

    let mut match_consts = String::new();
    let mut match_body = String::new();
    for (enum_value_expression, enum_key) in enum_info.value_expressions_to_enum_keys {
        // Use an intermediate const so that enums defined like `Two = ONE + 1u8` work properly.
        let match_const = format!("__num_enum_match_{}", enum_key);
        match_consts.push_str(&format!("const {match_const}: {discriminant} = {enum_value_expression};\n", match_const=match_const, discriminant=enum_info.discriminant, enum_value_expression=enum_value_expression));
        match_body.push_str(&format!("{match_const} => Ok({name}::{enum_key}),\n", match_const=match_const, name=enum_info.name, enum_key=enum_key));
    }

    let visibility = enum_info.visibility.join(" ");

    format!(r#"
{visibility} trait TryInto{name} {{
  type Error;

  fn try_into_{name}(self) -> Result<{name}, Self::Error>;
}}

impl TryInto{name} for {discriminant} {{
  type Error=String;

  fn try_into_{name}(self) -> Result<{name}, Self::Error> {{
    {match_consts}

    match self {{
      {match_body}
      _ => Err(format!("No value in enum {name} for value {{}}", self)),
    }}
  }}
}}
"#, name=enum_info.name, discriminant=enum_info.discriminant, visibility=visibility, match_consts=match_consts, match_body=match_body).parse().unwrap()
}

struct EnumInfo {
    name: Ident,
    discriminant: Ident,
    value_expressions_to_enum_keys: BTreeMap<String, Ident>,
    visibility: Vec<String>,
}

fn parse_enum(stream: TokenStream) -> EnumInfo {
    enum State {
        Waiting,
        SawHash,
        SawEnumIdent,
        SawEnumName,
    }

    let mut enum_name = None;
    let mut discriminant = None;
    let mut values = None;
    let mut visibility = vec![];

    let mut state = State::Waiting;

    for token in stream {
        match state {
            State::Waiting => {
                match token {
                    TokenTree::Punct(ref punct) if punct.as_char() == '#' => state = State::SawHash,
                    TokenTree::Ident(ident) => {
                        let s = ident.to_string();
                        if &s == "enum" {
                            state = State::SawEnumIdent;
                        } else if s.starts_with("pub") {
                            visibility.push(format!("{}", ident));
                        } else if !visibility.is_empty() {
                            visibility.push(format!("{}", ident));
                        } else {
                            panic!("Didn't expect token {}", ident);
                        }
                    },
                    val => {
                        if !visibility.is_empty() {
                            visibility.push(format!("{}", val));
                        } else {
                            panic!("Didn't expect token {}", val);
                        }
                    },
                }
            },
            State::SawHash => {
                // Check for #[repr(numeric_type)]
                if let TokenTree::Group(group) = token {
                    let items: Vec<_> = group.stream().into_iter().collect();
                    if items.len() == 2 {
                        if let TokenTree::Ident(ref maybe_repr) = items[0] {
                            if "repr" == &maybe_repr.to_string() {
                                if let TokenTree::Group(ref group) = items[1] {
                                    match group.stream().into_iter().next().unwrap() {
                                        TokenTree::Ident(ident) => {
                                            if &format!("{}", ident) == "C" {
                                                panic!("Can't generate num_enum traits for repr(C) enums because they don't have a generally defined size.")
                                            }
                                            discriminant = Some(ident);
                                        },
                                        val => {
                                            panic!("Got unexpected discriminant: {}", val);
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                // If we don't match repr, ignore this attribute and continue.
                // (If we do match repr, we've parsed it, and continue).
                state = State::Waiting;
            },
            State::SawEnumIdent => {
                if let TokenTree::Ident(ident) = token {
                    enum_name = Some(ident);
                } else {
                    panic!("Saw enum then not an ident: {}", token);
                }
                state = State::SawEnumName;
            },
            State::SawEnumName => {
                if let TokenTree::Group(group) = token {
                    values = Some(parse_body(group));
                } else {
                    panic!("Didn't see enum body, got: {}", token);
                }
            }
        }
    }

    EnumInfo {
        name: enum_name.expect("Couldn't find name of enum"),
        discriminant: discriminant.expect("Couldn't find repr of enum"),
        value_expressions_to_enum_keys: values.expect("Couldn't find body of enum"),
        visibility: visibility,
    }
}

fn parse_body(group: Group) -> BTreeMap<String, Ident> {
    enum State {
        WantKey,
        GotKey,
        InAttribute,
    }

    let mut values = BTreeMap::new();

    let mut state = State::WantKey;
    let mut key = None;

    // We parse values as expressions, so that we can parse (possibly complex) non-literals like
    // constants (`ONE`) or const-expressions (`1 + ONE + 2u8`).
    let mut next_value = String::from("0");

    for token in group.stream() {
        match state {
            State::WantKey => {
                match token {
                    TokenTree::Ident(ident) => {
                        key = Some(ident);
                        state = State::GotKey;
                    },
                    TokenTree::Punct(ref punct) if punct.as_char() == '#' => {
                        state = State::InAttribute;
                    },
                    _ => panic!("Parsing enum body, wanted key but got {}", token),
                }
            },
            State::GotKey => {
                match token {
                    TokenTree::Punct(ref punct) if punct.as_char() == '=' => {
                        // Reset next_value - we're about to get an override.
                        next_value = String::new();
                    },
                    TokenTree::Punct(ref punct) if punct.as_char() == ',' => {
                        values.insert(next_value.clone(), key.take().unwrap());

                        // In case there is no explicit value for the next, add one to the current.
                        next_value += " + 1";

                        state = State::WantKey;
                    },
                    rhs => {
                        // Literals, idents, and sub-expression parts, all just get appended.
                        next_value += &format!("{}", rhs);
                    }
                }
            },
            State::InAttribute => {
                state = State::WantKey;
            },
        }
    }

    if let Some(key) = key {
        values.insert(next_value, key);
    }
    values
}
