# Sharing to Copy

Maybe I need to figure out a simpler way to do interning / deduplication /
structural sharing.

Right now I use `rpds` for environment with structural sharing,
`typed-generational-arena` to store an arena with all environments (so they are
hashable), `lasso` for string interning and `internment` for sharing parts of
lambda terms.

If I didn't care about eventually be able to delete stuff, I could use the crate
`elsa` or something? I don't know. All solutions I used *enable* the deletion of
things, but they don't necessarily do it right now.

Anyway the point is, all this sharing isn't for performance (like, the
performance of comparing two expressions or anything like that): it's to be able
to have each expression as plain old data, and as such be `Copy`. This
eliminates problems with the borrow checker.

Eventually I might rever some of this, but in a deliberate way.

(Actually I didn't have any problem with the borrow checker, I just didn't like
the line noise that is to occasionally add `&`, other times add `.clone()` and
so on)

This could be a *good* architecture if the pieces worked together (or rather, if
I were using less crates to achieve this result) and it were integrated to a GC.

## Small improvement

The first step to improve this situation is to not intern environments anymore.
Add them to `typed-generational-arena` yes, but don't intern them alongside the
lambda expression into `internment`. This means that lambdas (the syntax tree)
are internet, but *closures* are not: they contain an interned lambda and an
environment.

(Note, maybe my terminology of lambda = AST node containing a function, closure =
AST node plus environment, isn't the best one)

Well to do this, one would require each environment to implement `Hash`.

But I have an idea: have the environment store a hash alongside it, something like

```rust
struct Env {
    cached_hash: u64,
    bindings: StandardIndex<EnvMap>,
}
```

rather than the current

```rust
type Env = StandardIndex<EnvMap>;
```

And every time I add a new binding (creating a new environment), recalculate the
hash. So the environment carries a cache of the hash of its own contents. (This
could even be upstreamed to `rpds`)

## Another, better improvement

I use `internment` to deduplicate parts of the AST, but perhaps this is not
needed. Maybe I just need some arena to store them and have they deallocate when
not used.. or, like, a GC. So one idea is to use `typed-generational-arena` to
store all terms, not only environments (it's supposed to store weak #
references, to deallocate after use)

That way I don't need to implement `Hash` on arenas so the above improvement is
moot.
