# ADR-0005: Parser Implementation Technique and Syntax-Error Code Range

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content. Per [ARCHITECTURE.md's decision-authority table](../../ARCHITECTURE.md#11-architecture-decision-records), "choosing a specific parsing technique inside `kyne_parser`" is explicitly an ADR-level decision, never a KIP.

## Context

Issue #7 required implementing `kyne_parser` (grammar enforcement) and
`kyne_cst` (the stable CST type) against the full grammar in
[`LANGUAGE_SPEC.md` §2](../LANGUAGE_SPEC.md#2-grammar). Two implementation
choices needed a documented decision: how the parser is built, and which
diagnostic code range its syntax errors use, since
[`COMPILER_ARCHITECTURE.md` §15.1](../COMPILER_ARCHITECTURE.md#151-diagnostic-code-namespace)
enumerates `KY01xx`–`KY07xx` by semantic concern (storage, types, events,
contract structure, authorization, visibility, control flow) but has no
line item for "syntax error" specifically.

## Decision

**Parsing technique.** `kyne_parser` is a hand-written recursive-descent
parser with Pratt-style precedence climbing for expressions (one function
per [`LANGUAGE_SPEC.md` §7.1](../LANGUAGE_SPEC.md#71-precedence) precedence
level, each calling the next-higher level before checking for its own
operators). No parser-generator or combinator library is used. This
mirrors the same reasoning `kyne_lexer` already established for hand-written
tokenization: Kyne's grammar is small and fixed by design (per
[`LANGUAGE_PRINCIPLES.md`'s Small Language Philosophy](../LANGUAGE_PRINCIPLES.md#small-language-philosophy)),
and a hand-written parser keeps every grammar production's implementation
directly traceable to its EBNF rule, with no generated-code layer between
the specification and the compiler.

The parser builds a lossless raw tree directly (`kyne_parser::SyntaxNode`,
tagged by a `NodeKind` mirroring every §2 production) via a builder-stack
pattern: `start_node`/`finish_node` push and pop nodes on an explicit
stack, and `bump` attaches the next token — draining any pending
trivia into the currently-open node first, so a doc comment or line
comment immediately preceding a declaration is attached to that
declaration's node rather than its predecessor's. `kyne_cst` then wraps
this raw tree, together with the source text it was parsed from, as the
stable `Cst` type other crates depend on — per
[`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates)'s dependency
table, `kyne_cst` depends on `kyne_parser`, not the reverse, so the raw
tree types have to live in the crate `kyne_cst` depends on.

**Two grammar disambiguations not spelled out in `LANGUAGE_SPEC.md` §2,
forced by its own worked examples and the six canonical contracts:**

1. A bare `identifier { ... }` is not parsed as a struct literal while
   parsing the head expression of `if`/`while`/`for`/`match` — mirroring
   Rust's identical restriction on condition expressions, for the same
   reason. Required for the Escrow canonical example's
   `if released { ... }`, where `released` is a bare `bool` state field,
   not a struct.
2. A `match` arm's non-block body accepts a bare `return`/`throw`
   (optionally followed by an expression, no trailing `;`) or bare
   `break`/`continue`, in addition to a plain `Expression`, per
   [`LANGUAGE_SPEC.md` §7.7](../LANGUAGE_SPEC.md#77-pattern-matching)'s
   own note that "`throw` and `return` used as a match arm's body have
   the bottom type" — confirmed by its worked example
   (`Status::Approved(by) if by == admin => return true,`), which the
   formal `MatchArm = Pattern [ "if" Expression ] "=>" ( Expression "," | Block )`
   production does not, on its own, admit.
3. A semicolon-less trailing expression immediately before a block's
   closing `}` is that block's value, required for `if`/`match` used in
   expression position per §7.6's own example
   (`if amount > 1000 { amount / 100 } else { 10 }`), even though
   `Block = "{" { Statement } "}"` does not show this as a distinct
   production from `ExprStmt = Expression ";"`.

All three are recorded here rather than as silent parser behavior because
they are genuine grammar-completion decisions, not merely engineering
technique — a future contributor reading §2 alone would not derive them.

**Syntax-error code range.** Parser diagnostics use `KY0001`, extending
the `KY00xx` ("Lexical") range from
[`COMPILER_ARCHITECTURE.md` §15.1](../COMPILER_ARCHITECTURE.md#151-diagnostic-code-namespace)
to cover syntax errors as well. Both lexical and syntax errors are
detected before any semantic category (storage, types, events, and so on)
could apply, so grouping them under one low range keeps the numbering
scheme's existing shape rather than introducing a new prefix range for a
single error kind. A future KIP or ADR MAY split syntax errors into their
own range if `KY00xx` grows crowded enough between lexical and syntax
concerns to justify it.

## Consequences

- `kyne_parser`'s `Parser` type and `kyne_cst`'s `Cst` type are additive,
  internal implementation details; nothing about this decision is visible
  in Kyne source syntax or in generated Rust output.
- The three grammar disambiguations above are exercised directly by
  `kyne_parser`'s own test suite (`bare_identifier_condition_is_not_a_struct_literal`,
  `match_statement_and_expression_with_guard`,
  `if_as_expression_requires_else`) so a future grammar change that
  regresses any of them fails a test, not just a canonical-example
  parse.
- `KY0001` is the only parser-produced code today; additional `KY00xx`
  codes MAY be added for more specific syntax-error categories
  (unterminated construct, mismatched delimiter, etc.) without requiring
  a new ADR, since the range itself is already decided here.
