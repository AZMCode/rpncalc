#![feature(extend_one)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[allow(non_snake_case)]
#[proc_macro_derive(ops_derive_SimpleOp)]
pub fn ops_derive_SimpleOp(input_stream: TokenStream) -> TokenStream {

    let mut out = TokenStream2::default();

    let input_enum: syn::ItemEnum = syn::parse(input_stream).unwrap();
    let mut exclude_fromstr = None;
    let mut simple_name_opt = None;
    let mut name_opt = None;
    let mut description_opt = None;
    let mut input_arity_op: Option<usize> = None;
    let input_enum_name = input_enum.ident;
    let mut input_enum_variants = vec![];
    for attr in input_enum.attrs {
        if matches!(attr.style,syn::AttrStyle::Outer) {
            let my_meta = attr.parse_meta().unwrap();
            if let syn::Meta::List(syn::MetaList {
                path,
                nested,
                ..
            })= my_meta {
                if let Some(true) = path.get_ident().and_then(|i| Some(i.to_string().trim().to_uppercase().as_str() == "SIMPLEOP")) {
                    for m in nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                            path, lit, ..
                        })) = m {
                            match (
                                path.get_ident().and_then(|i| Some(i.to_string().trim().to_uppercase())),
                                lit
                            ) {
                                (Some(m),syn::Lit::Str(s))  if "SIMPLE_NAME"     == m.as_str() => simple_name_opt   = Some(s.value()),
                                (Some(m),syn::Lit::Str(s))  if "NAME"            == m.as_str() => name_opt          = Some(s.value()),
                                (Some(m),syn::Lit::Str(s))  if "DESCRIPTION"     == m.as_str() => description_opt   = Some(s.value()),
                                (Some(m),syn::Lit::Int(i))  if "INPUT_ARITY"     == m.as_str() => input_arity_op    = Some(i.base10_parse().unwrap()),
                                (Some(m),syn::Lit::Bool(b)) if "EXCLUDE_FROMSTR" == m.as_str() => exclude_fromstr   = Some(b.value()),
                                _ => panic!("Unknown attribute name/type combination when parsing SimpleOp Attribute")
                            }
                        }
                    }
                }
            }
        }
    }
    let name = name_opt.ok_or("Name for SimpleOp not given").unwrap();
    let description = description_opt.ok_or("Description for SimpleOp not given").unwrap();
    let input_arity = input_arity_op.ok_or("Input arity not given").unwrap();
    for variant in input_enum.variants {
        let mut calc_closure = None;
        for attr in variant.attrs {
            if matches!(calc_closure,Some(_)) { break; }
            if let syn::Attribute {
                style: syn::AttrStyle::Outer,
                path,
                tokens,
                ..
            } = attr {
                if let Some(true) = path.get_ident().and_then(|i| Some(&i.to_string().trim().to_uppercase() == "SIMPLEOP")) {
                    let closure_expr: syn::Expr = syn::parse2(tokens).unwrap();
                    if let syn::Expr::Paren(syn::ExprParen {
                        expr: inner_expr,
                        ..
                    }) = closure_expr {
                        if let syn::Expr::Closure(closure) = *inner_expr {
                            let closure_input_count = closure.inputs.len();
                            if closure_input_count != input_arity {
                                panic!("Expected closure with {input_arity} inputs, found closure with {closure_input_count}.");
                            }
                            calc_closure = Some(closure);
                        } else {
                            panic!("Expected Closure")
                        }
                    } else {
                        panic!("Expected Parenthetized Closure")
                    }
                }
            }
        }
        if let Some(c) = calc_closure {
            input_enum_variants.push((variant.ident,c));
        } else {
            let variant_name = variant.ident.to_string();
            panic!("Expected variant {variant_name} to have calculation closure, found none")
        }
    }
    let simple_name_tok = match simple_name_opt {
        Some(v) => quote::quote!(Some(#v)),
        None => quote::quote!(None)
    };
    let op_desc_trait_path = quote::quote!(crate::input::command::CommandDesc);
    let comm_trait_path = quote::quote!(crate::input::command::Command);
    let fromstr_trait_path = quote::quote!(::std::str::FromStr);
    let error_type_path = quote::quote!(crate::error::Error);

    let input_enum_variants_fromstr_matches = input_enum_variants.iter()
        .map(|(id,_)| (id.to_string().trim().to_uppercase(),id.clone()))
        .map(|(s,id)| quote::quote!( #s => Ok( #input_enum_name :: #id ), ))
        .fold(TokenStream2::default(),|mut acc,elm| { acc.extend(TokenStream2::from(elm)); acc });
    let input_enum_variants_comm_matches = input_enum_variants.into_iter().map(|(id,c)| {
        let mut calling_parens = syn::ExprCall {
            args: syn::punctuated::Punctuated::default(),
            attrs: vec![],
            paren_token: syn::token::Paren::default(),
            func: Box::new(syn::Expr::Paren(syn::ExprParen {
                attrs: vec![],
                paren_token: syn::token::Paren::default(),
                expr: Box::new(syn::Expr::Closure(c))
            }))
        };
        for i in (0..input_arity).rev() {
            calling_parens.args.push(syn::parse2(quote::quote!( args[#i] )).unwrap());
        }
        quote::quote!( #input_enum_name :: #id =>  #calling_parens,)
    }).fold(TokenStream2::default(),|mut acc, elm| { acc.extend(elm); acc });
    let mut args_vec = syn::Macro {
        path: syn::parse2(quote::quote!(vec)).unwrap(),
        bang_token: syn::token::Bang::default(),
        delimiter: syn::MacroDelimiter::Bracket(Default::default()),
        tokens: TokenStream2::default()
    };
    for _ in 0..input_arity {
        args_vec.tokens.extend(quote::quote!( stack.pop().unwrap(), ));
    }
    let set_up_args = quote::quote!( let args: ::std::vec::Vec<f64> = #args_vec; );
    out.extend(TokenStream2::from(quote::quote!(
        impl #op_desc_trait_path for #input_enum_name {
            const SHORT_NAME: Option<&'static str> = #simple_name_tok ;
            const NAME: &'static str = #name ;
            const DESCRIPTION: &'static str = #description ;
        }
    )));
    if let None | Some(false) = exclude_fromstr {
        out.extend(TokenStream2::from(quote::quote!(
            impl #fromstr_trait_path for #input_enum_name {
                type Err = #error_type_path ;
                fn from_str(s: &str) -> std::result::Result<Self,Self::Err> {
                    let s_upper_trim = s.trim().to_uppercase();
                    match s_upper_trim.as_str() {
                        #input_enum_variants_fromstr_matches
                        _ => Err( #error_type_path ::ParseToken(< Self as #op_desc_trait_path >::NAME) )
                    }
                }
            }
        )));
    }
    out.extend(TokenStream2::from(quote::quote!(
        impl #comm_trait_path for #input_enum_name {
            fn comm(self,stack: &mut Vec<f64>) -> std::result::Result<Option<String>,#error_type_path> {
                let stack_len = stack.len();
                if stack_len < #input_arity {
                    return Err( #error_type_path ::StackEmpty(stack_len,#input_arity) )
                }
                #set_up_args
                stack.push(match self {
                    #input_enum_variants_comm_matches
                });
                Ok(None)
            }
        }
    )));
    TokenStream::from(out)
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn ops_attr_simpleop(_: TokenStream, item: TokenStream) -> TokenStream {
    let outer_e: Result<syn::ItemEnum,_> = syn::parse(item.clone());
    match outer_e {
        Ok(mut e) => {
            for var in e.variants.iter_mut() {
                var.attrs = var.attrs.clone().into_iter()
                    .filter(|attr| 
                        ! matches!(attr.path.get_ident()
                            .and_then(|i| Some(&i.to_string().trim().to_uppercase() == "SIMPLEOP")),
                            Some(true)
                        )
                    ).collect::<Vec<_>>()
            }
            TokenStream::from(quote::quote!{ #e })
        },
        Err(_) => item
    }
}