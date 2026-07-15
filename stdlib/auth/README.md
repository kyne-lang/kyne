# kyne_stdlib_auth

## Purpose

A single, non-consuming, read-only authorization query: `auth.has(addr)`.

## Responsibilities

Implement `auth.has`, per
[`STANDARD_LIBRARY.md` §5](../../docs/STANDARD_LIBRARY.md#5-authentication-module).
Layer 2 (Blockchain). Must never duplicate the `auth(addr)` language
keyword's mandatory, aborting semantics — that keyword cannot be wrapped
by library code at all.

## Dependencies

None.

## Future work

Not yet scheduled to a specific Phase 1 milestone.
