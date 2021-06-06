/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//use codemaker_python_impl as py;

use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{parse::{Parse, ParseStream}, Token, spanned::Spanned, punctuated::Punctuated};
use proc_macro2::{Span, TokenStream};

mod kw {
    syn::custom_keyword!(body);
    syn::custom_keyword!(def);
}


#[proc_macro]
pub fn quoted_rule(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as QuotedRule).tokens.into()
}

#[derive(Debug)]
struct QuotedRule {
    tokens: TokenStream,
}

impl Parse for QuotedRule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens = if input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            let ty: syn::Ident = input.parse()?;
            quote_spanned! { ty.span()=>
                ::codemaker_python::#ty
            }
        } else {
            input.parse::<kw::body>()?;
            let ty: syn::Ident = input.parse()?;
            let tokens: TokenStream = input.parse()?;
            match ty.to_string().as_str() {
                "Statement" => syn::parse2::<StatementBuilder>(tokens)?.to_token_stream(),
                "FunctionDefinition" => syn::parse2::<FunctionDefinitionBuilder>(tokens)?.to_token_stream(),
                _ => return Err(syn::Error::new_spanned(&ty, format!("unknown quoted Python syntax type: {}", ty.to_string()))),
            }
        };
        Ok( Self { tokens })
    }
}


#[derive(Debug)]
enum SingleSubstitutionPoint<T: std::fmt::Debug> {
    Quot(T),
    Sub(Span, TokenStream),
}

impl<T: Parse + std::fmt::Debug> Parse for SingleSubstitutionPoint<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            let span = input.span();
            let tokens; syn::parenthesized!(tokens in input);
            Self::Sub(span, tokens.parse()?)
        } else {
            Self::Quot(input.parse()?)
        })
    }
}

impl<T: ToTokens + std::fmt::Debug> ToTokens for SingleSubstitutionPoint<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Quot(q) => tokens.append_all(quote_spanned! {q.span()=>
                #q
            }),
            Self::Sub(span, ts) => tokens.append_all(quote_spanned! {*span=>
                #ts
            }),
        }
    }
}

#[derive(Debug)]
enum MultiSubstitutionPoint<T: std::fmt::Debug> {
    Quot(T),
    SubSingle(Span, TokenStream),
    SubMulti(Span, TokenStream),
}

impl<T: Parse + std::fmt::Debug> Parse for MultiSubstitutionPoint<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![$]) {
            input.parse::<Token![$]>()?;
            let span = input.span();
            let tokens; syn::parenthesized!(tokens in input);
            if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                Self::SubMulti(span, tokens.parse()?)
            } else {
                Self::SubSingle(span, tokens.parse()?)
            }
        } else {
            Self::Quot(input.parse()?)
        })
    }
}


impl<T: ToTokens + std::fmt::Debug> ToTokens for MultiSubstitutionPoint<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Quot(q) => tokens.append_all(quote_spanned! {q.span()=>
                .push(#q)
            }),
            Self::SubSingle(span, ts) => tokens.append_all(quote_spanned! {*span=>
                .push(#ts)
            }),
            Self::SubMulti(span, ts) => tokens.append_all(quote_spanned! {*span=>
                .extend(#ts)
            })
        }
    }
}


#[derive(Debug)]
struct Series<T: std::fmt::Debug> {
    items: Vec<T>,
}

impl<T: Parse + std::fmt::Debug> Parse for Series<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items: Vec<T> = Default::default();
        while ! input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}


impl<T: ToTokens + std::fmt::Debug> ToTokens for Series<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.items);
    }
}

#[derive(Debug)]
struct FunctionDefinitionBuilder {
    marker: kw::def,
    name: syn::Ident,
    args: Vec<syn::Ident>,
    body: Series<MultiSubstitutionPoint<StatementBuilder>>,
}


impl Parse for FunctionDefinitionBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let marker = input.parse::<kw::def>()?;
        let name: syn::Ident = input.parse()?;
        let args; syn::parenthesized!(args in input);
        let args = Punctuated::<syn::Ident, Token![,]>::parse_terminated(&args)?.into_iter().collect::<Vec<_>>();
        input.parse::<Token![:]>()?;
        let body; syn::braced!(body in input);
        let body: Series<MultiSubstitutionPoint<StatementBuilder>> = body.parse()?;
        Ok(FunctionDefinitionBuilder {
            marker,
            name,
            args,
            body,
        })
    }
}

