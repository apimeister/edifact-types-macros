extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, PathArguments};

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
    match x {
        Data::Struct(s) => {
            let f = &s.fields;
            for o in f {
                let id = o.ident.clone().unwrap();
                let t = &o.ty;
                match t {
                    syn::Type::Path(p) => {
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
                    _ => {}
                }
            }
        }
        _ => {}
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
    match x {
        Data::Struct(s) => {
            let f = &s.fields;
            for o in f {
                let id = o.ident.clone().unwrap();
                let t = &o.ty;
                match t {
                    syn::Type::Path(p) => {
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
                    _ => {}
                }
                idx += 1;
            }
        }
        _ => {}
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
    match x {
        Data::Struct(s) => {
            let f = &s.fields;
            for o in f {
                let id = o.ident.clone().unwrap();
                let t = &o.ty;
                match t {
                    syn::Type::Path(p) => {
                        let s = &p.path.segments;
                        let we = s.first().unwrap();
                        let w = &we.ident;
                        let arg = &we.arguments;
                        let sub_id = &we.ident;
                        let ty = w.to_string();
                        match ty.as_str() {
                            "Option" => {
                                // look for type inside
                                match arg {
                                    PathArguments::AngleBracketed(c) =>{
                                        let o = c.args.first().unwrap();
                                        match o {
                                            syn::GenericArgument::Type(po) => {
                                                match po {
                                                    syn::Type::Path(tpo) => {
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
                                                            },
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
                                                    },
                                                    _ => {},
                                                }
                                            },
                                            _ =>{}
                                        }
                                    }
                                    _ =>{}
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
                    _ => {}
                }
                idx += 1;
            }
        }
        _ => {}
    }
    output
}