# ADR-0006: Formatter Implementation Strategy and Two Rule Interpretations

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #10 required implementing `kyne_formatter` against
[`LANGUAGE_SPEC.md` §13](../LANGUAGE_SPEC.md#13-formatting-rules). Two
gaps in that section's literal text needed a documented decision,
discovered by testing against the six canonical examples in
`examples/canonical/` — the acceptance criterion issue #10 itself names
as "a good conformance check."

## Decision

**Implementation strategy.** `kyne_formatter` walks `kyne_cst`'s raw tree
directly (not `kyne_ast`) and re-emits every construct in canonical form
from scratch, rather than preserving and adjusting the original source's
whitespace. This is possible, and correct, specifically because §13 has
no "remove redundant parentheses" rule — that normalization is
`kyne_ast`'s job (`COMPILER_ARCHITECTURE.md` §7), not the formatter's — so
the formatter never needs precedence-aware reasoning about when a paren
is "redundant": any `ParenExpr` the parser produced is printed exactly
where it is, and none are ever synthesized. Comments are preserved by
walking `kyne_cst`'s leading-trivia attachment (a statement's or
declaration's leading whitespace/comments live nested inside its first
descendant node, per the parser's tree-building design — see
[ADR-0005](./ADR-0005-parser-implementation.md) — so trivia lookup walks
each node's full token stream in document order, not just its direct
children).

**Rule interpretation 1: struct/enum/error declaration bodies are always
multi-line.** All six canonical examples render every `error`, `struct`,
and `enum` declaration with one variant/field per line and a trailing
comma, even when the flat single-line form would easily fit under 100
columns (e.g. `error TokenError { InsufficientBalance, InvalidAmount }`
is 55 characters, yet renders across four lines in `token.kyn`). §13's
own text only says trailing commas are "required... that spans multiple
lines" — it does not say these three declaration bodies must *always*
span multiple lines. Given the unanimous, exception-free pattern across
every canonical example, `kyne_formatter` always expands these three
declaration kinds, unconditionally — never collapsing to one line
regardless of width. This does not apply to struct-literal *expressions*
(`Listing { seller, price, sold: false }`), which do collapse to one
line when they fit: the distinction tracks
[LANGUAGE_SPEC.md's Auditability principle](../LANGUAGE_SPEC.md#auditability-over-cleverness)
— a contract's storage/error surface is a declaration, always scanned
one item per line; a struct literal is an ordinary expression like any
other.

**Rule interpretation 2: width-driven wrapping for struct-literal and
`if`-expression values.** §13's line-length rule ("Max line length | 100
columns") is enforced by width-aware wrapping, not by preserving the
author's original line breaks: a `let` binding's value, or a bare
expression statement, renders flat if it fits and wraps otherwise.
Wrapping is implemented for exactly two shapes, since these are the only
ones the canonical examples exercise: a `StructLiteral` (fields expand
one per line, trailing comma) and an `if`/`else` used in expression
position (each branch's value re-checked for fit at its own, deeper
indent, recursively). A call or method call whose *last* argument is a
`StructLiteral` "hugs" that argument — the call's own text stays flat and
only the struct literal's fields expand, with the closing `}` sharing a
line with the call's closing `)` — matching the pattern used throughout
`multisig_wallet.kyn`. No general expression-wrapping algorithm is
implemented for every possible expression shape; unsupported shapes that
exceed 100 columns are left flat rather than guessed at.

**Fixture correction.** While testing rule interpretation 2, byte-counting
found that `examples/canonical/escrow.kyn`'s `init` function signature
was 101 characters — one over §13's own limit, and therefore not actually
in canonical form despite issue #10's framing of the six examples as
"already hand-formatted to the canonical style." Both
`examples/canonical/escrow.kyn` and the matching code block in
[`LANGUAGE_SPEC.md` §16.3](../LANGUAGE_SPEC.md#163-escrow) are corrected
to wrap that one parameter list, matching what `kyne_formatter` itself
produces. This is treated as a documentation/fixture bug fix (an
illustrative example brought into compliance with an existing,
unchanged rule), not a constitutional change, and needed no KIP.

## Consequences

- `kyne_formatter`'s output for `error`/`struct`/`enum` bodies is always
  multi-line; a future contributor should not "simplify" this to a
  width check without re-reading this ADR.
- The wrapping algorithm covers exactly the two shapes described above;
  extending it to more expression kinds (e.g. list/map literals, binary
  expression chains) is a natural, additive follow-up, not a breaking
  change to this ADR's decision.
- `examples/canonical/escrow.kyn` and `LANGUAGE_SPEC.md` §16.3 now agree
  byte-for-byte (modulo the surrounding prose), closing the gap this
  ADR found.
