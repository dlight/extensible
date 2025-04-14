#![allow(unused)]

use crate::lang::{Add, Call, Eq, Expr, If, Lambda, Value, Var, VarExt};

pub fn int(i: i32) -> Expr {
    Value::int(i)
}

pub fn bool(b: bool) -> Expr {
    Value::bool(b)
}

pub fn add(left: Expr, right: Expr) -> Expr {
    Add::expr(left, right)
}

pub fn eq(left: Expr, right: Expr) -> Expr {
    Eq::expr(left, right)
}

pub fn if_(cond: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
    If::expr(cond, then_branch, else_branch)
}

pub fn call(func: Expr, arg: Expr) -> Expr {
    Call::expr(func, arg)
}

pub fn lambda(param: &str, body: Expr) -> Expr {
    Lambda::expr(param, body)
}

pub fn var(name: &str) -> Expr {
    Var::expr(name)
}
