extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

#[proc_macro_derive(DisplayInnerSegment)]
pub fn display_inner(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_inner_display(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

#[proc_macro_derive(DisplayOuterSegment)]
pub fn display_outer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_outer_display(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

#[proc_macro_derive(DisplayEdifact)]
pub fn display_edifact(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_edifact(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

#[proc_macro_derive(DisplayEdifactSg)]
pub fn display_edifact_sg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_edifact_sg(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

fn generate_inner_display(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_types(ast);
    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt<'x>(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut str: Vec<String> = vec![];
                #(#output)*
                let joined = str.join(":");
                let joined = joined.trim_end_matches(":");
                write!(f, "{}", joined)
            }
        }
    })
}

fn generate_outer_display(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_types(ast);
    let s = format_ident!("{}", name).to_string().to_uppercase();
    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt<'x>(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut str: Vec<String> = vec![];
                str.push(#s.to_string());
                #(#output)*
                let joined = str.join("+");
                let joined = joined.trim_end_matches("+");
                if joined.len() > 3 {
                    write!(f, "{}", joined)
                }else{
                    write!(f, "")
                }
            }
        }
    })
}

fn generate_edifact(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_types(ast);
    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt<'x>(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut str: Vec<String> = vec![];
                #(#output)*
                // filter empty lines
                let str: Vec<String> = str
                    .iter()
                    .map(|v| v.clone())
                    .filter(|s| !s.is_empty())
                    .collect();
                let joined = str.join("'\n");
                write!(f, "{}'", joined)
            }
        }
    })
}

fn generate_edifact_sg(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_types(ast);
    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt<'x>(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut str: Vec<String> = vec![];
                #(#output)*
                // filter empty lines
                let str: Vec<String> = str
                    .iter()
                    .map(|v| v.clone())
                    .filter(|s| !s.is_empty())
                    .collect();
                let joined = str.join("'\n");
                write!(f, "{}", joined)
            }
        }
    })
}

fn gen_types(ast: &DeriveInput) -> Vec<TokenStream> {
    let x = &ast.data;
    let mut output = vec![];
    if let Data::Struct(s) = x {
        let f = &s.fields;
        for o in f {
            let id = o.ident.clone().unwrap();
            let t = &o.ty;
            if let syn::Type::Path(p) = t {
                let s = &p.path.segments;
                let w = &s.first().unwrap().ident;
                let ty = w.to_string();
                match ty.as_str() {
                    "Vec" => {
                        let ts = quote! {
                            if self.#id.is_empty() {
                                str.push("".to_string());
                            }else{
                                self.#id.iter().for_each(|x| str.push(format!("{}",x)));
                            }
                        };
                        output.push(ts);
                    }
                    "Option" => {
                        let ts = quote! {
                            str.push(self.#id.as_ref().map_or("".to_string(),|x| format!("{}",x)));
                        };
                        output.push(ts);
                    }
                    _ => {
                        let ts = quote! {
                            str.push(format!("{}",self.#id));
                        };
                        output.push(ts);
                    }
                }
            }
        }
    }
    output
}

#[proc_macro_derive(ParseInnerSegment)]
pub fn parse_inner(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_inner_parse(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

fn generate_inner_parse(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_inner_props(ast);
    let prop_count = output.len();
    Ok(quote! {
        impl #impl_generics ::core::str::FromStr for #name #ty_generics #where_clause {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() > #prop_count {
                    Err(ParseError {
                        msg: "too many segments".to_string(),
                    })
                } else {
                    Ok(#name {
                        #(#output)*
                    })
                }
            }
        }
    })
}

fn gen_inner_props(ast: &DeriveInput) -> Vec<TokenStream> {
    let x = &ast.data;
    let mut output = vec![];
    let mut idx: usize = 0;
    if let Data::Struct(s) = x {
        let f = &s.fields;
        for o in f {
            let id = o.ident.clone().unwrap();
            let t = &o.ty;
            if let syn::Type::Path(p) = t {
                let s = &p.path.segments;
                let w = &s.first().unwrap().ident;
                let ty = w.to_string();
                match ty.as_str() {
                    "Option" => {
                        let chunk = quote! {
                            #id: parts.get(#idx).map(|x| x.to_string()),
                        };
                        output.push(chunk);
                    }
                    _ => {
                        let chunk = quote! {
                            #id: parts.get(#idx).map(|x| x.to_string()).unwrap_or_default(),
                        };
                        output.push(chunk);
                    }
                }
            }
            idx += 1;
        }
    }
    output
}

#[proc_macro_derive(ParseElement)]
pub fn parse_element(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = generate_element_parser(&input).unwrap_or_else(|err| err.to_compile_error());
    #[cfg(feature = "debug")]
    println!("{output}");
    proc_macro::TokenStream::from(output)
}

