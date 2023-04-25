use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

#[derive(Clone, Default)]
pub(crate) struct Args {
    pub(crate) send: Option<syn::Ident>,
    pub(crate) sync: Option<syn::Ident>,
    pub(crate) vis: Option<syn::Visibility>,
    pub(crate) name: Option<syn::Ident>,
    pub(crate) crate_name: Option<syn::Path>,
}

#[derive(Clone)]
pub(crate) struct Func {
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) attrs_split: Option<usize>,
    pub(crate) vis: syn::Visibility,
    pub(crate) sig: syn::Signature,
    pub(crate) body: proc_macro2::TokenTree,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut result = Args::default();
        if !input.is_empty() {
            while !input.is_empty() {
                if input.peek(syn::Token![type]) {
                    let _: syn::Token![type] = input.parse()?;
                    let _: syn::Token![=] = input.parse()?;
                    result.vis = Some(input.parse()?);
                    result.name = Some(input.parse()?);
                } else if input.peek(syn::Token![crate]) {
                    let _: syn::Token![crate] = input.parse()?;
                    let _: syn::Token![=] = input.parse()?;
                    result.crate_name = Some(input.parse()?);
                } else {
                    let ident: syn::Ident = input.parse()?;
                    if ident == "Send" {
                        result.send = Some(ident);
                    } else if ident == "Sync" {
                        result.sync = Some(ident);
                    } else {
                        return Err(syn::Error::new_spanned(ident, "Unexpected input"));
                    }
                }

                if input.peek(syn::Token![,]) {
                    let _: syn::Token![,] = input.parse()?;
                } else if input.is_empty() {
                    break;
                } else {
                    return Err(input.error("Unexpected input"));
                }
            }
        }
        Ok(result)
    }
}

impl Parse for Func {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let attrs_split = attrs
            .iter()
            .enumerate()
            .find(|(_, attr)| {
                let Some(str) = get_doc_value(attr) else { return false; };
                if let Some(doc) = str.value().trim_start().strip_prefix('#') {
                    doc.trim().eq_ignore_ascii_case("struct")
                } else {
                    false
                }
            })
            .map(|(index, _)| index);
        Ok(Self {
            attrs,
            attrs_split,
            vis: input.parse()?,
            sig: input.parse()?,
            body: input.parse()?,
        })
    }
}

pub(crate) fn get_doc_value(attr: &syn::Attribute) -> Option<&syn::LitStr> {
    let syn::Meta::NameValue(kv) = &attr.meta else { return None; };
    let Some(ident) = kv.path.get_ident() else { return None; };
    if ident != "doc" {
        return None;
    }

    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = &kv.value
    {
        Some(lit)
    } else {
        None
    }
}

impl ToTokens for Func {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.vis.to_tokens(tokens);
        self.sig.to_tokens(tokens);
    }
}
