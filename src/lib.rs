use std::iter::repeat_n;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Generics, Ident, ImplItemType, ItemTrait, LitInt, TraitItem, TypeTuple, parse_macro_input,
    parse_str,
};

#[proc_macro_attribute]
pub fn tuple_to_typestack(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_trait_def: quote::__private::TokenStream = item.clone().into();
    let arg = parse_macro_input!(attr as LitInt);
    let number = arg
        .base10_parse::<usize>()
        .expect("macro attribute to be a number");

    let ast = parse_macro_input!(item as ItemTrait);
    let trait_name = ast.ident;
    let associated_type_name = ast
        .items
        .iter()
        .find_map(|ti| {
            if let TraitItem::Type(a) = ti {
                Some(a.ident.clone())
            } else {
                None
            }
        })
        .expect("there to be an associated type in the trait def");
    let tokenstreams_iter = (1..=number)
        .map(|x| {
            let mut generics_str =
                (0..x).fold(String::from("<"), |acc, y| acc + &format!("T{y}, "));
            generics_str.push('>');
            let generics: Generics = parse_str(&generics_str).expect("generics to work");
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
            let mut assoc_type_str = (0..x)
                .rev()
                .fold(format!("type {associated_type_name} = "), |acc, y| {
                    acc + &format!("(T{y}, ")
                });
            assoc_type_str += &format!("(){};", repeat_n(')', x).collect::<String>());
            let result_type: ImplItemType = parse_str(&assoc_type_str).expect("assoc type to work");
            let thing = quote! {
                impl #generics #trait_name for #type_signature {
                    #result_type
                }
            };
            println!("{}", &thing);
            thing
        })
        .collect::<Vec<_>>();
    quote! {
        #original_trait_def
        impl #trait_name for () {
            type #associated_type_name = ();
        }
        #(#tokenstreams_iter)*
    }
    .into()
}
