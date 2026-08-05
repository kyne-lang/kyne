# ADR-0012: Codegen's Print-Time Choices and Scope Boundary

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #17 required implementing `kyne_codegen`, per
[`COMPILER_ARCHITECTURE.md` §14](../COMPILER_ARCHITECTURE.md#14-rust-code-generation):
render RIR as formatted Rust source text applying the standard Soroban SDK
attribute macros, scoped to what Counter and Token require. §14 states
this stage should have "no remaining semantic decisions of its own" —
true of every *behavioral* choice (all of those were already made in
`kyne_rir`, per
[ADR-0011](./ADR-0011-rir-implementation.md)) but not quite true of
every choice: a handful of purely mechanical, non-behavioral print-time
decisions still had to be made to turn RIR into literal Rust tokens.
This ADR records them, plus this crate's explicit scope boundary,
alongside ADR-0011 for the same reason: so they can be checked against a
real `cargo build` once issue #18 makes that possible.

## Decisions

**Checked arithmetic, not bare operators.**
[LANGUAGE_SPEC.md §7.2](../LANGUAGE_SPEC.md#72-arithmetic) requires
`+ - * / %` to panic on overflow/underflow rather than silently wrap.
A bare Rust `+` wraps silently in a release build (`overflow-checks`
defaults to off) — relying on a Cargo profile setting to restore
checked behavior would make a language-level correctness guarantee
depend on a build flag a downstream `cargo build` could omit or
override. `src/print.rs` instead prints every arithmetic operator as an
explicit `checked_*` method call (`a.checked_add(b).expect("arithmetic
overflow")`, and so on for `sub`/`mul`/`div`/`rem`/unary `neg`),
independent of build profile. Comparison and logical operators have no
overflow concern and print as ordinary Rust operators.

**The literal Soroban storage-API call shape.** Realizes
[`RExpr::StorageGet`]/[`RStmt::StorageSet`]/[`RStmt::StorageMutate`]
exactly as ADR-0011 anticipated:
`env.storage().persistent().get::<Symbol, T>(&Symbol::new(&env,
"<key>")).unwrap_or(<default>)` (or `.unwrap()` when there is no
default — safe because `kyne_semantics`'s definite-assignment analysis
already guarantees the field is never read before `init` writes it),
and `.set(&Symbol::new(&env, "<key>"), &<value>)`. The `::<Symbol, T>`
turbofish is printed explicitly rather than left to inference, using
the field's own declared type (looked up from `RContract.state_fields`,
threaded through as `Ctx`) — several of Token's own call sites (an
`unwrap_or(0)` chained directly into further arithmetic) would leave
Rust nothing to infer `T` from otherwise.

**`RStmt::StorageMutate` realized as three explicit Rust statements.**
Per ADR-0011: read the field into a `let mut` local bound to the
field's own name (safe — LANGUAGE_SPEC.md §4.3 already forbids a `let`
binding sharing a `state` field's name, so no collision is possible),
call the mutating method on it, then write it back to storage under the
same key.

**`throw <expr>;` prints as `return Err(<expr>)`, verbatim.** Not a new
mapping decision — [LANGUAGE_SPEC.md
§8.7](../LANGUAGE_SPEC.md#87-throw) already specifies this exact
desugaring; `kyne_rir` deliberately keeps `RExpr::Throw` as a distinct
node (matching `Return`/`Break`/`Continue`, none of which are desugared
into Rust control flow at the RIR level either) rather than eagerly
expanding it, so realizing the substitution is this crate's job.

**The canonical `if`-desugared `match` is printed back as `if`/`else`.**
`kyne_hir::lower::lower_if_to_match` desugars every Kyne `if` into a
`match` on a `Literal(Bool(true))` arm and a `Wildcard` arm (see
`compiler/hir/src/lower.rs`), which survives unchanged through RIR.
Printing that shape literally (`match cond { true => ..., _ => ... }`)
would be correct but unusual Rust that would draw a
`clippy::match_bool` complaint from any reviewer running clippy on the
output — working against
[§14.3's reviewability requirement](../COMPILER_ARCHITECTURE.md#143-reviewability).
`src/print.rs`'s `try_print_as_if` detects exactly this shape (no
guards, no extra arms) and reconstructs `if cond { ... } else { ... }`
instead — a pure print-time syntactic simplification with identical
runtime meaning, not a new semantic decision. Any other `match` shape
prints as a literal Rust `match`.

**Same-contract function calls become `Self::name(env.clone(), ...)`.**
Every generated function is an inherent associated function on the
contract's zero-sized struct (`env: Env` is a parameter, not `&self` —
matching real Soroban SDK-generated contracts), so calling one from
another (`balance_of(to)` inside `mint`) has no implicit "same object"
call syntax the way a `&self` method would. `src/print.rs` recognizes a
call whose callee names a known contract function (from
`RContract.functions`, threaded through `Ctx`) and prints `Self::name(env.clone(),
<args>)`, prepending a clone of the caller's own `env` — `Env` is a
cheap handle clone in the real SDK.

