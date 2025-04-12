# TODO (I won't probably follow it but for a small personal project it's nice to have)

## Tidy up

* [ ] Move files to a new directory structure, to make them smaller

* [ ] Comment things. At least the "public" API of each module, as in, the
  `pub(crate)` things.

* [x] Add some syntax sugar to make tests smaller
  * [x] A macro for calling the function, then testing if it equals some value
  * [x] Stuff like `add(var(x), int(1))` rather than
        `Add::expr(Var::expr(x), Value::int(1))`

## Change code structure in more involved ways

* [ ] Have multiple instances of `internment`, one for each type, rather than a
      big one for `Expr`. This would enable not interning `Value`, which is dumb.
  * [ ] Indeed it's also dumb to intern the environments. See
    [`architecture.md`](./architecture.md) on a plan to improve on this.
    * [ ] Or better yet, drop the interning for AST nodes and store everything
      on arenas. See [`architecture.md`](./architecture.md).
* [ ] Rather than having `Expr` implement each trait, have some `Interpreter`
      struct implement it, and add `Expr` as an associated type.

## Actually make any progress *at all*

* [ ] Make a parser. This is actually a better investment than tidying up tests
      because it enables writing tests in the programming language rather than
      the raw AST. This is absolutely essential for having any kind of complex
      test.

## Eventually manage to progress in the thing I actually wanted to do (variables as effects)

* [ ] Investigate how to implement effects
  * [ ] Traits will need a new type, for effects
  * [ ] Rather than returning expressions, I think evaluation returns either an
        expresssion or an effect
* [ ] Do the thing

## Probably won't do it

* [ ] Add a GC to be able to actually free memory (right now I think some memory
      is freed, but I am not sure)
* [ ] Make it run in multiple threads (right now things are actually `Sync` and
  so on)
