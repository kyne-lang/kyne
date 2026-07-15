# STANDARD_LIBRARY.md

# The Kyne Standard Library Specification

**Phase:** 0 — Foundations
**Milestone:** 7 — Standard Library Specification
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on every future standard library implementation and contributor.

This document defines every module, and every conceptual API within it, that ships with Kyne. It describes **behavior, not implementation**: for every API, this document states what a conforming implementation MUST guarantee, never the Rust code that guarantee is realized with. The standard library is part of the language, not an SDK bolted onto it — every API in this document is designed to feel like it belongs to Kyne itself, not like a thin, mechanically generated wrapper around the Soroban SDK it ultimately compiles to.

This document implements — and MUST NOT contradict — [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), and [MEMORY_MODEL.md](./MEMORY_MODEL.md), all of which are locked, constitutional documents with respect to this one. Several of this document's most important decisions are, specifically, resolutions of tensions between what a conventional standard library might offer and what those five documents already fix as immutable — every such resolution is stated explicitly, with its reasoning, rather than silently decided. Where this document appears to conflict with any of those five, this document is wrong and MUST be corrected by KIP.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings.

---

# 1. Standard Library Philosophy

**Purpose.** The standard library exists to provide the small set of facilities nearly every Kyne contract needs, so that no contract author has to reimplement them, and so that every contract author who does need them uses the same, audited, consistent implementation. It is not a general-purpose utility collection; it is the minimum shared surface a Soroban smart contract written in Kyne requires.

**Scope.** The standard library covers exactly three concerns: general language facilities that require no blockchain context (Layer 1), facilities that require Soroban's execution context (Layer 2), and development-time tooling that has no bearing on deployed contract behavior (Layer 3). Anything outside these three concerns — networking, general-purpose data processing, application frameworks — is out of scope by design, per [§22](#22-future-library-evolution) and [Non-Goals](#non-goals).

**Relationship to Kyne.** The standard library is not an optional dependency a project chooses to add — every Kyne project has access to it without an explicit import step for its most fundamental facilities (Core), exactly as a language's own keywords require no import. It is specified with the same rigor, and is subject to the same KIP governance, as the language itself.

**Relationship to Soroban.** Every Layer 2 module in this specification is, at the implementation level, backed by a Soroban host function or host-provided guarantee. This document deliberately does not mirror the Soroban SDK's own API shape — per [Standard Library Philosophy Principle 2](#2-blockchain-native) below, every API here is designed around how a Kyne contract author thinks about a problem, with the Soroban mapping chosen to serve that design, not the reverse.

**Relationship to community packages.** Kyne's package management is explicitly out of scope for v1, per [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem). Until a package system exists, "community packages" cannot yet be a real distribution mechanism — but this document is written anticipating one: everything in this specification is what a contract author cannot reasonably be expected to write, or reuse consistently, without a package ecosystem; everything else, including the collection-processing capabilities a package ecosystem could eventually provide via ordinary Kyne libraries, is deliberately left out.