**Every non-`Copy` bound name is `.clone()`d at every read.**
`soroban_sdk::Address` (and every other SDK collection/handle type) is
`Clone` but not `Copy`. A parameter or `let` binding read more than
once — `to` in `mint`, first passed to `balances.set(to, ...)` and
later to `emit Mint(to, amount)` — moves on its first read under a
naive print, leaving the second a real Rust "use of moved value"
compile error. **This was not caught during issue #17 and was only
found once issue #18 ran a real `cargo build` against the real
`soroban-sdk`** — see Verification below. The fix (`print_fn`'s
`bound_names`/`copy_names`, `collect_bindings`, `print_bound_name`)
appends `.clone()` to every read of a name bound to a non-`Copy` type,
including reads that turn out to be the value's last use. This is
simpler than a real move/liveness analysis (which would clone only
all-but-the-last read) and always correct: Kyne's value semantics
([LANGUAGE_SPEC.md §4.0](../LANGUAGE_SPEC.md#40-memory-model-value-semantics-only))
guarantee a binding's value is never mutated out from under a read, so
cloning early versus cloning only when a later use exists cannot change
observable behavior — the cost is an occasional redundant clone (`.clone()`
on a value already about to be consumed anyway, or on a `let` binding
whose type wasn't provable as `Copy` because it had no explicit
annotation) rather than a real correctness gap. A future move/liveness
pass to avoid the redundant clones is real Ownership Planner work
belonging to `kyne_rir` per
[MEMORY_MODEL.md §21](../MEMORY_MODEL.md#21-memory-planning-architecture),
not this crate — noted as future work below.

**`string`-typed `const`s print as `&'static str`, not
`soroban_sdk::String`.** A `const` item has no `Env` available to
construct a runtime SDK value with (`String::from_str(&env, ...)`
requires one), and per [LANGUAGE_SPEC.md
§9.3](../LANGUAGE_SPEC.md#93-immutable-values), a `const` "has no
runtime storage representation at all" — it is inlined at compile
time. `src/print.rs`'s `print_const_type` special-cases `RType::String`
to `&str` for `const` position only; every other position (state
fields, params, struct fields) still prints `soroban_sdk::String`.
Every other type Counter/Token's own `const`s use (`u32`) was already a
real Rust primitive with no such gap.

**Import selection is computed, not fixed.** `src/print.rs` walks every
`RType` reachable from the contract (state fields, non-`String`
consts, event params, struct/enum fields, function params/returns) and
imports exactly the `soroban_sdk` names actually used, plus the always-
needed `contract`/`contractimpl`/`Env`/`Symbol` and the conditionally-
needed `contracttype`/`contracterror`, to avoid an unused-import
warning on the generated crate.

**`rustfmt` is invoked as a system subprocess, not linked as a
dependency.** Per the root `Cargo.toml`, no crate outside this
workspace's own members is part of the Kyne build — this workspace has
no external crate dependencies at all. `src/print.rs`'s `run_rustfmt`
shells out to the system `rustfmt` binary via `std::process::Command`
(a tool invocation, not a linked dependency), the same category of
external-tool boundary `kyne build`'s later stages cross when invoking
`cargo`/the Soroban CLI directly (issue #18).

## Scope boundary (issue #17: Counter and Token only)

- **`Cargo.toml` generation is deferred to issue #18.** §14 describes
  the generated crate's `Cargo.toml` declaring "the same dependencies
  any hand-written Soroban contract crate would," but issue #17's own
  acceptance criteria list only the `.rs` source text, formatting, and
  determinism — not a `Cargo.toml`. Issue #18 ("Cargo/Soroban build
  integration") is the natural place to own a real, buildable
  `Cargo.toml` together with the actual `cargo build` invocation that
  would first let it be verified.
- **§14.1's multi-file module mirroring is not implemented.** RIR
  itself (`RProgram { contract: RContract }`) carries no per-source-file
  boundary information to mirror — Counter and Token are each a single
  `.kyn` file, so this gap is invisible at their scope. Real
  multi-file mirroring needs multi-file tracking added upstream first,
  a Phase 2 follow-up.
- **`event` declarations do not get a generated Rust type.** Issue #17's
  acceptance criteria list `#[contract]`/`#[contractimpl]`/
  `#[contracttype]`/`#[contracterror]` only, not a `#[contractevent]`
  macro; `emit` realizes directly as an
  `env.events().publish((topic,), (data...))` call with no
  intermediate type.
- **An unqualified user-declared fieldless enum variant used directly in
  a pattern is not distinguished from a fresh binding.** `kyne_ast`'s
  `Pattern::Ident(name)` is structurally identical whether `name` is a
  prelude constructor (`None`) or a plain catch-all binding — Kyne
  resolves this the same way Rust's own pattern resolution does (an
  in-scope unit constant/variant name is a constructor pattern,
  anything else is a binding), so printing the identifier verbatim
  round-trips correctly for `None`/`Some`/`Ok`/`Err`. Neither Counter
  nor Token pattern-matches on a user-declared fieldless variant
  unqualified, so this case is implemented for structural completeness
  but untested.
- **A `string`-typed (or other `Env`-constructing type) value used as a
  general expression — not a `const` initializer — is printed as a bare
  Rust `&str` literal**, which will not type-check against a position
  expecting `soroban_sdk::String`. Neither Counter nor Token constructs
  a `soroban_sdk::String` value inside a function body, so this gap is
  untested; fixing it generally requires threading expected-type context
  through `print_expr`, deferred until a canonical example exercises it.

## Verification (added during issue #18)

ADR-0011 and this ADR's original text both flagged the assumed Soroban
SDK surface as "best-effort, not verified" because no crate before
issue #18 could reach a real `cargo build`. Issue #18 got real (if
limited) network access — `index.crates.io` is reachable and this
machine's `~/.cargo/registry` already had `soroban-sdk 23.5.3` and its
dependency closure cached — and confirmed, with a real `cargo build
--target wasm32-unknown-unknown --release --offline` against the real
crate:

- Both Counter's and Token's generated Rust (this repo's own golden-file
  fixtures) **compile successfully to a real, valid WASM binary** against
  `soroban-sdk 23.5.3`.
- This surfaced the real, non-`Copy`-clone bug documented above, fixed
  in this same issue.
- One pre-existing, non-blocking finding: `env.events().publish(...)`
  compiles but emits `warning: use of deprecated method
  soroban_sdk::events::Events::publish: use the #[contractevent] macro
  on a contract event type`. Not fixed here — adopting `#[contractevent]`
  needs a real generated Rust type per `event` declaration, a design
  change out of this issue's scope (see the "`event` declarations do
  not get a generated Rust type" scope note above) — tracked as future
  work below.
- `soroban-env-host` (needed only for native/test builds, not the real
  `wasm32-unknown-unknown` contract build) pulls in a yanked `spin
  0.9.8` and could not be resolved in this environment — irrelevant to
  the actual contract build, which uses `soroban-env-guest` instead
  (see `soroban-sdk`'s own `Cargo.toml`,
  `[target.'cfg(target_family="wasm")'.dependencies.soroban-env-guest]`).

## Consequences

- The Soroban SDK type/storage-API mapping (ADR-0011) and this ADR's
  print-time choices are now verified against a real build for
  Counter and Token specifically, not merely inspected — see
  Verification above. Coverage beyond these two examples remains
  unverified.
- The two scope gaps noted above (unqualified fieldless-variant
  patterns, non-`const` `string`-typed expressions) are the concrete
  things a future issue expanding RIR coverage beyond Counter/Token
  needs to revisit first.
- Two further follow-ups surfaced by real verification, deferred as
  future work: adopting `#[contractevent]` for `emit` (avoiding the
  deprecation warning above), and a real move/liveness pass in
  `kyne_rir` so a non-`Copy` value is cloned only when it is actually
  read again, not at every read unconditionally.
