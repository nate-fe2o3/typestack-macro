use std::iter::repeat_n;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Generics, ImplItemType, ItemTrait, LitInt, TraitItem, TypeTuple, parse_macro_input, parse_str,
};

/// Provides blanket trait implementations for any tuple type with a number of values up to and including
/// the number argument provided. These impl blocks contain an associated type that is the TypeStack
/// representation of the tuple type.
///
/// # Example:
/// ```
/// #[tuple_to_typestack(2)]
/// pub trait SomeTrait {
///     type Assoc;
/// }
/// ```
/// # Expanded Macro Output:
/// ```
/// pub trait SomeTrait {
///     type Assoc;
/// }
/// impl SomeTrait for () {
///     type Assoc = ();
/// }
/// impl<T0> SomeTrait for (T0,) {
///     type Assoc = (T0, ());
/// }
/// impl<T0, T1> SomeTrait for (T0, T1) {
///     type Assoc = (T1, (T0, ()));
/// }
/// ```
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
    // are none or multiple, the macro will throw a compiler error
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
    let tokenstreams_iter = (0..=number).map(|num_types| {
        let types = (0..num_types)
            .map(|e| format!("T{e}, "))
            .collect::<Vec<_>>();
        let types_string = types.iter().cloned().collect::<String>();

        //construct generic string, parse into Generics AST node
        let generics_str = format!("<{types_string}>");
        let generics: Generics = parse_str(&generics_str).expect("generics to work");

        //construct type sig string, parse into TypeTuple AST node
        let type_signature_str = format!("({types_string})");
        let type_signature: TypeTuple = parse_str(&type_signature_str).expect("type sig to work");

        //construct associated type string
        let mut assoc_type_str = types
            .iter()
            .rev()
            .fold(format!("type {associated_type_name} = "), |acc, t| {
                acc + &format!("({t}")
            });
        assoc_type_str += &format!("(){};", repeat_n(')', num_types).collect::<String>());

        //parse into ImplItemType AST node
        let result_type: ImplItemType = parse_str(&assoc_type_str).expect("assoc type to work");

        // return all nodes as a single formatted proc_macro2::TokenStream
        quote! {
            impl #generics #trait_name for #type_signature {
                #result_type
            }
        }
    });

    // return original trait def and all typestack impls
    quote! {
        #original_trait_def
        #(#tokenstreams_iter)*
    }
    .into()
}
