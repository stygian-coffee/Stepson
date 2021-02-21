//use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Variant};

#[proc_macro_derive(FromRepl)]
pub fn derive_from_repl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;

    // data must be an enum; get its variants
    let variants = match input.data {
        Data::Enum(d) => d,
        _ => panic!("{} must be an enum", enum_name),
    }.variants;

    // now, turn each variant into a match arm for FromRepl
    let arms_from_repl = variants.iter().map(|v| variant_to_from_repl(&enum_name, v));

    // also, turn each variant into a match arm for ReplCompletion
    let arms_complete = variants.iter().map(|v| variant_to_repl_completion(v));

    // finally, create the impl
    let expanded = quote! {
        impl FromRepl for #enum_name {
            fn from_repl<'a, T>(words: &mut T)
                -> Result<Self, crate::repl::from_repl::ParseError> where
                T: Iterator<Item=&'a str> {
                use crate::repl::from_repl::ParseError;

                let word = match words.next() {
                    Some(w) => w,
                    None => return Err(ParseError::ExpectedArgument),
                };

                Ok(match word {
                    #(#arms_from_repl)*
                    _ => return Err(ParseError::UnknownArgument(word.to_string())),
                })
            }
        }

        impl crate::repl::ReplCompletion for #enum_name {
            fn complete<'__a, __T>(words: __T, pos: usize) -> (usize, Vec<String>) where
                __T: Iterator<Item=&'__a str> {
                //TODO
                unimplemented!()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn variant_to_from_repl(enum_name: &proc_macro2::Ident, variant: &Variant)
    -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;

    //TODO find some way to not force the existence of a variant named "Unknown"
    if variant_name == "Unknown" {
        return quote! {
            _ => {
                let mut v = vec![<u8 as std::str::FromStr>::from_str(word)?];
                v.append(&mut Vec::<u8>::from_repl(words)?);
                #enum_name::#variant_name(v)
            },
        }
    }

    // if discriminant, than no variant
    if let Some(_) = variant.discriminant {
        return quote! {
            stringify!(#variant_name) => Self::#variant_name,
        }
    }

    // otherwise, enum variant has one unnamed field; get it
    let field = match variant.fields.clone() {
        Fields::Unnamed(f) => f,
        _ => unreachable!(), //TODO is this really unreachable?
    }.unnamed.into_iter().nth(0).unwrap();
    //TODO helpful error message? check length of iter?

    //TODO helpful error message if field.ident is None
    let field_type = field.ty;

    quote! {
        stringify!(#variant_name) => #enum_name::#variant_name(#field_type::from_repl(words)?),
    }
}

fn variant_to_repl_completion(variant: &Variant) -> proc_macro2::TokenStream {
    //TODO
    let variant_name = &variant.ident;

    //TODO find some way to not force the existence of a variant named "Unknown"
    if variant_name == "Unknown" {
        return proc_macro2::TokenStream::new();
    }

    // if discriminant, then no field
    if let Some(_) = variant.discriminant {
        return quote! {
            completions.insert(stringify!(#variant_name), None);
        }
    }

    // otherwise, enum variant has one unnamed field; get it
    let field = match variant.fields.clone() {
        Fields::Unnamed(f) => f,
        _ => unreachable!(), //TODO is this really unreachable?
    }.unnamed.into_iter().nth(0).unwrap();
    //TODO helpful error message? check length of iter?

    //TODO helpful error message if field.ident is None
    let field_type = field.ty;

    quote! {
        completions.insert(stringify!(#variant_name), Some(
            #field_type::possible_completions as
            fn(__T) -> crate::repl::CompletionMap<'__a, __T>));
    }
}
