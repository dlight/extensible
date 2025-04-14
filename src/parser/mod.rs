#![allow(unused)]

use std::fmt;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;

use itertools::Itertools;

use crate::lang::{Add, Call, Eq, Expr, If, Lambda, Value, Var, VarExt};

#[cfg(test)]
mod tests;

type Span = SimpleSpan<usize>;
type Spanned<T> = (T, Span);
type ParserError<'tokens> = extra::Err<Rich<'tokens, Token>>;

type ParserErrors<'tokens> = Vec<Rich<'tokens, Token>>;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//.*")]
pub enum Token {
    Error,

    #[regex(r"[+-]?[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i32),

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("if")]
    If,

    #[token("then")]
    Then,

    #[token("else")]
    Else,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[token("+")]
    Add,

    #[token("==")]
    Eq,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("lambda")]
    Lambda,

    #[token("->")]
    Arrow,

    #[token(";")]
    Semicolon,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::If => write!(f, "if"),
            Self::Then => write!(f, "then"),
            Self::Else => write!(f, "else"),
            Self::Ident(s) => write!(f, "{}", s),
            Self::Add => write!(f, "+"),
            Self::Eq => write!(f, "=="),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Lambda => write!(f, "lambda"),
            Self::Arrow => write!(f, "->"),
            Self::Semicolon => write!(f, ";"),
            Self::Error => write!(f, "<error>"),
        }
    }
}

fn parser<'tokens, Input>() -> impl Parser<'tokens, Input, Expr, ParserError<'tokens>>
where
    Input: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        // atoms
        let atom = select! {
            Token::Int(i) => Value::int(i),
            Token::True => Value::bool(true),
            Token::False => Value::bool(false),
            Token::Ident(s) => Var::expr(&s),
        };

        // parenthesized expressions
        let paren_expr = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        // lambda expressions
        let lambda = just(Token::Lambda)
            .ignore_then(select! { Token::Ident(s) => s })
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map(|(param, body)| Lambda::expr(&param, body));

        // factor-level expressions (atoms, parenthesized, and lambdas)
        let factor = choice((atom, paren_expr, lambda));

        // function application
        let app_args = factor.clone().repeated().at_least(1);
        let app = factor
            .clone()
            .foldl(app_args, |func, arg| Call::expr(func, arg));

        // addition
        let add_args = just(Token::Add).ignore_then(factor.clone()).repeated();
        let add = factor
            .clone()
            .foldl(add_args, |acc, next| Add::expr(acc, next));

        // Equality
        let eq = factor
            .clone()
            .then(just(Token::Eq).ignore_then(factor.clone()))
            .map(|(a, b)| Eq::expr(a, b));

        // If-then-else
        let if_then_else = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then_ignore(just(Token::Else))
            .then(expr.clone())
            .map(|((cond, then_branch), else_branch)| If::expr(cond, then_branch, else_branch));

        // Final expression
        choice((if_then_else, app, eq, add, factor))
    })
}

pub fn parse<'input>(src: &'input str) -> Result<Expr, Vec<Rich<'input, Token>>> {
    let token_iter = Token::lexer(src).spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Error, span.into()),
    });

    let token_stream =
        Stream::from_iter(token_iter).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));
    let (output, errors) = parser().parse(token_stream).into_output_errors();

    if !errors.is_empty() {
        return Err(errors);
    }

    output.ok_or_else(Vec::new)
}

// Helper function to report errors nicely
pub fn report_errors(src: &str, errors: &[Rich<Token>]) {
    use std::io::Write;

    let mut stdout = std::io::stdout().lock();

    writeln!(&mut stdout).unwrap();

    for err in errors {
        let err_span = err.span().into_range();
        Report::build(ReportKind::Error, err_span.clone())
            .with_message(err.to_string())
            .with_label(
                Label::new(err_span.clone())
                    .with_message(err.reason().to_string())
                    .with_color(Color::Red),
            )
            .with_labels(err.contexts().map(|(label, label_span)| {
                Label::new(label_span.into_range())
                    .with_message(format!("while parsing this {}", label))
                    .with_color(Color::Yellow)
            }))
            .finish()
            .write_for_stdout(Source::from(src), &mut stdout)
            .unwrap();
    }
    writeln!(&mut stdout).unwrap();
}
