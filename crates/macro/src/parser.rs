use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::spanned::Spanned;
use syn::{
    parse2, FnArg, ForeignItem, ForeignItemFn, Ident, Index, Item, ItemFn, ItemForeignMod, LitByte,
    LitInt, LitStr, ReturnType, Signature, Type, Visibility,
};

use crate::util::{join_errors, prepend, tag_length};

macro_rules! bail_syn {
    ($span:expr, $fmt:literal $(, $v:tt)* $(,)?) => {
        return Err(syn::parse::Error::new($span, format!($fmt $(, $v)*)))
    };
}

#[derive(Default)]
pub struct BindgenMetadata {
    use_native_types: bool,
}

#[derive(Default)]
struct BindgenMetatadaBuilder(BindgenMetadata);

impl BindgenMetatadaBuilder {
    fn finish(self) -> BindgenMetadata {
        self.0
    }

    fn use_native_types(&mut self, span: Span) -> ParseResult<&mut Self> {
        match &mut self.0.use_native_types {
            v @ false => {
                *v = true;
                Ok(self)
            }
            true => Err(ParseError::new(
                span,
                "Attribute already set (maybe duplicate?)",
            )),
        }
    }
}

impl Parse for BindgenMetadata {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut ret = BindgenMetatadaBuilder::default();
        if input.is_empty() {
            return Ok(ret.finish());
        }

        let mut errs = None;
        let mut start = true;
        while !input.is_empty() {
            if start {
                start = false;
            } else if join_errors(&mut errs, input.parse::<Token![,]>()).is_none() {
                break;
            }

            if input.is_empty() {
                break;
            }

            let attr_ident: Ident = match join_errors(&mut errs, input.parse()) {
                Some(v) => v,
                None => break,
            };
            let attr_span = attr_ident.span();
            let attr_name = attr_ident.to_string();

            match &attr_name as &str {
                "use_native_types" => join_errors(&mut errs, ret.use_native_types(attr_span)),
                _ => {
                    if input.peek(Token![=]) {
                        join_errors(&mut errs, input.parse::<Token![=]>());
                        join_errors(
                            &mut errs,
                            input.step(|cursor| match cursor.token_tree() {
                                Some(v) => Ok(v),
                                None => Err(ParseError::new(cursor.span(), "No value assigned")),
                            }),
                        );
                    }

                    join_errors(
                        &mut errs,
                        Err(ParseError::new(
                            attr_span,
                            format!("Unknown attribute {}", attr_name),
                        )),
                    )
                }
            };
        }

        match errs {
            Some(e) => Err(e),
            None => Ok(ret.finish()),
        }
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

        if metadata.use_native_types {
            if item.sig.abi.is_none() {
                return Err(ParseError::new_spanned(
                    item.sig,
                    "Natively exporting functions must be marked as extern",
                ));
            }
        } else {
            item.vis = Visibility::Inherited;
        }

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

