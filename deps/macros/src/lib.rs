extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
mod utils;
use utils::IntoError;

const ATTR_MACRO_NAME: &'static str = "simple_op";

#[allow(non_snake_case)]
#[proc_macro_derive(SimpleOp)]
pub fn ops_derive_SimpleOp(input_stream: TokenStream) -> TokenStream {
    let macro_res: Result<TokenStream2,syn::Error> = (|| {
        let syn::ItemEnum {
            attrs: external_attrs,
            ident: self_ident,
            variants,
            ..
        } = syn::parse(input_stream)?;
        enum KeyValue {
            String(String),
            Usize(usize),
            Bool(bool)
        }
        let enum_props = external_attrs.into_iter()
            .filter(|a| a.path.get_ident().map(|i| &i.to_string() == ATTR_MACRO_NAME) == Some(true) )
            .map(|a| match a.parse_meta()? {
                syn::Meta::List(l) => Ok(l),
                _ => Err(a.into_error("Expected SimpleOp Attribute to be of List kind"))
            }).try_fold(vec![], |mut acc, l| {
                acc.extend(l?.nested);
                Result::<_,syn::Error>::Ok(acc)
            })?.into_iter()
            .map(|m| match m {
                syn::NestedMeta::Meta(m) => Ok(m),
                _ => Err(m.into_error("Expected all metas at this level to not be a literal"))
            }).map(|m| match m? {
                syn::Meta::NameValue(p) => Ok(p),
                m => Err(m.into_error("Expected all metas at this level to be name-value pairs"))
            }).map(|p| match (p.clone()?.path.get_ident().map(|i| i.to_string().trim().to_uppercase()).as_ref().map(|i| i.as_str()),p.clone()?.lit) {
                (None,_) => Err(p?.into_error("Expected name to be only ident, not also path")),
                (Some(key @ ( "SHORT_NAME" | "NAME" | "DESCRIPTION")),syn::Lit::Str(s)) => Ok((key.to_string(),KeyValue::String(s.value()))),
                (Some("INPUT_ARITY"),syn::Lit::Int(i)) => Ok(("INPUT_ARITY".to_string(),KeyValue::Usize(i.base10_parse()?))),
                (Some("EXCLUDE_FROMSTR"),syn::Lit::Bool(b)) => Ok(("EXCLUDE_FROMSTR".to_string(),KeyValue::Bool(b.value))),
                _ => Err(p?.into_error("Combination Name and Value not recoognized"))
            }).collect::<Result<std::collections::HashMap<String,_>,_>>()?;
        let short_name = enum_props.get("SHORT_NAME")
            .map(|v| match v {
                KeyValue::Usize(i) => quote::quote!( Some(#i) ),
                _ => unreachable!()
            }).unwrap_or(quote::quote!(None));
        let name = enum_props.get("NAME").map(|v| match v {
            KeyValue::String(s) => s.to_string(),
            _ => unreachable!()
        }).ok_or(self_ident.into_error("Name not specified in enum attributes"))?;
        let description = enum_props.get("DESCRIPTION").map(|v| match v {
            KeyValue::String(s) => s.to_string(),
            _ => unreachable!()
        }).ok_or(self_ident.into_error("Description not specified in enum attributes"))?;
        let input_arity = enum_props.get("INPUT_ARITY").map(|v| match v {
            KeyValue::Usize(i) => *i,
            _ => unreachable!()
        }).ok_or(self_ident.into_error("Input Arity not specified in enum attributes"))?;
        let exclude_fromstr = enum_props.get("EXCLUDE_FROMSTR").map(|v| match v {
            KeyValue::Bool(b) => *b,
            _ => unreachable!()
        }).unwrap_or(false);

        let construct_vec_args = syn::punctuated::Punctuated
            ::<TokenStream2,syn::token::Comma>
            ::from_iter(vec![quote::quote!(stack.pop().unwrap());input_arity]);
        let destruct_vec_args = syn::punctuated::Punctuated
            ::<TokenStream2,syn::token::Comma>
            ::from_iter(vec![quote::quote!(args .pop().unwrap());input_arity]);

        let variant_to_closure_map = variants.into_iter().map(|v| Ok((
            v.ident.clone(),
            v.attrs.clone().into_iter()
                .filter(|a| a.path.get_ident()
                    .map(|i| &i.to_string() == ATTR_MACRO_NAME)
                    == Some(true)
                ).map(|a| syn::parse2::<syn::ExprParen>(a.tokens) )
                .map(|p| match *(p.clone()?.expr) {
                    syn::Expr::Closure(c) => Ok(c),
                    _ => Err(p.clone()?.into_error("Expected Closure Here"))
                }).map(|a| (a.clone()?.inputs.len() == input_arity)
                    .then(|| a.clone())
                    .ok_or(a?.into_error(format!("Expected closure to have {input_arity} arguments")))
                ).collect::<Result<Vec<_>,_>>()
                .and_then(|internal_v| (internal_v.len() == 1)
                    .then(|| (&internal_v[0]).clone())
                    .ok_or(v.into_error("Expected variant to have 1 associated closure for calculation"))
                )??
        ))).collect::<Result<std::collections::HashMap<_,_>,syn::Error>>()?;
        let variants_iter = variant_to_closure_map.keys().map(|v| v.clone()).collect::<Vec<_>>();
        let variants_str_iter = variant_to_closure_map.keys().map(|i| i.to_string().trim().to_uppercase()).collect::<Vec<_>>();
        let closures_iter = variant_to_closure_map.values().map(|v| v.clone()).collect::<Vec<_>>();
        let command_desc = quote::quote!(crate::input::command::CommandDesc);
        let command = quote::quote!(crate::input::command::Command);
        let from_str = quote::quote!(::std::str::FromStr);
        let result = quote::quote!(::std::result::Result);
        let error = quote::quote!(crate::error::Error);
        let mut out = quote::quote!(
            impl #command_desc for #self_ident {
                const SHORT_NAME: Option<&'static str> = #short_name;
                const NAME: &'static str = #name;
                const DESCRIPTION: &'static str = #description;
            }

            impl #command for #self_ident {
                fn comm(self, stack: &mut Vec<f64>) -> #result <Option<String>, #error > {
                    if stack.len() < #input_arity {
                        return Err(#error :: StackEmpty(stack.len(), #input_arity ));
                    }
                    let mut args: ::std::vec::Vec::<f64> = vec![#construct_vec_args];
                    stack.push(match self {
                        #( #self_ident :: #variants_iter => (#closures_iter)(#destruct_vec_args) ),*
                    });
                    Ok(None)
                }
            }
        );
        if !exclude_fromstr {
            out.extend(quote::quote!(
                impl #from_str for #self_ident {
                    type Err = #error;
                    fn from_str(input: &str) -> #result <#self_ident,#error> {
                        match input.to_uppercase().trim() {
                            #( #variants_str_iter => Ok(#self_ident :: #variants_iter) ),* ,
                            _ => Err(#error :: ParseToken( #name ))
                        }
                    }
                }
            ))
        }
        Ok(out)
    })();
    match macro_res {
        Ok(ts) => TokenStream::from(ts),
        Err(e) => TokenStream::from(e.to_compile_error())
    }
}

#[proc_macro_attribute]
pub fn simple_op(_: TokenStream, input: TokenStream) -> TokenStream {
    let macro_res: Result<_,syn::Error> = (|| {
        let mut self_enum = syn::parse::<syn::ItemEnum>(input)?;
        for var_ref in self_enum.variants.iter_mut() {
            var_ref.attrs = var_ref.attrs.clone().into_iter()
                .filter(|a| a.path.get_ident().map(|i| &i.to_string() == ATTR_MACRO_NAME) != Some(true))
                .collect::<Vec<_>>();
        }
        Ok(quote::quote!( #self_enum ))
    })();
    match macro_res {
        Ok(ts) => TokenStream::from(ts),
        Err(e) => TokenStream::from(e.to_compile_error())
    }
}