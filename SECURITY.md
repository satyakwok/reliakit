# Security Policy

## Supported Versions

Reliakit currently supports the latest published version of each crate.

| Crate | Supported |
|---|---|
| `reliakit-primitives` `0.2.x` | Yes |
| `reliakit-secret` `0.1.x` | Yes |
| `reliakit-validate` `0.1.x` | Yes |
| `reliakit-collections` `0.1.x` | Yes |

Older versions may receive fixes when the issue is severe and the fix can be
backported safely.

## Reporting a Vulnerability

Please do not open a public issue for security vulnerabilities.

Report security issues through GitHub Security Advisories:

https://github.com/satyakwok/reliakit/security/advisories/new

Include:

- affected crate and version,
- a minimal reproduction if possible,
- expected behavior,
- actual behavior,
- impact and exploitability notes,
- any suggested fix.

## Scope

Security reports are most useful when they involve:

- validation bypasses that can cause unsafe downstream assumptions,
- panic paths reachable through safe public APIs,
- incorrect parsing that accepts malformed input as valid,
- feature flag combinations that break documented safety or `no_std` behavior,
- accidental secret exposure in future crates.

This repository does not operate a hosted service and does not collect user data.
