# Redact secrets in logs

## Problem

API keys, passwords, and tokens leak through the most ordinary code: a
`println!`, a `tracing` field, a `#[derive(Debug)]` on a struct, a panic message.
Once a secret lands in a log aggregator it is effectively public. You want values
that refuse to print themselves, while still being usable where the real bytes
are needed.

## Use

- `reliakit-secret`: wrappers whose `Display`/`Debug` render `[REDACTED]`, with
  an explicit, greppable method to reach the inner value.

## Example

```rust
use reliakit_secret::{ExposeSecret, Secret, SecretString};

fn main() {
    let api_key = Secret::new("rk_live_example");
    let password = SecretString::from_string("correct horse battery staple");

    // Formatting never reveals the value.
    assert_eq!(format!("{api_key}"), "[REDACTED]");
    assert_eq!(format!("{password:?}"), "Secret([REDACTED])");

    // Reaching the real bytes is explicit and easy to audit for.
    assert_eq!(api_key.expose_secret().len(), "rk_live_example".len());
}
```

Put a `Secret` field inside a normal struct and its `Debug` stays clean while the
other fields print as usual:

```rust
use reliakit_secret::Secret;

#[derive(Debug)]
struct DbConfig {
    host: String,
    password: Secret<String>,
}
```

## Run it

```sh
cargo run -p reliakit-secret --example secret_basic
```

## Why this works

Redaction is the default, not a discipline. The only way to read the inner value
is `expose_secret()`, so a code review (or a grep) can find every place a secret
is actually used. Accidental exposure paths (`Display`, `Debug`, derived `Debug`
on a containing struct, panic formatting) all render `[REDACTED]` automatically.

## Common mistakes

- **Logging the raw value before wrapping it.** Wrap at the source (where the
  secret enters the process), not right before you would have logged it.
- **Calling `expose_secret()` into a log line.** That defeats the wrapper. Only
  expose where the bytes are consumed (an HTTP header, a DB driver).
- **Trusting `Display` to be safe on your own types.** A custom `Display` that
  formats the inner value re-leaks it; keep secrets behind the wrapper end to end.

## When not to use this

- This is **redaction, not encryption**. The bytes sit in memory in the clear;
  `reliakit-secret` does not encrypt and does not promise zeroization. For
  at-rest or in-transit protection, use real cryptography.
- It does not stop a determined caller who has `expose_secret()`: it stops
  *accidental* logging, which is the common failure.
- `SecretString` needs `alloc`; `Secret<T>` for a `Copy`/borrowed `T` works in
  pure `no_std`.
