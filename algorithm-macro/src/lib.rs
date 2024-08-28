
use proc_macro::TokenStream;
use syn;
use syn::{Token, parse_quote};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use quote::quote;
use proc_macro2;

mod config;

use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;

struct Attr {
    cache_type: syn::Type,
    cache_creation_expr: syn::Expr,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let cache_type: syn::Type = input.parse()?;
        input.parse::<Token![:]>()?;
        let cache_creation_expr: syn::Expr = input.parse()?;
        Ok(Attr {
            cache_type,
            cache_creation_expr,
        })
    }
}

#[proc_macro_attribute]
pub fn cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Attr);

    match algorithm_cache_impl(attr, item.clone()) {
        Ok(tokens) => return tokens,
        Err(e) => {
            panic!("error = {:?}", e);
        }
    }
}

// The main entry point for the macro.
fn algorithm_cache_impl(attr: Attr, item: TokenStream) -> syn::parse::Result<TokenStream> {
    let mut original_fn: syn::ItemFn = syn::parse(item.clone())?;
    let (macro_config, out_attributes) =
        {
            let attribs = &original_fn.attrs[..];
            config::Config::parse_from_attributes(attribs)?
        };
    original_fn.attrs = out_attributes;

    let mut new_fn = original_fn.clone();
    let return_type = get_cache_fn_return_type(&original_fn)?;
    let new_name = format!("__cache_auto_{}", original_fn.sig.ident.to_string());
    original_fn.sig.ident = syn::Ident::new(&new_name[..], original_fn.sig.ident.span());
    let (call_args, types, cache_args) = get_args_and_types(&original_fn, &macro_config)?;
    let cloned_args = make_cloned_args_tuple(&cache_args);
    let fn_path = path_from_ident(original_fn.sig.ident.clone());
    let fn_call = syn::ExprCall {
        attrs: Vec::new(),
        paren_token: syn::token::Paren::default(),
        args: call_args,
        func: Box::new(fn_path)
    };

    let tuple_type = syn::TypeTuple {
        paren_token: syn::token::Paren::default(),
        elems: types,
    };

    let cache_type = &attr.cache_type;
    let cache_type_with_generics: syn::Type = parse_quote! {
        #cache_type<#tuple_type, #return_type, algorithm::DefaultHasher>
    };
    let lru_body = build_cache_body(&cache_type_with_generics, &attr.cache_creation_expr, &cloned_args,
        &fn_call, &macro_config);

    new_fn.block = Box::new(lru_body);
    let out = quote! {
        #original_fn
        #new_fn
    };
    Ok(out.into())
}

// Build the body of the caching function. What is constructed depends on the config value.
fn build_cache_body(full_cache_type: &syn::Type, cache_new: &syn::Expr,
                    cloned_args: &syn::ExprTuple, inner_fn_call: &syn::ExprCall,
                    config: &config::Config) -> syn::Block
{
    if config.use_thread {
        build_mutex_cache_body(full_cache_type, cache_new, cloned_args, inner_fn_call)
    } else {
        build_tls_cache_body(full_cache_type, cache_new, cloned_args, inner_fn_call)
    }
}

