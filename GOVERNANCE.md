# Governance

This document describes how reliakit is maintained and how that is expected to
grow. It is deliberately lightweight; the project is small and values staying
that way.

## Current model

reliakit is currently led by a single maintainer ([@satyakwok](https://github.com/satyakwok)),
who reviews and merges changes and has the final say on direction. Decisions are
made in the open: design discussion happens on issues and pull requests, and the
[project principles](CONTRIBUTING.md#project-principles) (zero third-party
dependencies, no `unsafe`, `no_std`-friendly, small and explicit surface) are
the standard every change is measured against.

When a decision is a judgment call (an API shape, whether something belongs in
the project at all), it is discussed on the relevant issue before code is
written. "Let's not add X" is a legitimate outcome; keeping the surface small is
a feature.

## Becoming a maintainer

The project grows its maintainer set from its contributors. There is no
application form; it follows from sustained, high-quality work:

- a track record of merged non-trivial pull requests (**around five** is a rough
  guide, but it is a signal, not a quota),
- good judgment about the project's constraints (the principles above), and
- helpful, respectful participation in reviews and discussions.

The number is deliberately soft: maintainership is about demonstrated trust and
judgment, not a count to grind. A contributor who consistently shows this will be
invited to become a **collaborator** with review and merge rights. Maintainers are expected to uphold
the principles and the [Code of Conduct](CODE_OF_CONDUCT.md), and to merge only
changes that meet the project's bar.

## Path to an organization

While reliakit has a single maintainer it lives under a personal account. **Once
there are two or three regular maintainers**, the repository will move to a
GitHub organization so ownership is shared and the project does not depend on one
person. This is a criteria-gated step, not a scheduled one; the transfer
preserves issues, pull requests, stars, and history, so there is no rush to do it
before there is an actual team to share ownership with.

## Releases

Release mechanics (versioning, the per-crate tag-and-publish flow, and crates.io
Trusted Publishing) are documented in [RELEASING.md](RELEASING.md). Publishing is
restricted to maintainers.

## Code of Conduct

Participation in the project is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md).
