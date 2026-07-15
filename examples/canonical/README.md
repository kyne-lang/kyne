# examples/canonical/

The six canonical example contracts from
[`LANGUAGE_SPEC.md` §16](../../docs/LANGUAGE_SPEC.md#16-examples), copied
here **verbatim** as standalone `.kyn` files:

| File | Demonstrates |
|---|---|
| [`counter.kyn`](./counter.kyn) | The minimal contract shape: `state`, `event`, `auth`, `init`. |
| [`token.kyn`](./token.kyn) | `const`, `error`, `Result`, `map<K,V>`, fungible-token accounting. |
| [`escrow.kyn`](./escrow.kyn) | Multi-party authorization, boolean release conditions. |
| [`marketplace.kyn`](./marketplace.kyn) | Local `struct` types, `match` on `Option`. |
| [`voting.kyn`](./voting.kyn) | `if`-as-expression, nested `struct` reconstruction. |
| [`multisig_wallet.kyn`](./multisig_wallet.kyn) | `list<T>`, nested `map<K, map<K,V>>`, `internal fn`, the `?` operator, `for` loops. |

**These files are not independently editable content.** They are a
faithful materialization of `LANGUAGE_SPEC.md` §16's own prose examples,
per [`ARCHITECTURE.md` §7](../../ARCHITECTURE.md#7-examples): "A change to
any file in `examples/canonical/` is, definitionally, a change to
`LANGUAGE_SPEC.md` itself and MUST go through `GOVERNANCE.md` §25's
constitutional process, never an ordinary pull request." If
`LANGUAGE_SPEC.md` §16 is ever amended by an Accepted KIP, these files
MUST be updated to match in the same change.

## Role in testing

Per [`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy),
these six files are the compiler's own golden-file corpus for code
generation tests. No test currently reads them — no compiler exists yet —
but their presence here as real, standalone files (rather than only as
prose embedded in `LANGUAGE_SPEC.md`) is itself a Phase 1 readiness
prerequisite identified in `KYNE_WORKSPACE_AUDIT.md` §18.
