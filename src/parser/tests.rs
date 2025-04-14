use super::*;
use crate::util::*;

macro_rules! parse_eq {
    ($expr:expr, $expected:expr) => {{
        match parse($expr) {
            Ok(result) => {
                assert_eq!(result, $expected);
                Ok(())
            }
            Err(err) => {
                report_errors($expr, &err);
                Err(err)
            }
        }
    }};
}

type Result<T, E = ParserErrors<'static>> = std::result::Result<T, E>;

#[test]
fn literal() -> Result<()> {
    let r1 = parse_eq!("42", int(42));
    let r2 = parse_eq!("true", bool(true));

    r1.or(r2)
}

#[test]
fn addition() -> Result<()> {
    parse_eq!("1 + 2", add(int(1), int(2)))
}

#[test]
fn equality() -> Result<()> {
    parse_eq!("2 == 3", eq(int(2), int(3)))
}

#[test]
fn if_then_else() -> Result<()> {
    parse_eq!("if true then 1 else 2", if_(bool(true), int(1), int(2)))
}

#[test]
fn precedence() -> Result<()> {
    parse_eq!("1 + 2 == 3", eq(add(int(1), int(2)), int(3)))
}

#[test]
fn lambda_application() -> Result<()> {
    let src = "(lambda x -> (x + 1))(5)";
    let expr = call(lambda("x", add(var("x"), int(1))), int(5));

    parse_eq!(src, expr)
}

#[test]
fn complex_expressions() -> Result<()> {
    let src = "if ((1 + 2) == 3) then (lambda x -> (x + 10))(5) else 0";

    let expr = if_(
        eq(add(int(1), int(2)), int(3)),
        call(lambda("x", add(var("x"), int(10))), int(5)),
        int(0),
    );

    parse_eq!(src, expr)
}

#[test]
fn should_err_42x() {
    let src = "42x";
    match parse(src) {
        Ok(expr) => panic!("Expected an error parsing {src}, but it parses into this: {expr:?}"),
        Err(_) => {}
    }
}

#[test]
fn but_42_x_is_ok() -> Result<()> {
    parse_eq!("42 x", call(int(42), var("x")))
}