**Why the library intentionally remains small.** Every API added to the standard library is added forever, in the same sense every keyword in [LANGUAGE_SPEC.md §1.3](./LANGUAGE_SPEC.md#13-keywords) is added forever — it becomes part of what every future Kyne compiler must support, what every future contract author must learn to recognize (even if they never call it), and what every future audit must account for as part of the trusted base. [LANGUAGE_SPEC.md's Small Language Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) applies to this document with equal force: an API belongs here only if nearly every contract needs it, not merely if it would be convenient for some contracts to have.

## The Five Standard Library Principles

### 1. Small by Default

Only functionality that nearly every contract requires belongs in the standard library. A capability useful to a minority of contracts — however useful to that minority — belongs in a community package once Kyne's package ecosystem exists, not in the trusted, always-available base every contract carries. Several modules in this specification ([§8](#8-event-module), [§15](#15-error-utilities)) are, by design, smaller than a conventional standard library's equivalent module, precisely because this principle was applied honestly rather than nominally — see each module's own reasoning.

### 2. Blockchain Native

Every API in this specification is designed around smart contract development, not around wrapping the Soroban SDK mechanically. Where the Soroban SDK exposes a capability in a general, low-level shape, Kyne's standard library exposes the *contract-relevant* shape of that capability — for example, [§11](#11-math-module)'s `mul_div` exists because overflow-safe ratio computation is a real, common smart-contract need, not because Soroban happens to expose 128-bit multiplication.

### 3. Safe by Construction

The safe API MUST always be the easiest API to reach for. Where a genuinely unsafe or dangerous operation would otherwise be offered, this specification either omits it entirely (per [§14](#14-crypto-module)'s prohibition on insecure primitives) or names it in a way that makes its risk unmistakable at the call site (per [§11](#11-math-module)'s `wrapping_*`/`saturating_*` naming), consistent with [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience).

### 4. Consistent

Every module in this specification follows the same naming pattern (`module.verb_object`), the same error-handling pattern (fallible operations return `Result`, never a sentinel value or a silent default), and the same documentation shape (per [APIs as Behavioral Specifications](#apis-as-behavioral-specifications)). A developer who has learned one module's conventions has learned every module's conventions.

### 5. Deterministic

No standard library API may introduce nondeterministic behavior. There is no networking, no randomness, no filesystem access, and no operating system interaction anywhere in this specification — every module is either a pure computation over its inputs or a deterministic query against the execution context [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context) already defines. This is not a stylistic preference; it is a hard requirement inherited directly from [RUNTIME_MODEL.md §2.1](./RUNTIME_MODEL.md#21-every-execution-is-deterministic) — a standard library API that introduced nondeterminism would make every contract that calls it a non-conforming Kyne program.

---

# Overall Library Architecture

The standard library is organized into three strictly layered conceptual layers.

```
Layer 1 — Core
    ↓
Layer 2 — Blockchain
    ↓
Layer 3 — Utilities
```

**Layer 1 — Core.** General language facilities with **no dependency on Soroban's execution context whatsoever**. Core modules would, in principle, make sense in a Kyne program that was not a blockchain contract at all — they concern values, computation, and control flow, not the chain.

**Layer 2 — Blockchain.** Soroban-specific functionality: storage, authentication, ledger context, time, events, and cryptography. Every module here depends on the execution context [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context) defines and has no meaning outside a contract invocation.

**Layer 3 — Utilities.** Debugging and testing facilities whose entire purpose is to support contract *development*, not contract *execution* — every Layer 3 API either has no effect on deployed, optimized contract behavior at all ([§13](#13-debug-module)) or exists purely to support a separate, off-chain testing context ([§14](#14-test-module)).

## Dependency Direction

Dependency flows in exactly one direction: **Layer 3 MAY depend on Layer 2 and Layer 1. Layer 2 MAY depend on Layer 1 only. Layer 1 MUST NOT depend on Layer 2 or Layer 3.** This mirrors the acyclic, forward-only dependency discipline [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture) already applies to the compiler's own internal crates, applied here to the standard library's module graph. A Core module that depended on Blockchain would mean a "general language facility" secretly required a live contract execution context to function correctly — which would mean it was never actually a Core facility, and belongs in Layer 2 instead. This rule is what prevents circular dependencies from ever becoming possible: a cycle would require some Layer 1 module to depend on a Layer 2 or Layer 3 module, which this rule forbids outright.

## Module-to-Layer Assignment

| Layer | Modules |
|---|---|
| 1 — Core | `core/`, `collections/`, `math/`, `convert/` |
| 2 — Blockchain | `storage/`, `auth/`, `ledger/`, `time/`, `events/`, `crypto/` |
| 3 — Utilities | `debug/`, `test/` |

---

# 2. Library Organization

```
std/
  core/
  collections/
  math/
  convert/
  storage/
  auth/
  ledger/
  time/
  events/
  crypto/
  debug/
  test/
```

| Module | Layer | Responsibility |
|---|---|---|
| `core/` | 1 | Assertions, panics, and unconditional-failure markers — the language-level facilities every contract implicitly relies on. |
| `collections/` | 1 | Ergonomic helpers over `list<T>` and `map<K, V>` beyond the intrinsic method tables already fixed by [LANGUAGE_SPEC.md §6.2](./LANGUAGE_SPEC.md#62-collections). |
| `math/` | 1 | Checked, wrapping, and saturating arithmetic helpers, and ratio/percentage computation, over Kyne's existing numeric types. |
| `convert/` | 1 | Explicit, closed-set conversions between primitive types and `string`/`bytes`. |
| `storage/` | 2 | Persistent storage lifetime management — **not** an alternative way to read or write `state`, per [§4](#4-storage-module)'s reasoning. |
| `auth/` | 2 | A non-consuming, read-only authorization query complementing the `auth(...)` keyword, per [§5](#5-authentication-module)'s reasoning. |
| `ledger/` | 2 | Direct, named accessors for the execution context [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context) already defines but does not name. |
| `time/` | 2 | Ergonomic, derived helpers over `ledger.timestamp()` — not a second clock. |
| `events/` | 2 | Intentionally near-empty, per [§8](#8-event-module)'s reasoning — the `event`/`emit` keywords already exhaust this concern. |
| `crypto/` | 2 | Hashing, signature verification, and encoding, delegated entirely to Soroban's audited host cryptography. |
| `debug/` | 3 | Development-only inspection output, fully elided in optimized builds. |
| `test/` | 3 | Conceptual primitives for mocking execution context in an off-chain test harness. |

**Dependency rules.** Every module MUST respect the layer-ordering rule in [Overall Library Architecture](#overall-library-architecture). Within a layer, modules MAY depend on one another only where this document explicitly says so (for example, `time/` depends on `ledger/`, per [§7](#7-time-module)) — an undocumented cross-module dependency within a layer is not permitted, keeping each module's own responsibility boundary sharp.

---

# APIs as Behavioral Specifications

Every conceptual API described in this document is specified using the following fixed shape: **Purpose**, **Parameters**, **Return value**, **Failure behavior**, **Runtime guarantees**, **Memory behavior**, **Storage behavior** (where applicable), **Security considerations**, **Time complexity**, and a short usage example. No API in this document is implemented — every description states the observable contract a conforming implementation MUST uphold, exactly as [MEMORY_MODEL.md](./MEMORY_MODEL.md) states observable memory contracts without prescribing an algorithm.

---

# 3. Core Module

### `core.panic(message: string) -> !`

| Field | Specification |
|---|---|
| Purpose | Unconditionally abort the current invocation. |
| Parameters | `message` — a diagnostic string, surfaced only to off-chain tooling (per [RUNTIME_MODEL.md §9.6](./RUNTIME_MODEL.md#96-observable-behavior), never part of on-chain, consensus-relevant state). |
| Return value | Never returns — its type is the bottom type, usable anywhere an expression of any type is expected, per [LANGUAGE_SPEC.md §7.7](./LANGUAGE_SPEC.md#77-pattern-matching)'s treatment of diverging constructs. |
| Failure behavior | Always a Runtime Failure, per [RUNTIME_MODEL.md §9.4](./RUNTIME_MODEL.md#94-runtime-failure) — never catchable, always rolls back the entire top-level transaction. |
| Runtime guarantees | Deterministic: given the same reachability, `panic` always fires. |
| Memory behavior | No allocation; any staged state from the current and enclosing invocations is discarded per [RUNTIME_MODEL.md §9.5](./RUNTIME_MODEL.md#95-scoped-rollback-and-catching). |
| Storage behavior | Not applicable — see Memory behavior. |
| Security considerations | None — `panic` cannot be misused to bypass a guarantee; it can only halt execution. |
| Time complexity | O(1). |
| Example | `if amount < 0 { core.panic("amount must not be negative"); }` |

### `core.assert(condition: bool, message: string) -> unit`

| Field | Specification |
|---|---|
| Purpose | Assert that `condition` holds; abort if it does not. |
| Parameters | `condition` — the boolean expression to check. `message` — a diagnostic string. |
| Return value | `unit` if `condition` is `true`. |
| Failure behavior | Calls `core.panic(message)` if `condition` is `false` — identical Runtime Failure semantics. |
| Runtime guarantees | Deterministic. |
| Memory behavior | No allocation beyond evaluating `condition`. |
| Storage behavior | Not applicable. |
| Security considerations | `assert` is appropriate for invariants a contract author believes can never be false; it is not a substitute for `throw`-based validation of caller-supplied input, which SHOULD produce a Recoverable Error instead, per [RUNTIME_MODEL.md §9.2](./RUNTIME_MODEL.md#92-recoverable-error), so a caller can respond to it as an ordinary value rather than losing the entire transaction unconditionally. |
| Time complexity | O(1) plus the complexity of evaluating `condition`. |
| Example | `core.assert(total_supply >= 0, "total supply invariant violated");` |

### `core.assert_eq(left: T, right: T, message: string) -> unit`

| Field | Specification |
|---|---|
| Purpose | Assert that `left` and `right` are structurally equal. |
| Parameters | `left`, `right` — any type `T` for which `==` is defined, per [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)'s automatic structural equality. `message` — a diagnostic string. |
| Return value | `unit` if `left == right`. |
| Failure behavior | Calls `core.panic(message)` if `left != right`. |
| Runtime guarantees | Deterministic, since `==` itself is deterministic. |
| Memory behavior | No allocation beyond the underlying `==` comparison. |
| Storage behavior | Not applicable. |
| Security considerations | Same guidance as `assert`. |
| Time complexity | O(1) for primitives; proportional to structural size for `struct`/`enum`/collection types, since structural equality compares every field/element. |
| Example | `core.assert_eq(balance_of(admin), expected, "unexpected admin balance");` |

### `core.todo(message: string) -> !`

| Field | Specification |
|---|---|
| Purpose | Mark a code path as not yet implemented. |
| Parameters | `message` — a description of what remains to be implemented. |
| Return value | Never returns; bottom type, identical to `panic`. |
| Failure behavior | Always a Runtime Failure, identical to `core.panic`. |
| Runtime guarantees | Deterministic. |
| Memory behavior | Identical to `core.panic`. |
| Storage behavior | Not applicable. |
| Security considerations | A `core.todo()` call reachable from any `public fn` entry point in a project intended for deployment indicates incomplete work; a conforming Security Analyzer implementation ([COMPILER_ARCHITECTURE.md §11](./COMPILER_ARCHITECTURE.md#11-security-analyzer)) SHOULD flag it, though this document does not amend COMPILER_ARCHITECTURE.md's fixed analysis list to require it. |
| Time complexity | O(1). |
| Example | `public fn liquidate(position_id: u64) -> Result<bool, LiquidationError> { core.todo("liquidation logic pending audit"); }` |

### `core.unreachable(message: string) -> !`

| Field | Specification |
|---|---|
| Purpose | Assert that a code path can never execute. |
| Parameters | `message` — an explanation of why this path is believed unreachable. |
| Return value | Never returns; bottom type, identical to `panic`. |
| Failure behavior | Always a Runtime Failure if actually reached. |
| Runtime guarantees | Deterministic. |
| Memory behavior | Identical to `core.panic`. |
| Storage behavior | Not applicable. |
| Security considerations | Distinct from `todo` in intent, not in mechanism: `todo` marks known-incomplete work; `unreachable` marks a path the author asserts is impossible given the surrounding logic — reaching it indicates a defect in that reasoning, not missing work. |
| Time complexity | O(1). |
| Example | Used in the fall-through arm of a `match` the author believes is already exhaustively handled by preceding arms for all reachable inputs, distinct from the compiler's own exhaustiveness requirement, which already forces every `match` to cover every variant per [LANGUAGE_SPEC.md §7.7](./LANGUAGE_SPEC.md#77-pattern-matching). |

## Why `print()` Is Not a Core API

The brief for this module anticipated a development-only `print()` function in Core. This specification places development-time output exclusively in [`debug.print()`](#13-debug-module) instead, and Core intentionally has no `print` of its own. Two names for the identical capability — one in `core/`, one in `debug/` — would violate [One Obvious Way](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place), and `debug/` is the correct home on responsibility grounds: development-only output that must be fully elided from optimized builds ([§13](#13-debug-module)) belongs with the other development-only, elided facilities, not in the module every contract depends on unconditionally.

---

# 4. Storage Module

## Why This Module Does Not Provide `get`/`set`/`has`/`remove`/`clear`

The brief for this module anticipated a conventional key-value storage API (`storage.get()`, `storage.set()`, `storage.has()`, `storage.remove()`, `storage.clear()`) operating on arbitrary keys. This specification deliberately does **not** provide one, because doing so would directly contradict [RUNTIME_MODEL.md §2.2](./RUNTIME_MODEL.md#22-every-state-change-is-explicit): "a `state` field changes value if and only if a Kyne statement in the executing contract's own source explicitly assigns to it." A generic, arbitrary-key storage API would let a contract read and write persistent storage under keys that were never declared via `state`, bypassing [LANGUAGE_SPEC.md §4.4](./LANGUAGE_SPEC.md#44-definite-assignment-of-state)'s definite-assignment checking and [§3.2](./LANGUAGE_SPEC.md#32-canonical-member-order)'s canonical-ordering guarantee that every field an auditor needs to find is declared in one visible place. It would, in effect, create a second, unaudited way to touch persistent storage alongside the audited one `state` already provides — precisely the kind of redundant, competing mechanism [One Obvious Way](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) forbids.

The capabilities the brief's five function names gesture at already exist, correctly scoped, through existing mechanisms:

| Requested capability | Existing mechanism |
|---|---|
| `get` | Reading a `state` field by name, or `map<K,V>.get(key)` / `list<T>.get(index)`, per [LANGUAGE_SPEC.md §6.2](./LANGUAGE_SPEC.md#62-collections). |
| `set` | Assigning to a `state` field, or `map<K,V>.set(key, value)`. |
| `has` | `map<K,V>.has(key)`, for a collection-typed `state` field. For a scalar `state` field, existence is unconditionally guaranteed once past [definite assignment](./LANGUAGE_SPEC.md#44-definite-assignment-of-state) — "has" has no meaning for it. |
| `remove` | `map<K,V>.remove(key)`, for a collection-typed `state` field. A scalar `state` field cannot be removed, only reassigned — Kyne's `state` model has no undeclare operation. |
| `clear` | **Not provided.** There is no operation in Kyne v1 to delete a `state` field's underlying ledger entry outright, as distinct from reassigning it to an empty or default value. Ledger-level entry deletion, as a capability distinct from reassignment, is reserved for future evaluation per [§22](#22-future-library-evolution). |

## What This Module Actually Provides: Storage Lifetime Management

[RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) explicitly reserves the mechanism for extending a contract instance's storage lifetime, ahead of Soroban's state-expiration archival, to this document: "storage entries that are not kept alive (through whatever explicit lifetime-extension mechanism the standard library exposes — a stdlib-level concern, deferred to a future Standard Library Specification) MAY become archived." This is the Storage module's real, substantive purpose.

### `storage.ttl() -> u32`

| Field | Specification |
|---|---|
| Purpose | Query the current remaining time-to-live, in ledger sequence terms, of the contract instance's persistent storage. |
| Parameters | None. |
| Return value | `u32` — the number of ledger sequences remaining before the instance's storage becomes eligible for archival, per [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle). |
| Failure behavior | Never fails — this is a pure, always-available query. |
| Runtime guarantees | Deterministic within a given ledger; the same call within the same invocation always returns the same value. |
| Memory behavior | No allocation beyond the returned `u32`. |
| Storage behavior | Read-only; does not participate in the [Storage Lifecycle](./MEMORY_MODEL.md#9-storage-lifecycle) staging/commit sequence, since it reads metadata about storage, not a `state` field's value. |
| Security considerations | Informational only — has no bearing on authorization or state correctness. |
| Time complexity | O(1). |
| Example | `if storage.ttl() < RENEWAL_THRESHOLD { storage.extend_ttl(TARGET_TTL); }` |

### `storage.extend_ttl(extend_to: u32) -> unit`

| Field | Specification |
|---|---|
| Purpose | Request that the contract instance's persistent storage lifetime be extended. |
| Parameters | `extend_to` — the target minimum remaining time-to-live, in ledger sequence terms. |
| Return value | `unit`. |
| Failure behavior | A request for a shorter-than-current extension is a no-op, never an error; a request exceeding the Soroban host's own maximum permitted extension is a Runtime Failure, per [RUNTIME_MODEL.md §9.4](./RUNTIME_MODEL.md#94-runtime-failure). |
| Runtime guarantees | The extension request is staged and committed exactly like any other invocation effect: if the invocation that called `extend_ttl` ultimately fails and rolls back, the extension request MUST be discarded along with every other staged effect of that invocation, per [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline). |
| Memory behavior | No allocation. |
| Storage behavior | Operates on the contract instance's storage lifetime metadata as a whole — it is not scoped to an individual `state` field, since Kyne's `state` fields are not individually addressable as first-class values outside their declaring contract. |
| Security considerations | Extension consumes resources metered by the Soroban host; a contract SHOULD extend conservatively rather than requesting the maximum possible extension on every invocation. |
| Time complexity | O(1). |
| Example | See above. |

---

# 5. Authentication Module

## Why This Module Does Not Duplicate `auth(...)`

The brief for this module anticipated `auth.require()`, `auth.optional()`, and `auth.verify()`. `auth.require()` would be a direct duplicate of the `auth(addr)` **language keyword** already locked by [LANGUAGE_SPEC.md §10.1](./LANGUAGE_SPEC.md#101-the-auth-statement) and [RUNTIME_MODEL.md §12](./RUNTIME_MODEL.md#12-authorization-model) — introducing a second spelling for the identical mandatory, aborting, transaction-critical operation would violate [One Obvious Way](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) and is not offered. `auth(addr)` remains the sole, canonical way to require authorization in Kyne, and — per [LANGUAGE_SPEC.md §10.1](./LANGUAGE_SPEC.md#101-the-auth-statement) — it remains a statement usable only within a contract member function; it cannot be wrapped by a stdlib function at all, since library code is categorically forbidden from calling it.

What the brief's `optional` and `verify` names gesture at is a genuinely different, additive capability this module does provide: a **non-consuming, read-only query** — "would `auth(addr)` succeed right now, without actually requiring it or aborting if it would not."

### `auth.has(addr: address) -> bool`

| Field | Specification |
|---|---|
| Purpose | Query whether a valid, unconsumed authorization entry currently exists for `addr`, without requiring it and without aborting if it does not. |
| Parameters | `addr` — the `address` to check. |
| Return value | `bool` — `true` if `addr` has a valid, currently-applicable authorization entry for this invocation; `false` otherwise. |
| Failure behavior | Never fails — this is a pure query, in contrast to `auth(addr)`, which aborts on a negative result. |
| Runtime guarantees | Deterministic: reflects exactly the same authorization entries [RUNTIME_MODEL.md §12.1–§12.2](./RUNTIME_MODEL.md#121-identity-verification)'s Authorization pipeline stage already verified for the enclosing transaction. |
| Memory behavior | No allocation beyond the returned `bool`. |
| Storage behavior | Not applicable. |
| Security considerations | `auth.has` MUST NOT be used as a substitute for `auth(addr)` where authorization is actually required for correctness — since it does not abort on failure, code that checks `auth.has(addr)` and proceeds regardless of the result has not actually enforced anything. Its legitimate use is choosing between multiple optional signers (for example, "if the admin authorized this call, apply an admin discount; otherwise proceed as an ordinary call") where no single address's authorization is unconditionally required. |
| Time complexity | O(1). |
| Example | `let is_admin_call = auth.has(admin); let fee = if is_admin_call { 0 } else { standard_fee };` |

Unlike `storage/`, this module is available to both contract member functions and library free functions, since it performs no side-effecting consumption of an authorization entry and therefore does not require the invocation-scoping `auth(addr)` itself needs.

---

# 6. Ledger Module

This module provides the named accessors [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context) requires to exist, but explicitly leaves unnamed: "the exact standard-library function names for network, ledger, transaction, and timestamp access are a Standard Library Specification concern... not fixed by [RUNTIME_MODEL.md]."

### `ledger.sequence() -> u32`

| Field | Specification |
|---|---|
| Purpose | Return the current ledger's sequence number. |
| Parameters | None. |
| Return value | `u32`. |
| Failure behavior | Never fails. |
| Runtime guarantees | Deterministic within a given invocation and stable across the entire transaction, per [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context). |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | Suitable for deadline and ordering logic; MUST NOT be treated as a source of unpredictability or randomness, per [Standard Library Principle 5](#5-deterministic). |
| Time complexity | O(1). |
| Example | `let deadline = ledger.sequence() + 100;` |

### `ledger.timestamp() -> u64`

| Field | Specification |
|---|---|
| Purpose | Return the current ledger's close time, as a Unix timestamp in seconds. |
| Parameters | None. |
| Return value | `u64`. |
| Failure behavior | Never fails. |
| Runtime guarantees | Deterministic and stable across the transaction, identical in character to `ledger.sequence()`. |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | The sole legitimate source of "current time" in Kyne — see [§7](#7-time-module). |
| Time complexity | O(1). |
| Example | `state created_at: u64 = 0;` … `created_at = ledger.timestamp();` |

### `ledger.network() -> string`

| Field | Specification |
|---|---|
| Purpose | Return an identifier for the network the current transaction was submitted to. |
| Parameters | None. |
| Return value | `string`. |
| Failure behavior | Never fails. |
| Runtime guarantees | Deterministic and stable across the transaction. |
| Memory behavior | No allocation beyond the returned `string`. |
| Storage behavior | Not applicable. |
| Security considerations | Useful for contracts that must refuse to operate on an unintended network (for example, guarding a migration or test-only code path); not a substitute for `auth`-based access control. |
| Time complexity | O(1). |
| Example | `if ledger.network() != EXPECTED_NETWORK { throw ContractError::WrongNetwork; }` |

### `ledger.protocol() -> u32`

| Field | Specification |
|---|---|
| Purpose | Return the Soroban protocol version the current ledger is operating under. |
| Parameters | None. |
| Return value | `u32`. |
| Failure behavior | Never fails. |
| Runtime guarantees | Deterministic and stable across the transaction. |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | Useful for contracts whose logic depends on protocol-version-gated host behavior. |
| Time complexity | O(1). |
| Example | `if ledger.protocol() < MIN_SUPPORTED_PROTOCOL { throw ContractError::UnsupportedProtocol; }` |

---

# 7. Time Module

**Time comes only from the blockchain, never from the operating system.** Every function in this module is a pure, deterministic computation over [`ledger.timestamp()`](#6-ledger-module)'s value; this module introduces no second source of "now."

### `time.now() -> u64`

| Field | Specification |
|---|---|
| Purpose | Return the current ledger time. |
| Parameters | None. |
| Return value | `u64` — defined to be exactly `ledger.timestamp()`'s value. This is not an independent clock; it exists in `time/` rather than `ledger/` because the remaining functions in this module are ergonomic, derived operations that belong together with it. |
| Failure behavior | Never fails. |
| Runtime guarantees | Identical to `ledger.timestamp()`. |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | Identical to `ledger.timestamp()`. |
| Time complexity | O(1). |
| Example | `let now = time.now();` |

### `time.before(target: u64) -> bool` / `time.after(target: u64) -> bool`

| Field | Specification |
|---|---|
| Purpose | Compare the current ledger time against `target`. |
| Parameters | `target` — a Unix timestamp in seconds. |
| Return value | `bool` — `time.before(target)` is `time.now() < target`; `time.after(target)` is `time.now() > target`. |
| Failure behavior | Never fails. |
| Runtime guarantees | Deterministic, following `time.now()`. |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | The natural building block for deadline and vesting logic. |
| Time complexity | O(1). |
| Example | `if time.after(unlock_time) { release_funds(beneficiary); }` |

### `time.elapsed(since: u64) -> u64`

| Field | Specification |
|---|---|
| Purpose | Compute the duration, in seconds, that has passed since `since`. |
| Parameters | `since` — a Unix timestamp in seconds, expected to be in the past. |
| Return value | `u64` — `time.now() - since`, **saturating to `0`** if `since` is in the future. |
| Failure behavior | Never fails — this function deliberately does not adopt Kyne's default checked-subtraction-panics behavior (per [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic)); a negative elapsed duration has an unambiguous, safe interpretation (zero elapsed time), and panicking on it would make ordinary, non-adversarial comparisons ("has enough time passed since X") needlessly fragile. This exception is stated explicitly here, by name, exactly as [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic) requires of any function that deviates from checked-by-default arithmetic. |
| Runtime guarantees | Deterministic. |
| Memory behavior | No allocation. |
| Storage behavior | Not applicable. |
| Security considerations | Suitable for cooldown and rate-limiting logic. |
| Time complexity | O(1). |
| Example | `if time.elapsed(last_claim) >= COOLDOWN_SECONDS { allow_claim(); }` |

---

# 8. Event Module

## Why This Module Adds Almost Nothing

The brief for this module anticipated `event.emit()`, `event.topic()`, and `event.data()`. This specification provides none of them, and the reasoning is worth stating in full, since an empty-looking module is an unusual outcome that deserves explicit justification rather than silent omission.

`event.emit()` would duplicate the `emit` **keyword** already locked by [LANGUAGE_SPEC.md §11.2](./LANGUAGE_SPEC.md#112-emission) — the same One-Obvious-Way violation already ruled out for `auth.require()` in [§5](#5-authentication-module).

`event.topic()` and `event.data()` would require exposing a topic/data split that does not exist at the Kyne source level at all: [LANGUAGE_SPEC.md §11.1](./LANGUAGE_SPEC.md#111-declaration)'s `event` declarations are flat, positional parameter lists with no topic/data distinction — how a declared event's fields map onto Soroban's own topic/data event structure is a Rust Code Generation decision governed by [COMPILER_ARCHITECTURE.md §14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation), not something Kyne source expresses or a stdlib function could meaningfully parameterize.

A read-back or query capability — "what events has this invocation emitted so far" — is not offered because [RUNTIME_MODEL.md §11.4](./RUNTIME_MODEL.md#114-guarantees) forbids it outright: "A conforming runtime MUST NOT provide any mechanism by which contract code queries previously emitted events." Unlike [§5](#5-authentication-module)'s `auth.has`, there is no safe, non-side-effecting query this module could legitimately add here, because RUNTIME_MODEL.md has already ruled out the one capability that would justify it.

**This module's entire specification is therefore: use the `event` and `emit` keywords, exactly as [LANGUAGE_SPEC.md §11](./LANGUAGE_SPEC.md#11-events) and [RUNTIME_MODEL.md §11](./RUNTIME_MODEL.md#11-event-model) already define them.** This is [Small by Default](#1-small-by-default) applied honestly: a module with nothing genuinely additive to offer is correctly specified as offering nothing, not padded with redundant or forbidden capabilities to appear complete.

---

# 9. Crypto Module

## Delegation to Host Cryptography

Every function in this module MUST be implemented by delegating entirely to the Soroban host's own audited, deterministic cryptographic functions. **The Kyne compiler MUST NOT implement any cryptographic primitive itself** — not for performance, not for a missing host capability, and not as a fallback. This is a permanent rule: rolling one's own cryptography is a well-established source of real-world vulnerabilities, and a language whose primary claim is security-by-construction cannot make an exception for the one domain where that claim is hardest to verify by inspection. It is also a determinism requirement: the Soroban host's cryptographic functions are guaranteed identical across every validator, per [RUNTIME_MODEL.md §2.1](./RUNTIME_MODEL.md#21-every-execution-is-deterministic); an independently implemented primitive would carry no such guarantee.

**Never expose insecure primitives.** This module MUST NOT, now or in any future version, expose an algorithm not currently backed by an audited Soroban host function — concretely, this permanently excludes cryptographic primitives known to be broken or deprecated for the purposes this module serves (for example, MD5 or SHA-1 for any security-relevant hashing purpose), regardless of any future request for them.

### `crypto.hash(data: bytes) -> bytes<32>`

| Field | Specification |
|---|---|
| Purpose | Compute a cryptographic hash of `data` using Kyne's standard hash algorithm (SHA-256, matching Soroban's own host-provided hashing function). |
| Parameters | `data` — the byte sequence to hash. |
| Return value | `bytes<32>` — a fixed-size 32-byte digest. |
| Failure behavior | Never fails — hashing is total over all `bytes` inputs. |
| Runtime guarantees | Fully deterministic: identical `data` always produces an identical digest, on every conforming implementation. |
| Memory behavior | Allocates only the fixed-size `bytes<32>` result, per [MEMORY_MODEL.md §12](./MEMORY_MODEL.md#12-collections)'s treatment of `bytes<N>` as stack-eligible. |
| Storage behavior | Not applicable. |
| Security considerations | Suitable for commit-reveal schemes, content addressing, and integrity checks. |
| Time complexity | O(n) in the length of `data`. |
| Example | `let commitment = crypto.hash(secret_bytes);` |

### `crypto.verify_signature(public_key: bytes, message: bytes, signature: bytes) -> bool`

| Field | Specification |
|---|---|
| Purpose | Verify that `signature` is a valid signature over `message` under `public_key`, using Kyne's standard signature scheme (Ed25519, matching Soroban's own host-provided verification function). |
| Parameters | `public_key`, `message`, `signature` — byte sequences in the encoding the underlying host function requires. |
| Return value | `bool` — `true` if the signature is valid, `false` otherwise. This function returns a `bool` rather than aborting on an invalid signature, since signature verification is frequently one input to a larger authorization decision, not itself the entirety of one — a contract combining a signature check with other logic needs the boolean result, not an unconditional abort. |
| Failure behavior | Never fails in the Runtime Failure sense for a malformed-but-well-typed input; returns `false` for an invalid signature rather than aborting. |
| Runtime guarantees | Fully deterministic. |
| Memory behavior | No allocation beyond the `bool` result. |
| Storage behavior | Not applicable. |
| Security considerations | This function verifies a cryptographic signature; it does **not** perform Soroban-native authorization, which remains the sole responsibility of the `auth(...)` keyword per [LANGUAGE_SPEC.md §10](./LANGUAGE_SPEC.md#10-authentication) — the two are complementary, not interchangeable: `auth(...)` should be preferred for ordinary caller authorization, while `crypto.verify_signature` serves cases requiring verification of an externally-supplied, off-chain-signed payload (for example, a signed price attestation from an oracle). |
| Time complexity | O(n) in the length of `message`, dominated by a constant-cost signature verification operation. |
| Example | `if !crypto.verify_signature(oracle_pubkey, price_payload, signature) { throw OracleError::InvalidSignature; }` |

---

# 10. Collections Module

## Scope Limitation: No Higher-Order Functions

The brief for this module anticipated general-purpose `filter`, `map`, and `sort`-by-comparator operations. **Kyne v1 has no function-value type** — `fn` is a declaration, never a value, per the grammar in [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar), and there is no closure or lambda syntax anywhere in the language. A generic `filter(items: list<T>, predicate: fn(T) -> bool)` is therefore not expressible as a Kyne function signature at all — this is a direct, mechanical consequence of decisions already locked in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), not an oversight of this document.

This module instead provides a small, closed set of concretely specified operations sufficient for common contract logic. A developer needing processing beyond this set SHOULD express it directly as an explicit `for` loop, which — per [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first) — remains fully readable and auditable without requiring higher-order abstraction. Generic higher-order collection operations are reserved as future work, contingent on Kyne someday introducing function values, per [§22](#22-future-library-evolution).

## Scope Limitation: No Map Enumeration

This module also does not provide any function that enumerates a `map<K, V>`'s keys, values, or entries. [LANGUAGE_SPEC.md §6.2.2](./LANGUAGE_SPEC.md#622-mapk-v-methods) explicitly reserves map iteration for a future version behind an `.entries()`-style API; a stdlib function that achieved the equivalent effect (for example, `collections.keys(m) -> list<K>`) would silently work around that reservation rather than respecting it. This module MUST NOT provide such a function until [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) itself is amended by KIP to support map iteration.

### `list<T>` helpers

| Function | Purpose | Complexity |
|---|---|---|
| `collections.contains(items: list<T>, value: T) -> bool` | Whether `value` structurally equals ([LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)) any element of `items`. | O(n) |
| `collections.index_of(items: list<T>, value: T) -> Option<u32>` | The index of the first element structurally equal to `value`, or `None`. | O(n) |
| `collections.reverse(items: list<T>) -> list<T>` | A new list with `items`'s elements in reverse order. | O(n) |
| `collections.is_empty(items: list<T>) -> bool` | Whether `items.len() == 0`. | O(1) |
| `collections.sort_ascending(items: list<T>) -> list<T>` | A new list sorted ascending. **Restricted to numeric element types** (`i32`/`i64`/`i128`/`u32`/`u64`/`u128`), since `<`/`>` are defined only for numeric types per [LANGUAGE_SPEC.md §7.3](./LANGUAGE_SPEC.md#73-comparison-boolean-and-logical-expressions). | O(n log n) |
| `collections.sort_descending(items: list<T>) -> list<T>` | As above, descending. | O(n log n) |

Every function above MUST return a new `list<T>` (or `Option`/`bool`) rather than mutating `items` in place, consistent with [Value Semantics](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only) — the input `items` remains fully intact and independently usable after any of these calls.

### `map<K, V>` helpers

| Function | Purpose | Complexity |
|---|---|---|
| `collections.is_empty(m: map<K, V>) -> bool` | Whether `m.len() == 0`. | O(1) |

This is the only `map<K, V>` helper this module provides, for the reasons stated in [Scope Limitation: No Map Enumeration](#scope-limitation-no-map-enumeration) above — every other conceivable helper would require enumerating the map's contents.

**Runtime guarantees, memory behavior, and determinism** for every function in this table are identical across the whole module: every function is a pure, deterministic computation over its input with no storage interaction, and — per [MEMORY_MODEL.md §12](./MEMORY_MODEL.md#12-collections) — obeys ordinary value semantics for its result.

---

# 11. Math Module

## Overflow Policy

Bare arithmetic operators (`+`, `-`, `*`, `/`, `%`) remain checked by default and panic on overflow, exactly as fixed by [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic) — this module does not, and cannot, change that default. This module instead names the two categories of explicit deviation from that default that [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic) already anticipates by name (`wrapping_add`, `saturating_sub`), plus a third, additive category: a checked variant that converts an overflow into a `None` value rather than a panic, letting a developer route an overflow into a Recoverable Error via `throw` rather than an uncatchable Runtime Failure, where that is the more appropriate response for a given contract.

### Per-type function families

For every integer type `T` in `{i32, i64, i128, u32, u64, u128}`, this module provides:

| Function | Behavior on overflow |
|---|---|
| `math.checked_add_T(a: T, b: T) -> Option<T>` (and `_sub`, `_mul`) | Returns `None` instead of panicking. |
| `math.wrapping_add_T(a: T, b: T) -> T` (and `_sub`, `_mul`) | Wraps around the type's range. |
| `math.saturating_add_T(a: T, b: T) -> T` (and `_sub`, `_mul`) | Clamps to the type's minimum or maximum value. |

(Per-type naming — `checked_add_i128`, `checked_add_u64`, and so on — is required by [LANGUAGE_SPEC.md §5.5](./LANGUAGE_SPEC.md#55-no-overloading)'s prohibition on function overloading: Kyne has no mechanism for one function name to serve multiple concrete types, so this family is, by necessity, one concretely named function per type per operation, not a single generic signature.)

Every function above is specified as follows:

| Field | Specification |
|---|---|
| Purpose | Perform the named arithmetic operation with the named non-default overflow behavior. |
| Parameters | `a`, `b` — operands of type `T`. |
| Return value | `Option<T>` for `checked_*`; `T` for `wrapping_*`/`saturating_*`. |
| Failure behavior | Never a Runtime Failure — this is precisely the point of each variant: `checked_*` returns `None`, `wrapping_*`/`saturating_*` always return a value. |
| Runtime guarantees | Fully deterministic. |
| Memory behavior | No allocation beyond the primitive result. |
| Storage behavior | Not applicable. |
| Security considerations | `wrapping_*` and `saturating_*` deliberately mask overflow rather than reporting it; a contract using either MUST have a specific, documented reason a silent non-panicking result is safe in that context, since the whole-language default exists specifically to make overflow unmissable. |
| Time complexity | O(1). |
| Example | `let new_balance = match math.checked_sub_i128(balance, amount) { Some(value) => value, None => throw TokenError::InsufficientBalance }; ` |

### `math.mul_div(a: i128, b: i128, denominator: i128) -> Result<i128, MathError>`

| Field | Specification |
|---|---|
| Purpose | Compute `(a * b) / denominator` using an intermediate representation wide enough to avoid overflowing during the multiplication step, even when the final result would fit in `i128`. |
| Parameters | `a`, `b` — the values to multiply. `denominator` — the divisor. |
| Return value | `Result<i128, MathError>` — `Ok` with the computed value, or `Err(MathError::DivisionByZero)` if `denominator` is zero, or `Err(MathError::Overflow)` if the final result does not fit in `i128`. |
| Failure behavior | Never panics — every failure mode is a typed `Result::Err`, a Recoverable Error per [RUNTIME_MODEL.md §9.2](./RUNTIME_MODEL.md#92-recoverable-error). |
| Runtime guarantees | Fully deterministic. |
| Memory behavior | No allocation beyond the `Result<i128, MathError>` result. |
| Storage behavior | Not applicable. |
| Security considerations | This is the standard, safe building block for ratio and percentage computation — computing `a * b` and then dividing with two separate, bare operations risks an intermediate overflow that `mul_div` avoids by construction. Contracts computing fees, shares, or exchange rates SHOULD use `mul_div` rather than two bare operations. |
| Time complexity | O(1). |
| Example | `let fee = math.mul_div(amount, fee_basis_points, 10_000)?;` |

### `math.percentage_of(amount: i128, basis_points: u32) -> Result<i128, MathError>`

| Field | Specification |
|---|---|
| Purpose | Compute `amount * basis_points / 10,000` — a percentage of `amount`, expressed in basis points (hundredths of a percent) to avoid floating-point representation entirely, since Kyne has no floating-point type at all, per [LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types) and [RUNTIME_MODEL.md §1.1](./RUNTIME_MODEL.md#11-deterministic). |
| Parameters | `amount` — the base value. `basis_points` — the percentage, in basis points (`10_000` = 100%). |
| Return value | `Result<i128, MathError>`, defined identically to `math.mul_div(amount, basis_points, 10_000)`. |
| Failure behavior | Identical to `mul_div`. |
| Runtime guarantees | Fully deterministic. |
| Memory behavior | No allocation beyond the result. |
| Storage behavior | Not applicable. |
| Security considerations | Identical to `mul_div` — this function exists purely as a named, self-documenting convenience over it. |
| Time complexity | O(1). |
| Example | `let interest = math.percentage_of(principal, annual_rate_bps)?;` |

## On "Large Integer Helpers"

The brief's "large integer helpers" refers to ergonomic operations over `i128`/`u128` — the largest numeric types Kyne provides — not a new arbitrary-precision integer type. Kyne v1's closed primitive type set, fixed by [LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types), does not include an arbitrary-precision integer, and this module MUST NOT introduce one, since doing so would be a language-level type-system change outside this document's authority.

---

# 12. Conversion Module

**Conversions are always explicit, never implicit** — this module exists precisely because [LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types) forbids any implicit conversion between types, including between numeric types of different width or signedness. Every conversion in Kyne is a named function call, visible at its call site.

## Naming Pattern

Because Kyne has neither user-facing generics ([LANGUAGE_SPEC.md §6.7](./LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature)) nor function overloading ([LANGUAGE_SPEC.md §5.5](./LANGUAGE_SPEC.md#55-no-overloading)), this module cannot offer a single polymorphic `to_string(value: T)` or `parse<T>(s: string)` — every conversion is a separately named, concretely typed function, following the pattern `convert.<source>_to_<target>` for value-to-representation conversions and `convert.parse_<target>` for representation-to-value conversions.

| Function | Purpose | Failure behavior |
|---|---|---|
| `convert.i64_to_string(v: i64) -> string` (and equivalents for every integer type) | Render an integer as its decimal string representation. | Never fails. |
| `convert.bool_to_string(v: bool) -> string` | Render `"true"` or `"false"`. | Never fails. |
| `convert.address_to_string(a: address) -> string` | Render an address in its standard string encoding. | Never fails. |
| `convert.parse_i64(s: string) -> Result<i64, ParseError>` (and equivalents for every integer type) | Parse a decimal string as the named integer type. | `Err(ParseError::InvalidFormat)` on malformed input; `Err(ParseError::OutOfRange)` if the value does not fit the target type. |
| `convert.parse_bool(s: string) -> Result<bool, ParseError>` | Parse `"true"`/`"false"`. | `Err(ParseError::InvalidFormat)` otherwise. |
| `convert.parse_address(s: string) -> Result<address, ParseError>` | Parse an address from its standard string encoding. | `Err(ParseError::InvalidFormat)` otherwise. |
| `convert.bytes_to_hex(b: bytes) -> string` | Render a byte sequence as a lowercase hexadecimal string. | Never fails. |
| `convert.hex_to_bytes(s: string) -> Result<bytes, ParseError>` | Parse a hexadecimal string into bytes. | `Err(ParseError::InvalidFormat)` on malformed input. |

**On `parse` versus `try_parse`.** The brief anticipated both an infallible `parse` and a fallible `try_parse`. Kyne provides only the fallible form, named `parse_*`, returning `Result`. There is no infallible parsing variant that panics on malformed input, because string input is frequently attacker-influenced, and silently choosing to make bad input a Runtime Failure rather than a value a caller can handle would violate [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience) — the brief's two-name distinction collapses to a single, always-fallible family in Kyne.

Every function in this module shares the following non-varying fields: **Runtime guarantees** — fully deterministic. **Memory behavior** — allocates only its own return value, per [MEMORY_MODEL.md §12](./MEMORY_MODEL.md#12-collections)'s treatment of `string`/`bytes`. **Storage behavior** — not applicable; no conversion function touches `state`. **Security considerations** — parsing functions MUST validate their input fully before returning `Ok`; a parse function that accepts malformed input as a side effect of a lenient implementation is a defect. **Time complexity** — O(n) in the length of the string/byte input for every function in this module.

---

# 13. Debug Module

**Development-only.** Every function in this module has effect only in a development build and MUST be entirely elided — including the evaluation of its arguments — in an optimized/release build, producing zero generated Rust and zero WASM footprint.

**Why elision is always safe.** Kyne expressions cannot themselves produce side effects: assignment is a statement, never an expression, per [LANGUAGE_SPEC.md §7.1](./LANGUAGE_SPEC.md#71-precedence), and there is no other mechanism by which evaluating an expression could mutate `state` or emit an event. Because of this, eliding a `debug.*` call's arguments along with the call itself can never change any behavior [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) guarantees — debug output was never part of a contract's on-chain observable behavior to begin with, so removing it removes nothing [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)'s "optimizations must never change observable behavior" rule protects.

### `debug.print(message: string) -> unit`

| Field | Specification |
|---|---|
| Purpose | Emit a diagnostic string to the development environment's output. |
| Parameters | `message` — the string to print. |
| Return value | `unit`. |
| Failure behavior | Never fails, in a development build. Fully elided in an optimized build. |
| Runtime guarantees | Has no bearing on [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)'s guarantees at all — it is not part of the runtime model's observable-behavior surface. |
| Memory behavior | Allocates its `message` argument in a development build; allocates nothing in an optimized build, since the call and its arguments are removed entirely. |
| Storage behavior | Not applicable. |
| Security considerations | MUST NOT be relied upon for any contract-correctness purpose, since it does not exist at all in the deployed artifact. |
| Time complexity | O(1) plus the cost of formatting `message`, in a development build only. |
| Example | `debug.print("balance after transfer: " + convert.i128_to_string(balance));` |

### `debug.inspect(value: T) -> unit`

| Field | Specification |
|---|---|
| Purpose | Print a structured representation of `value`, of any type, for development-time inspection. |
| Parameters | `value` — any Kyne value. |
| Return value | `unit`. |
| Failure behavior | Identical to `debug.print`. |
| Runtime guarantees | Identical to `debug.print`. |
| Memory behavior | Identical to `debug.print`. |
| Storage behavior | Not applicable. |
| Security considerations | Identical to `debug.print`. |
| Time complexity | Proportional to the structural size of `value`, in a development build only. |
| Example | `debug.inspect(transaction);` |

### `debug.dump() -> unit`

| Field | Specification |
|---|---|
| Purpose | Print the enclosing contract's full current `state` for development-time inspection. |
| Parameters | None. |
| Return value | `unit`. |
| Failure behavior | Identical to `debug.print`. |
| Runtime guarantees | Identical to `debug.print`. |
| Memory behavior | Identical to `debug.print`, proportional to the contract's total `state` size, in a development build only. |
| Storage behavior | Read-only; does not participate in the [Storage Lifecycle](./MEMORY_MODEL.md#9-storage-lifecycle)'s staging/commit sequence in any build. |
| Security considerations | MUST NOT be reachable in a deployed contract's actual execution — because it is fully elided in optimized builds, this is guaranteed rather than merely conventional. |
| Time complexity | Proportional to total `state` size, in a development build only. |
| Example | `debug.dump();` |

---

# 14. Test Module

This module defines the **conceptual primitives** a testing framework needs to construct a deterministic, mocked execution context. It does not specify the test-runner CLI or framework mechanics that consume these primitives — that belongs to a future `TOOLCHAIN.md`, per [Non-Goals](#non-goals); this module fixes only the behavioral contract of the mocking primitives themselves.

| Function | Purpose |
|---|---|
| `test.mock_ledger(sequence: u32, timestamp: u64) -> unit` | Within a test harness, fix the values [`ledger.sequence()`](#6-ledger-module) and [`ledger.timestamp()`](#6-ledger-module) return for subsequent invocations in the same test. |
| `test.mock_auth(addr: address) -> unit` | Within a test harness, register `addr` as having a valid authorization entry, satisfying both `auth(addr)` and [`auth.has(addr)`](#5-authentication-module), without requiring a real signature — since a test harness runs off-chain and is not itself processing a real, signed transaction. |
| `test.mock_state(field_initializers) -> unit` | Within a test harness, pre-seed a contract instance's `state` fields to specific values before invoking a function under test, bypassing the ordinary `init` constructor flow for test setup convenience. |

Every function in this module is available **only** within a test harness context, never within a deployed contract's own compiled artifact — a conforming implementation MUST reject any attempt to call a `test.*` function from code reachable in a `kyne build`-produced deployment artifact, mirroring [§13](#13-debug-module)'s elision guarantee but as a compile-time rejection rather than a runtime elision, since test mocking, unlike debug output, has no meaningful "elided no-op" interpretation — a contract that could call `test.mock_auth` in production would have a real, exploitable authorization bypass, not an inert statement.

**Runtime guarantees.** Every mocked value is deterministic within a given test run, and MUST be fully isolated between separate test cases — one test's `test.mock_ledger` call MUST NOT leak into a subsequently run, unrelated test.

---

# 15. Error Utilities

## Why This Module Is Deliberately Thin

`Option<T>` and `Result<T, E>`'s core operations — `is_some`/`is_none`/`is_ok`/`is_err`/`unwrap_or`, construction via `Some`/`None`/`Ok`/`Err`, the `?` propagation operator, and exhaustive `match` — are already fully specified as **language-level intrinsics** in [LANGUAGE_SPEC.md §6.6](./LANGUAGE_SPEC.md#66-option-and-result) and [§7.7–§7.8](./LANGUAGE_SPEC.md#77-pattern-matching). This module does not redefine, wrap, or duplicate any of them.

Beyond those intrinsics, a conventional standard library's error-utilities module would typically offer generic helpers such as mapping a `Result`'s error type or chaining fallible computations — every such helper requires a function-value parameter (`map_err(f: fn(E) -> E2)`) and is therefore unavailable in Kyne v1 for the same reason [§10](#10-collections-module) cannot offer generic `filter`/`map`: **Kyne has no function-value type.**

This module's actual content is therefore guidance, not new callable APIs, consistent with [Small by Default](#1-small-by-default):

**Error type structure.** A contract's `error` declaration ([LANGUAGE_SPEC.md §6.5](./LANGUAGE_SPEC.md#65-error)) SHOULD enumerate every distinct failure reason a caller might need to branch on differently, and SHOULD NOT collapse unrelated failure reasons into a single, ambiguous variant — since [LANGUAGE_SPEC.md §7.7](./LANGUAGE_SPEC.md#77-pattern-matching)'s exhaustive `match` requirement means every variant a caller cares about is already forced to be handled explicitly wherever it is matched.

**`throw` versus propagation.** A function SHOULD use `throw` directly, per [LANGUAGE_SPEC.md §8.7](./LANGUAGE_SPEC.md#87-throw), at the specific point a precondition fails, rather than constructing an error value earlier and returning it later — this keeps the failure's source visually adjacent to the condition that caused it, per [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first).

**Recovery.** Recovery from a Recoverable Error is always an explicit `match` on a `Result` at the call site that receives it, per [RUNTIME_MODEL.md §9.5](./RUNTIME_MODEL.md#95-scoped-rollback-and-catching) — there is no separate "recovery" API, since recovery is simply what an ordinary `match` arm that does not re-`throw` already accomplishes.

---

# 16. Serialization

This chapter is a cross-reference and elaboration chapter, not a source of new callable APIs — every serialization guarantee it states is already fixed by [MEMORY_MODEL.md §8](./MEMORY_MODEL.md#8-storage-mapping).

**Default encoding.** Every `state` field's serialization format is chosen by the compiler, per [MEMORY_MODEL.md §8](./MEMORY_MODEL.md#8-storage-mapping), and is not developer-configurable — there is no Kyne syntax to select or customize a field's on-the-wire representation.

**Deterministic encoding.** Serialization MUST be a deterministic function of a value's in-memory representation — the same value always serializes to the same bytes, per [Standard Library Principle 5](#5-deterministic) and [MEMORY_MODEL.md §8](./MEMORY_MODEL.md#8-storage-mapping)'s round-trip guarantee.

**Round-trip guarantees.** Deserializing a field's stored bytes MUST always reproduce a value structurally identical to the one originally serialized, for every type permitted in `state` position, per [MEMORY_MODEL.md §8](./MEMORY_MODEL.md#8-storage-mapping).

**Versioning.** Because a deployed contract's WASM is immutable in Kyne v1 — there is no upgrade mechanism, per [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) — a given contract instance's serialization format, once deployed, never needs to change or be migrated within that instance's lifetime. Serialization-format *versioning*, as a distinct concern from the format itself, therefore has no meaningful role to play until Kyne someday introduces contract upgradability, which remains explicitly out of scope, per [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle).

---

# 17. Security Guarantees

| Category | Guarantee |
|---|---|
| Can panic (Runtime Failure) | `core.panic`, `core.assert`, `core.assert_eq`, `core.todo`, `core.unreachable`; bare arithmetic operators on overflow, per [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic); `auth(addr)` on failed authorization, per [LANGUAGE_SPEC.md §10.1](./LANGUAGE_SPEC.md#101-the-auth-statement); `storage.extend_ttl` on an over-large request. |
| Returns `Result` (Recoverable Error) | `math.mul_div`, `math.percentage_of`, every `convert.parse_*` function. |
| Never panics, never returns `Result` | `auth.has`, `ledger.*`, `time.*`, `crypto.hash`, `crypto.verify_signature` (returns `bool`, not `Result`, since an invalid signature is an ordinary negative answer, not an exceptional one), `collections.*`, `debug.*` (in development builds), every `math.checked_*`/`wrapping_*`/`saturating_*` function. |
| Deterministic | Every function in this specification, without exception, per [Standard Library Principle 5](#5-deterministic). |
| Security-sensitive | `auth.has` (see [§5](#5-authentication-module)'s misuse warning), `crypto.verify_signature` (see [§9](#9-crypto-module)'s scope note distinguishing it from `auth`), every `wrapping_*`/`saturating_*` math function (see [§11](#11-math-module)'s misuse warning). |

---

# 18. API Design Rules

**Verb-first naming.** Every function name begins with the action it performs (`get`, `set`, `extend_ttl`, `verify_signature`), not the object it acts on, so a developer scanning a module's function list reads a list of actions, not a list of nouns requiring further investigation to know what can be done with them.

**No abbreviations.** `sequence`, not `seq`; `timestamp`, not `ts`; `signature`, not `sig`. An abbreviation saves the author a few keystrokes at the cost of every future reader needing to recognize it — a trade [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first) rejects.

**Predictable return types.** A function that can fail always returns `Result` or `Option`, never a sentinel value (a magic number, an empty string standing in for "not found"). A function that cannot fail never returns `Result` "just in case" — matching [§17](#17-security-guarantees)'s explicit categorization exactly, with no function left ambiguous between the two.

**No hidden allocations.** A function's [Memory behavior](#apis-as-behavioral-specifications) field states plainly what it allocates; no function in this specification allocates persistent storage, heap memory proportional to an unbounded input, or any other resource not evident from its parameters and documented complexity.

**No hidden storage writes.** A function not documented as touching `state` — which is every function outside [§4](#4-storage-module)'s `storage.extend_ttl` — MUST NOT write to persistent storage under any circumstance. A developer reading a call to `math.mul_div` or `crypto.hash` never needs to wonder whether it might have side-effected the ledger.

**Explicit failure behavior.** Every API in this document states its failure behavior as one of exactly three categories from [§17](#17-security-guarantees) — panics, returns `Result`, or never fails — with no fourth, ambiguous category ("usually succeeds," "fails silently under some conditions") permitted anywhere in this specification or in any future addition to it.

**Why these rules exist.** Each rule exists to make the standard library's own behavior exactly as auditable as [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) and [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) already require Kyne source itself to be — a developer should never need to distrust or specially investigate a standard library call the way they might a third-party dependency, because the standard library is held to the language's own specification discipline, not a lesser one.

---

# 19. Performance Philosophy

Every function in this specification documents its time complexity, per [APIs as Behavioral Specifications](#apis-as-behavioral-specifications), using standard asymptotic notation (O(1), O(log n), O(n), O(n log n)) relative to its documented input size.

**Document expensive operations.** [§10](#10-collections-module)'s `sort_ascending`/`sort_descending` (O(n log n)) and every collection-scanning function ([§10](#10-collections-module)'s `contains`/`index_of`, both O(n)) state their cost explicitly, so a developer choosing between them and a hand-written loop is choosing with full information, not guessing.

**No hidden costs.** A function's stated complexity accounts for its entire behavior — there is no function in this specification whose real cost is meaningfully higher than its documented complexity suggests, such as an O(1)-looking call that secretly performs an O(n) storage scan underneath.

**No surprising storage access.** Every function's [Storage behavior](#apis-as-behavioral-specifications) field states plainly whether it touches persistent storage at all; a function documented as "not applicable" for Storage behavior MUST NOT issue a single storage read or write, since Soroban storage access carries a real, metered resource cost distinct from ordinary computation, and a developer budgeting that cost needs the documentation to be exact.

---

# 20. Stability Guarantees

**Versioning.** This specification is versioned alongside [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) and [RUNTIME_MODEL.md](./RUNTIME_MODEL.md); a standard library function's signature and documented behavior, once shipped in a numbered Kyne release, is a stability commitment of the same weight as a language keyword's behavior.

**Deprecation.** A standard library function MAY be deprecated — marked for eventual removal with a compiler-surfaced warning — but MUST NOT be removed outright within the same major version; removal is a breaking change subject to [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution), exactly as a language keyword's removal would be.

**Backward compatibility.** A new standard library function is always additive; an existing function's documented behavior MUST NOT be redefined to mean something new in a later release. Where a function's original design is found to be a mistake, the correction is a new, differently named function alongside a deprecation of the old one, never a silent behavior change under the same name.

**Future additions.** Every new standard library function or module MUST go through the KIP process defined in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution), evaluated against [Small by Default](#1-small-by-default) with the same rigor a new language keyword would receive — "would be convenient for some contracts" is not sufficient justification on its own, per [LANGUAGE_SPEC.md's Design Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place).

**Relationship to KIPs.** Every reasoning note in this document (why `storage` has no `get`/`set`, why `events` is nearly empty, why `collections` has no `filter`) is itself a standing precedent for future KIP review: a KIP proposing to reverse one of these decisions MUST address the original reasoning directly, not merely restate the original brief's request.

---

# 21. Module Invariants

**Storage never bypasses the Runtime Model.** Every persistent effect a standard library function produces — currently, only [§4](#4-storage-module)'s `storage.extend_ttl` — MUST be staged and committed through exactly the same pipeline [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline) already defines for `state` itself, with no separate, parallel persistence mechanism. A future standard library function that persisted anything outside this pipeline would silently violate every atomicity and rollback guarantee [RUNTIME_MODEL.md §9](./RUNTIME_MODEL.md#9-failure-model) makes.

**Crypto never exposes insecure primitives.** Restated permanently from [§9](#9-crypto-module): no algorithm not currently backed by an audited Soroban host function may ever be added to `crypto/`, regardless of a future request's justification. This invariant exists because a single insecure primitive, once shipped, could be relied upon by deployed, immutable, unpatchable contracts before its weakness is discovered.

**Math never silently overflows.** Every function in `math/` either preserves the checked-by-default panic behavior [LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic) already establishes, or names its deviation explicitly in its own function name (`wrapping_*`, `saturating_*`, `checked_*`), per [§11](#11-math-module). A future math function with an unnamed, silent overflow-masking behavior is never permitted.

**Collections preserve value semantics.** Every function in `collections/` returns a new value rather than mutating its input in place, per [§10](#10-collections-module) and [Value Semantics](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only) generally. A future collections function that mutated its argument in place would introduce exactly the aliasing behavior Kyne's language design has never permitted.

**Debug utilities never change observable behavior.** Restated permanently from [§13](#13-debug-module): every `debug.*` function's effect is confined to development builds and is fully absent — not merely inert, but entirely unrepresented in generated code — from optimized builds, per [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)'s optimization-safety rule applied to elision specifically.

**Why contributors must preserve them.** Every invariant above closes off a specific, previously reasoned-through way the standard library could silently regress a guarantee [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), or [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) already makes. A future contributor proposing a standard library change that appears to violate one of these invariants has, in every case this document has considered, actually proposed a change to one of those three underlying documents wearing a standard-library-shaped disguise — and MUST pursue that change explicitly, through a KIP against the actual document it affects, rather than through a standard library addition that appears narrower than it is.

---

# 22. Future Library Evolution

The following are intentionally **outside the scope of Kyne v1** and MUST NOT be introduced without a KIP explicitly amending this document:

- **Networking** — direct network I/O from contract code, which [Standard Library Principle 5](#5-deterministic) rules out categorically: no Kyne execution can depend on a nondeterministic external resource.
- **HTTP** — a specific case of the above.
- **JSON** — a general-purpose serialization format distinct from the deterministic, storage-oriented serialization [§16](#16-serialization) already specifies; reserved pending a concrete contract-relevant need, per [Small by Default](#1-small-by-default).
- **Regex** — general-purpose pattern matching over strings, judged, absent a concrete smart-contract-specific need, to belong in a future community package rather than the trusted base.
- **Filesystem** — contracts have no filesystem; this is a categorical exclusion, not a deferred feature.
- **Threads** — Kyne's execution model is single-threaded and sequential, per [RUNTIME_MODEL.md §1.1](./RUNTIME_MODEL.md#11-deterministic); threading has no meaning within it.
- **Async runtime** — already excluded at the language level for v1, per [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)'s enumerated v1 exclusions.
- **Environment variables** — contracts have no environment; every piece of legitimate ambient context is already exhaustively enumerated in [RUNTIME_MODEL.md §5](./RUNTIME_MODEL.md#5-execution-context) and named in [§6](#6-ledger-module) and [§7](#7-time-module) of this document.
- **Random number generation** — true randomness is definitionally non-reproducible and therefore categorically incompatible with [§2.1 of RUNTIME_MODEL.md](./RUNTIME_MODEL.md#21-every-execution-is-deterministic); this is a permanent exclusion, not a temporary one, unless a future KIP defines a specifically deterministic, ledger-seeded pseudo-randomness source with an explicit, bounded predictability model.
- **Plugins** — standard library extensibility mechanisms are governed by [COMPILER_ARCHITECTURE.md §18](./COMPILER_ARCHITECTURE.md#18-plugin-architecture)'s existing "no plugins in v1" decision, which this document inherits without modification.

**These are intentionally outside Kyne v1.** Their absence reflects the same deliberate, evidence-driven deferral [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), and [MEMORY_MODEL.md](./MEMORY_MODEL.md) already apply to their own reserved features. Any future addition MUST go through the KIP process in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution) and MUST demonstrate it does not weaken [Standard Library Principle 5 (Deterministic)](#5-deterministic) or any [Module Invariant](#21-module-invariants).

---

# Library Layering

Restated concisely from [Overall Library Architecture](#overall-library-architecture): dependency flows strictly **Core → Blockchain → Utilities**, never in reverse. A Layer 1 module MUST be usable, and MUST behave identically, whether or not any Layer 2 or Layer 3 module is ever invoked in the same program — this is what makes it legitimately "Core" rather than "Blockchain functionality that happens to have simple inputs." Import rules follow [LANGUAGE_SPEC.md §12.2](./LANGUAGE_SPEC.md#122-imports)'s ordinary `use` mechanism, with no special-cased import syntax for standard library modules versus project-local ones — the standard library is imported exactly as any other module would be, reinforcing that it is part of the language's own surface, not a separately privileged SDK. **Extension philosophy:** a new module MAY be added to a new or existing layer only through the KIP process in [§20](#20-stability-guarantees), and MUST be assigned to the layer matching its actual dependency needs, per [Overall Library Architecture](#overall-library-architecture)'s assignment table — a module's layer is a consequence of what it depends on, never a stylistic choice.

---

# Standard Library Commandments

1. **The standard library remains intentionally small.** Every module in this specification was evaluated, and several were deliberately narrowed or emptied ([§4](#4-storage-module), [§8](#8-event-module), [§15](#15-error-utilities)), rather than padded to match a conventional library's shape. This remains the standing bar for every future addition.

2. **Every API must be deterministic.** No exception exists, and none may ever be introduced, per [Standard Library Principle 5](#5-deterministic) and [§22](#22-future-library-evolution)'s permanent exclusions.

3. **Every API must earn its place.** An API belongs here only if nearly every contract needs it — not merely if some contract would find it convenient — mirroring [LANGUAGE_SPEC.md's keyword discipline](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) applied to function names instead of reserved words.

4. **Safety is preferred over convenience.** Where a convenient, general design (unrestricted `storage.set`, a single polymorphic `parse`) would conflict with an existing security or auditability guarantee, this specification has, in every case, chosen the guarantee — see [§4](#4-storage-module) and [§12](#12-conversion-module) for the two most consequential instances of this choice.

5. **Storage interactions must remain explicit.** No standard library function may read or write persistent storage under a key not already visible as a declared `state` field, per [§4](#4-storage-module)'s central reasoning — this commandment is the reason that module looks the way it does.

6. **Behavior must remain predictable.** Every function's failure mode is one of exactly three documented categories ([§17](#17-security-guarantees)), every function's complexity is documented ([§19](#19-performance-philosophy)), and every function's naming follows one fixed pattern ([§18](#18-api-design-rules)) — nothing in this library asks a developer to remember an exception to its own rules.

7. **Generated Rust must remain understandable.** Every function in this specification, when compiled, MUST produce Rust a reviewer would recognize as an ordinary, idiomatic call into the Soroban SDK or a small, obviously correct helper — never a large, opaque code-generation expansion, per [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) and [COMPILER_ARCHITECTURE.md §14.3](./COMPILER_ARCHITECTURE.md#143-reviewability).

**Why these rules are permanent.** The standard library is trusted by every Kyne contract unconditionally, in the same way the language's own keywords are — a defect or a scope-creeping addition here has the same blast radius as a defect in the compiler itself. These seven commandments exist to keep that trust earned rather than assumed, indefinitely, as the library grows.

---

# Non-Goals

This document does **not** define, and explicitly defers to other documents:

- **Language syntax** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Compiler implementation** — defined in [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md).
- **Memory algorithms** — defined in [MEMORY_MODEL.md](./MEMORY_MODEL.md).
- **Package manager** — reserved per [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem), to be further specified in a future `PACKAGE_MANAGER.md`.
- **Third-party packages** — not a concern of the standard library at all; see [§1](#1-standard-library-philosophy)'s "Relationship to community packages."
- **Networking** — permanently excluded, per [§22](#22-future-library-evolution).
- **Operating system APIs** — contracts have no operating system; permanently excluded.
- **General-purpose application frameworks** — Kyne is not a general-purpose language, per [Contract-Oriented Design](./LANGUAGE_PRINCIPLES.md#why-contracts-are-the-unit-of-everything), and its standard library does not attempt to serve use cases outside smart contract development.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission and values this standard library exists to serve.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy and KIP governance process this document's own evolution follows.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the type system, grammar, and language-level intrinsics (`auth`, `emit`, `Option`/`Result`, collection methods) this document builds on without duplicating.
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — the execution context, failure model, and event/authorization guarantees every Layer 2 module implements against.
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) — the Security Analyzer, optimization, and plugin-architecture decisions this document references and inherits.
- [MEMORY_MODEL.md](./MEMORY_MODEL.md) — the storage lifecycle and value-semantics guarantees this document's storage and collections modules depend on directly.

The following documents are anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `TOOLCHAIN.md`, `GOVERNANCE.md`, `ROADMAP.md`, and a future `PACKAGE_MANAGER.md` to be defined by KIP. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is the single authoritative specification for Kyne's standard library. A future contributor implementing or extending it should be able to determine, from this document alone, exactly what every module MUST guarantee — and should find, on every page, a design that was reasoned through against the language's own locked decisions rather than assumed by default from convention. Where this document declined to provide a capability a conventional standard library might offer, that decision is recorded, with its reasoning, precisely so a future contributor does not need to re-derive it, and does not casually reverse it without engaging the reasoning directly. Where a future engineer finds a genuine gap this document does not address, that is a gap to be closed by a KIP — not a decision to be made silently in an implementation.
