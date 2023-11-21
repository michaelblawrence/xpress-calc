use std::{fmt::Display, str::FromStr};

use crate::parser;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LiteralNum(f64),
    Plus,
    Sub,
    Mul,
    Div,
    Sine,
    Cosine,
    Log,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    Pow,
    Mod,
    Rand,
    Identifier(String),
    Let,
    LeftArrow,
    Equals,
    Pi,
    E,
    Sqrt,
    Comma,
    Semicolon,
}

pub fn tokenize<'a>(source: parser::Bite<'a>) -> impl Iterator<Item = Result<Token, String>> + 'a {
    let mut bite = source;
    let mut done = false;

    let mut closure_stack = vec![];
    let mut closure_stack_iter = None;

    std::iter::from_fn(move || {
        bite = bite.chomp(parser::Chomp::whitespace());

        let has_next = !bite.is_empty() && !done;
        if !has_next {
            // once token stream has ended append any missing open parens/brackets
            return closure_stack_iter
                .get_or_insert_with(|| closure_stack.clone().into_iter().map(|x| Ok(x)))
                .next_back();
        }

        let next_token = tokenize_impl(&mut bite);
        match &next_token {
            Ok(Token::OpenParen) => closure_stack.push(Token::CloseParen),
            Ok(token) if closure_stack.last() == Some(token) => {
                closure_stack.pop();
            }
            Err(_) => {
                done = true;
            }
            _ => (),
        }

        Some(next_token)
    })
}

fn tokenize_impl(bite: &mut parser::Bite<'_>) -> Result<Token, String> {
    let token = if let Some(_) = bite.nibble(parser::Chomp::literal("sin")) {
        Token::Sine
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("log")) {
        Token::Log
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("cos")) {
        Token::Cosine
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("rand")) {
        Token::Rand
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("let")) {
        Token::Let
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("pi").or(parser::Chomp::char('𝜋'))) {
        Token::Pi
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("E")) {
        Token::E
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("sqrt")) {
        Token::Sqrt
    } else if let Some(literal) = bite.nibble(parser::Chomp::any_number()) {
        Token::LiteralNum(parse(literal)?)
    } else if let Some(_) = bite.nibble(parser::Chomp::char('(')) {
        Token::OpenParen
    } else if let Some(_) = bite.nibble(parser::Chomp::char(')')) {
        Token::CloseParen
    } else if let Some(_) = bite.nibble(parser::Chomp::char('{')) {
        Token::OpenCurly
    } else if let Some(_) = bite.nibble(parser::Chomp::char('}')) {
        Token::CloseCurly
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("=>")) {
        Token::LeftArrow
    } else if let Some(_) = bite.nibble(parser::Chomp::char(',')) {
        Token::Comma
    } else if let Some(_) = bite.nibble(parser::Chomp::char(';')) {
        Token::Semicolon
    } else if let Some(_) = bite.nibble(parser::Chomp::char('=')) {
        Token::Equals
    } else if let Some(_) = bite.nibble(parser::Chomp::char('+')) {
        Token::Plus
    } else if let Some(_) = bite.nibble(parser::Chomp::char('-')) {
        Token::Sub
    } else if let Some(_) = bite.nibble(parser::Chomp::char('*').or(parser::Chomp::char('×'))) {
        Token::Mul
    } else if let Some(_) = bite.nibble(parser::Chomp::char('/').or(parser::Chomp::char('÷'))) {
        Token::Div
    } else if let Some(_) = bite.nibble(parser::Chomp::char('^')) {
        Token::Pow
    } else if let Some(_) = bite.nibble(parser::Chomp::char('%').or(parser::Chomp::literal("mod")))
    {
        Token::Mod
    } else if let Some(indent) = bite.nibble(parser::Chomp::alphanumeric()) {
        Token::Identifier(indent.to_string())
    } else if let Some(indent) = bite.nibble(parser::Chomp::char_any(&['𝒂', '𝒃', '𝒙', '𝒚']))
    {
        Token::Identifier(indent.to_string())
    } else {
        Err(format!("Could not parse: {}", bite.as_str()))?
    };

    Ok(token)
}

fn parse<T: FromStr>(literal: &str) -> Result<T, String>
where
    <T as FromStr>::Err: Display,
{
    literal
        .parse()
        .map_err(|e| format!("Could not parse '{literal}': {e}"))
}
