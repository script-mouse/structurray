/*
Copyright 2024 Benjamin Richcreek

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
//! # Psuedo-Array Generation
//! This crate allows for the generation of [`struct`]s with an arbitrary, programmer-provided number (less than [`u32::MAX`]) of identical fields with different names. 
//! Generally speaking, it is also useful to use [`structinator`](https://crates.io/crates/structinator)
//! on [`struct`]s of this size to allow them to be automatically constructed from [`Iterator`]s.
//!
//! Psuedo-Array [`struct`]s like this are ideal for reducing data spent on identifiers in online databases like [Google Firebase](https://firebase.google.com).
//!
//! [`struct`]: https://doc.rust-lang.org/1.58.1/std/keyword.struct.html
//! 
//! To learn more about what this crate does, look at the documentation for this crates only public attribute, [`macro@faux_array`].
//! 
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn;
use syn::{Type,parse,ItemStruct,Ident,Token};
use syn::token::Pound;
use syn::parse::{Parse,ParseStream};
use std::str::FromStr;
use quote::quote;
use ascii_basing::encoding::encode;
const ARGUMENT_ERROR_MESSAGE: &'static str = "The faux_array attribute should be given two arguments, the first of which should be a type and the second should be an integer";
struct Arguments {
    field_count: u32,
    field_type: Type,
}
impl Parse for Arguments {
    fn parse(input: ParseStream) -> Result<Self,syn::Error> {
        let inner_type: Type = input.parse()?;
        Ok(Arguments {
            field_count: 0,
            field_type: inner_type,
        })
    }
}
#[proc_macro_attribute]
/// Converts your [`struct`] to a psuedo-array
///
/// # Requirements
/// This attribute must be attached to the definition of a [`struct`] that implements [serde::Serialize](https://docs.rs/serde/latest/serde). [`Serialize`] must be implemented because all fields will be `rename`d to their identifier with the leading underscore removed.
/// This is because the intended use case of creating such a long [`struct`] is to save storage space in online databases, so [`struct`]s with this attribute should already have implemented [`Serialize`]. In a later version of this
/// library, a third [`bool`] argument will allow for this attribute to apply to [`struct`]s not implementing [`Serialize`]. If you have a use case where it's ideal to have the option to attach this attribute to a [`struct`] not
/// implementing [`Serialize`], feel free to look at this crate's [Github repository](https://github.com/script-mouse/structurray) and contribute or simply open an issue to let me know that there is demand for such a use case. Note that in order to derive [`Serialize`] on a
/// user-defined [`struct`], as shown here, requires use of `derive` features from [`serde`](https://docs.rs/serde/latest/serde).
///
/// # Arguments
/// This attribute macro should be invoked with two arguments. The first argument should be a type, such as [`u8`] or [`String`]. The second argument should be an [integer](u32) literal.
/// # Example
/// Let's imagine you needed to make a [`struct`] with 3 identical fields. If you were feeling particularly lazy that day, you could use this library to quickly generate all the fields you needed. This snippet:
/// ```
/// # use structurray::faux_array;
/// # use serde::Serialize;
/// 
/// #[faux_array(T,3)]
/// #[derive(Serialize)]
/// struct Lazyrray<T> {}
/// ```
/// Expands to this:
/// ```
/// # use structurray::faux_array;
/// # use serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct Lazyrray<T> {
///     #[serde(rename = "0")]
///     _0: T,
///     #[serde(rename = "1")]
///     _1: T,
///     #[serde(rename = "2")]
///     _2: T,
/// }
/// ```
/// Of course, this is a rather trivial example, but this attribute can be quite useful when creating longer pseudo-arrays.
/// # Identifier Generation
/// Identifiers are generated using a [Base62](https://en.wikipedia.org/wiki/Base62) algorithm described in detail in the documentation of [`ascii_basing`](https://docs.rs/ascii_basing/latest/ascii_basing).
/// The algorithm uses the following 62 characters, in order from least value (0 = 0) to greatest value (Z = 61):
/// ```no_run
/// # /*
/// 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
/// # */
/// ```
/// # Panics
/// Panics if the arguments are out of order or formatted incorrectly (most common cause of incorrect formatting is missing a comma). Panics if the first type can't be parsed to a type. Panics if the second argument cannot be parsed and stored in a [`u32`]. Panics if 
/// the [`struct`] this attribute is attached to does not implement [`Serialize`].
///
/// [`struct`]: https://doc.rust-lang.org/1.58.1/std/keyword.struct.html
/// [`Serialize`]: https://docs.rs/serde/latest/serde
pub fn faux_array(args: TokenStream, actual: TokenStream) -> TokenStream {
    let string_holder = args.to_string();
    let mut string_args = string_holder.split(',');
    let first_string = string_args.next().expect(format!("{}. No arguments were found",ARGUMENT_ERROR_MESSAGE).as_str());
    let mut arguments: Arguments = parse(TokenStream::from_str(first_string).expect("The arguments given could not be converted back to a TokenStream after being converted to a String. Make sure your arguments list is also a valid Rust String and TokenStream")).expect(format!("{}. The first argument was {} , which could not be converted to a type",ARGUMENT_ERROR_MESSAGE,first_string).as_str());
    arguments.field_count = string_args.next().expect(format!("{}. Only one argument was found",ARGUMENT_ERROR_MESSAGE).as_str()).trim().parse().expect(format!("{}. The second argument could not be parsed to a u32. Make sure the second argument is an integer that can be stored in a u32",ARGUMENT_ERROR_MESSAGE).as_str());
    let build_length = usize::try_from(arguments.field_count).expect(format!("{}. The second argument was successfully parsed to a u32, but failed conversion to a usize integer. Make sure the second argument is less than or equal to {}",ARGUMENT_ERROR_MESSAGE,usize::MAX).as_str());
    let structure: ItemStruct = parse(actual).expect("The faux_array attribute should only be attached to struct definitions");
    let attributes = &structure.attrs;
    let visibility = &structure.vis;
    let name = &structure.ident;
    let generics = &structure.generics;
    let tipe = arguments.field_type;
    let mut names: Vec<String> = Vec::with_capacity(build_length);
    let hashtag: Pound = Token![#](Span::call_site());
    let mut idents: Vec<Ident> = Vec::with_capacity(build_length);
    let mut copyscore = String::with_capacity(7);
    let mut looper: u32 = 0;
    while looper < arguments.field_count {
        copyscore.push('_');
        let new_name = encode(looper,None).expect("An unexpected error occurred. Please try again. If the error persists, contact me at richcreekbenjamin@gmail.com with a description of what is causing the bug");
        copyscore.push_str(new_name.as_str());
        names.push(new_name);
        idents.push(Ident::new(&copyscore,Span::call_site()));
        looper += 1;
        copyscore.clear();
    }
    quote! {
        #(#attributes)*
        #visibility struct #name #generics {
            #(#hashtag[serde(rename = #names)]
            #idents : #tipe),*
        }

    }.into()
}