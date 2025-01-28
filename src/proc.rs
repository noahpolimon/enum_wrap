use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, parse_str,
    punctuated::Punctuated,
    token, Attribute, Ident, ItemTrait, Path, Result, Token, Visibility,
};

pub(crate) static ITEM_TRAITS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) struct VariantPath {
    pub attrs: Vec<Attribute>,
    pub path: Path,
}

impl Parse for VariantPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let path = input.parse()?;

        Ok(Self { attrs, path })
    }
}

impl ToTokens for VariantPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attrs;
        let path = &self.path;

        tokens.extend(quote! {
            #(#attrs)*
            #path
        });
    }
}

#[allow(dead_code)]
pub(crate) struct TypeWrapInput {
    attrs: Vec<Attribute>,
    auto_impls: Vec<Path>,
    vis: Visibility,
    ident: Ident,
    brace_token: token::Brace,
    variants: Punctuated<VariantPath, Token![,]>,
}

impl Parse for TypeWrapInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Attribute::parse_outer(input)?;
        let impl_attr_ident = "auto_impl";

        let auto_impls = attrs
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(index, attr)| match &attr.meta {
                syn::Meta::List(list) => {
                    if let Some(id) = list.path.get_ident() {
                        if id == impl_attr_ident {
                            attrs.remove(index);
                            let tokens = &list.tokens;
                            let punc: Punctuated<Path, Token![,]> = parse_quote!(#tokens);
                            Some(punc.into_pairs().map(|pair| pair.into_value()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();

        let vis = input.parse()?;
        let ident = input.parse()?;

        let content;
        let brace_token = syn::braced!(content in input);
        let variants = Punctuated::parse_terminated(&content)?;

        Ok(Self {
            attrs,
            auto_impls,
            vis,
            ident,
            brace_token,
            variants,
        })
    }
}

// TODO: make it work with fn defs with self with no ref param or no self param at all
// TODO: refactor
pub(crate) fn enum_wrap2(input: TypeWrapInput) -> TokenStream {
    let attrs = input.attrs;

    let auto_impls = input.auto_impls;

    let trait_defs = ITEM_TRAITS.lock().unwrap_or_else(|e| {
        ITEM_TRAITS.clear_poison();
        e.into_inner()
    });

    let vis = input.vis;
    let ident = input.ident;

    let mut variants = vec![];
    let variants_inner =
        input
            .variants
            .iter()
            .map(|path| {
                variants.push(path.path.segments.last().unwrap_or_else(|| {
                    panic!("failed to obtain variant ident from {:?}", path.path)
                }));
                &path.path
            })
            .collect::<Vec<_>>();

    let trait_fn_tokens = auto_impls.into_iter().map(|path| {
        let trait_path = path.to_token_stream().to_string();

        let trait_def_str = trait_defs.get(&trait_path).unwrap_or_else(|| {
            panic!(
                "{} not annotated with #[enum_wrap_impl] or does not exist",
                trait_path
            )
        });

        let trait_def = parse_str::<ItemTrait>(trait_def_str.as_str())
            .expect("unable to parse trait definition");

        let def_ident = trait_def.ident;
        let def_generics = trait_def.generics;

        let fn_def = trait_def.items.into_iter().filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                let method_sig = &method.sig;
                let method_ident = &method_sig.ident;
                let method_inputs = &method_sig.inputs;
                let _self_ref = method_inputs
                    .first()
                    .map(|s| match s {
                        syn::FnArg::Receiver(receiver) => {
                            // FIXME: allow types Self, &Self, &mut Self
                            if receiver.colon_token.is_some() {
                                panic!("receiver should not have an arbitrary type")
                            }

                            receiver.reference.as_ref().map(|r| r.0)
                        }
                        _ => {
                            panic!("{}::{} does not have a receiver", def_ident, method_ident)
                        }
                    })
                    .unwrap_or_else(|| {
                        panic!("{}::{} does not have a receiver", def_ident, method_ident)
                    });

                let method_args = method_inputs
                    .pairs()
                    .skip(1)
                    .filter_map(|pair| match pair.into_tuple().0 {
                        syn::FnArg::Typed(pat_type) => {
                            let pat = pat_type.pat.to_token_stream();

                            Some(quote! {#pat,})
                        }
                        _ => None,
                    })
                    .collect::<TokenStream>();

                Some(quote!(
                    #method_sig {
                        match self {
                            #(#ident::#variants(var) =>
                                var.#method_ident(#method_args)),*
                        }
                    }
                ))
            } else {
                None
            }
        });

        quote! {
            #[automatically_derived]
            impl #def_generics #def_ident #def_generics for #ident {
                #(#fn_def)*
            }
        }
    });

    let into_impls = variants.iter().zip(&variants_inner).map(|(variant, ty)| {
        quote! {
            #[automatically_derived]
            impl ::core::convert::Into<#ident> for #ty {
                fn into(self) -> #ident {
                    #ident::#variant(self)
                }
            }
        }
    });

    quote! {
        #(#attrs)*
        #vis enum #ident {
            #(#variants(#variants_inner)),*
        }

        #(#trait_fn_tokens)*
        #(#into_impls)*
    }
}

pub(crate) fn enum_wrap_impl2(input: ItemTrait) -> TokenStream {
    let mut traits = ITEM_TRAITS.lock().unwrap_or_else(|e| {
        ITEM_TRAITS.clear_poison();
        e.into_inner()
    });

    let input_tokens = input.to_token_stream();

    traits.insert(input.ident.to_string(), input_tokens.to_string());

    input_tokens
}
