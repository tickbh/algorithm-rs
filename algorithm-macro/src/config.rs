use std::default::Default;
use std::collections::HashSet;

use quote::ToTokens;
use syn::{self, Token, parenthesized};
use syn::parse::{Parse, ParseStream};

pub struct Config {
    pub ignore_args: HashSet<syn::Ident>,
    pub use_thread: bool,
}

struct IgnoreArgsAttrib {
    ignore_args: HashSet<syn::Ident>,
}

enum ConfigAttrib {
    IgnoreArgs(IgnoreArgsAttrib),
    UseTread,
}

const CONFIG_ATTRIBUTE_NAME: &'static str = "cache_cfg";

impl Config {
    // Parse any additional attributes present after `lru_cache` and return a configuration object
    // created from their contents. Additionally, return any attributes that were not handled here.
    pub fn parse_from_attributes(attribs: &[syn::Attribute]) -> syn::Result<(Config, Vec<syn::Attribute>)> {
        let mut parsed_attributes = Vec::new();
        let mut remaining_attributes = Vec::new();

        for attrib in attribs {
            let segs = &attrib.path().segments;
            if segs.len() > 0 {
                if segs[0].ident == CONFIG_ATTRIBUTE_NAME {
                    let tokens = attrib.meta.to_token_stream();
                    let parsed = syn::parse2::<ConfigAttrib>(tokens)?;
                    parsed_attributes.push(parsed);
                }
                else {
                    remaining_attributes.push(attrib.clone());
                }
            }
        }

        let mut config: Config = Default::default();

        for parsed_attrib in parsed_attributes {
            match parsed_attrib {
                ConfigAttrib::IgnoreArgs(val) => config.ignore_args = val.ignore_args,
                ConfigAttrib::UseTread => config.use_thread = true,
            }
        }

        Ok((config, remaining_attributes))
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            ignore_args: HashSet::new(),
            use_thread: false,
        }
    }
}

impl Parse for ConfigAttrib {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let _name = input.parse::<syn::Ident>()?;
        let content;
        let _paren = parenthesized!(content in input);
        let name = content.parse::<syn::Ident>()?;
        match &name.to_string()[..] {
            "ignore_args" => Ok(ConfigAttrib::IgnoreArgs(content.parse::<IgnoreArgsAttrib>()?)),
            "thread" => Ok(ConfigAttrib::UseTread),
            _ => Err(syn::parse::Error::new(
                name.span(), format!("unrecognized config option '{}'", name.to_string())
            ))
        }
    }
}

impl Parse for IgnoreArgsAttrib {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        input.parse::<Token![=]>()?;
        let elems = syn::punctuated::Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;
        Ok(IgnoreArgsAttrib {
            ignore_args: elems.into_iter().collect(),
        })
    }
}