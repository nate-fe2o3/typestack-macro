use std::iter::repeat_n;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Generics, ImplItemType, ItemTrait, LitInt, TraitItem, TypeTuple, parse_macro_input, parse_str,
};

#[proc_macro_attribute]
pub fn tuple_to_typestack(attr: TokenStream, item: TokenStream) -> TokenStream {
    // clone original. since it is fully replaced by the output of this macro
    let original_trait_def: quote::__private::TokenStream = item.clone().into();

    // The number passed to the macro
    let arg = parse_macro_input!(attr as LitInt);
    let number = arg
        .base10_parse::<usize>()
        .expect("macro attribute to be a number");

    // ItemTrait AST node from original trait definition
    let ast = parse_macro_input!(item as ItemTrait);
    let trait_name = ast.ident;

    // this assumes that there is only one associated type inside the trait definition. If there
    // are multiple, the macro will only define the first one.
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

    // Generates each trait implementation up to the number passed to the macro
    let tokenstreams_iter = (1..=number)
        .map(|x| {
            // construct generics string
            let mut generics_str =
                (0..x).fold(String::from("<"), |acc, y| acc + &format!("T{y}, "));
            generics_str.push('>');

            //parse into Generics AST node
            let generics: Generics = parse_str(&generics_str).expect("generics to work");

            //construct type signature string from generics string
            let type_signature_str = generics_str
                .chars()
                .map(|c| match c {
                    '<' => '(',
                    '>' => ')',
                    _ => c,
                })
                .collect::<String>();

            //parse into TypeTuple AST node
            let type_signature: TypeTuple =
                parse_str(&type_signature_str).expect("type sig to work");

            //construct associated type string
            let mut assoc_type_str = (0..x)
                .rev()
                .fold(format!("type {associated_type_name} = "), |acc, y| {
                    acc + &format!("(T{y}, ")
                });
            assoc_type_str += &format!("(){};", repeat_n(')', x).collect::<String>());

            //parse into ImplItemType AST node
            let result_type: ImplItemType = parse_str(&assoc_type_str).expect("assoc type to work");

            // return all nodes as a single formatted proc_macro2::TokenStream
            quote! {
                impl #generics #trait_name for #type_signature {
                    #result_type
                }
            }
        })
        .collect::<Vec<_>>();

    // return original trait def, unit type impl, and all typestack impls
    quote! {
        #original_trait_def
        impl #trait_name for () {
            type #associated_type_name = ();
        }
        #(#tokenstreams_iter)*
    }
    .into()
}
