#![allow(unused)]

use std::sync::{Arc, LazyLock, Mutex};

use enum_as_inner::EnumAsInner;

use rpds::HashTrieMapSync;

use internment::Intern;
use lasso::{Spur, ThreadedRodeo};
use typed_generational_arena::{StandardArena, StandardIndex};

type EnvMap = HashTrieMapSync<Var, Expr>;

type Var = Spur;
type Env = StandardIndex<EnvMap>;

static RODEO: LazyLock<ThreadedRodeo> = LazyLock::new(|| ThreadedRodeo::new());
static ENV_ARENA: LazyLock<Mutex<StandardArena<EnvMap>>> =
    LazyLock::new(|| Mutex::new(StandardArena::new()));

trait VarExt {
    fn new(name: &str) -> Self;
    fn expr(name: &str) -> Expr;
    fn str(self) -> &'static str;
}

impl VarExt for Var {
    fn new(name: &str) -> Self {
        RODEO.get_or_intern(name)
    }

    fn expr(name: &str) -> Expr {
        Intern::new(ExprNode::Var(Var::new(name)))
    }

    fn str(self) -> &'static str {
        RODEO.resolve(&self)
    }
}

trait EnvExt {
    fn empty() -> Env;

    fn insert(&self, key: Var, value: Expr) -> Self;

    fn insert_str(&self, key: &str, value: Expr) -> Self;

    fn get(&self, key: Var) -> Option<Expr>;
}

impl EnvExt for Env {
    fn empty() -> Env {
        ENV_ARENA
            .lock()
            .unwrap()
            .insert(HashTrieMapSync::new_sync())
    }

    fn insert(&self, key: Var, value: Expr) -> Env {
        let map = ENV_ARENA.lock().unwrap().get(*self).unwrap().clone();
        let new_map = map.insert(key, value);
        ENV_ARENA.lock().unwrap().insert(new_map)
    }

    fn insert_str(&self, key: &str, value: Expr) -> Env {
        self.insert(Var::new(key), value)
    }

    fn get(&self, key: Var) -> Option<Expr> {
        ENV_ARENA
            .lock()
            .unwrap()
            .get(*self)
            .unwrap()
            .get(&key)
            .cloned()
    }
}

trait HasEnv {
    type Env;
}

trait Int: HasEnv {
    fn int(i: i32) -> Self;

    fn add(self, oth: Self, env: Self::Env) -> Self;
}

trait Cond: HasEnv {
    fn bool(b: bool) -> Self;

    fn if_then_else(self, then_branch: Self, else_branch: Self, env: Self::Env) -> Self;
    fn eq(self, oth: Self, env: Self::Env) -> Self;
}

trait App: HasEnv {
    fn app(self, oth: Self, env: Self::Env) -> Self;
}

trait Eval: App + Int + Cond {
    fn eval(self, env: Self::Env) -> Self;
}

impl HasEnv for Expr {
    type Env = Env;
}

type Expr = Intern<ExprNode>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, EnumAsInner)]
enum ExprNode {
    Var(Var),
    Value(Value),
    Add(Add),
    Eq(Eq),
    If(If),
    Call(Call),
    Closure(Lambda, Option<Env>),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Value {
    Int(i32),
    Bool(bool),
}

impl Value {
    fn int(i: i32) -> Expr {
        Intern::new(ExprNode::Value(Value::Int(i)))
    }

