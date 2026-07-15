# kyne_stdlib_math

## Purpose

Checked, wrapping, and saturating arithmetic helpers, and ratio/percentage
computation (`mul_div`, `percentage_of`).

## Responsibilities

Implement the per-type `checked_*`/`wrapping_*`/`saturating_*` families and
`mul_div`/`percentage_of`, per [`STANDARD_LIBRARY.md` §11](../../docs/STANDARD_LIBRARY.md#11-math-module).
Layer 1 (Core). Every deviation from Kyne's checked-by-default arithmetic
must be named explicitly in the function's own name.

## Dependencies

None.

## Future work

Not yet scheduled to a specific Phase 1 milestone.
