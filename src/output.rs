use crate::{BinOp, Function, Output, UnOp, Value};

use std::str::FromStr;

use proc_macro::TokenStream;
use syn::{
    parenthesized, parse,
    parse::{Parse, ParseStream},
    Ident, LitBool, LitFloat, LitInt, LitStr, Result, Token,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(boolean);
    custom_keyword!(ceiling);
    custom_keyword!(concat);
    custom_keyword!(floor);
    custom_keyword!(integer);
    custom_keyword!(real);
    custom_keyword!(round);
    custom_keyword!(string);
}

impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let operator = if input.peek(Token![+]) {
            BinOp::Add
        } else if input.peek(Token![-]) {
            BinOp::Sub
        } else if input.peek(Token![*]) {
            BinOp::Mul
        } else if input.peek(Token![/]) {
            BinOp::Div
        } else if input.peek(Token![%]) {
            BinOp::Mod
        } else if input.peek(Token![==]) {
            BinOp::Eq
        } else if input.peek(Token![!=]) {
            BinOp::Ne
        } else if input.peek(Token![>]) {
            BinOp::Gt
        } else if input.peek(Token![>=]) {
            BinOp::Ge
        } else if input.peek(Token![<]) {
            BinOp::Lt
        } else if input.peek(Token![<=]) {
            BinOp::Le
        } else {
            let message = format!("expected a binary operator, found '{input}'");
            Err(input.error(message))?
        };

        input.parse::<syn::BinOp>()?;

        Ok(operator)
    }
}

fn primary(input: ParseStream) -> Result<Output> {
    if input.peek(Ident) {
        let ident: Ident = input.parse()?;
        Ok(Output::Identifier(ident))
    } else if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        Ok(Output::Literal(Value::String(lit.value())))
    } else if input.peek(LitBool) {
        let lit: LitBool = input.parse()?;
        Ok(Output::Literal(Value::Boolean(lit.value)))
    } else if input.peek(LitInt) {
        let lit: LitInt = input.parse()?;
        Ok(Output::Literal(Value::Integer(
            lit.base10_parse().unwrap_or_else(|err| panic!("{err}")),
        )))
    } else if input.peek(LitFloat) {
        let lit: LitFloat = input.parse()?;
        Ok(Output::Literal(Value::Real(
            lit.base10_parse().unwrap_or_else(|err| panic!("{err}")),
        )))
    } else {
        let message = format!("expected a literal or an identifier, found '{input}'");
        Err(input.error(message))
    }
}

fn arg(input: ParseStream) -> Result<Output> {
    let content;
    parenthesized!(content in input);
    content.parse()
}

fn args(input: ParseStream) -> Result<(Output, Output)> {
    let content;
    parenthesized!(content in input);
    let arg1 = content.parse()?;
    content.parse::<Token![,]>()?;
    let arg2 = content.parse()?;
    Ok((arg1, arg2))
}

fn function(input: ParseStream) -> Result<Output> {
    if input.peek(kw::boolean) {
        input.parse::<kw::boolean>()?;
        Ok(Output::Function(Function::Boolean(Box::new(arg(input)?))))
    } else if input.peek(kw::ceiling) {
        input.parse::<kw::ceiling>()?;
        Ok(Output::Function(Function::Ceiling(Box::new(arg(input)?))))
    } else if input.peek(kw::concat) {
        input.parse::<kw::concat>()?;
        let (arg1, arg2) = args(input)?;
        Ok(Output::Function(Function::Concat(
            Box::new(arg1),
            Box::new(arg2),
        )))
    } else if input.peek(kw::floor) {
        input.parse::<kw::floor>()?;
        Ok(Output::Function(Function::Floor(Box::new(arg(input)?))))
    } else if input.peek(kw::integer) {
        input.parse::<kw::integer>()?;
        Ok(Output::Function(Function::Integer(Box::new(arg(input)?))))
    } else if input.peek(kw::real) {
        input.parse::<kw::real>()?;
        Ok(Output::Function(Function::Real(Box::new(arg(input)?))))
    } else if input.peek(kw::round) {
        input.parse::<kw::round>()?;
        Ok(Output::Function(Function::Round(Box::new(arg(input)?))))
    } else if input.peek(kw::string) {
        input.parse::<kw::string>()?;
        Ok(Output::Function(Function::String(Box::new(arg(input)?))))
    } else {
        primary(input)
    }
}

fn unary(input: ParseStream) -> Result<Output> {
    if input.peek(Token![-]) {
        input.parse::<Token![-]>()?;
        let right = Box::new(unary(input)?);
        Ok(Output::Unary {
            operator: UnOp::Negate,
            right,
        })
    } else if input.peek(Token![!]) {
        input.parse::<Token![-]>()?;
        let right = Box::new(unary(input)?);
        Ok(Output::Unary {
            operator: UnOp::Not,
            right,
        })
    } else {
        function(input)
    }
}

fn binary(input: ParseStream) -> Result<Output> {
    let mut output = unary(input)?;

    while input.peek(Token![+])
        || input.peek(Token![-])
        || input.peek(Token![*])
        || input.peek(Token![/])
        || input.peek(Token![%])
    {
        let operator: BinOp = input.parse()?;
        let right = Box::new(unary(input)?);
        output = Output::Binary {
            left: Box::new(output),
            operator,
            right,
        };
    }

    Ok(output)
}

fn comparison(input: ParseStream) -> Result<Output> {
    let mut output = binary(input)?;

    while input.peek(Token![==])
        || input.peek(Token![!=])
        || input.peek(Token![<])
        || input.peek(Token![<=])
        || input.peek(Token![>])
        || input.peek(Token![>=])
    {
        let operator: BinOp = input.parse()?;
        let right = Box::new(binary(input)?);
        output = Output::Binary {
            left: Box::new(output),
            operator,
            right,
        };
    }

    Ok(output)
}

impl Parse for Output {
    fn parse(input: ParseStream) -> Result<Self> {
        comparison(input)
    }
}

pub(crate) fn parse_output(input: &str) -> Output {
    let tokens = match TokenStream::from_str(input) {
        Ok(stream) => stream,
        Err(e) => panic!("{e}"),
    };

    match parse(tokens) {
        Ok(output) => output,
        Err(e) => panic!("{e}"),
    }
}
