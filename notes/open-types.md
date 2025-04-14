# Open types

## Pitch

Here is an idea: open types, a form of late binding. That is, types that are
extensible. It's like OCaml `type .. += ..` but in OCaml's case, this actually
createss another type (in another module); but in my language I want to modify
the existing type, and even have two modifications happen without one knowing
about the other (which could cause a diamond problem if I'm not careful)

Any operation on open types that checks it on a case by case basis needs to be
open, which means that whoever is extending the type must also implement the
operation for the new variants they are adding.

Open operations may also be declared with a catch all `_` (which is like the `_`
at the end of a `match` in languages like Rust). This means that who is
extending the type may implement the operation for the new variantsss, but
doesn't need to.

Operations that treat the type as a whole can't be declared open, and thus
aren't implemented when extending the type.

(Note: operations could check the type on a case by case basis with a catch all,
but still not be open. This just means that for new variants there won't be a
possibility to add custom code for them - they will always hit the `_` branch.
Not sure how useful is that)

Anyway, suppose you create a file `item.ext` with this:

```ext
open type Item {
    Sword { attack: number, durability: number }
    Potion { heal: number }
}

open fn use(item: Item)

use(Sword { damage, durability }) {
    say "Attacked for {attack} damage, durability is now {durability - 1}"
}

use(Potion { heal }) {
    say "Healed {heal} HP"
}

fn action(item: Item) {
    say "Will perform some action.."

    use(item)
}

open fn describe(item: Item)

describe(item: Sword) {
    say "A rusty sword. (DEBUG: {item})"
}

describe(item: Potion) {
    say "A glowing potion (DEBUG: {item})"
}

describe(item: _) {
    say "Unknown item (DEBUG: {item})
}
```

Since `Item` was declared as an open type, you can later modify it. But if you
just modify it without also implementing `use` for the new variants, there will
be a compiler error. You may also implement `describe` for some or all new
variants, but it is not needed, because there is a default implementation.

So you put this on `newitems.ext`:

```ext
import item.Item, item.Item.Sword, item.Item.Potion;

extend type Item {
    Boots { speed: number }
    Amulet { magic: number }
}

extend fn use(item: Item)

use(Boots { speed }) {
    say "Wearing boots, speed is now {speed}"
}

use(Amulet { magic }) {
    say "Wearing an amulet, magic is now {magic}"
}

extend fn describe(item: Item)

describe(Boots { speed }) {
    say "A pair of boots. (DEBUG: {item})"
}

open fn break(item Item);

break(Sword) {
    say "Breaking sword, attack is decreased"
}

break(Potion) {
    say "Breaking potion. What a waste!"
}

break(Boots) {
    say "Breaking boots, speed is decreased"
}

break(Amulet) {
    say "Breaking amulet, magic is decreased"
}

break(item: _) {
    say "Breaking unknown item (DEBUG: {item})"
}
```

In there `use` was fully implemented, but `describe` only partially implemented.
There is now a new function `break`, which can be further extended by other code.

## Keyword

I initially wanted the keyword to extend a function or type to be `add` (as in,
`add type ..` and `add fn ..`, or even `add to open type ..` and `add to open fn
..`), but then maybe it could be confused with the mathematical operation?

One alternative is `ext`, but I'm not sure if it's clear enough:

```ext
ext type Notification {
    Email { email: string, subject: string, body: string }
    Sms { number: string, message: string }
}

ext fn send(notification: Notification)

send(Email { email, subject, body }) {
    ...
}

send(Sms { number, message }) {
    ...
}
```

Another option is `grow`

```ext
grow type Notification {
    Email { email: string, subject: string, body: string }
    Sms { number: string, message: string }
}

grow fn send(notification: Notification)

send(Email { email, subject, body }) {
    ...
}

send(Sms { number, message }) {
    ...
}
```

I am partial to `extend`, since this language experiment is called *extensible*.
But `grow` looks very nice.


# Further syntax bikeshedding

If I'm extending a function with a single variant, it should be possible to
write it in a single definition, like this:

```ext
grow type Vehicle {
    ElectricCar { speed: number }
}

grow fn drive(vehicle: ElectricCar) {
    say "Driving electric car at {speed} km/h"
}
```

## A note on function parameters

Implicit in this discusssion is that function parameter names are meaningful -
they are used for (optional) keyword arguments. So a function `fn f(x: 1)` may
be called as either `f(1)` or `f(x: 1)` (I am thinking about `f(x = 1)` but I'm
not sure yet)

However, when defining a function that has a known signature (like an open
function, that needs to be declared beforehand, like in Haskell), it might be a
hassle to also define the parameter name if I'm doing pattern matching on the
parameter.

Maybe I even want to omit function parameters sometimes, even if the function is
not growable - this would mean this particular function would not be callable
with keyword arguments (Maybe this could be explicit by prefixing `_` into the
argument or something). One could imagine functions that can only be called with
keyword arguments, too.