        let func_export = if self.metadata.use_native_types {
            quote!()
        } else {
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
        leb128::write::unsigned(&mut bytes, 64).unwrap();
        leb128::write::unsigned(&mut bytes, name.len() as _).unwrap();
        bytes.extend(name.as_bytes());

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

        let param_data = if !self.metadata.use_native_types {
            let v =
                iter::repeat(Ident::new("GodotValue", Span::call_site())).take(self.param_count);
            quote!(#(<DataTypeValue<#v>>::value() ,)*)
        } else {
            let v = sig.inputs.iter().map(|v| match &*v {
                FnArg::Typed(v) => &*v.ty,
                FnArg::Receiver(_) => unreachable!("Method function should get filtered"),
            });
            quote!(#(<DataTypeValue<#v>>::value() ,)*)
        };

        let result_data = if !self.metadata.use_native_types {
            let v =
                iter::repeat(Ident::new("GodotValue", Span::call_site())).take(self.result_count);
            quote!(#(<DataTypeValue<#v>>::value() ,)*)
        } else {
            match &sig.output {
                ReturnType::Default => quote!(),
                ReturnType::Type(_, v) => match &**v {
                    Type::Tuple(v) => {
                        let v = v.elems.iter();
                        quote!(#(<DataTypeValue<#v>>::value() ,)*)
                    }
                    v => quote!(<DataTypeValue<#v>>::value(),),
                },
            }
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
                    #param_data
                    #(#result_len ,)*
                    #result_data
                ];
            };
        )
        .to_tokens(tokens);
    }
}

pub struct ModuleName(String);

impl Parse for ModuleName {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let name: Ident = input.parse()?;
        if name.to_string() != "wasm_import_module" {
            return Err(ParseError::new(name.span(), "Invalid link structure"));
        }
        input.parse::<Token![=]>()?;
        let name: LitStr = input.parse()?;
        Ok(Self(name.value()))
    }
}

pub struct BindgenImport {
    metadata: BindgenMetadata,
    item: ItemForeignMod,

    module_name: String,
}

impl BindgenImport {
    pub fn new(metadata: BindgenMetadata, mut item: ItemForeignMod) -> ParseResult<Self> {
        let mut module_name = None;
        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 {
                if let Some(v) = attr.path.segments.first() {
                    let i = v.ident.to_string();
                    if i != "link" {
                        continue;
                    }

                    let name: ModuleName = parse2(attr.tokens.to_token_stream())?;
                    module_name = Some(name.0);
                }
            }
        }

        let mut errs = None;
        for i in item.items.iter() {
            if !matches!(i, ForeignItem::Fn(_)) {
                join_errors::<()>(
                    &mut errs,
                    Err(ParseError::new_spanned(
                        i,
                        "Non-function item is currently unsupported",
                    )),
                );
            }
        }

        if let Some(e) = errs {
            return Err(e);
        }

        let module_name = module_name.unwrap_or_else(|| {
            item.attrs
                .push(parse_quote!(#[link(wasm_import_module = "host")]));
            "host".into()
        });

        Ok(Self {
            metadata,
            item,
            module_name,
        })
    }
}

impl ToTokens for BindgenImport {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.metadata.use_native_types {
            self.item.to_tokens(tokens);
        } else {
            let item = &self.item;
            for i in item.items.iter().filter_map(|v| match v {
                ForeignItem::Fn(f) => Some(f),
                _ => None,
            }) {
                let func_ident = &i.sig.ident;
                let ext_item = {
                    let ItemForeignMod { attrs, abi, .. } = &*item;
                    let ForeignItemFn {
                        attrs: in_attrs,
                        sig:
                            Signature {
                                fn_token,
                                ident,
                                inputs,
                                output,
                                ..
                            },
                        semi_token,
                        ..
                    } = &*i;

                    let inputs = inputs.iter().enumerate().map(|(i, _)| {
                        let ident = format_ident!("arg{}", i);
                        quote!(#ident : GodotValue)
                    });

                    let output = match output {
                        o @ ReturnType::Default => quote!(#o),
                        ReturnType::Type(arrow, ty) => match &**ty {
                            Type::Tuple(t) => {
                                let v = iter::repeat(Ident::new("GodotValue", Span::call_site()))
                                    .take(t.elems.len());
                                quote!(#arrow (#(#v),*))
                            }
                            _ => quote!(#arrow GodotValue),
                        },
                    };

                    quote!(
                        #(#attrs)*
                        #abi {
                            #(#in_attrs)*
                            #fn_token #ident ( #(#inputs),* ) #output #semi_token
                        }
                    )
                };

                let ForeignItemFn { vis, sig, .. } = &*i;
                let Signature {
                    constness,
                    asyncness,
                    unsafety,
                    abi,
                    fn_token,
                    output,
                    ..
                } = &*sig;

                let param_args = sig.inputs.iter().enumerate().map(|(i, v)| match v {
                    v @ FnArg::Receiver(_) => quote!(#v),
                    FnArg::Typed(v) => {
                        let ty = &*v.ty;
                        let i = format_ident!("arg{}", i);
                        quote!(#i : #ty)
                    }
                });

                let param_cvt = sig.inputs.iter().enumerate().map(|(i, v)| {
                    let name = match v {
                        FnArg::Receiver(v) => Ident::new("self", v.self_token.span),
                        FnArg::Typed(_) => format_ident!("arg{}", i),
                    };
                    quote!(#name.into())
                });

                let ret_cvt = match &sig.output {
                    ReturnType::Default => quote!(ret),
                    ReturnType::Type(_, t) => match &**t {
                        Type::Tuple(t) => {
                            let r = (0..t.elems.len()).map(Index::from);
                            quote!( ( #(ret.#r.into() ,)* ) )
                        }
                        _ => quote!(ret.try_into().unwrap()),
                    },
                };

                quote!(
                    #vis #constness #asyncness #unsafety #abi #fn_token #func_ident ( #(#param_args),* ) #output {
                        use godot_wasm_bindgen::__hidden::{DataTypeValue, GodotValue};

                        #ext_item

                        unsafe {
                            let ret = #func_ident ( #(#param_cvt),* );

                            #ret_cvt
                        }
                    }
                )
                .to_tokens(tokens);
            }

            let ItemForeignMod {
                attrs, abi, items, ..
            } = item;
            let items = items.iter().filter(|v| !matches!(v, ForeignItem::Fn(_)));

            quote!(
                #(#attrs)* #abi {
                    #(#items)*
                }
            )
            .to_tokens(tokens);
        }

        for i in self.item.items.iter().filter_map(|v| match v {
            ForeignItem::Fn(f) => Some(f),
            _ => None,
        }) {
            let name = i.sig.ident.to_string();

            let mut bytes = Vec::new();
            leb128::write::unsigned(&mut bytes, 0).unwrap();
            leb128::write::unsigned(&mut bytes, self.module_name.len() as _).unwrap();
            bytes.extend(self.module_name.as_bytes());
            leb128::write::unsigned(&mut bytes, name.len() as _).unwrap();
            bytes.extend(name.as_bytes());

            let param_count = i.sig.inputs.len();
            let result_count = match &i.sig.output {
                ReturnType::Default => 0,
                ReturnType::Type(_, t) => match &**t {
                    Type::Tuple(t) => t.elems.len(),
                    _ => 1,
                },
            };

            let (param_len, param_bcount) = {
                let mut bytes = Vec::new();
                leb128::write::unsigned(&mut bytes, param_count as _).unwrap();

                let l = bytes.len();
                let ret = bytes
                    .into_iter()
                    .map(|b| LitByte::new(b, Span::call_site()));
                (ret, l)
            };
            let (result_len, result_bcount) = {
                let mut bytes = Vec::new();
                leb128::write::unsigned(&mut bytes, result_count as _).unwrap();

                let l = bytes.len();
                let ret = bytes
                    .into_iter()
                    .map(|b| LitByte::new(b, Span::call_site()));
                (ret, l)
            };

            let extra_count = param_bcount + result_bcount + param_count + result_count;
            bytes.extend(iter::repeat(0).take(extra_count));
            tag_length(&mut bytes);
            bytes.truncate(bytes.len() - extra_count);
            prepend(&mut bytes, &[1, 0, 0, 0]);

            let bytes_len_token =
                LitInt::new(&format!("{}", bytes.len() + extra_count), Span::call_site());
            let bytes_token = bytes
                .into_iter()
                .map(|b| LitByte::new(b, Span::call_site()));

            let param_data = if !self.metadata.use_native_types {
                let v = iter::repeat(Ident::new("GodotValue", Span::call_site())).take(param_count);
                quote!(#(<DataTypeValue<#v>>::value() ,)*)
            } else {
                let v = i.sig.inputs.iter().map(|v| match &*v {
                    FnArg::Typed(v) => &*v.ty,
                    FnArg::Receiver(_) => unreachable!("Method function should get filtered"),
                });
                quote!(#(<DataTypeValue<#v>>::value() ,)*)
            };

            let result_data = if !self.metadata.use_native_types {
                let v =
                    iter::repeat(Ident::new("GodotValue", Span::call_site())).take(result_count);
                quote!(#(<DataTypeValue<#v>>::value() ,)*)
            } else {
                match &i.sig.output {
                    ReturnType::Default => quote!(),
                    ReturnType::Type(_, v) => match &**v {
                        Type::Tuple(v) => {
                            let v = v.elems.iter();
                            quote!(#(<DataTypeValue<#v>>::value() ,)*)
                        }
                        v => quote!(<DataTypeValue<#v>>::value(),),
                    },
                }
            };

            quote!(
                const _: () = {
                    use godot_wasm_bindgen::__hidden::{DataTypeValue, GodotValue};

                    #[link_section = "__godot_wasm_bindgen_data"]
                    #[doc(hidden)]
                    static DATA: [u8; #bytes_len_token] = [
                        #(#bytes_token ,)*
                        #(#param_len ,)*
                        #param_data
                        #(#result_len ,)*
                        #result_data
                    ];
                };
            )
            .to_tokens(tokens);
        }
    }
}

pub enum BindgenInput {
    Function(BindgenFunction),
    Import(BindgenImport),
}

impl BindgenInput {
    pub fn process(metadata: BindgenMetadata, input: TokenStream) -> ParseResult<Self> {
        match syn::parse2(input)? {
            Item::Fn(f) => Ok(Self::Function(BindgenFunction::new(metadata, f)?)),
            Item::ForeignMod(i) => Ok(Self::Import(BindgenImport::new(metadata, i)?)),
            item => bail_syn!(item.span(), "Unknown or unsupported item type"),
        }
    }
}

impl ToTokens for BindgenInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Function(f) => f.to_tokens(tokens),
            Self::Import(i) => i.to_tokens(tokens),
        }
    }
}
