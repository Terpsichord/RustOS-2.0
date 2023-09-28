#![feature(box_patterns)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Abi, ItemFn, LitStr, ReturnType, Token, Type,
};

#[proc_macro_attribute]
pub fn interrupt_handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(input as ItemFn);

    item_fn.sig.abi = Some(Abi {
        extern_token: Token![extern](item_fn.sig.span()),
        name: Some(LitStr::new("x86-interrupt", Span::call_site())),
    });

    if let ReturnType::Type(_, box Type::Never(_)) = item_fn.sig.output {
        let quote = quote! {
            loop { /* x86_64::instructions::hlt(); */ }
        };
        item_fn.block.stmts.push(parse_quote!(quote as Stmt));
    }
    println!("{:?}", item_fn.to_token_stream());
    item_fn.into_token_stream().into()
}
