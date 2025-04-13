# What this is

Currently this is a toy interpreter for lambda calculus with integers and
booleans, and just three operations (addition, equality and if-then-else).
Variables are supposed to be lexically scoped, and evaluation order is strict.

# How to run it

It actually doesn't run run, like, it's not a program yet. But you can run the tests:

```sh
cargo test
```

# What's this supposed to be?

It might eventually become an interpreter that implements this stuff: [Free
Variable as Effect, in Practice](https://okmij.org/ftp/Computation/var-effect/).
See [`notes.md`](./notes.md) for more details.

# What are the issues with it?

Well apart from the fact that it's just a toy, it also has a convoluted
architecture in order to make all expressions implement `Copy`, simplifying
ownership (or rather, sidestepping it). The idea is actually pretty cool - I am
calling it *Sharing to Copy* - I just wanted to focus on the lambda calculus
rather than make the code too noisy. But I used too many dependencies for it and
I would like to cut down a bit. See [`architecture.md`](./architecture.md) for
details.

# What about the name?

It comes from [Having an Effect](https://okmij.org/ftp/Computation/having-effect.html)

> This research has been a journey following the tantalizing lead: the founding
> papers on monads, monad transformers, free monads and extensible-effects were
> all about extensible interpreters. Along the way we have realized that the
> expression problem, stable denotations, and extensible interpreters are all
> different names for the same thing.

In there, the author begins with interpreters that can be extended without
modifying previous code, and ends up *extending* it further (heh) with the idea
that variable bindings could be resolved by effect handlers, and well, there it
is.