    fn bool(b: bool) -> Expr {
        Intern::new(ExprNode::Value(Value::Bool(b)))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Add {
    left: Expr,
    right: Expr,
}

impl Add {
    fn expr(left: Expr, right: Expr) -> Expr {
        Intern::new(ExprNode::Add(Add { left, right }))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Eq {
    left: Expr,
    right: Expr,
}

impl Eq {
    fn expr(left: Expr, right: Expr) -> Expr {
        Intern::new(ExprNode::Eq(Eq { left, right }))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct If {
    cond: Expr,
    then_branch: Expr,
    else_branch: Expr,
}

impl If {
    fn expr(cond: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
        Intern::new(ExprNode::If(If {
            cond,
            then_branch,
            else_branch,
        }))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Call {
    func: Expr,
    arg: Expr,
}

impl Call {
    fn expr(func: Expr, arg: Expr) -> Expr {
        Intern::new(ExprNode::Call(Call { func, arg }))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Lambda {
    param: Var,
    body: Expr,
}

impl Lambda {
    fn expr(param: &str, body: Expr) -> Expr {
        Intern::new(ExprNode::Closure(
            Lambda {
                param: RODEO.get_or_intern(param),
                body,
            },
            None,
        ))
    }

    fn with_env(self, env: Env) -> Expr {
        Intern::new(ExprNode::Closure(self, Some(env)))
    }
}

impl App for Expr {
    fn app(self, oth: Self, env: Env) -> Expr {
        let func_eval = self.eval(env);
        let (lambda, closure_env) = func_eval
            .as_closure()
            .expect("Attempted to apply a non-function: {lambda:?}");
        let arg_value = oth.eval(env);

        let new_closure_env = closure_env
            .expect("Attempted to apply closure without an environment: {lambda:?}")
            .insert(lambda.param, arg_value);
        lambda.body.eval(new_closure_env)
    }
}

use std::ops::Deref;

impl Eval for Expr {
    fn eval(self, env: Self::Env) -> Self {
        match *self {
            ExprNode::Value(_) => self,
            ExprNode::Var(name) => env.get(name).expect(&format!(
                "Variable {:?} not found in environment",
                Var::str(name)
            )),
            ExprNode::If(expr) => expr
                .cond
                .if_then_else(expr.then_branch, expr.else_branch, env),
            ExprNode::Call(call) => call.func.app(call.arg, env),
            ExprNode::Add(add) => add.left.add(add.right, env),
            ExprNode::Eq(eq) => eq.left.eq(eq.right, env),
            ExprNode::Closure(lambda, None) => Lambda::with_env(lambda, env),
            ExprNode::Closure(_, _) => self,
        }
    }
}

impl Int for Expr {
    fn int(i: i32) -> Self {
        Value::int(i)
    }

    fn add(self, oth: Self, env: Env) -> Self {
        let left_eval = self.eval(env).into_value();
        let right_eval = oth.eval(env).into_value();

        match (left_eval, right_eval) {
            (Ok(Value::Int(i1)), Ok(Value::Int(i2))) => Value::int(i1 + i2),
            (e1, e2) => {
                panic!("Addition requires two integer values, values found: {e1:?} and {e2:?}")
            }
        }
    }
}

impl Cond for Expr {
    fn bool(b: bool) -> Self {
        Value::bool(b)
    }

    fn eq(self, oth: Self, env: Env) -> Self {
        let left_eval = self.eval(env).into_value();
        let right_eval = oth.eval(env).into_value();

        match (left_eval, right_eval) {
            (Ok(Value::Int(i1)), Ok(Value::Int(i2))) => Value::bool(i1 == i2),
            (Ok(Value::Bool(b1)), Ok(Value::Bool(b2))) => Value::bool(b1 == b2),
            (e1, e2) => panic!(
                "Equality requires two values of the same type, values found: {e1:?} and {e2:?}"
            ),
        }
    }

    fn if_then_else(self, then_branch: Self, else_branch: Self, env: Env) -> Self {
        let cond_eval = self.eval(env).into_value();
        match cond_eval {
            Ok(Value::Bool(true)) => then_branch.eval(env),
            Ok(Value::Bool(false)) => else_branch.eval(env),
            e => panic!("Condition did not evaluate to a boolean, value found: {e:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static EMPTY_ENV: LazyLock<Env> = LazyLock::new(|| Env::empty());

    macro_rules! eval_eq {
        ($expr:expr, $expected:expr) => {{
            let result = $expr.eval(*EMPTY_ENV).into_value();
            assert_eq!(result, Ok($expected));
        }};
    }

    fn int(i: i32) -> Expr {
        Value::int(i)
    }

    fn bool(b: bool) -> Expr {
        Value::bool(b)
    }

    fn add(left: Expr, right: Expr) -> Expr {
        Add::expr(left, right)
    }

    fn eq(left: Expr, right: Expr) -> Expr {
        Eq::expr(left, right)
    }

    fn if_(cond: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
        If::expr(cond, then_branch, else_branch)
    }

    fn call(func: Expr, arg: Expr) -> Expr {
        Call::expr(func, arg)
    }

    fn lambda(param: &str, body: Expr) -> Expr {
        Lambda::expr(param, body)
    }

    fn var(name: &str) -> Expr {
        Var::expr(name)
    }

    #[test]
    fn test_int_addition() {
        let a = int(2);
        let b = int(3);

        eval_eq!(add(a, b), Value::Int(5));
    }

    #[test]
    fn test_int_addition_panic() {
        let a = int(2);
        let b = bool(true);

        let result = std::panic::catch_unwind(|| {
            a.add(b, *EMPTY_ENV);
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_bool_equality() {
        let a = bool(true);
        let b = bool(true);
        let c = bool(false);

        eval_eq!(eq(a, b), Value::Bool(true));
        eval_eq!(eq(a, c), Value::Bool(false));
    }

    #[test]
    fn test_int_equality() {
        let env = Env::empty();
        let a = int(10);
        let b = int(10);
        let c = int(20);

        eval_eq!(eq(a, b), Value::Bool(true));
        eval_eq!(eq(a, c), Value::Bool(false));
    }

    #[test]
    fn test_eq_panic_mixed_types() {
        let a = int(1);
        let b = bool(true);
        let result = std::panic::catch_unwind(|| {
            eq(a, b).eval(*EMPTY_ENV);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_if_then_else() {
        let istrue = bool(true);
        let isfalse = bool(false);
        let then_branch = int(42);
        let else_branch = int(7);

        eval_eq!(if_(istrue, then_branch, else_branch), Value::Int(42));
        eval_eq!(if_(isfalse, then_branch, else_branch), Value::Int(7));
    }

    #[test]
    fn test_lambda_and_application() {
        // λx. x + 1
        let body = add(Var::expr("x"), int(1));
        let lambda = lambda("x", body);

        eval_eq!(call(lambda, int(41)), Value::Int(42));
    }

    #[test]
    fn test_combined_expression() {
        let env = Env::empty();

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
    fn test_function_composition() {
        // f(x) = x + 1
        let f_body = add(var("x"), int(1));
        let f = lambda("x", f_body);

        // g(x) = x + 10
        let g_body = add(var("x"), int(10));
        let g = lambda("x", g_body);

        // r = g(5), then evaluate r
        let r = call(g, int(5)).eval(*EMPTY_ENV);

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
    fn test_nested_lambdas() {
        // outer_func = λx. λy. x + y
        let inner_body = add(var("x"), var("y"));
        let func = lambda("y", inner_body);
        let outer_func = lambda("x", func);

        // Apply it to 10
        let applied_once = call(outer_func, int(10)).eval(*EMPTY_ENV);

        // Further apply the result to 5
        let result = call(applied_once, int(5)).eval(*EMPTY_ENV).into_value();

        assert_eq!(result, Ok(Value::Int(15)));

        // Call outer_func(20)(6) in a single expresssion
        let all_at_once = call(call(outer_func, int(20)), int(6))
            .eval(*EMPTY_ENV)
            .into_value();

        assert_eq!(all_at_once, Ok(Value::Int(26)));
    }

    #[test]
    fn test_triangular_with_z_combinator() {
        // We use the z combinator here rather than the more common y combinator because the
        // language is currently strict (not lazy)

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
}