fn generate_element_parser(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let tok = parse_all(ast);
    let s = format_ident!("{}", name).to_string().to_uppercase();
    let res = quote! {
        impl<'a> crate::util::Parser<&'a str, #name, nom::error::Error<&'a str>> for #name {
            fn parse(input: &'a str) -> ::nom::IResult<&'a str, #name> {
                #[cfg(feature = "logging")]
                log::debug!("Parser is inside {}", #s);
                let (_, vars) = crate::util::parse_colon_section(input)?;
                #[cfg(feature = "logging")]
                log::debug!("Variables created {vars:?}");
                let output = #name {
                    #(#tok)*
                };
                Ok(("", output))
            }
        }
    };
    Ok(res)
}

#[proc_macro_derive(ParseSg)]
pub fn parse_sg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = generate_sg_parser(&input, true).unwrap_or_else(|err| err.to_compile_error());
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(ParseMsg)]
pub fn parse_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = generate_sg_parser(&input, false).unwrap_or_else(|err| err.to_compile_error());
    #[cfg(feature = "debug")]
    println!("{output}");
    proc_macro::TokenStream::from(output)
}

fn generate_sg_parser(ast: &DeriveInput, is_sg: bool) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let mut lefties = vec![];
    let mut attries = vec![];
    if let Data::Struct(left_vec) = &ast.data {
        if let Fields::Named(f) = &left_vec.fields {
            for (idx, ff) in (&f.named).into_iter().enumerate() {
                let left = match &ff.ident {
                    Some(l) => l.clone(),
                    None => Ident::new("", Span::call_site()),
                };

                // right side can be any of String, Struct, Enum, Option<..>, Vec<..>

                let all = if let Type::Path(tyty) = &ff.ty {
                    let inside = &tyty.path.segments[0];
                    let inside_str = inside.ident.to_string();
                    // First occurence, can be Option, Vec or Type
                    match inside_str.as_str() {
                        "Vec" | "Option" => {
                            if let PathArguments::AngleBracketed(inside_optvec) = &inside.arguments
                            {
                                if let GenericArgument::Type(Type::Path(t)) = &inside_optvec.args[0]
                                {
                                    let ti = &t.path.segments[0].ident;
                                    (
                                        quote! {
                                            #left
                                        },
                                        if inside_str == "Vec" {
                                            // if we are in msg_type stage, or if vec is the first element of struct
                                            // many0 will come back positive and the surroung element will not collaps
                                            if !is_sg || idx != 0 {
                                                quote! {
                                                // let (outer_rest, dtm) = many0(DTM::parse)(outer_rest)?;
                                                    let (outer_rest, #left) = nom::multi::many0(#ti::parse)(outer_rest)?;
                                                }
                                            } else {
                                                quote! {
                                                    let (outer_rest, #left) = nom::multi::many1(#ti::parse)(outer_rest)?;
                                                }
                                            }
                                        } else {
                                            quote! {
                                                let (outer_rest, #left) = nom::combinator::opt(#ti::parse)(outer_rest)?;
                                            }
                                        },
                                    )
                                } else {
                                    (quote! {}, quote! {})
                                }
                            } else {
                                (quote! {}, quote! {})
                            }
                        }
                        _ => {
                            let i = inside.ident.clone();
                            (
                                quote! {
                                    #left
                                },
                                quote! {
                                    // let (outer_rest, loc) = LOC::parse(input)?;
                                    let (outer_rest, #left) = #i::parse(outer_rest)?;
                                },
                            )
                        }
                    }
                } else {
                    (quote! {}, quote! {})
                };
                lefties.push(all.0);
                attries.push(all.1);
            }
        }
    };
    let s = format_ident!("{}", name).to_string().to_uppercase();
    let res = quote! {
        // impl<'a> Parser<&'a str, IftminSg1, nom::error::Error<&'a str>> for IftminSg1 {
        //     fn parse(input: &'a str) -> IResult<&'a str, IftminSg1> {
        //         let (outer_rest, loc) = LOC::parse(input)?;
        //         let (outer_rest, dtm) = many0(DTM::parse)(outer_rest)?;
        //         Ok((outer_rest, IftminSg1 { loc, dtm }))
        //     }
        // }
        impl<'a> crate::util::Parser<&'a str, #name, nom::error::Error<&'a str>> for #name {
            fn parse(input: &'a str) -> ::nom::IResult<&'a str, #name> {
                #[cfg(feature = "logging")]
                log::debug!("Parser is inside {}", #s);
                let outer_rest = input;
                #(#attries)*
                Ok((outer_rest, #name { #(#lefties),* }))
            }
        }
    };
    #[cfg(feature = "debug")]
    println!("{res}");
    Ok(res)
}

// impl<'a> Parser<&'a str, C002, nom::error::Error<&'a str>> for C002 {
//     fn parse(input: &'a str) -> IResult<&'a str, C002> {
//         let (_, vars) = crate::util::parse_colon_section(input)?;
//         let output = C002 {
//             _010: vars.first().map(|x| _1001::from_str(clean_num(x)).unwrap()),
//             _020: vars.get(1).map(|x| _1131::from_str(clean_num(x)).unwrap()),
//             _030: vars.get(2).map(|x| _3055::from_str(clean_num(x)).unwrap()),
//             _040: vars.get(3).map(|x| x.to_string()),
//         };
//         Ok(("", output))
//     }
// }

#[proc_macro_derive(ParseSegment)]
pub fn parse_segment(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = generate_segment_parser(&input).unwrap_or_else(|err| err.to_compile_error());
    #[cfg(feature = "debug")]
    println!("{output}");
    proc_macro::TokenStream::from(output)
}

// impl<'a> Parser<&'a str, COM, nom::error::Error<&'a str>> for COM {
//     fn parse(input: &'a str) -> IResult<&'a str, COM> {
//         let (output_rest, vars) = crate::util::parse_line(input, "COM")?;
//         let output = COM {
//             _010: vars.first().map(|x| C076::parse(x).unwrap().1).unwrap(),
//         };
//         Ok((output_rest, output))
//     }
// }

fn generate_segment_parser(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let tok = parse_all(ast);
    let s = format_ident!("{}", name).to_string().to_uppercase();
    let res = quote! {
        impl<'a> crate::util::Parser<&'a str, #name, nom::error::Error<&'a str>> for #name {
            fn parse(input: &'a str) -> ::nom::IResult<&'a str, #name> {
                #[cfg(feature = "logging")]
                log::debug!("Parser is inside {}", #s);
                let (output_rest, vars) = crate::util::parse_line(input, #s)?;
                #[cfg(feature = "logging")]
                log::debug!("Variables created {vars:?}");
                #[cfg(feature = "logging")]
                log::debug!("Left over string {output_rest:?}");
                let output = #name {
                    #(#tok)*
                };
                Ok((output_rest, output))
            }
        }
    };
    Ok(res)
}

fn parse_all(ast: &DeriveInput) -> Vec<TokenStream> {
    let x = &ast.data;
    let name = format!("{}",&ast.ident);
    let mut output = vec![];
    if let Data::Struct(s) = x {
        let f = &s.fields;
        for (idx, o) in f.into_iter().enumerate() {
            // _010, _020, etc
            let struct_field = o.ident.clone().unwrap();
            let sf_string = struct_field.to_string();
            let syn::Type::Path(tp) = &o.ty else {
                panic!("Path type not found!")
            };
            let s = tp.path.segments.first().unwrap();
            let opt_vec = s.ident.clone();
            let ov_string = opt_vec.to_string();
            match opt_vec.to_string().as_str() {
                "Option" | "Vec" => {
                    // List, String, Segment inside option or vec
                    let mut inside_opt_vec: Ident = Ident::new("placeholder", Span::call_site());
                    if let PathArguments::AngleBracketed(abga) = &s.arguments {
                        if let GenericArgument::Type(syn::Type::Path(tp)) =
                            abga.args.first().unwrap()
                        {
                            inside_opt_vec = tp.path.segments.first().unwrap().ident.clone();
                        }
                    }
                    let iov_string = inside_opt_vec.to_string();

                    // Can be String, _XXX (List), or CXXX,SXXX (Segment)
                    if inside_opt_vec == "String" {
                        output.push(quote! {
                            #struct_field: vars.get(#idx).map(|x| x.to_string()),
                        });
                    } else if inside_opt_vec.to_string().starts_with('_') {
                        // List (types.rs)
                        output.push(quote! {
                                #struct_field: vars.get(#idx).filter(|&f| !f.is_empty()).map(|x| match #inside_opt_vec::from_str(clean_num(x)) {
                                    Ok(f) => f,
                                    Err(e) => {
                                        #[cfg(feature = "logging")]
                                        log::error!("Line: {input}\nFor struct {}, parsing optional list item {} failed. Enum {} encountered the following error: {}", #name, #sf_string, #iov_string, e);
                                        panic!("Parsing optional list item failed");
                                    },
                                }),
                            });
                    } else {
                        // Segment or Element
                        output.push(quote! {
                                #struct_field: vars.get(#idx).filter(|&f| !f.is_empty()).map(|x| match #inside_opt_vec::parse(x) {
                                    Ok((_,r)) => r,
                                    Err(e) => {
                                        #[cfg(feature = "logging")]
                                        log::error!("Line: {input}\nFor struct {}, parsing optional segment or element {} failed. Struct {} encountered the following error: {}", #name, #sf_string, #iov_string, e);
                                        panic!("Parsing optional segment or element failed");
                                    },
                                }),
                            });
                    }
                }
                "String" => {
                    output.push(quote! {
                        #struct_field: match vars.get(#idx).filter(|&f| !f.is_empty()).map(|x| x.to_string()) {
                            Some(f) => f,
                            None => {
                                #[cfg(feature = "logging")]
                                log::error!("Line: {input}\nFor struct {}, parsing mandatory {}.to_string() was not found", #name, #sf_string);
                                panic!("Parsing mandatory to_string() failed");
                        },
                        },
                    });
                }
                _ => {
                    // Can be _XXX (List), or CXXX,SXXX (Segment)
                    if opt_vec.to_string().starts_with('_') {
                        // List (types.rs)
                        output.push(quote! {
                            #struct_field: vars.get(#idx).filter(|&f| !f.is_empty()).map(|x|match #opt_vec::from_str(clean_num(x)){
                                Ok(f) => f,
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    log::error!("Line: {input}\nFor struct {}, parsing list item {} failed. Enum {} encountered the following error: {}", #name, #sf_string, #ov_string, e);
                                    panic!("Parsing list item failed");
                                },
                            }).expect("Parsing List: not found"),
                        });
                    } else {
                        // Segment or Element
                        output.push(quote! {
                            #struct_field: vars.get(#idx).filter(|&f| !f.is_empty()).map(|x| match #opt_vec::parse(x) {
                                Ok((_,r)) => r,
                                Err(e) => {
                                    #[cfg(feature = "logging")]
                                    log::error!("Line: {input}\nFor struct {}, parsing segment or element {} failed. Struct {} encountered the following error: {}", #name, #sf_string, #ov_string, e);
                                    panic!("Parsing list item failed");
                                },
                            }).expect("Parsing Segement or Element: not found"),
                        });
                    }
                }
            }
        }
    }
    output
}

