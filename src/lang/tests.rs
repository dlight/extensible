use super::*;
use crate::util::*;

macro_rules! eval_eq {
    ($expr:expr, $expected:expr) => {{
        let result = $expr.eval(Env::empty()).into_value();
        assert_eq!(result, Ok($expected));
    }};
}

#[test]
fn int_addition() {
    let a = int(2);
    let b = int(3);

    eval_eq!(add(a, b), Value::Int(5));
}

#[test]
fn int_addition_panic_mixed_types() {
    let a = int(2);
    let b = bool(true);

    let result = std::panic::catch_unwind(|| {
        a.add(b, Env::empty());
    });

    assert!(result.is_err());
}

#[test]
fn equality() {
    let a = bool(true);
    let b = bool(true);
    let c = bool(false);

    let x = int(10);
    let y = int(10);
    let z = int(20);

    eval_eq!(eq(a, b), Value::Bool(true));
    eval_eq!(eq(a, c), Value::Bool(false));

    eval_eq!(eq(x, y), Value::Bool(true));
    eval_eq!(eq(x, z), Value::Bool(false));
}

#[test]
fn bool_eq_panic_mixed_types() {
    let a = int(1);
    let b = bool(true);
    let result = std::panic::catch_unwind(|| {
        eq(a, b).eval(Env::empty());
    });
    assert!(result.is_err());
}

#[test]
fn if_then_else() {
    let istrue = bool(true);
    let isfalse = bool(false);
    let then_branch = int(42);
    let else_branch = int(7);

    eval_eq!(if_(istrue, then_branch, else_branch), Value::Int(42));
    eval_eq!(if_(isfalse, then_branch, else_branch), Value::Int(7));
}

#[test]
fn lambda_and_application() {
    // λx. x + 1
    let body = add(Var::expr("x"), int(1));
    let lambda = lambda("x", body);

    eval_eq!(call(lambda, int(41)), Value::Int(42));
}

#[test]
fn combined_expression() {
    // Compute (5 + 3 == 8) && (2 + 2 == 4) via nested ifs
    //
    // (language doesn't have conjunction yet)

    let expr1 = add(int(5), int(3)); // 5 + 3
    let cond1 = eq(expr1, int(8)); // expr1 == 8

    let expr2 = add(int(2), int(2)); // 2 + 2
    let cond2 = eq(expr2, int(4)); // expr2 == 4

    // if cond1 then cond2 else false is the same as cond1 && cond2
    eval_eq!(if_(cond1, cond2, bool(false)), Value::Bool(true));
}
#[test]
fn function_composition() {
    // f(x) = x + 1
    let f_body = add(var("x"), int(1));
    let f = lambda("x", f_body);

    // g(x) = x + 10
    let g_body = add(var("x"), int(10));
    let g = lambda("x", g_body);

    // r = g(5), then evaluate r
    let r = call(g, int(5)).eval(Env::empty());

    // evaluate f(r)
    // (no function composition, actually just calls f(15))
    eval_eq!(call(f, r), Value::Int(16));

    // h(x) = f(g(x))
    let h_body = call(f, call(g, var("x")));
    let h = lambda("x", h_body);

    // evaluate h(5)
    eval_eq!(call(h, int(5)), Value::Int(16));
}

#[test]
fn nested_lambdas() {
    // outer_func = λx. λy. x + y
    let inner_body = add(var("x"), var("y"));
    let func = lambda("y", inner_body);
    let outer_func = lambda("x", func);

    // Apply it to 10
    let applied_once = call(outer_func, int(10)).eval(Env::empty());

    // Further apply the result to 5
    eval_eq!(call(applied_once, int(5)), Value::Int(15));

    // Call outer_func(20)(6) in a single expresssion
    eval_eq!(call(call(outer_func, int(20)), int(6)), Value::Int(26));
}

#[test]
fn triangular_with_z_combinator() {
    // We use the z combinator here rather than the more common y combinator
    // because the language is currently strict (not lazy)

    // Z combinator: λf. (λx. f (λv. (x x) v)) (λx. f (λv. (x x) v))
    let inner_app = |x: Expr| call(x, x);
    let inner_lambda = lambda("v", call(inner_app(var("x")), var("v")));
    let outer_lambda = lambda("x", call(var("f"), inner_lambda));
    let z_combinator = lambda("f", call(outer_lambda, outer_lambda));

    // Define triangular function:
    // tri = λtri_rec. λn. if n == 0 then 0 else n + tri_rec(n - 1)
    let condition = eq(var("n"), int(0));
    let n_minus_one = add(var("n"), int(-1));
    let recursive_call = call(var("tri_rec"), n_minus_one);
    let sum = add(var("n"), recursive_call);

    let inner_lambda = lambda("n", if_(condition, int(0), sum));
    let triangular = lambda("tri_rec", inner_lambda);

    // Create triangular function via applying z_combinator
    let triangular_func = call(z_combinator, lambda("f", call(triangular, var("f"))));

    // call triangular(3) which should be 1 + 2 + 3
    eval_eq!(call(triangular_func, int(3)), Value::Int(6));

    // call triangular(53) which should be 1 + 2 + 3 + ... + 53
    eval_eq!(call(triangular_func, int(53)), Value::Int(1431));
}
