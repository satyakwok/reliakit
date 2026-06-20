# Validate input at the boundary

## Problem

External input (config files, HTTP bodies, CLI args) arrives as raw `String`s and
integers with no guarantees. If you pass those raw values inward, every function
downstream has to re-check them or risk acting on garbage. You want to validate
once, at the edge, and then carry a type that proves the value is good.

## Use

- `reliakit-primitives`: per-value constrained types (`Port`, `NonEmptyStr`,
  `BoundedStr`, `Percent`, ...).
- `reliakit-validate`: collect every error in a struct at once instead of
  failing on the first.

## Example

Construct validated primitives at the edge:

```rust
use reliakit_primitives::{BoundedStr, NonEmptyStr, Port};

// A username is 3..=32 bytes; the bound is part of the type.
type Username = BoundedStr<3, 32>;

fn accept(raw_name: &str, raw_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let display = NonEmptyStr::new(raw_name)?; // rejects ""
    let username = Username::new(raw_name)?;   // rejects too short / too long
    let port = Port::new(raw_port)?;           // rejects 0
    // From here inward, these carry their invariant; nothing re-checks them.
    println!("{display} ({username}) on port {port}");
    Ok(())
}
```

When a whole form should report every problem at once, implement `Validate`:

```rust
use reliakit_validate::{Validate, ValidationError, Violation};

struct Signup {
    username: String,
    age: u32,
}

impl Validate for Signup {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        let mut errors = ValidationError::empty();
        if self.username.len() < 3 {
            errors.push(Violation::with_field("username", "must be at least 3 characters"));
        }
        if self.age < 18 {
            errors.push(Violation::with_field("age", "must be 18 or older"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

## Run it

```sh
cargo run -p reliakit-primitives --example primitives_basic
cargo run -p reliakit-validate --example basic
```

## Why this works

The invariant lives in the type, not in a convention. A function taking `Port`
cannot be handed `0`; a function taking `Username` cannot be handed an empty or
oversized string. You check once, at construction, and the compiler carries the
guarantee everywhere the value goes. `Validate` complements this when a caller
needs the full list of problems (a form, a config block) rather than the first.

## Common mistakes

- **Validating too late**: threading raw `String`/`u16` deep into the system and
  checking in the handler that finally uses them. Validate at the boundary.
- **Re-validating everywhere**: once you hold a `Port`, stop checking it. The
  type is the proof.
- **Stringly-typed config**: passing `String` for things that are really a port,
  a percentage, or a bounded name. Use the matching primitive.
- **Failing on the first error** for user-facing forms. Use `Validate` so the
  user sees every problem in one pass.

## When not to use this

- Not every value needs a newtype. Reach for a primitive when the constraint is
  real and reused, not for one-off locals.
- `reliakit-validate` describes *what* is wrong; it does not parse or coerce.
  Combine it with primitives or your own parsing for the *how*.
- Owned types (`BoundedStr`, `NonEmptyStr`) need `alloc`; in pure `no_std` without
  `alloc`, prefer the core primitives (`Port`, `Percent`, ...).
