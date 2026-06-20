# Reliakit cookbook

Task-oriented recipes: find your problem, the crate that solves it, and the
smallest correct example.

Each recipe mirrors a compile-tested example in the workspace and links to it, so
the code you copy is the code that CI builds. Reliakit stays runtime-agnostic, so
every recipe drives the clock and the waiting itself; nothing sleeps or spawns
for you.

## Which recipe do I need?

| Problem | Crate(s) | Recipe |
|---|---|---|
| Reject malformed external input at the edge | `reliakit-primitives`, `reliakit-validate` | [Validate input at the boundary](validate-input-at-the-boundary.md) |
| Keep credentials out of logs and panics | `reliakit-secret` | [Redact secrets in logs](redact-secrets-in-logs.md) |
| Retry a flaky call without a runtime | `reliakit-retry`, `reliakit-backoff` | [Retry with backoff](retry-with-backoff.md) |
| Cap how fast a worker calls something | `reliakit-ratelimit` | [Rate-limit a worker](rate-limit-a-worker.md) |
| Hash or sign data with a stable byte layout | `reliakit-codec` | [Deterministic encode/decode for signing](deterministic-codec-for-signing.md) |

For the wider problem-to-crate map (circuit breaking, timeouts, health, strict
JSON/CSV, decisions), see the root [README](../../README.md) sections "Which
resilience block do I use?" and "Real-world use cases".

## How to read a recipe

Every page has the same sections:

- **Problem**: the engineering situation it addresses.
- **Use**: the exact crate(s).
- **Example**: a minimal, API-accurate snippet.
- **Run it**: the `cargo run` command for the full example it mirrors.
- **Why this works**: the design value.
- **Common mistakes**: concrete errors to avoid.
- **When not to use this**: honest limits.

## Conventions used across recipes

- **Validate once, at the boundary.** Construct a validated type from untrusted
  input, then pass the type inward. Code deeper in never re-checks.
- **You own the clock.** Time-aware crates take an explicit `now` (or a delay you
  wait yourself), so the same code runs under any runtime, in tests, and in
  `no_std`.
- **No hidden work.** Nothing allocates a thread, sleeps, or reads a global
  clock behind your back.
