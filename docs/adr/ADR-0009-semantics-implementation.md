# ADR-0009: Implicit Collection Defaults and Ordering-Diagnostic Baseline

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #14 required implementing `kyne_semantics`'s minimal v0.2 rule set,
per [`COMPILER_ARCHITECTURE.md` §10](../COMPILER_ARCHITECTURE.md#10-semantic-analysis):
canonical member order, definite assignment of `state`, exhaustive
`match`, and `emit` argument validation. Testing the definite-assignment
rule against the six canonical examples surfaced a real, unanimous
pattern the literal text of
[`LANGUAGE_SPEC.md` §4.4](../LANGUAGE_SPEC.md#44-definite-assignment-of-state)
does not account for.

## Decision

**`list<T>`/`map<K, V>`-typed `state` fields are exempt from definite
assignment.** §4.4 requires every `state` field to either carry a
literal initializer or be unconditionally assigned in `init`, with no
stated exception. Yet four of the six canonical examples
(`token.kyn`'s `balances`, `marketplace.kyn`'s `listings`,
`voting.kyn`'s `proposals` and `voted`, `multisig_wallet.kyn`'s
`transactions` and `approvals_index`) declare a `map<K, V>` `state`
field with *no* literal initializer that is *never* assigned in `init`
either — a pattern present in every canonical example that has such a
field, with zero counterexamples. This is not plausibly six independent
oversights in the language's own normative example corpus; it implies
collection-typed `state` was always intended to have an implicit empty
default, the same way many languages default a container to empty
rather than requiring an explicit `= []`/`= {}` at every declaration
site. `kyne_semantics` implements this reading:
`has_implicit_default` in `src/definite_assignment.rs` exempts exactly
`Type::List`/`Type::Map`, nothing else — `address`, numeric types,
`bytes<N>`, `Option<T>`, `Result<T, E>`, and user `struct`/`enum` types
still require an explicit initializer or unconditional `init`
assignment, since none of those has an equally obvious, universally
safe zero value (a zero `address` or a defaulted enum variant is not
self-evidently correct the way an empty collection is).

**Canonical-order diagnostics compare against the immediately preceding
member, not the historical maximum.** An earlier implementation of
`src/order.rs` never lowered its "highest category seen" baseline after
reporting a violation, so a single leading out-of-place member (e.g. an
`error` block placed first) caused every correctly-ordered member after
it to also be flagged, cascading one real mistake into several
misleading ones. The baseline now always advances to the category of
the member just processed, whether or not that member was itself
flagged — still catching every adjacent-pair ordering violation
LANGUAGE_SPEC.md §3.2 requires as a hard error, but reporting each
genuine mistake once rather than repeating it against everything that
follows.

## Consequences

- A `state` field of a type this ADR does not name (including a
  `bytes<N>`, which is fixed-size and therefore has no obviously "empty"
  value) still requires an explicit initializer or `init` assignment;
  only `list<T>` and `map<K, V>` get the implicit-default treatment.
- If a future KIP formalizes collection-default semantics directly in
  `LANGUAGE_SPEC.md` §4.4 (the more durable home for this rule), this
  crate's `has_implicit_default` check should be updated to match
  whatever that KIP specifies, superseding the reasoning here rather
  than contradicting it.
- The ordering-diagnostic behavior change is diagnostic-quality only —
  no ordering that previously passed now fails, and no genuinely
  out-of-order member goes unreported; only the count of *duplicate*
  reports for a single mistake changed.
