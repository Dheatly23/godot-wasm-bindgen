use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::spanned::Spanned;
use syn::{FnArg, Ident, Index, Item, ItemFn, LitByte, LitInt, ReturnType, Type, Visibility};

use crate::util::{prepend, tag_length};

macro_rules! bail_syn {
    ($span:expr, $fmt:literal $(, $v:tt)* $(,)?) => {
        return Err(syn::parse::Error::new($span, format!($fmt $(, $v)*)))
    };
}

#[derive(Default)]
pub struct BindgenMetadata {}

impl Parse for BindgenMetadata {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let ret = Self::default();
        if input.is_empty() {
            return Ok(ret);
        }

        // Currently there is no attributes
        Err(ParseError::new(
            input.cursor().span(),
            "Attributes currently not supported",
        ))
    }
}

pub struct BindgenFunction {
    metadata: BindgenMetadata,
    item: ItemFn,

    name: String,
    param_count: usize,
    result_count: usize,
}

impl BindgenFunction {
    pub fn new(metadata: BindgenMetadata, mut item: ItemFn) -> ParseResult<Self> {
        if !matches!(&item.vis, Visibility::Public(_)) {
            bail_syn!(item.vis.span(), "Visibility must be public");
        }
        item.vis = Visibility::Inherited;

        let name;
        let (param_count, result_count);
        {
            let sig = &item.sig;
            sig.inputs
                .iter()
                .map(|t| match t {
                    FnArg::Typed(_) => Ok(()),
                    FnArg::Receiver(t) => {
                        bail_syn!(t.span(), "Support for methods is not currently supported")
                    }
                })
                .fold(Ok(()), |a, b| match (a, b) {
                    (Ok(()), x) | (x, Ok(())) => x,
                    (Err(mut e1), Err(e2)) => {
                        e1.combine(e2);
                        Err(e1)
                    }
                })?;

            name = sig.ident.to_string();

            param_count = sig.inputs.len();
            result_count = match &sig.output {
                ReturnType::Default => 0,
                ReturnType::Type(_, t) => match &**t {
                    Type::Tuple(t) => t.elems.len(),
                    _ => 1,
                },
            };
        }

        Ok(Self {
            metadata,
            item,

            name,
            param_count,
            result_count,
        })
    }
}

impl ToTokens for BindgenFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item.to_tokens(tokens);

        let name = &self.name;
        let sig = &self.item.sig;

        let func_export = {
            let param_args = (0..self.param_count).map(|i| {
                let i = format_ident!("arg{}", i);
                quote!(#i : GodotValue)
            });

            let result_args = match &sig.output {
                v @ ReturnType::Default => quote!(#v),
                ReturnType::Type(arrow, t) => match &**t {
                    Type::Tuple(t) => {
                        let r = iter::repeat(Ident::new("GodotValue", Span::call_site()))
                            .take(t.elems.len());
                        quote!(#arrow ( #(#r ,)* ))
                    }
                    _ => quote!(#arrow GodotValue),
                },
            };

            let param_cvt = (0..self.param_count).map(|i| {
                let i = format_ident!("arg{}", i);
                quote!(#i.try_into().unwrap())
            });

            let ret_cvt = match &sig.output {
                ReturnType::Default => quote!(ret),
                ReturnType::Type(_, t) => match &**t {
                    Type::Tuple(t) => {
                        let r = (0..t.elems.len()).map(Index::from);
                        quote!( ( #(ret.#r.into() ,)* ) )
                    }
                    _ => quote!(ret.into()),
                },
            };

            let name_ident = format_ident!("{}", name);
            quote!(
                #[export_name = #name]
                #[doc(hidden)]
                pub extern "C" fn export_function(#(#param_args),*) #result_args {
                    use std::convert::TryFrom;
                    let ret = #name_ident ( #(#param_cvt),* );
                    #ret_cvt
                }
            )
        };

        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(name.as_bytes());
        tag_length(&mut bytes);

        let (param_len, param_bcount) = {
            let mut bytes = Vec::new();
            leb128::write::unsigned(&mut bytes, self.param_count as _).unwrap();

            let l = bytes.len();
            let ret = bytes
                .into_iter()
                .map(|b| LitByte::new(b, Span::call_site()));
            (ret, l)
        };
        let (result_len, result_bcount) = {
            let mut bytes = Vec::new();
            leb128::write::unsigned(&mut bytes, self.result_count as _).unwrap();

            let l = bytes.len();
            let ret = bytes
                .into_iter()
                .map(|b| LitByte::new(b, Span::call_site()));
            (ret, l)
        };

        let extra_count = param_bcount + result_bcount + self.param_count + self.result_count;
        bytes.extend(iter::repeat(0).take(extra_count));
        tag_length(&mut bytes);
        bytes.truncate(bytes.len() - extra_count);
        prepend(&mut bytes, &[1, 0, 0, 0]);

        let bytes_len_token =
            LitInt::new(&format!("{}", bytes.len() + extra_count), Span::call_site());
        let bytes_token = bytes
            .into_iter()
            .map(|b| LitByte::new(b, Span::call_site()));

        let param_data = sig.inputs.iter().map(|v| match &*v {
            FnArg::Typed(v) => &v.ty,
            FnArg::Receiver(_) => unreachable!("Method function should get filtered"),
        });

        let result_data = match &sig.output {
            ReturnType::Default => Vec::new(),
            ReturnType::Type(_, v) => match &**v {
                Type::Tuple(v) => v.elems.iter().collect(),
                v => vec![v],
            },
        };

        quote!(
            const _: () = {
                use godot_wasm_bindgen::__hidden::{DataTypeValue, GodotValue};

                #func_export

                #[link_section = "__godot_wasm_bindgen_data"]
                #[doc(hidden)]
                static DATA: [u8; #bytes_len_token] = [
                    #(#bytes_token ,)*
                    #(#param_len ,)*
                    #(<DataTypeValue<#param_data>>::value() ,)*
                    #(#result_len ,)*
                    #(<DataTypeValue<#result_data>>::value()),*
                ];
            };
        )
        .to_tokens(tokens);
    }
}

pub enum BindgenInput {
    Function(BindgenFunction),
}

impl BindgenInput {
    pub fn process(metadata: BindgenMetadata, input: TokenStream) -> ParseResult<Self> {
        match syn::parse2(input)? {
            Item::Fn(f) => Ok(Self::Function(BindgenFunction::new(metadata, f)?)),
            item => bail_syn!(item.span(), "Unknown or unsupported item type"),
        }
    }
}

impl ToTokens for BindgenInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Function(f) => f.to_tokens(tokens),
        }
    }
}
