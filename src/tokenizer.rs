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
    OpenParen,
    CloseParen,
    Pow,
    Mod,
    Rand,
    Identifier(String),
    Let,
    Equals,
}

pub fn tokenize_iter<'a>(
    source: parser::Bite<'a>,
) -> impl Iterator<Item = Result<Token, String>> + 'a {
    let mut bite = source;
    let mut done = false;

    std::iter::from_fn(move || {
        let has_next = !bite.is_empty() && !done;
        has_next.then(|| {
            let token: Result<Token, String> = tokenize_impl(&mut bite);
            if token.is_err() {
                done = true;
            }
            token
        })
    })
}

fn tokenize_impl(bite: &mut parser::Bite<'_>) -> Result<Token, String> {
    *bite = bite.chomp(parser::Chomp::whitespace());
    let token = if let Some(_) = bite.nibble(parser::Chomp::literal("sin")) {
        Token::Sine
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("cos")) {
        Token::Cosine
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("rand")) {
        Token::Rand
    } else if let Some(_) = bite.nibble(parser::Chomp::literal("let")) {
        Token::Let
    } else if let Some(literal) = bite.nibble(parser::Chomp::any_number()) {
        Token::LiteralNum(parse(literal)?)
    } else if let Some(indent) = bite.nibble(parser::Chomp::alphanumeric()) {
        Token::Identifier(indent.to_string())
    } else if let Some(_) = bite.nibble(parser::Chomp::char('(')) {
        Token::OpenParen
    } else if let Some(_) = bite.nibble(parser::Chomp::char(')')) {
        Token::CloseParen
    } else if let Some(_) = bite.nibble(parser::Chomp::char('=')) {
        Token::Equals
    } else if let Some(_) = bite.nibble(parser::Chomp::char('+')) {
        Token::Plus
    } else if let Some(_) = bite.nibble(parser::Chomp::char('-')) {
        Token::Sub
    } else if let Some(_) = bite.nibble(parser::Chomp::char('*')) {
        Token::Mul
    } else if let Some(_) = bite.nibble(parser::Chomp::char('/')) {
        Token::Div
    } else if let Some(_) = bite.nibble(parser::Chomp::char('^')) {
        Token::Pow
    } else if let Some(_) = bite.nibble(parser::Chomp::char('%')) {
        Token::Mod
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
