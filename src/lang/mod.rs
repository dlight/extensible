#![allow(unused)]

use std::sync::{Arc, LazyLock, Mutex};

use enum_as_inner::EnumAsInner;

use rpds::HashTrieMapSync;

use internment::Intern;
use lasso::{Spur, ThreadedRodeo};
use typed_generational_arena::{StandardArena, StandardIndex};

#[cfg(test)]
mod tests;

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
