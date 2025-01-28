# **enum_wrap! { ... }**

`A rust macro to wrap types into enum variants`

_Note: This project is currently being developed, it's capabilities will change/improve over time_

This macro aims to:

1. Reduce the amount of boilerplate code needed to wrap types using enums.
2. Avoid reliance on dynamic dispatching.
3. Avoid having to manually add arms to every `match` expressions in trait implementations when
   new variants are added.
4. Provide `Into<Enum>` for wrapped types to allow them be passable as `impl Into<Enum>`.

For now, this crate is not published on [crates.io](crates.io) as it is mostly just a way for
me to learn more about rust and proc macros.

#

## Add as a dependency

To add enum_wrap to your project, run:

```bash
cargo add --git "https://github.com/noahpolimon/enum_wrap.git"
```

or, add this line to Cargo.toml under `[dependencies]`:

```toml
enum_wrap = { git = "https://github.com/noahpolimon/enum_wrap.git" }
```

#

## Creating a wrapper

To create a wrapper around 2 types:

```rust
use enum_wrap::enum_wrap;

enum_wrap! {
    pub Wrapper {
        TypeA,
        TypeB
    }
}

// Creates an enum with 2 variants with the same name
// as the type name and automatically implements
// `Into<Wrapper>` for both types
```

Attributes such as `#[derive(...)]` can be used **inside** the `enum_wrap!` definition.

```rust
use enum_wrap::enum_wrap;

enum_wrap! {
    #[derive(Debug)]
    pub Wrapper { ... }
}

// Note that using those attributes on the macro will not work
```

#

## Auto-impl a trait

There are some conditions for a trait to be "auto-implementable":

1. All its methods must have a receiver (`self`, `&self`, `&mut self`). Explicit types are not
   accepted at the moment.
2. It must only have methods (no type aliases), at least for now.
3. It must be decorated with `#[enum_wrap_impl]` to expose its definition.
4. All wrapped types **should** implement it.

To auto-impl a trait, use the `#[auto_impl(...)]` attribute.

```rust
use enum_wrap::{enum_wrap, enum_wrap_impl};

#[enum_wrap_impl]
trait Trait {
    fn func(&self) -> String;
    fn func1(&self, param: String);
}

enum_wrap! {
    #[auto_impl(Trait)]
    pub Wrapper { ... }
}
```

Note that `#[auto_impl(...)]` can only be used inside `enum_wrap!` as it is not a real attribute:

```rust
// This does not compile
#[auto_impl(Trait)]
pub enum Wrapper {
    TypeA(TypeA),
    TypeB(TypeB)
}
```

#

## Example

```rust
use enum_wrap::{enum_wrap, enum_wrap_impl};

#[enum_wrap_impl]
trait Trait {
    fn func(&self) -> String;
    fn func1(&self, param: String);
}

#[derive(Debug)]
struct TypeA {}
impl Trait for TypeA { ... }

#[derive(Debug)]
struct TypeB {}
impl Trait for TypeB { ... }

enum_wrap! {
    #[derive(Debug)]
    #[auto_impl(Trait)]
    pub Wrapper {
        TypeA,
        TypeB
    }
}

fn main() {
    let a: Wrapper = TypeA { }.into();
    let b: Wrapper = TypeB { }.into();
    let _ = a.func(); // works
    b.func1(); // works
}
```

#

## Todo List:

- [ ] support renaming variants
- [ ] make #[auto_impl(...)] a real attribute (maybe?)
- [ ] support explicit receivers in trait methods

_Note: This list is not definitive_

#

## License

```
MIT License

Copyright (c) 2025 noahpolimon

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