// Build the body of the caching function which puts the cache in thread-local storage.
fn build_tls_cache_body(full_cache_type: &syn::Type, cache_new: &syn::Expr,
                     cloned_args: &syn::ExprTuple, inner_fn_call: &syn::ExprCall) -> syn::Block
{
    parse_quote! {
        {
            use std::cell::RefCell;
            use std::thread_local;
            thread_local!(
                static cache: RefCell<#full_cache_type> =
                    RefCell::new(#cache_new);
            );
            cache.with(|c| {
                let mut cache_ref = c.borrow_mut();
                let cloned_args = #cloned_args;

                let stored_result = cache_ref.get_mut(&cloned_args);
                if let Some(stored_result) = stored_result {
                    return stored_result.clone()
                }

                // Don't hold a mutable borrow across
                // the recursive function call
                drop(cache_ref);

                let ret = #inner_fn_call;
                c.borrow_mut().insert(cloned_args, ret.clone());
                ret
            })
        }
    }
}

// Build the body of the caching function which guards the static cache with a mutex.
fn build_mutex_cache_body(full_cache_type: &syn::Type, cache_new: &syn::Expr,
                     cloned_args: &syn::ExprTuple, inner_fn_call: &syn::ExprCall) -> syn::Block
{
    parse_quote! {
        {
            use lazy_static::lazy_static;
            use std::sync::Mutex;

            lazy_static! {
                static ref cache: Mutex<#full_cache_type> =
                    Mutex::new(#cache_new);
            }

            let cloned_args = #cloned_args;

            let mut cache_unlocked = cache.lock().unwrap();
            let stored_result = cache_unlocked.get_mut(&cloned_args);
            if let Some(stored_result) = stored_result {
                return stored_result.clone();
            };

            // must unlock here to allow potentially recursive call
            drop(cache_unlocked);

            let ret = #inner_fn_call;
            let mut cache_unlocked = cache.lock().unwrap();
            cache_unlocked.insert(cloned_args, ret.clone());
            ret
        }
    }
}

fn get_cache_fn_return_type(original_fn: &syn::ItemFn) -> syn::Result<Box<syn::Type>> {
    if let syn::ReturnType::Type(_, ref ty) = original_fn.sig.output {
        Ok(ty.clone())
    } else {
        return Err(syn::Error::new_spanned(original_fn, "There's no point of caching the output of a function that has no output"))
    }
}

fn path_from_ident(ident: syn::Ident) -> syn::Expr {
    let mut segments: Punctuated<_, Token![::]> = Punctuated::new();
    segments.push(syn::PathSegment { ident: ident, arguments: syn::PathArguments::None });
    syn::Expr::Path(syn::ExprPath { attrs: Vec::new(), qself: None, path: syn::Path { leading_colon: None, segments: segments} })
}

fn make_cloned_args_tuple(args: &Punctuated<syn::Expr, Token![,]>) -> syn::ExprTuple {
    let mut cloned_args = Punctuated::<_, Token![,]>::new();
    for arg in args {
        let call = syn::ExprMethodCall {
            attrs: Vec::new(),
            receiver: Box::new(arg.clone()),
            dot_token: syn::token::Dot { spans: [arg.span(); 1] },
            method: syn::Ident::new("clone", proc_macro2::Span::call_site()),
            turbofish: None,
            paren_token: syn::token::Paren::default(),
            args: Punctuated::new(),
        };
        cloned_args.push(syn::Expr::MethodCall(call));
    }
    syn::ExprTuple {
        attrs: Vec::new(),
        paren_token: syn::token::Paren::default(),
        elems: cloned_args,
    }
}

fn get_args_and_types(f: &syn::ItemFn, config: &config::Config) ->
        syn::Result<(Punctuated<syn::Expr, Token![,]>, Punctuated<syn::Type, Token![,]>, Punctuated<syn::Expr, Token![,]>)>
{
    let mut call_args = Punctuated::<_, Token![,]>::new();
    let mut types = Punctuated::<_, Token![,]>::new();
    let mut cache_args = Punctuated::<_, Token![,]>::new();

    for input in &f.sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new(input.span(), "`self` arguments are currently unsupported by algorithm_cache"));

            }
            syn::FnArg::Typed(p) => {
                let mut segments: syn::punctuated::Punctuated<_, Token![::]> = syn::punctuated::Punctuated::new();
                let arg_name;
                if let syn::Pat::Ident(ref pat_ident) = *p.pat {
                    arg_name = pat_ident.ident.clone();
                    segments.push(syn::PathSegment { ident: pat_ident.ident.clone(), arguments: syn::PathArguments::None });
                } else {
                    return Err(syn::Error::new(input.span(), "unsupported argument kind"));
                }

                let arg_path = syn::Expr::Path(syn::ExprPath { attrs: Vec::new(), qself: None, path: syn::Path { leading_colon: None, segments } });
                if !config.ignore_args.contains(&arg_name) {
                    // If the arg type is a reference, remove the reference because the arg will be cloned
                    if let syn::Type::Reference(type_reference) = &*p.ty {
                        if let Some(_) = type_reference.mutability {
                            call_args.push(arg_path);
                            continue;
                            // return Err(io::Error::new(io::ErrorKind::Other, "`mut` reference arguments are not supported as this could lead to incorrect results being stored"));
                        }
                        types.push(type_reference.elem.as_ref().to_owned()); // as_ref -> to_owned unboxes the type
                    } else {
                        types.push((*p.ty).clone());
                    }

                    cache_args.push(arg_path.clone());
                }
                call_args.push(arg_path);
            }
        }
    }

    if types.len() == 1 {
        types.push_punct(syn::token::Comma { spans: [proc_macro2::Span::call_site(); 1] })
    }

    Ok((call_args, types, cache_args))
}