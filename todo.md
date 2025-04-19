# TODO

(I won't probably follow most of it it but for a small personal project it's nice to have)

## Tidy up

* [ ] Move files to a new directory structure, to make them smaller. (This
      proved to be harder than anticipated, because I wanted to make each module
      self-contained and I would need to make `Add`, `Lambda` etc generic on the
      expression type. This sucked so I need to organize in some other way)

* [ ] Comment things. At least the public API of each module.

* [x] Add some syntax sugar to make tests smaller
  * [x] A macro for calling the function, then testing if it equals some value
  * [x] Stuff like `add(var(x), int(1))` rather than
        `Add::expr(Var::expr(x), Value::int(1))`

## Other side quests

* [ ] Make a small CLI symbolic calculator with line editing

## Actually make any progress *at all*

* [x] Make a parser. This is actually a better investment than tidying up tests
      because it enables writing tests in the programming language rather than
      the raw AST. This is absolutely essential for having any kind of complex
      test.

* [ ] Make end-to-end tests with 3 values, source, expression and value, testing
      both parser and lang. (I could not test source -> expr but I think I
      should)

## Parser issues

* [ ] The string "42x" is accepted as a function application, like if it were
      "42 x". Sounds like an issue with logos

* [ ] Break down the large, monolithic recursive parser into smaller functions.
      That proved to be a hard task the last time I messed with chumsky. Maybe
      using
      [Recursive::declare](https://docs.rs/chumsky/1.0.0-alpha.8/chumsky/recursive/struct.Recursive.html#method.declare)
      and
      [Recursive::define](https://docs.rs/chumsky/1.0.0-alpha.8/chumsky/recursive/struct.Recursive.html#method.define)
      directly would help.

* [ ] Use pratt parsing to give + more priority than ==, so that `1 + 2 == 3`
      works, rather than having to parenthize, like `(1 + 2) == 3`

* [ ] When printing a parser error during tests, the ariadne output doesn't
      match Cargo output (I added some stdout lock to at least not interleave,
      but the Cargo output of failing test `parser::tests::precedence` still
      doesn't show together its parser error)

## Eventually manage to progress in the thing I actually wanted to do (variables as effects)

* [ ] Investigate how to implement effects
  * [ ] Traits will need a new type, for effects
  * [ ] Rather than returning expressions, I think evaluation returns either an
        expresssion or an effect
* [ ] Do the thing

## Change code structure in more involved ways

* [ ] Have multiple instances of `internment`, one for each type, rather than a
      big one for `Expr`. This would enable not interning `Value`, which is
      dumb.
  * [ ] Indeed it's also dumb to intern the environments. See
        [`notes/sharing-to-copy.md`](./notes/sharing-to-copy.md) on a plan to
        improve on this.
    * [ ] Or better yet, drop the interning for AST nodes and store everything
          on arenas. See [`notes/sharing-to-copy.md`](./notes/sharing-to-copy.md).
* [ ] Rather than having `Expr` implement each trait, have some `Interpreter`
      struct implement it, and add `Expr` as an associated type.

## Probably won't do it

* [ ] Add a GC to be able to actually free memory (right now I think some memory
      is freed, but I am not sure)
* [ ] Make it run in multiple threads (right now things are actually `Sync` and
      so on)


* [ ] Rewrite the AST using the [recursion
      schemes](https://www.tweag.io/blog/2025-04-10-rust-recursion-schemes/)
      pattern ([/r/rust
      discussion](https://old.reddit.com/r/rust/comments/1k1gfyi/practical_recursion_schemes_in_rust_traversing/))
