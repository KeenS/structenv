extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use syn::punctuated::{Pair, Punctuated};
use syn::token::Comma;
use syn::{Data, DeriveInput, FieldValue, Fields};

use std::iter::IntoIterator;

fn path_name(path: &syn::Path) -> String {
    let mut s = String::new();
    if let Some(_colon2) = path.leading_colon {
        s.push_str("::")
    }
    for pair in path.segments.pairs() {
        use syn::punctuated::Pair::*;
        match pair {
            Punctuated(segment, _) => {
                s.push_str(&segment.ident.to_string());
                s.push_str("::")
            }
            End(segment) => {
                s.push_str(&segment.ident.to_string());
            }
        }

        // TODO: handle it
        // use syn::PathArguments::*;
        // match segment.arguments {
        //     None => (),
        //     AngleBracketed(arg) => (),
        //     Parenthesized(arg) => (),
        // }
    }
    s
}

#[test]
fn test_path_name() {
    assert_eq!(path_name(&parse_quote!(default)), "default");
    assert_eq!(path_name(&parse_quote!(self::default)), "self::default");
}

struct CodeGenOption {
    default: Option<syn::Expr>,
}

impl CodeGenOption {
    fn from_options(options: impl Iterator<Item = StructEnvOption>) -> Self {
        let mut default = None;
        for option in options {
            use StructEnvOption::*;
            match option {
                Default(value) => default = Some(value),
            }
        }

        Self { default }
    }
}

enum StructEnvOption {
    Default(syn::Expr),
}

fn collect_options(
    attr: impl Iterator<Item = syn::Attribute>,
) -> impl Iterator<Item = StructEnvOption> {
    use syn::{Lit, Meta, MetaNameValue, NestedMeta};

    attr.filter(|attr| path_name(&attr.path) == "structenv")
        .map(|attr| {
            attr.interpret_meta()
                .expect(&format!("invalid structenv syntax: {}", quote!(attr)))
        }).flat_map(|meta| match meta {
            Meta::List(l) => l.nested,
            tokens => panic!("unsupported syntax: {}", quote!(#tokens).to_string()),
        }).map(|meta| match meta {
            NestedMeta::Meta(m) => m,
            ref tokens => panic!("unsupported syntax: {}", quote!(#tokens).to_string()),
        }).map(|attr| match attr {
            Meta::NameValue(MetaNameValue {
                ident,
                lit: Lit::Str(value),
                ..
            }) => {
                if &ident.to_string() == "default_value" {
                    let default = value.value();
                    let default = syn::parse_str(&default).expect("failed to parse default value");
                    StructEnvOption::Default(default)
                } else {
                    panic!("ops")
                }
            }
            attr => panic!("invalid structenv syntax: {}", quote!(#attr)),
        })
}

/// implement from_env for the struct.
/// for `snake_case` fields, `SNAKE_CASE` variable is used.
#[proc_macro_derive(StructEnv, attributes(structenv))]
pub fn derive_env(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => panic!("only structs are supported"),
    };

    let fields = match fields {
        Fields::Named(fields) => fields,
        _ => panic!("only named fields are supported"),
    };

    let inits = fields
        .named
        .into_iter()
        .map(|field| -> FieldValue {
            let ident = field
                .ident
                .expect("fields of named struct must have idents");
            let large_snake = ident.to_string().to_uppercase();
            let ty = field.ty;
            let var_message = format!("{} is not set or invalid unicode", large_snake);
            let parse_message = format!("failed to parse {} ", large_snake);
            let options = collect_options(field.attrs.into_iter());
            let option = CodeGenOption::from_options(options);

            let fallback: syn::Expr;
            if let Some(expr) = option.default {
                fallback = parse_quote!(#expr)
            } else {
                fallback = parse_quote!(panic!(#var_message))
            }

            parse_quote!(#ident: match ::std::env::var(#large_snake) {
                Ok(s)=> s.parse::<#ty>()
                    .expect(#parse_message),
                Err(_) => {#fallback},
            }
            )
        }).map(|init| Pair::new(init, Some(parse_quote!(,))))
        .collect::<Punctuated<FieldValue, Comma>>();

    let expanded = quote! {
        impl ::structenv::StructEnv for #struct_name {
            fn from_env() -> Self {
                Self {
                    #inits
                }
            }
        }
    };
    TokenStream::from(expanded).into()
}
