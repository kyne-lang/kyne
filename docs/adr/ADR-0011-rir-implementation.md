# ADR-0011: RIR's Soroban SDK Type and Storage-API Mapping

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #16 required implementing `kyne_rir`, per
[`COMPILER_ARCHITECTURE.md` §12](../COMPILER_ARCHITECTURE.md#12-intermediate-representations)
and [`MEMORY_MODEL.md` §21](../MEMORY_MODEL.md#21-memory-planning-architecture):
map `address` to `soroban_sdk::Address`, `state` reads/writes to explicit
storage-API calls, and `auth(addr)` to `require_auth()`, scoped to what
Counter and Token require. Every one of these mappings targets a real,
external Rust crate (`soroban-sdk`) this workspace does not depend on
and has no way to compile against yet — `kyne_codegen` (issue #17) only
produces Rust source *text*, and no crate before issue #18 invokes a
real `cargo build`.

## Decision

**The exact Soroban SDK surface below is this crate's best-effort
target, not a verified one.** It reflects the author's understanding of
`soroban-sdk`'s public API and is deliberately recorded here, in one
place, specifically so it can be checked against a real `cargo build`
once issue #18 makes that possible, and corrected in one place if it's
wrong — rather than the mapping being scattered as implicit assumptions
across this crate's code with no single point of truth to verify or fix.

**Type mapping** (`src/lower.rs`'s `lower_type`):

| Kyne type | Rust/Soroban type |
|---|---|
| `bool`, `i32`/`i64`/`i128`, `u32`/`u64`/`u128` | themselves (Rust's own primitives) |
| `address` | `soroban_sdk::Address` |
| `symbol` | `soroban_sdk::Symbol` |
| `string` | `soroban_sdk::String` |
| `bytes` | `soroban_sdk::Bytes` |
| `bytes<N>` | `soroban_sdk::BytesN<N>` |
| `list<T>` | `soroban_sdk::Vec<T>` (the SDK's own host-managed vector, distinct from `std::vec::Vec`) |
| `map<K, V>` | `soroban_sdk::Map<K, V>` |
| `Option<T>`, `Result<T, E>` | Rust's own, used directly — Soroban contract functions may return either |
| a user `struct`/`enum`/`error` name | unchanged |

**Storage API mapping**: a `state` field read/write becomes
[`RExpr::StorageGet`]/[`RStmt::StorageSet`], each carrying the field's
own name as its storage key, per
[`LANGUAGE_SPEC.md` §4.3](../LANGUAGE_SPEC.md#43-state-persistent-contract-storage)'s
"keyed by the field's name." `kyne_codegen` (issue #17) is expected to
print these as `env.storage().persistent().get(&Symbol::new(&env, "<key>"))`
and `.set(&Symbol::new(&env, "<key>"), &<value>)` — `Symbol::new` rather
than the shorter `symbol_short!` macro specifically because
`symbol_short!` caps names at 9 characters and Token's own
`total_supply` field is 12, so the macro isn't generally applicable to
real Kyne field names.

**`auth` mapping**: `auth(addr)` becomes `RStmt::RequireAuth(addr)`,
printed as `addr.require_auth()` — an instance method on
`soroban_sdk::Address`, per
[`LANGUAGE_SPEC.md` §10.1](../LANGUAGE_SPEC.md#101-the-auth-statement)'s
own text ("compiling directly to Soroban's `require_auth()` call *on
that address*"), not a free function taking the address as an argument.

**The synthetic `env: Env` parameter.** Every real Soroban contract
function needs a `soroban_sdk::Env` handle to reach storage or perform
authorization, but Kyne source never declares one — `RFnDecl.params`
therefore holds only Kyne's own declared parameters, and prepending the
fixed `env: Env` parameter is left to `kyne_codegen` at print time,
since it is unconditional and carries no per-function information this
stage would otherwise compute.

**Compound `state` assignment is expanded, not preserved.** `count +=
1` has no Rust place to apply `+=` to (a `state` field is never a real
Rust binding), so `src/lower.rs`'s `lower_assign` always expands a
compound assignment into an explicit read-modify-write:
`RStmt::StorageSet { key: "count", value: <StorageGet count> + 1 }`.

**Implicit collection defaults are inherited and realized here.**
[ADR-0009](./ADR-0009-semantics-implementation.md) established that a
`list<T>`/`map<K, V>` `state` field with no literal initializer has an
implicit empty default. This crate is where that default becomes a real
value: every `StorageGet` for such a field carries a synthesized empty
`RExpr::ListLiteral`/`RExpr::MapLiteral` as its `default`, so
`kyne_codegen` can print a `.get(...).unwrap_or(<default>)` call that
never observably reads as unset.

## Consequences

- If the real `soroban-sdk` API differs from this table once issue #18
  can actually compile generated output, the fix is a `lower_type`/
  storage-call change here (and the matching print logic in
  `kyne_codegen`), not a re-derivation from scratch — this ADR is the
  single record of what was assumed and why.
- `kyne_codegen` (issue #17) owns exactly two things this crate
  deliberately leaves undone: prepending `env: Env` to every function
  signature, and choosing the literal Rust syntax (`env.storage()...`)
  that realizes `RExpr::StorageGet`/`RStmt::StorageSet`/`RStmt::RequireAuth`.
