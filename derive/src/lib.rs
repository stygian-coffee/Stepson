use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(FromRepl)]
pub fn derive_from_repl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let from_repl = from_repl(input.clone());
    let repl_completion = repl_completion(input);

    proc_macro::TokenStream::from(quote! { #from_repl #repl_completion })
}

fn from_repl(input: DeriveInput) -> TokenStream {
    let type_name = input.ident;

    let inner = match input.data {
        Data::Struct(data) => from_repl_struct(data),
        Data::Enum(data) => from_repl_enum(data),
        Data::Union(_) => unimplemented!(),
    };

    quote! {
        impl FromRepl for #type_name {
            fn from_repl<'a, T>(words: &mut T)
                -> Result<Self, crate::repl::from_repl::ParseError> where
                T: Iterator<Item=&'a str> {
                #inner
            }
        }
    }
}

fn from_repl_struct(data: DataStruct) -> TokenStream {
    match data.fields {
        Fields::Named(_fields) => {
            unimplemented!()
        }
        Fields::Unnamed(fields) => {
            let mut lines = vec![];
            for f in fields.unnamed {
                let ty = f.ty;
                lines.push(quote! {
                    #ty::from_repl(&mut words.take(1))?,
                });
            }
            quote! {
                Ok(Self(
                    #(#lines)*
                ))
            }
        }
        Fields::Unit => quote! {
            vec![].into_iter()
        },
    }
}

fn from_repl_enum(data: DataEnum) -> TokenStream {
    let variants = data.variants.into_iter();

    let mut match_arms = vec![];
    for variant in variants {
        let variant_name = variant.ident;

        //TODO find some way to not force the existence of a variant named "Unknown"
        if variant_name == "Unknown" {
            match_arms.push(quote! {
                _ => {
                    let mut v = vec![<u8 as std::str::FromStr>::from_str(word)?];
                    v.append(&mut Vec::<u8>::from_repl(words)?);
                    Self::#variant_name(v) // i.e. Self::Unknown
                },
            });
            continue;
        }

        // if discriminant, then no field
        if let Some(_) = variant.discriminant {
            match_arms.push(quote! {
                stringify!(#variant_name) => Self::#variant_name,
            });
            continue;
        }

        // otherwise, variant has one unnamed field
        // TODO support multiple fields or named fields?
        let fields = match variant.fields {
            Fields::Unnamed(f) => f,
            _ => unimplemented!(),
        }
        .unnamed;
        let field = match fields.into_iter().nth(0) {
            Some(f) => f,
            None => unimplemented!(),
        };
        let ty = field.ty;

        match_arms.push(quote! {
            stringify!(#variant_name) => Self::#variant_name(#ty::from_repl(words)?),
        });
    }

    quote! {
        use crate::repl::from_repl::ParseError;

        let word = match words.next() {
            Some(w) => w,
            None => return Err(ParseError::ExpectedArgument),
        };

        Ok(match word {
            #(#match_arms)*
            _ => return Err(ParseError::UnknownArgument(word.to_string())),
        })
    }
}

fn repl_completion(input: DeriveInput) -> TokenStream {
    let type_name = input.ident;

    let inner = match input.data {
        Data::Struct(data) => repl_completion_struct(data),
        Data::Enum(data) => repl_completion_enum(data),
        Data::Union(_) => unimplemented!(),
    };

    quote! {
        impl ReplCompletion for #type_name {
            fn completion_tree() -> crate::repl::CompletionTree {
                #inner
            }
        }
    }
}

fn repl_completion_struct(data: DataStruct) -> TokenStream {
    match data.fields {
        Fields::Named(_fields) => {
            unimplemented!()
        }
        Fields::Unnamed(fields) => {
            let mut sub_tree_fns = vec![];

            let num_fields = fields.unnamed.len();
            for (i, f) in fields.unnamed.into_iter().enumerate() {
                let field_type = f.ty;
                let current_sub_tree_fn = format_ident!("sub_tree_{}", i);
                let next_sub_tree_fn = if i == num_fields - 1 {
                    quote! {
                        crate::repl::CompletionTree::lazy_empty()
                    }
                } else {
                    let next_ident = format_ident!("sub_tree_{}", i + 1);
                    quote! {
                        Box::new(#next_ident)
                    }
                };

                sub_tree_fns.push(quote! {
                    fn #current_sub_tree_fn() -> crate::repl::CompletionTree {
                        use crate::repl::CompletionTree;
                        CompletionTree {
                            branches: #field_type::lazy_completion_tree()()
                                .branches
                                .into_iter()
                                .map(|(s, _)| (s, Box::new(#next_sub_tree_fn) as _))
                                .collect(),
                        }
                    }
                });
            }

            quote! {
                #(#sub_tree_fns)*
                sub_tree_0()
            }
        }
        Fields::Unit => quote! {
            crate::repl::CompletionTree::lazy_empty()
        },
    }
}

fn repl_completion_enum(data: DataEnum) -> TokenStream {
    let variants = data.variants.into_iter();

    let mut lines = vec![];
    for variant in variants {
        let variant_name = variant.ident;

        //TODO find some way to not force the existence of a variant named "Unknown"
        if variant_name == "Unknown" {
            continue;
        }

        // if discriminant, then no field
        if let Some(_) = variant.discriminant {
            lines.push(quote! {
                (stringify!(#variant_name).to_string(), crate::repl::CompletionTree::lazy_empty()),
            });
            continue;
        }

        // otherwise, variant has one unnamed field
        // TODO support multiple fields or named fields?
        let fields = match variant.fields {
            Fields::Unnamed(f) => f,
            _ => unimplemented!(),
        }
        .unnamed;
        let field = match fields.into_iter().nth(0) {
            Some(f) => f,
            None => unimplemented!(),
        };
        let ty = field.ty;

        lines.push(quote! {
            (
                stringify!(#variant_name).to_string(),
                <#ty as crate::repl::ReplCompletion>::lazy_completion_tree()
            ),
        });
    }

    quote! {
        crate::repl::CompletionTree::new(vec![
            #(#lines)*
        ])
    }
}
