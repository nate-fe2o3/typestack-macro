use std::{collections::HashMap, iter::repeat_n};

use proc_macro::{Span, TokenStream};
use quote::{TokenStreamExt, quote};
use syn::{
    Attribute, Data, DeriveInput, GenericParam, Generics, ImplItemType, Item, ItemImpl, ItemTrait,
    LitInt, Token, TypeTuple, parse_macro_input, parse_quote, parse_str, parse2, token::Trait,
};

struct ImplTrait {
    generic_impl: String,
    trait_name: String,
    type_signature: String,
    result_type: String,
}
impl ImplTrait {
    fn new(
        generic_impl: String,
        trait_name: String,
        type_signature: String,
        result_type: String,
    ) -> Self {
        Self {
            generic_impl,
            trait_name,
            type_signature,
            result_type,
        }
    }
}

#[proc_macro_attribute]
pub fn tuple_to_typestack(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_as_stream: quote::__private::TokenStream = item.clone().into();
    let arg = parse_macro_input!(attr as LitInt);
    let number = arg
        .base10_parse::<usize>()
        .expect("macro attribute to be a number");

    let ast = parse_macro_input!(item as ItemTrait);
    let trait_name = ast.ident;
    let tokenstreams_iter = (1..=number)
        .map(|x| {
            // let mut new_impl_block: ItemImpl = parse_quote! {
            //     impl #trait_name {
            //         type Result = String;
            //     }
            // };
            // let gens = (0..x)
            //     .map(|e| parse_str::<GenericParam>(&format!("T{e}")).expect("generic params"));
            // new_impl_block.generics.params.extend(gens);
            // new_impl_block.
            let mut generics_str =
                (0..x).fold(String::from("<"), |acc, y| acc + &format!("T{y}, "));
            generics_str.push('>');
            let generics: Generics = parse_str(&generics_str).expect("generics to work");
            // let generics: Generics = parse_quote! {
            //     <#(T #nums,)*>
            // };
            // let type_signature: TypeTuple = parse2(parse_quote! {
            //     (#(T #nums),*)
            // })
            // .expect("type sig to work");
            let type_signature_str = generics_str
                .chars()
                .map(|c| match c {
                    '<' => '(',
                    '>' => ')',
                    _ => c,
                })
                .collect::<String>();
            dbg!("{}", &type_signature_str);
            let type_signature: TypeTuple =
                parse_str(&type_signature_str).expect("type sig to work");
            let mut result_type_str =
                (0..x).rev().fold(String::from("type Result = "), |acc, y| {
                    acc + &format!("(T{y}, ")
                });
            result_type_str += "()";
            result_type_str += &repeat_n(')', x).collect::<String>();
            result_type_str += ";";
            let result_type: ImplItemType =
                parse_str(&result_type_str).expect("result type to work");
            let thing = quote! {
                impl #generics #trait_name for #type_signature {
                    #result_type
                }
            };
            println!("{}", &thing);
            thing
        })
        .collect::<Vec<_>>();
    let output = quote! {
        #item_as_stream
        #(#tokenstreams_iter)*
        pub struct Thing(usize);
    };
    // output.append_all(tokenstreams_iter);
    output.into()
}