#[proc_macro_derive(ParseOuterSegment)]
pub fn parse_outer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let toks = generate_outer_parse(&ast).unwrap_or_else(|err| err.to_compile_error());
    toks.into()
}

fn generate_outer_parse(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let output = gen_outer_props(ast);
    let prop_count = output.len();
    let upper_name = format_ident!("{}", name).to_string().to_uppercase();
    Ok(quote! {
        impl #impl_generics ::core::str::FromStr for #name #ty_generics #where_clause {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let x = s.trim_end_matches('\'');
                let parts: Vec<&str> = x.split('+').collect();
                if parts[0] == #upper_name {
                    if parts.len() > #prop_count +1  {
                        Err(ParseError {
                            msg: "too many segments".to_string(),
                        })
                    } else {
                        let mut obj = #name::default();
                        #(#output)*
                        Ok(obj)
                    }
                } else {
                    Err(ParseError {
                        msg: "segment name wrong".to_string(),
                    })
                }
            }
        }
    })
}

fn gen_outer_props(ast: &DeriveInput) -> Vec<TokenStream> {
    let x = &ast.data;
    let mut output = vec![];
    let mut idx: usize = 1;
    if let Data::Struct(s) = x {
        let f = &s.fields;
        for o in f {
            let id = o.ident.clone().unwrap();
            let t = &o.ty;
            if let syn::Type::Path(p) = t {
                let s = &p.path.segments;
                let we = s.first().unwrap();
                let w = &we.ident;
                let arg = &we.arguments;
                let sub_id = &we.ident;
                let ty = w.to_string();
                match ty.as_str() {
                    "Option" => {
                        // look for type inside
                        if let PathArguments::AngleBracketed(c) = arg {
                            let o = c.args.first().unwrap();
                            if let syn::GenericArgument::Type(syn::Type::Path(tpo)) = o {
                                let sg = tpo.path.segments.first().unwrap();
                                let ident = &sg.ident;
                                let type_name = ident.to_string();
                                match type_name.as_str() {
                                    "String" => {
                                        let chunk = quote! {
                                            if let Some(val) = parts.get(#idx) {
                                                obj.#id = Some(val.to_string());
                                            }
                                        };
                                        output.push(chunk);
                                    }
                                    _ => {
                                        let chunk = quote! {
                                            if let Some(val) = parts.get(#idx) {
                                                let t = #ident::from_str(val).unwrap();
                                                obj.#id = Some(t);
                                            }
                                        };
                                        output.push(chunk);
                                    }
                                }
                            }
                        }
                    }
                    "String" => {
                        let chunk = quote! {
                            if let Some(val) = parts.get(#idx) {
                                obj.#id = val.to_string();
                            }
                        };
                        output.push(chunk);
                    }
                    _ => {
                        let chunk = quote! {
                            if let Some(val) = parts.get(#idx) {
                                obj.#id = #sub_id::from_str(val).unwrap();
                            }
                        };
                        output.push(chunk);
                    }
                }
            }
            idx += 1;
        }
    }
    output
}
