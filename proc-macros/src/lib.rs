// Copyright (c) 2023 René Kijewski <crates.io@k6i.de>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// --- LLVM Exceptions to the Apache 2.0 License ----
//
// As an exception, if, as a result of your compiling your source code, portions
// of this Software are embedded into an Object form of such source code, you
// may redistribute such embedded portions in such Object form without complying
// with the conditions of Sections 4(a), 4(b) and 4(d) of the License.
//
// In addition, if you combine or link compiled forms of this Software with
// software that is licensed under the GPLv2 ("Combined Software") and if a
// court of competent jurisdiction determines that the patent provision (Section
// 3), the indemnity provision (Section 9) or other Section of the License
// conflicts with the conditions of the GPLv2, you may retroactively and
// prospectively choose to deem waived or otherwise exclude such Section(s) of
// the License, but only in their entirety and only with respect to the Combined
// Software.

#![allow(unknown_lints)]
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(meta_variable_misuse)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(single_use_lifetimes)]
#![warn(unused_extern_crates)]
#![warn(unused_lifetimes)]
#![cfg_attr(miri, ignore)]

mod config;

use heck::ToPascalCase as _;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{parse_quote_spanned, Ident};

#[proc_macro_attribute]
pub fn named_future(args: TokenStream, input_stream: TokenStream) -> TokenStream {
    let args: config::Args = syn::parse_macro_input!(args);
    let mut func: config::Func = syn::parse_macro_input!(input_stream);
    let body = &func.body;

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Names
    // ////////////////////////////////////////////////////////////////////////////////////////////

    let function_name = func.sig.ident.clone();
    let function_name_span = function_name.span();

    let crate_name = args
        .crate_name
        .as_ref()
        .cloned()
        .unwrap_or_else(|| parse_quote_spanned!(function_name_span => ::named_future));

    let struct_name = if let Some(ref name) = args.name {
        name.clone()
    } else {
        Ident::new(
            &function_name.to_string().to_pascal_case(),
            function_name_span,
        )
    };
    let struct_name_string = struct_name.to_string();

    let gen_ident = function_name;
    let impl_ident = Ident::new("__implementation", function_name_span);

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Attributes
    // ////////////////////////////////////////////////////////////////////////////////////////////

    let (func_attrs, struct_attrs) = match func.attrs_split {
        Some(index) => {
            let (func_attrs, struct_attrs) = func.attrs.split_at(index);
            (func_attrs, &struct_attrs[1..])
        },
        None => (func.attrs.as_slice(), &[][..]),
    };

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Types
    // ////////////////////////////////////////////////////////////////////////////////////////////

    let arg_types_as_tuple = match arg_types_as_tuple(&func) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let args_pats_as_tuple = match args_pats_as_tuple(&func) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let arg_exprs_with_commas = match arg_exprs_with_commas(&func) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let args_exprs_as_tuple = syn::ExprTuple {
        attrs: vec![],
        paren_token: func.sig.paren_token,
        elems: arg_exprs_with_commas.clone(),
    };
    let phantom = phantom(&func.sig.generics, function_name_span);

    let ty_self = ty_self(&func, &struct_name);

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Signatures
    // ////////////////////////////////////////////////////////////////////////////////////////////

    let func_vis = &func.vis;
    let func_gen = &func.sig.generics;
    let func_output = match &func.sig.output {
        syn::ReturnType::Default => parse_quote_spanned!(function_name_span => ()),
        syn::ReturnType::Type(_, ty) => syn::Type::clone(ty),
    };
    let (impl_generics, ty_generics, where_clause) = func_gen.split_for_impl();
    let struct_vis = args.vis.as_ref().unwrap_or(func_vis);

    let mut func_sig = func.sig.clone();
    func_sig.asyncness = None;
    func_sig.output = parse_quote_spanned! {
        function_name_span => -> #struct_name #ty_generics
    };

    let gen_sig = gen_sig(&func, &gen_ident, &args_pats_as_tuple, &arg_types_as_tuple);
    let impl_send = impl_send(&args, &struct_name, func_gen, &gen_ident, &crate_name);
    let impl_sync = impl_sync(&args, &struct_name, func_gen, &gen_ident, &crate_name);

    func.sig.ident = impl_ident.clone();
    for arg in &mut func_sig.inputs {
        if let syn::FnArg::Typed(arg) = arg {
            if let syn::Pat::Ident(arg) = &mut *arg.pat {
                arg.mutability = None;
            }
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////////
    // Implementation
    // ////////////////////////////////////////////////////////////////////////////////////////////

    let struct_name_span = struct_name.span();
    let struct_definition = quote_spanned! {
        struct_name_span =>

        #(#struct_attrs)*
        #[repr(transparent)]
        #[must_use = "futures do nothing unless you `.await` or poll them"]
        #struct_vis struct #struct_name #func_gen #where_clause {
            _future: #crate_name::machinery::Bytes<
                { <#ty_self as #crate_name::machinery::NamedFuture>::SIZE_OF },
                { <#ty_self as #crate_name::machinery::NamedFuture>::ALIGN_OF },
            >,
            _not_send_or_sync: ::core::marker::PhantomData<*mut ()>,
            _pin: ::core::marker::PhantomPinned,
            _phantom: #phantom,
        }
    };

    TokenStream::from(quote_spanned! {
        function_name_span =>

        #(#func_attrs)*
        #[inline]
        #[allow(clippy::needless_lifetimes)]
        #func_vis #func_sig {
            <#struct_name #ty_generics as #crate_name::machinery::NamedFuture>::
                new(#args_exprs_as_tuple)
        }

        #struct_definition

        #[allow(clippy::needless_lifetimes)]
        const _: () = {
            #[inline(always)]
            #func #body

            const _: () = {
                #gen_sig {
                    #impl_ident(#arg_exprs_with_commas).await
                }

                impl #impl_generics #crate_name::machinery::NamedFuture
                for #struct_name #ty_generics #where_clause {
                    const ALIGN_OF: ::core::primitive::usize =
                        #crate_name::machinery::align_of(&#gen_ident);
                    const SIZE_OF: ::core::primitive::usize =
                        #crate_name::machinery::size_of(&#gen_ident);

                    type Args = #arg_types_as_tuple;

                    #[inline(always)]
                    fn new(args: Self::Args) -> Self {
                        let fut = #gen_ident(args);
                        unsafe { ::core::mem::transmute(fut) }
                    }
                }

                impl #impl_generics ::core::ops::Drop
                for #struct_name #ty_generics #where_clause {
                    #[inline]
                    fn drop(&mut self) {
                        unsafe { #crate_name::machinery::drop(&#gen_ident, self) };
                    }
                }

                impl #impl_generics ::core::future::Future
                for #struct_name #ty_generics #where_clause {
                    type Output = #func_output;

                    #[inline]
                    fn poll(
                        self: ::core::pin::Pin<&mut Self>,
                        cx: &mut ::core::task::Context<'_>,
                    ) -> ::core::task::Poll<Self::Output> {
                        unsafe { #crate_name::machinery::poll(&#gen_ident, self, cx) }
                    }
                }

                impl #impl_generics ::core::fmt::Debug
                for #struct_name #ty_generics #where_clause {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.debug_struct(#struct_name_string).finish_non_exhaustive()
                    }
                }

                #impl_send
                #impl_sync
            };
        };
    })
}

