Initially I was trying to follow the contents of some writings of Oleg Kiselyov
regarding implementing variable bindings as effects, but right now this is only
a simple lambda calculus implementation, with lexical scoping.

I think it all began with this from 2016, which deals on the expression problem
among other things:

* [Having an Effect](https://okmij.org/ftp/Computation/having-effect.html)

(As an aside, [Data types à la
carte](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/data-types-a-la-carte/14416CB20C4637164EA9F77097909409)
as an example of how the expression problem can be tackled in Haskell)

That linked those sources

* https://okmij.org/ftp/Haskell/extensible/EDSLNG.hs
* https://okmij.org/ftp/Computation/edlsng.ml

And this, from 2023 - 2024

* [Free Variable as Effect, in Practice](https://okmij.org/ftp/Computation/var-effect/)
* [Higher-order Programming is an Effect](https://okmij.org/ftp/Computation/variables-effects.html)
* [Compilers: Incrementally and Extensibly](https://okmij.org/ftp/tagless-final/Compiler/)

(Note: I first saw it by seeing the third link on HN, [Compilers: Incrementally
and Extensibly (2024)](https://news.ycombinator.com/item?id=43593088)]. Someone
mentioned [An Incremental Approach to Compiler
Construction](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf) which is pretty
cool as well)

The first link had accompanying papers

* [Free Variable as Effect, in Practice](https://okmij.org/ftp/Computation/var-effect/var-effect.pdf)
* [Higher-Order Programming is an Effect](https://okmij.org/ftp/Computation/HOPE.pdf)

And this presentation

* [Higher-Order Programming is an Effect](https://okmij.org/ftp/Computation/HOPE-talk.pdf)

Anyway the idea is that variable bindings can be treated as effects, in the
sense that whenever we encounter a variable binding, we can call an effect
handler that will check the environment to decide what will it resolve to (and
thus different effect handlers can decide whether to apply lexical scoping,
dynamic scoping, etc)

This isn't probably terribly useful but I wanted to implement it anyway.

Eventually I decided to first implement a simple lambda calculus the usual way
(which means, attaching each function to an environment that closes over its
scope - a closure - and not doing variable substitution as in the mathematical
definition).

So anyway that's it.