impl ToTokens for FunctionDefinitionBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.to_string();
        let args = self.args.iter().map(|arg|arg.to_string());
        let body = &self.body;
        tokens.append_all(quote_spanned! {self.marker.span()=>
            ::codemaker_python::FunctionDefinition::new(#name)
                #(.add_arg(#args))*
                #body
        })
    }
}


#[derive(Debug)]
enum ExpressionBuilder {
    Equals(syn::Ident, Token![==], SingleSubstitutionPoint<syn::Lit>),
}


impl Parse for ExpressionBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Actually only parses `variable == literal` for now...
        let lhs = input.parse()?;
        let marker = input.parse::<Token![==]>()?;
        let rhs = input.parse()?;
        Ok(ExpressionBuilder::Equals(lhs, marker, rhs))
    }
}

impl ToTokens for ExpressionBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Equals(lhs, marker, rhs) => {
                let lhs = lhs.to_string();
                tokens.append_all(quote_spanned! {marker.span()=>
                    ::codemaker_python::Expression::new_equals(
                        ::codemaker_python::Expression::new_variable(#lhs),
                        #rhs
                    )
                })
            },
        }
    }
}


#[derive(Debug)]
struct IfElseBuilder {
    marker: Token![if],
    condition: ExpressionBuilder,
    body_if: Series<MultiSubstitutionPoint<StatementBuilder>>,
    body_else: Series<MultiSubstitutionPoint<StatementBuilder>>,
}


impl Parse for IfElseBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let marker = input.parse::<Token![if]>()?;
        let condition: ExpressionBuilder = input.parse()?;
        input.parse::<Token![:]>()?;
        let body_if; syn::braced!(body_if in input);
        let body_if = body_if.parse()?;
        let body_else = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            input.parse::<Token![:]>()?;
            let be; syn::braced!(be in input);
            be.parse()?
        } else {
            Series { items: vec![] }
        };
        Ok(IfElseBuilder {
            marker,
            condition,
            body_if,
            body_else,
        })
    }
}

impl ToTokens for IfElseBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let condition = &self.condition;
        let body_if = &self.body_if;
        let body_else = &self.body_else;
        tokens.append_all(quote_spanned! {self.marker.span()=>
            ::codemaker_python::IfElse::new(#condition)
                .with_body_if(|body__| { // TODO: hygiene?
                    body__#body_if
                })
                .with_body_else(|body__| { // TODO: hygiene?
                    body__#body_else
                })
        })
    }
}

#[derive(Debug)]
struct ReturnBuilder {
    marker: Token![return],
    value: SingleSubstitutionPoint<syn::Lit>,
}


impl Parse for ReturnBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let marker = input.parse::<Token![return]>()?;
        let value = input.parse()?;
        Ok(ReturnBuilder {
            marker,
            value,
        })
    }
}

impl ToTokens for ReturnBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value;
        tokens.append_all(quote_spanned! {self.marker.span()=>
            ::codemaker_python::Return::new(#value)
        })
    }
}



#[derive(Debug)]
enum StatementBuilder {
    FuncDef(FunctionDefinitionBuilder),
    IfElse(IfElseBuilder),
    Return(ReturnBuilder),
}

impl Parse for StatementBuilder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(kw::def) {
            Self::FuncDef(input.parse()?)
        } else if input.peek(Token![if]) {
            Self::IfElse(input.parse()?)
        } else if input.peek(Token![return]) {
            Self::Return(input.parse()?)
        } else {
            return Err(input.error("unable to parse Python statement"))
        })
    }
}

impl ToTokens for StatementBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stmt = match self {
            Self::FuncDef(f) => f.to_token_stream(),
            Self::IfElse(ie) => ie.to_token_stream(),
            Self::Return(r) => r.to_token_stream(),
        };
        tokens.append_all(quote! {
            ::std::convert::Into::<::codemaker_python::Statement>::into(#stmt)
        });
    }
}