fn gen_sig(
    func: &config::Func,
    gen_ident: &Ident,
    args_pats_as_tuple: &syn::Pat,
    arg_types_as_tuple: &syn::Type,
) -> syn::Signature {
    let function_name_span = func.sig.ident.span();

    let mut gen_sig = func.sig.clone();
    gen_sig.ident = gen_ident.clone();
    gen_sig.inputs = parse_quote_spanned! {
        function_name_span => #args_pats_as_tuple: #arg_types_as_tuple,
    };
    gen_sig
}

/// A type "PhantomData<(fn() -> *const A, fn() -> *const B)>"
fn phantom(func_gen: &syn::Generics, function_name_span: proc_macro2::Span) -> syn::Type {
    let mut result = func_gen
        .type_params()
        .map(|ty| -> syn::Type {
            let ty = &ty.ident;
            parse_quote_spanned!(function_name_span => fn() -> *const #ty)
        })
        .collect::<Punctuated<_, syn::Token![,]>>();
    for syn::LifetimeParam { lifetime, .. } in func_gen.lifetimes() {
        result.push(parse_quote_spanned!(function_name_span => &#lifetime ()));
    }
    if !result.is_empty() && !result.trailing_punct() {
        result.push_punct(Default::default());
    }
    let result = syn::Type::Tuple(syn::TypeTuple {
        paren_token: Default::default(),
        elems: result,
    });
    parse_quote_spanned! {
        function_name_span => ::core::marker::PhantomData<#result>
    }
}

/// "impl Send for Type {}"
fn impl_send(
    args: &config::Args,
    struct_name: &Ident,
    func_gen: &syn::Generics,
    gen_ident: &Ident,
    crate_name: &syn::Path,
) -> Option<proc_macro2::TokenStream> {
    let span = args.send.as_ref()?.span();
    let (impl_generics, ty_generics, where_clause) = func_gen.split_for_impl();
    Some(quote_spanned! {
        span =>
        const _: () = #crate_name::machinery::ensure_send(&#gen_ident);

        unsafe impl #impl_generics ::core::marker::Send
        for #struct_name #ty_generics #where_clause {}
    })
}

/// "impl Sync for Type {}"
fn impl_sync(
    args: &config::Args,
    struct_name: &Ident,
    func_gen: &syn::Generics,
    gen_ident: &Ident,
    crate_name: &syn::Path,
) -> Option<proc_macro2::TokenStream> {
    let span = args.sync.as_ref()?.span();
    let (impl_generics, ty_generics, where_clause) = func_gen.split_for_impl();
    Some(quote_spanned! {
        span =>
        const _: () = #crate_name::machinery::ensure_sync(&#gen_ident);

        unsafe impl #impl_generics ::core::marker::Sync
        for #struct_name #ty_generics #where_clause {}
    })
}

/// Comma separated expression "a, b, c"
fn arg_exprs_with_commas(
    func: &config::Func,
) -> Result<Punctuated<syn::Expr, syn::token::Comma>, TokenStream> {
    let result = func
        .sig
        .inputs
        .iter()
        .map(|input| {
            if let syn::FnArg::Typed(item) = input {
                let span = item.span();
                let mut pat = (*item.pat).clone();
                if let syn::Pat::Ident(pat) = &mut pat {
                    pat.mutability = None;
                }
                let expr: syn::Expr = parse_quote_spanned!(span => #pat);
                Ok(expr)
            } else {
                Err(syn::Error::new_spanned(input, "Not implemented for Self"))
            }
        })
        .collect::<Result<Punctuated<_, syn::Token![,]>, _>>();
    let mut result = match result {
        Ok(result) => result,
        Err(err) => return Err(err.into_compile_error().into()),
    };
    if !result.is_empty() && !result.trailing_punct() {
        result.push_punct(Default::default());
    }
    Ok(result)
}

/// A type "(A, B, C)"
fn arg_types_as_tuple(func: &config::Func) -> Result<syn::Type, TokenStream> {
    let result = func
        .sig
        .inputs
        .iter()
        .map(|input| {
            if let syn::FnArg::Typed(item) = input {
                Ok(syn::Type::clone(&item.ty))
            } else {
                Err(syn::Error::new_spanned(input, "Not implemented for Self"))
            }
        })
        .collect::<Result<Punctuated<_, syn::Token![,]>, _>>();
    let mut result = match result {
        Ok(result) => result,
        Err(err) => return Err(err.into_compile_error().into()),
    };
    if !result.is_empty() && !result.trailing_punct() {
        result.push_punct(Default::default());
    }

    Ok(syn::Type::Tuple(syn::TypeTuple {
        paren_token: Default::default(),
        elems: result,
    }))
}

/// A pattern "(a, b, c)"
fn args_pats_as_tuple(func: &config::Func) -> Result<syn::Pat, TokenStream> {
    let result = func
        .sig
        .inputs
        .iter()
        .map(|input| {
            if let syn::FnArg::Typed(item) = input {
                let mut pat = (*item.pat).clone();
                if let syn::Pat::Ident(pat) = &mut pat {
                    pat.mutability = None;
                }
                Ok(pat)
            } else {
                Err(syn::Error::new_spanned(input, "Not implemented for Self"))
            }
        })
        .collect::<Result<Punctuated<_, syn::Token![,]>, _>>();
    let mut result = match result {
        Ok(result) => result,
        Err(err) => return Err(err.into_compile_error().into()),
    };
    if !result.is_empty() && !result.trailing_punct() {
        result.push_punct(Default::default());
    }

    Ok(syn::Pat::Tuple(syn::PatTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: result,
    }))
}

/// The type "Self<'static, …>"
fn ty_self(func: &config::Func, name: &Ident) -> proc_macro2::TokenStream {
    let lifetimes: Vec<_> = func
        .sig
        .generics
        .lifetimes()
        .map(|lt| &lt.lifetime.ident)
        .collect();

    let span = name.span();
    let (_, ty_generics, _) = func.sig.generics.split_for_impl();
    let stream = quote_spanned!(span => #name #ty_generics);

    lifetimes_to_static(&lifetimes, stream)
}

/// Replace lifetimes in `lifetimes` with `'static`
fn lifetimes_to_static(
    lifetimes: &[&Ident],
    stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut had_apos = false;
    stream
        .into_iter()
        .map(|tt| {
            let is_lt = had_apos;
            had_apos = false;
            match tt {
                TokenTree::Ident(mut ident) => {
                    if is_lt && lifetimes.iter().any(|lt| **lt == ident) {
                        ident = Ident::new("static", ident.span());
                    }
                    TokenTree::Ident(ident)
                },
                TokenTree::Punct(punct) => {
                    if punct.as_char() == '\'' {
                        had_apos = true;
                    }
                    TokenTree::Punct(punct)
                },
                TokenTree::Literal(lit) => TokenTree::Literal(lit),
                TokenTree::Group(group) => {
                    let delimiter = group.delimiter();
                    let tt = lifetimes_to_static(lifetimes, group.stream());
                    TokenTree::Group(proc_macro2::Group::new(delimiter, tt))
                },
            }
        })
        .collect()
}
