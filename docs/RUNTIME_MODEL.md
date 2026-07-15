# RUNTIME_MODEL.md

# The Kyne Runtime Model

**Phase:** 0 — Foundations
**Milestone:** 4 — Runtime and Execution Model
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on the compiler implementation and on every conforming Kyne runtime.

This document defines how every Kyne program behaves at runtime. It formalizes the philosophy in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), the mission in [FOUNDATION.md](./FOUNDATION.md), and the syntax and static semantics in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) into a single, authoritative account of execution. Where this document and those documents appear to conflict, this document governs runtime behavior, and the conflict MUST be raised as a KIP to correct whichever document is wrong.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings. A conforming Kyne compiler and a conforming Kyne runtime MUST NOT produce behavior that violates a MUST/MUST NOT rule in this document. Where this document identifies more than one plausible interpretation of a runtime behavior, it resolves the ambiguity and states the chosen behavior explicitly — it does not leave the choice to the implementer.

---

# 1. Runtime Philosophy

Every Kyne program executes according to three immutable properties: it is **Deterministic**, it is **Transactional**, and it is **Transparent**. These properties are not aspirational qualities the runtime tries to achieve — they are the definition of what it means for an execution to be a valid Kyne execution at all. A runtime that produces a correct-looking result through a non-deterministic, non-transactional, or opaque mechanism has not implemented Kyne; it has implemented something else with Kyne's syntax.

## 1.1 Deterministic

Given the same on-chain state, the same invocation arguments, and the same execution context (see [§5](#5-execution-context)), a Kyne contract MUST produce exactly the same result, the same state changes, and the same emitted events, every time, on every conforming implementation. Determinism is not a performance property here — it is a correctness property, and its absence is a security defect. A blockchain that cannot re-derive the same outcome from the same inputs cannot reach consensus, and a contract whose outcome depends on anything other than its declared inputs is not auditable, because an auditor reasoning about its logic can no longer trust that the logic they read is the logic that ran.

Determinism constrains the language itself, not only its runtime: Kyne has no floating-point type ([LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types)) because floating-point arithmetic is not guaranteed bit-identical across hardware and toolchains; Kyne provides no randomness primitive, because true randomness is definitionally non-reproducible; and every piece of ambient information a contract can observe — time, ledger sequence, invoker identity — MUST come from an explicit, deterministic source recorded in the transaction itself (see [§5](#5-execution-context)), never from an implementation's local clock, local entropy pool, or environment.

## 1.2 Transactional

Every top-level invocation of a Kyne contract executes as a single, indivisible unit. Either every state change staged during that invocation and everything it called becomes visible on the ledger together, or none of it does. There is no runtime-observable intermediate state where some of an invocation's effects are visible and others are not — see [§7](#7-execution-pipeline) and [§10](#10-storage-behavior) for the staging and commit mechanism that makes this guarantee possible, and [§9](#9-failure-model) for exactly which failures cause a rollback and at what scope.

Transactionality is what allows a contract author to reason locally: a function that checks a precondition and then acts on it never needs to worry that another invocation observed and acted on the state in between, because from the perspective of any other invocation, this one either has not happened yet or has already happened in full.

## 1.3 Transparent

Every effect a Kyne program has at runtime MUST be traceable to a specific, visible construct in its source: a `state` assignment, an `emit`, an `auth` call, a `throw`. There is no runtime behavior that exists only in the generated Rust or only in the compiler's implementation and not in the Kyne source a developer and an auditor both read. This is the runtime-level expression of [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction): the abstraction may remove repetition, but it MUST NOT remove the reader's ability to predict, from source alone, exactly what will happen on-chain.

Transparency is also why this document exists as a standalone specification rather than as a description embedded in compiler comments: the runtime model is a contract with every future compiler implementation, every auditor, and every developer, independent of any one implementation's internal design.

---

# 2. Runtime Manifesto

The following five principles are permanent language guarantees. They MUST hold for every conforming Kyne program, on every conforming implementation, indefinitely. A future KIP MAY refine how these principles are achieved; no future KIP may weaken the principle itself without that change being treated as a foundational break requiring the extraordinary process described in [LANGUAGE_PRINCIPLES.md's Ten Commandments](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne).

## 2.1 Every Execution Is Deterministic

A conforming Kyne runtime MUST guarantee that re-executing a contract invocation against identical prior ledger state, identical arguments, and identical execution context produces identical outputs: identical return values, identical staged state changes, and identical emitted events, in identical order. This is not merely a testing convenience — determinism is the property that allows independent validators to agree on the outcome of a transaction without trusting one another's hardware, operating system, or floating-point unit.

Determinism has a specific, narrow set of legitimate inputs, enumerated exhaustively in [§5](#5-execution-context): the current ledger's timestamp and sequence number, the invocation's arguments, the invoker's verified identity, and persistent state already committed to the ledger. Anything not on that list — wall-clock time, a random seed, a network call to an off-chain system, thread-scheduling order — MUST NOT influence a contract's outcome, and Kyne provides no syntax capable of expressing such an influence in the first place. This is a stronger guarantee than "the compiler discourages non-determinism" — it is "the language has no construct through which non-determinism could enter."

The consequence for implementers is direct: a conforming runtime MUST NOT introduce any source of variation between two executions of the same invocation against the same state — not for performance, not for convenience, and not as an unobservable internal implementation detail, because "unobservable" is exactly the property that cannot be assumed of anything touching consensus-relevant output.

## 2.2 Every State Change Is Explicit

A `state` field changes value if and only if a Kyne statement in the executing contract's own source explicitly assigns to it. There is no implicit persistence, no automatic caching of a computed value into storage, and no state mutation that occurs as a side effect of a language feature that does not, on its face, look like an assignment. This mirrors [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit) at the level of runtime behavior rather than syntax: every write an auditor needs to find is findable by locating assignments to that field's name.

This principle is also why [§9.4 of LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md#94-compiler-rules) forbids library code from mutating a contract's `state` directly — a state change that originated inside code the reading contract did not itself write would still be, from the runtime's perspective, an explicit assignment in some source file, but it would not be explicit *to the contract being audited*, which defeats the purpose. Every state change must be explicit relative to the contract whose behavior is under review, not merely explicit somewhere in the dependency graph.

The runtime MUST NOT perform any write to persistent storage that does not correspond, one-to-one, to an assignment statement executed by the contract that owns that storage. This rules out, categorically, any future optimization or convenience feature — however well-intentioned — that would write to `state` as an inferred side effect of some other operation.

## 2.3 Every Authorization Is Verifiable

Every effect that depends on a specific party having authorized it MUST be traceable to a specific `auth(...)` statement in source, evaluated against a specific, independently verifiable authorization entry attached to the transaction (see [§12](#12-authorization-model)). There is no implicit authorization, no ambient "the caller is assumed authorized" default, and no authorization that is inferred from context rather than checked against a cryptographically verified entry.

This principle is the runtime enforcement of [LANGUAGE_SPEC.md §10.2](./LANGUAGE_SPEC.md#102-no-implicit-sender)'s "no implicit sender" rule: because Kyne has no ambient caller identity, every authorization claim a contract relies on is necessarily the result of an explicit runtime check against an explicit address, which makes every privileged effect in a Kyne contract independently verifiable by re-checking that one `auth` call against the transaction's recorded authorization entries — without needing to trust the contract's own bookkeeping about who called it.

A conforming runtime MUST reject, with a Runtime Failure (see [§9.4](#94-runtime-failure)), any `auth(addr)` evaluation for which no valid, unconsumed authorization entry for `addr` exists. There is no permissive fallback and no partial-authorization state; authorization for a given address at a given `auth` call site is binary and MUST be resolved before the next statement executes.

## 2.4 Every Failure Is Atomic

When an invocation fails — whether by an explicit `throw` that propagates uncaught, or by an unstructured Runtime Failure such as a checked-arithmetic panic — every state change staged by that invocation, and by everything it called, MUST be discarded as though it had never been attempted. Section [§7](#7-execution-pipeline) and [§9](#9-failure-model) define the precise scope of this rollback: it is scoped to the invocation frame that fails and everything nested beneath it, not necessarily to the entire top-level transaction, because a calling contract MAY catch a nested failure as an ordinary `Result` value and continue toward its own success (see [§8](#8-function-invocation-model)). What MUST always hold, without exception, is that no partial effect of a *failed* frame is ever visible on the ledger — not one field, not one event.

Atomicity is what allows a contract author to write a function that performs several related state changes without manually implementing compensation logic for every possible mid-function failure: the runtime's staging and rollback mechanism ([§10](#10-storage-behavior)) makes "some of this happened and some of it didn't" an impossible outcome by construction, not merely an unlikely one.

## 2.5 Every Abstraction Is Transparent

Every convenience Kyne's syntax offers — the direct-by-name access to `state`, the `?` error-propagation operator, the `auth(...)` statement's translation into a lower-level authorization check — MUST correspond to runtime behavior a reader can predict from the Kyne source alone, without needing to inspect generated Rust or compiler internals to know what will happen on-chain. This document exists specifically to make that prediction possible: every stage of every pipeline described here (§7, §10, §12) is a public, stable contract between the language and its users, not an implementation detail subject to silent change.

A future optimization to the reference compiler MAY change *how* an effect is achieved. It MUST NOT change *which* effects occur, *in what order they become observable*, or *under what conditions they roll back*, because those are the runtime-level promises this specification makes, and Kyne's users — developers and auditors alike — are entitled to rely on them exactly as written here.

---

# 3. Project Lifecycle

A Kyne contract project moves through the following stages between a developer's keystrokes and a transaction executing on Stellar:

```
.kyn source
    ↓
Compiler
    ↓
Generated Rust
    ↓
Soroban Toolchain
    ↓
WASM
    ↓
Deployment
    ↓
Contract Instance
    ↓
Blockchain Execution
```

**`.kyn` source.** A developer writes one or more `.kyn` files forming a single contract project, per [LANGUAGE_SPEC.md §12](./LANGUAGE_SPEC.md#12-modules): at most one `ContractDecl` across the whole project, with any number of supporting files providing `struct`, `enum`, `error`, `const`, and `fn` declarations. This stage is purely textual; nothing here is yet checked for correctness.

**Compiler.** The Kyne compiler parses, statically checks, and lowers the project's source according to [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) in full — lexical structure, grammar, the canonical member order, definite assignment, exhaustive matching, and every other MUST-level static rule. A project that fails any check here never proceeds to a later stage; there is no such thing as a Kyne program that "mostly" compiles. Compiler internals are explicitly out of scope for this document — see [Non-Goals](#non-goals).

**Generated Rust.** The compiler's output is a complete, idiomatic, Soroban-compatible Rust crate, satisfying [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction): every Kyne construct maps to specific, readable Rust. This stage is a compiled artifact, not a hand-edited one — modifying generated Rust directly and recompiling from `.kyn` would silently discard the edit, so generated Rust is treated as a build output, reviewable but not a second source of truth.

**Soroban Toolchain.** The generated Rust crate is compiled by the standard Soroban/Rust toolchain exactly as any other Soroban contract crate would be — Kyne introduces no fork of, or special case in, this stage. This is the guarantee behind "Kyne is not a Rust replacement": from this point forward, a Kyne contract and a hand-written Soroban Rust contract are indistinguishable to the toolchain.

**WASM.** The toolchain output is a WASM binary, the actual artifact that will be uploaded to the network and executed by the Soroban host.

**Deployment.** The WASM binary is uploaded to the network and installed, and a contract instance is created referencing it. This is the point at which the project's single contract (per [Contract-Oriented Design](./LANGUAGE_PRINCIPLES.md#why-contracts-are-the-unit-of-everything)) becomes a real, addressable, on-chain entity.

**Contract Instance.** The deployed instance owns its own persistent storage, distinct from every other instance of even the same WASM code, per [§6](#6-state-model). If the contract declares an `init` function, this is invoked exactly once here, per [LANGUAGE_SPEC.md §3.5](./LANGUAGE_SPEC.md#35-the-constructor).

**Blockchain Execution.** From this point on, the contract instance is invoked by transactions according to the execution pipeline defined in [§7](#7-execution-pipeline), for as long as the instance remains active on the ledger (see [§4](#4-contract-lifecycle)).

---

# 4. Contract Lifecycle

Independently of the project pipeline in [§3](#3-project-lifecycle), every deployed contract *instance* moves through its own lifecycle over time:

```
Source
    ↓
Compiled
    ↓
Deployed
    ↓
Executed
    ↓
Archived
```

**Source.** The `.kyn` project exists but has not yet been compiled. Nothing about this stage is observable on-chain; it is purely a developer-side artifact.

**Compiled.** The project has produced a WASM binary (via [§3](#3-project-lifecycle)) but that binary has not yet been installed on any network. A compiled contract is fully deterministic and fully specified but has no on-chain identity yet.

**Deployed.** The WASM has been installed and a contract instance created. **From this point forward, the contract's code is immutable.** Kyne v1 defines no upgrade mechanism, and this specification takes no position on one: a deployed contract's logic MUST be treated, for every purpose within this document, as fixed for the lifetime of the instance. This is a deliberate scope boundary, not an oversight — contract upgradability raises storage-compatibility and authorization questions significant enough to deserve its own KIP once real deployed contracts exist to inform the design, rather than being speculated on ahead of need, per [Small Language Philosophy](./LANGUAGE_PRINCIPLES.md#small-language-philosophy).

**Executed.** The contract instance is invoked by transactions, each proceeding through the [Execution Pipeline](#7-execution-pipeline). What changes during this stage is exclusively the instance's persistent `state`, per the [State Model](#6-state-model); the contract's code, its declared `event` and `error` shapes, and its public API never change during this stage, because they were fixed at deployment.

**Archived.** A contract instance's persistent storage entries are not guaranteed to remain in immediately accessible ledger state indefinitely. Consistent with Soroban's ledger-state-expiration model, storage entries that are not kept alive (through whatever explicit lifetime-extension mechanism the standard library exposes — a stdlib-level concern, deferred to a future Standard Library Specification) MAY become archived after a sufficiently long period without qualifying activity. Archival is not deletion: an archived entry's data is preserved and MUST be explicitly restored before it can be read or written again; a conforming runtime MUST NOT allow an archived entry to be treated as absent (`None`) or silently reset to a default — that would violate [§2.2](#22-every-state-change-is-explicit) by producing an apparent state change (loss of a stored value) that no `.kyn` source ever expressed. A Kyne contract's logic MUST NOT assume its storage is exempt from archival; contracts holding long-lived state SHOULD budget for periodic lifetime extension using standard library facilities.

Nothing about a contract instance's **code** is ever archived or restored — only its **persistent storage** is subject to this lifecycle. The distinction matters because it means a contract's deterministic behavior (§2.1) is never itself in question during archival; only the availability of specific stored values is.

---

# 5. Execution Context

Every Kyne function executes inside a deterministic execution context. This context is the exhaustive set of ambient information a contract MAY observe; nothing outside this list is available, and every conforming runtime MUST make exactly this information, and no more, available to executing contract code:

| Context value | Description | How it is obtained |
|---|---|---|
| Network | The identity of the network the transaction was submitted to. | Explicit accessor call (standard library), never an implicit global. |
| Ledger | The current ledger sequence number. | Explicit accessor call. |
| Transaction | Identifying data for the enclosing transaction. | Explicit accessor call. |
| Timestamp | The current ledger close time. | Explicit accessor call. |
| Invoker | The address(es) that have supplied authorization for this invocation. | Explicit function parameters only, per [LANGUAGE_SPEC.md §10.2](./LANGUAGE_SPEC.md#102-no-implicit-sender) — never an ambient "current caller" value. |
| Authorization | Whether a specific address has a valid, unconsumed authorization entry. | Explicit `auth(addr)` statement, per [§12](#12-authorization-model). |
| Storage | The contract instance's own persistent `state`. | Direct-by-name access to declared `state` fields, per [LANGUAGE_SPEC.md §4.3](./LANGUAGE_SPEC.md#43-state-persistent-contract-storage). |
| Events | The append-only record this invocation is contributing to. | `emit` statements only, per [§11](#11-event-model) — contracts cannot read previously emitted events back. |

**There are no hidden globals and no implicit runtime state.** Every row in the table above is obtained either through an explicit standard-library function call (network, ledger, transaction, timestamp) or through language constructs whose whole purpose is to make the access visible in source (`state` names, `auth(...)`, `emit`, and explicit `address` parameters for invoker identity). A conforming runtime MUST NOT expose any of this information through a bare identifier, an implicitly injected variable, or any mechanism that does not appear as an explicit call or declared construct at the point of use. This is the direct runtime consequence of [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit): a reader auditing a function MUST be able to see every piece of ambient context it consults by reading that function's own text, without needing to know an implicit convention.

The exact standard-library function names for network, ledger, transaction, and timestamp access are a Standard Library Specification concern (see [Cross References](#cross-references)) and are not fixed by this document; what this document fixes is that such access is always an explicit call, never a global.

---

# 6. State Model

Kyne has exactly three categories of data. This is a deliberately closed set — see [§6.7](#67-why-exactly-three-categories) — matching [LANGUAGE_SPEC.md §4](./LANGUAGE_SPEC.md#4-variables)'s `let` / `const` / `state` triad, restated here in terms of runtime behavior rather than syntax.

## 6.1 Local

| Property | Behavior |
|---|---|
| Lifetime | The current function invocation's call frame only. |
| Mutability | Mutable via reassignment (`let`); no separate immutable-local form exists. |
| Visibility | The enclosing block and any nested block, per lexical scope. |
| Storage location | Execution memory (the generated Rust's stack/heap, per [§4.0 of LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)); never the ledger. |
| Compiler behavior | Must be initialized at declaration; ordinary lexical scoping and shadowing rules apply. |
| Generated Rust behavior | Compiled to ordinary Rust local bindings; the compiler is free to choose owned values, clones, or references underneath, invisibly to Kyne source. |
| Persistence | None. A Local value that is not written into `state` before the invocation ends ceases to exist, with no runtime trace. |

## 6.2 Persistent

| Property | Behavior |
|---|---|
| Lifetime | The contract instance's entire on-chain existence, subject to [archival](#4-contract-lifecycle). |
| Mutability | Mutable, exclusively via assignment inside a member function of the contract that declares it. |
| Visibility | Direct-by-name, within any function of the declaring contract; never accessible from library code directly ([LANGUAGE_SPEC.md §9.4](./LANGUAGE_SPEC.md#94-compiler-rules)); never accessible from a different contract instance at all. |
| Storage location | The Soroban ledger's persistent storage, keyed to this specific contract instance. |
| Compiler behavior | Requires an explicit type; subject to [definite-assignment analysis](./LANGUAGE_SPEC.md#44-definite-assignment-of-state) before any non-`init` entry point can read it. |
| Generated Rust behavior | Compiled to explicit ledger storage `get`/`set` calls at each read/write site — never cached silently across statements within one invocation beyond what the staging model in [§10](#10-storage-behavior) describes. |
| Persistence | Survives across separate invocations and separate transactions, subject to the [Archived](#4-contract-lifecycle) stage of the contract lifecycle. |

## 6.3 Constant

| Property | Behavior |
|---|---|
| Lifetime | Compile time only; has no runtime lifetime at all. |
| Mutability | Immutable; a `const` MUST have a constant-expression initializer ([LANGUAGE_SPEC.md §4.2](./LANGUAGE_SPEC.md#42-const-compile-time-constants)). |
| Visibility | Wherever declared: contract scope or module scope. |
| Storage location | None. A `const` has no ledger footprint and no runtime memory footprint distinct from the literal values the compiler inlines at its use sites. |
| Compiler behavior | Fully evaluated at compile time; never re-evaluated at runtime. |
| Generated Rust behavior | Compiled to a Rust `const`, inlined identically to how Rust itself would inline it. |
| Persistence | Not applicable — a value with no runtime existence cannot persist or fail to persist. |

## 6.4 Storage Location Summary

```
Local       → execution memory only, discarded at end of invocation
Persistent  → ledger storage, keyed to the contract instance, survives across invocations
Constant    → no runtime storage at all; resolved entirely at compile time
```

## 6.5 Mutability Summary

Local and Persistent data are both mutable; Constant data is not. Kyne does not additionally distinguish "mutable" from "immutable" within the Local or Persistent categories (there is no `mut`/non-`mut` split within `let`, per [LANGUAGE_SPEC.md §4.1](./LANGUAGE_SPEC.md#41-let-local-variables)) — mutability is a property of *which category a value belongs to*, not an independent modifier layered on top of it.

## 6.6 Compiler and Generated-Rust Behavior, Restated

The compiler MUST treat these three categories as the exhaustive classification of every value a Kyne program can produce. Every read of a `state` field within a given invocation MUST observe that field's most recently staged value from within the same invocation (including values staged by frames that have already successfully returned into this invocation, per [§7](#7-execution-pipeline)), or, if unmodified so far in this invocation, its last-committed value from the ledger. The compiler MUST NOT generate Rust that caches a `state` read across a boundary where another part of the same invocation could have modified it, since that would violate [§2.2](#22-every-state-change-is-explicit) by making a write's visibility depend on an implementation detail rather than on program order.

## 6.7 Why Exactly Three Categories

Kyne intentionally has only three data categories, not more. A fourth category — for example, Soroban's Temporary or Instance storage durability classes, distinct from the Persistent class Kyne exposes today — would require the language to expose a durability *choice* to every developer at every `state` declaration, and that choice is exactly the kind of low-level, easy-to-get-wrong decision [Design Philosophy](./LANGUAGE_PRINCIPLES.md#design-philosophy) exists to remove from the common path. Kyne v1 makes that choice once, for every contract, in favor of the safest and most predictable option (Persistent), and defers exposing the other durability classes to a future version once real usage patterns show which contracts actually need the more advanced, and more error-prone, tradeoffs those classes offer — consistent with [LANGUAGE_SPEC.md §9.1](./LANGUAGE_SPEC.md#91-persistent-state).

Three categories are also the minimum necessary to answer the one question every value in a smart contract must answer: does this survive the invocation, and if so, whose responsibility is it? Local answers "no." Constant answers "the question doesn't apply — it never had a runtime existence to survive." Persistent answers "yes, and it belongs to this contract instance." There is no legitimate fourth answer within the scope of what Kyne v1 needs to express, and inventing one ahead of a concrete need would be exactly the kind of unjustified complexity [Every Keyword Must Earn Its Place](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) rejects.

---

# 7. Execution Pipeline

Every invocation of a Kyne contract — whether the top-level entry point of a transaction or a nested call reached via [§8](#8-function-invocation-model) — proceeds through the following stages, in this exact order:

```
Receive Invocation
    ↓
Validate Inputs
    ↓
Authorization
    ↓
Load State
    ↓
Execute Function
    ↓
Stage State Changes
    ↓
Emit Events
    ↓
Commit
    ↓
Return Result
```

**Receive Invocation.** The runtime identifies the target contract instance, the target function, and the supplied arguments. Argument values are decoded according to the function's declared parameter types ([LANGUAGE_SPEC.md §5.2](./LANGUAGE_SPEC.md#52-parameters-and-return-types)); a decoding failure at this stage is a Runtime Failure ([§9.4](#94-runtime-failure)) and the invocation proceeds no further.

**Validate Inputs.** Argument values that decoded successfully are checked against any type-level constraints the compiler can enforce before contract logic runs (for example, that a value claimed to be a particular `struct` in fact supplies every declared field). This stage precedes authorization deliberately: a structurally malformed call should fail cheaply, before the runtime spends effort verifying who authorized it.

**Authorization.** The runtime verifies the cryptographic validity of every authorization entry attached to the transaction — signatures check out, and any entry-level constraints (such as expiration) are satisfied. This stage is distinct from, and precedes, any specific `auth(addr)` statement the contract's own code executes during **Execute Function**: this pipeline stage establishes which authorization entries are valid *at all* for this transaction; the contract's own `auth(...)` calls later determine which of those already-valid entries are actually *relied upon*, and for what. A transaction carrying no structurally valid authorization entries fails here, before any contract-specific logic runs, as a Runtime Failure.

**Load State.** The invocation's initial view of the contract instance's `state` is established from the ledger's last-committed values, layered underneath any changes already staged by an enclosing invocation in the same transaction (see [§8](#8-function-invocation-model) and [§10](#10-storage-behavior)).

**Execute Function.** The target function's body runs: its statements execute in source order, its own `auth(...)` calls are checked against the entries validated in the **Authorization** stage, its `state` reads observe the view established in **Load State** (as subsequently modified by its own assignments), and any nested calls recursively proceed through this same pipeline (see [§8](#8-function-invocation-model)).

**Stage State Changes.** Every assignment to `state` performed during **Execute Function** is recorded in this invocation's own staging layer. Nothing is written directly to ledger storage yet — see [§10](#10-storage-behavior) for why this indirection exists and exactly what it guarantees.

**Emit Events.** Every `emit` statement executed during **Execute Function** is recorded, in the order it executed, as part of this invocation's contribution to the transaction's event record. Events are staged alongside state changes and become part of the durable record only if this invocation, and every invocation depending on it, ultimately succeeds — see [§11](#11-event-model).

**Commit.** If **Execute Function** completed without failure, this invocation's staged state changes are merged into its caller's staging layer (if it was itself a nested call) or, if this was the top-level invocation, written to ledger storage as a single, indivisible unit alongside every nested invocation's merged changes. Staged events are finalized in the same step.

**Return Result.** The function's return value (or, for a nested call, the value observed by its caller) is produced. For the top-level invocation, this is also the value returned to whatever submitted the transaction.

## 7.1 Why State Is Staged Before Commit

State is never written directly to ledger storage during **Execute Function** because doing so would make [§2.4 (Every Failure Is Atomic)](#24-every-failure-is-atomic) impossible to guarantee: if a function's third state assignment executed a direct ledger write and its fifth statement then threw an error, a direct-write model would leave that third write permanently visible despite the overall invocation having failed. Staging defers every write's real effect until the point at which the runtime can be certain the writing invocation — and everything depending on its success — actually succeeded. This is what makes rollback a matter of discarding an in-memory staging layer rather than a matter of undoing already-durable ledger writes, which would be slower, more complex, and in the presence of concurrent access, potentially impossible to do safely.

## 7.2 Rollback Behavior

If any stage from **Execute Function** onward fails — an uncaught `throw`, a Runtime Failure, or a failure surfaced by a nested call that this invocation does not catch — this invocation's staging layer (state changes and events alike) MUST be discarded in its entirety, as though **Execute Function** had never run. The failure then propagates to whatever invoked this one, per the scoped rollback and catching rules in [§8](#8-function-invocation-model) and [§9](#9-failure-model). No stage after **Stage State Changes** MUST ever run for a failed invocation — an invocation that fails is never committed, in whole or in part.

---

# 8. Function Invocation Model

Kyne recognizes three distinct call relationships, each with different consequences for state visibility and transaction boundaries.

## 8.1 Internal Function Calls

An **internal function call** is a contract function calling another function declared in the same contract — for example, a `public fn` calling a `private` helper or an `internal fn`. The callee executes in a nested invocation frame (per [§7](#7-execution-pipeline)) that shares the same contract instance's `state` and therefore the same view of storage as its caller: a `state` assignment made by the callee is immediately visible to the caller once the callee returns successfully, because the callee's staging layer merges directly into the caller's.

Internal calls do not cross a transaction boundary — they are ordinary nested execution within the same top-level invocation, and a failure in an internal call is caught or propagated exactly as described in [§9](#9-failure-model), scoped to the callee's own frame.

## 8.2 Cross-Function Calls

A **cross-function call** is a contract function calling a free function provided by an imported library ([LANGUAGE_SPEC.md §12.1](./LANGUAGE_SPEC.md#121-contracts-and-libraries)). Because a library MUST NOT declare `state` at all, a cross-function call crosses a module boundary but not a storage boundary: the library function operates purely on the values passed to it and returns new values, per [Composition Over Inheritance](./LANGUAGE_PRINCIPLES.md#composition-over-inheritance) and [LANGUAGE_SPEC.md §9.4](./LANGUAGE_SPEC.md#94-compiler-rules). A cross-function call's failure behaves identically to an internal call's failure — it is scoped to that call's own invocation frame — because there is no additional contract-instance boundary for it to interact with.

## 8.3 Cross-Contract Calls

A **cross-contract call** invokes a function belonging to a *different* deployed contract instance. This is the one call relationship that crosses both a module boundary and a storage boundary: the callee executes with its own `state`, entirely inaccessible to the caller, and its own authorization checks against entries relevant to its own invocation.

Cross-contract calls do **not** cross a transaction boundary: the callee's invocation is still part of the same top-level transaction as the caller, subject to the same overall atomicity guarantee, and its staged changes merge into the calling invocation's staging layer on success, per [§7](#7-execution-pipeline) — exactly as an internal call's would, just across a storage boundary rather than within one. What is different about a cross-contract call is that its failure is *observable and catchable as an ordinary value* at the call site: because the callee's return type is known statically, a caller receiving a `Result<T, E>` from a cross-contract call MAY `match` on it and, if it does not itself fail in response, continue toward its own success — in which case only the *callee's* staged changes are discarded, while the caller's own state changes, made before or after the cross-contract call, MAY still commit as part of an ultimately successful top-level transaction. See [§9.5](#95-scoped-rollback-and-catching) for the precise rule.

## 8.4 Stack Behavior

Invocation frames nest exactly as function calls nest in any ordinary call stack: an internal call, a cross-function call, and a cross-contract call are all, from the runtime's perspective, a new frame pushed onto the current transaction's execution, with the distinctions above governing only what data each frame can see and how its failure is scoped — not whether a stack frame exists at all. A conforming runtime MUST enforce a maximum call depth consistent with the Soroban host's own resource limits; exceeding it is a Runtime Failure.

## 8.5 State Visibility Summary

| Call type | Crosses module boundary | Crosses storage boundary | Crosses transaction boundary |
|---|---|---|---|
| Internal function call | No | No | No |
| Cross-function call (library) | Yes | No (library holds no state) | No |
| Cross-contract call | Yes | Yes | No |

---

# 9. Failure Model

Every Kyne invocation resolves to exactly one of four outcomes.

## 9.1 Success

**Execute Function** completed without a `throw` and without a Runtime Failure. All state changes staged during the invocation, and everything it called, are merged upward per [§7](#7-execution-pipeline); if this was the top-level invocation, they are committed to the ledger, and any staged events are finalized.

## 9.2 Recoverable Error

The invocation executed `throw <expr>;` (directly, or via an unhandled `?` propagating one, per [LANGUAGE_SPEC.md §8.7](./LANGUAGE_SPEC.md#87-throw) and [§7.8](./LANGUAGE_SPEC.md#78-error-propagation)), producing a structured `Result::Err` value of the function's declared error type. This is a **typed, catchable** outcome: it is an ordinary value that a calling invocation MAY inspect via `match` and respond to without itself failing. If no calling invocation catches it — if it propagates, uncaught, all the way out of the top-level entry point — the entire top-level transaction fails, and per [§9.5](#95-scoped-rollback-and-catching), rolls back in full.

## 9.3 Compiler Error

The `.kyn` project failed static checking during the **Compiler** stage of [§3](#3-project-lifecycle) and never produced a deployable artifact at all. This outcome has no runtime manifestation whatsoever — it is included here only to complete the taxonomy of ways a Kyne program can fail to produce a successful result; a program that has a Compiler Error was never executed and therefore never reached this document's pipeline.

## 9.4 Runtime Failure

The invocation aborted through a mechanism that produces no structured error value at all — a checked-arithmetic overflow ([LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic)), a failed `auth(addr)` check ([§12](#12-authorization-model)), a decoding failure at **Receive Invocation**, or an exhaustion of the Soroban host's metered resources. A Runtime Failure is **never catchable** by any Kyne construct — there is no `Result` to `match` on, because none was produced. A Runtime Failure occurring anywhere in a transaction's call graph MUST cause the entire top-level transaction to roll back in full, with no exception, regardless of how many enclosing invocations exist between the point of failure and the top level.

## 9.5 Scoped Rollback and Catching

The distinction between Recoverable Error and Runtime Failure exists specifically to define how far a failure's rollback extends:

- A **Recoverable Error** rolls back the failing invocation's own staged changes only. If a calling invocation receives it as a `Result::Err` and handles it — via `match`, without itself throwing or triggering a Runtime Failure — the calling invocation MAY proceed to its own success, and its own staged changes (from before or after the failed call) MAY commit. If no invocation up the chain catches it, the failure reaches the top level uncaught, and the entire transaction rolls back — not because Recoverable Errors are inherently transaction-fatal, but because an uncaught failure at the top level means the top-level invocation itself did not succeed, and per [§1.2 (Transactional)](#12-transactional), an invocation that does not succeed commits nothing.
- A **Runtime Failure** always rolls back the entire top-level transaction. It cannot be caught, so it is definitionally never handled by an enclosing invocation, so it always propagates to the top.

This distinction MUST be preserved by every conforming implementation: it is not acceptable for an implementation to treat all failures as uniformly transaction-fatal (which would make cross-contract error handling meaningless) or to make Runtime Failures selectively catchable (which would make [§2.3 (Every Authorization Is Verifiable)](#23-every-authorization-is-verifiable) and similar hard guarantees negotiable at the call site).

## 9.6 Observable Behavior

A Recoverable Error is observable to a catching invocation as an ordinary typed value; it is observable to an external caller of a transaction that ultimately failed as a transaction failure carrying that error value. A Runtime Failure is observable only as an undifferentiated transaction failure — a conforming runtime MUST NOT expose partial diagnostic detail about a Runtime Failure to on-chain logic, though it MAY surface implementation-level diagnostic detail off-chain (for example, in a local test harness) for developer convenience, since that detail plays no role in consensus-relevant execution.

---

# 10. Storage Behavior

Every persistent `state` access proceeds through the same five-stage lifecycle within an invocation:

```
Read
    ↓
Modify
    ↓
Validate
    ↓
Stage
    ↓
Commit
```

**Read.** A bare reference to a `state` field's name resolves to its current value: either a value already staged earlier in this same transaction (by this invocation or an enclosing one that has already run), or, if untouched so far, its last-committed ledger value.

**Modify.** An assignment statement computes the new value to be associated with that field.

**Validate.** The compiler's static checks (types, [definite assignment](./LANGUAGE_SPEC.md#44-definite-assignment-of-state)) have already ruled out an entire class of invalid writes before runtime; at runtime, the only remaining validation is that the write is being performed by a member function of the contract that owns the field, per [LANGUAGE_SPEC.md §9.4](./LANGUAGE_SPEC.md#94-compiler-rules) — a rule the compiler enforces statically, making this stage, in a conforming implementation, always trivially satisfied by the time a program has compiled at all.

**Stage.** The new value is recorded in the current invocation's staging layer, per [§7](#7-execution-pipeline), and does not yet touch ledger storage.

**Commit.** If the invocation succeeds, the staged value merges upward into the caller's staging layer, or, at the top level, becomes the new last-committed ledger value.

## 10.1 Why Writes Are Never Immediately Committed

An immediate-write model would make every `state` assignment a durable ledger mutation the instant it executed, which is incompatible with [§2.4 (Every Failure Is Atomic)](#24-every-failure-is-atomic): a function that writes to three fields and then fails on the fourth statement would otherwise leave three real, permanent ledger writes behind despite having never successfully completed. Deferring every write's real effect to **Commit** is what allows failure, at any point in an invocation's execution, to be handled by discarding an in-memory layer rather than by attempting to undo already-durable changes.

## 10.2 Consistency Guarantees

Within a single invocation and everything it calls, every `state` read MUST observe the most recent write to that field made anywhere within the same top-level transaction so far, whether that write was staged by the current invocation, by an enclosing one, or by an already-successfully-returned nested call. A conforming runtime MUST NOT allow a read to observe a stale, pre-transaction value once that field has been written within the current transaction, and MUST NOT allow a read to observe a value staged by an invocation that has not yet successfully returned (since that invocation might still fail and have its staged value discarded).

## 10.3 Rollback Guarantees

If an invocation fails, per [§9](#9-failure-model), every value it staged — read, modified, validated, and staged, but never committed — MUST be discarded as though the invocation had never executed. A subsequent read of the same field, by any invocation still running after the failure has been handled or propagated, MUST observe the value as it stood immediately before the failing invocation began, not any intermediate value the failing invocation computed.

---

# 11. Event Model

## 11.1 Declaration and Emission

Events are declared with `event` and emitted with `emit`, per [LANGUAGE_SPEC.md §11](./LANGUAGE_SPEC.md#11-events). At runtime, an `emit` statement's arguments are evaluated at the point it executes and recorded, in order, as part of the executing invocation's contribution to the transaction's event record, per [§7](#7-execution-pipeline).

## 11.2 Ordering

Events are recorded in strict program execution order: within a single invocation, in the order their `emit` statements executed; across nested invocations, interleaved exactly as the invocations themselves interleaved (a nested call's events appear at the point in the caller's sequence where that call occurred). This ordering MUST be deterministic and MUST be identical across every conforming implementation observing the same execution, per [§2.1](#21-every-execution-is-deterministic).

## 11.3 Persistence and Visibility

Events emitted by an invocation that ultimately fails — whether that invocation's own failure or an enclosing invocation's failure that discards it, per [§9.5](#95-scoped-rollback-and-catching) — MUST be discarded along with that invocation's staged state changes. Only events belonging to invocations that are part of an ultimately successful top-level transaction become part of the durable, externally visible event record.

## 11.4 Guarantees

**Events NEVER modify blockchain state.** An event is a append-only, informational record of something that happened; it has no read API from within any Kyne contract, and no future invocation of any contract MAY observe, branch on, or be affected by a previously emitted event. Events exist for external observers — indexers, client applications, off-chain systems — not for on-chain logic. A conforming runtime MUST NOT provide any mechanism by which contract code queries previously emitted events, because doing so would make on-chain behavior depend on a side channel outside the state model in [§6](#6-state-model), which would violate [§2.2 (Every State Change Is Explicit)](#22-every-state-change-is-explicit)'s guarantee that `state` is the only thing that persists and influences future execution.

**Events only describe successful execution.** Per [§11.3](#113-persistence-and-visibility), an event that was emitted during a frame that did not ultimately succeed never becomes part of the durable record. An external observer MUST be able to trust that every event they see corresponds to logic that actually took effect — an event is not a log of "this was attempted," it is a record of "this happened."

---

# 12. Authorization Model

## 12.1 Identity Verification

Every authorization entry attached to a transaction identifies a specific `address`. The runtime's **Authorization** pipeline stage ([§7](#7-execution-pipeline)) establishes which addresses have supplied a structurally valid entry for this transaction, before any contract-specific code executes.

## 12.2 Signature Verification

For each authorization entry, the runtime cryptographically verifies that it was validly signed by (or otherwise validly authorized on behalf of) the address it claims to authorize, and that any entry-level constraints — such as an expiration ledger sequence — are currently satisfied. An entry that fails signature verification is not made available to any `auth(...)` call during execution, exactly as if it had never been submitted.

## 12.3 Permission Verification

During **Execute Function**, each `auth(addr)` statement checks whether a validly verified entry for `addr` exists and applies to the current invocation. This is the runtime enforcement point for [LANGUAGE_SPEC.md §10.1](./LANGUAGE_SPEC.md#101-the-auth-statement): the compiler generates the call that performs this check ([Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) — the mapping from `auth(addr)` to this check is fixed and documented), but the **runtime enforces it** at execution time, against the transaction's actual, cryptographically verified authorization entries. The compiler cannot make this determination statically, because whether a given transaction actually carries a valid entry for a given address is runtime information, not something knowable from source alone.

## 12.4 Continuation of Execution

If permission verification succeeds, execution proceeds to the statement following `auth(addr)`. If it fails, the invocation immediately becomes a Runtime Failure ([§9.4](#94-runtime-failure)) — no further statements in that invocation execute, and per [§9.5](#95-scoped-rollback-and-catching), the failure is uncatchable and propagates to a full transaction rollback.

## 12.5 Why Authorization Is Explicit

Authorization is checked only where a `auth(...)` statement appears in source, on an address the contract itself names explicitly as a parameter, because any ambient or inferred notion of "the authorized party" would reintroduce exactly the confused-deputy risk [LANGUAGE_SPEC.md §10.2](./LANGUAGE_SPEC.md#102-no-implicit-sender) rejects: a function written assuming a single implicit caller behaves incorrectly the moment it is invoked on someone else's behalf through a relayed or delegated call. Making every authorization check name its subject explicitly, and making that check a runtime-enforced verification against cryptographic entries rather than a compiler-trusted assumption, is what allows an auditor to answer "who must have approved this effect?" by reading the function alone.

---

# 13. Runtime Guarantees

A conforming Kyne runtime provides the following guarantees, without exception, to every executing contract:

**Deterministic execution.** Identical prior state, arguments, and execution context always produce identical results, state changes, and events, per [§2.1](#21-every-execution-is-deterministic). This holds regardless of which conforming implementation performs the execution.

**Atomic transactions.** A transaction's effects are visible in full or not at all, per [§1.2](#12-transactional) and [§2.4](#24-every-failure-is-atomic). There is no runtime-observable partially-applied transaction.

**Explicit authorization.** No effect requiring a specific party's approval occurs without a runtime-verified `auth(...)` check against that party's address, per [§12](#12-authorization-model). There is no default-authorized caller.

**Predictable storage.** A `state` read within a transaction always observes the most recent write within that same transaction, and never observes a value from an invocation that has not yet successfully returned, per [§10.2](#102-consistency-guarantees).

**Ordered events.** Emitted events are recorded in strict, deterministic program order, per [§11.2](#112-ordering), and only for invocations that are ultimately part of a successful transaction, per [§11.3](#113-persistence-and-visibility).

**No hidden mutations.** Every `state` change corresponds to an explicit assignment statement in the contract that owns the field, per [§2.2](#22-every-state-change-is-explicit). There is no mutation a reader cannot find by searching for assignments to a field's name.

**No undefined runtime behavior.** Every construct in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) has exactly one defined runtime meaning, specified either there (for static semantics) or in this document (for execution behavior). A conforming compiler MUST NOT accept a program whose runtime behavior this specification does not define; if such a gap is found, it MUST be closed by a KIP amending this document, not by an implementation choosing behavior unilaterally.

---

# 14. Runtime Invariants

The following invariants MUST always be true of every execution of every conforming Kyne contract, on every conforming implementation, without exception. They are the load-bearing rules the rest of this document is built to guarantee, restated here as an explicit checklist against which any future compiler change — including any optimization — MUST be validated.

**Contract execution is deterministic.** No optimization, caching strategy, or implementation choice MUST introduce any variation in output between two executions sharing identical prior state, arguments, and execution context. An optimization that is merely *usually* deterministic is not conforming.

**Persistent state changes only after successful completion.** No `state` write MUST become visible on the ledger before the invocation that staged it — and every invocation depending on it, up to the top level — has completed successfully, per [§7](#7-execution-pipeline) and [§10](#10-storage-behavior). A compiler optimization that speculatively writes state early, intending to roll back on failure, would violate this invariant the moment any external observer could read ledger state mid-transaction, and is therefore prohibited outright, not merely discouraged.

**Authorization is always explicit.** No effect MUST occur on the basis of an inferred, cached, or ambient authorization; every privileged effect traces to a specific `auth(addr)` statement checked against a specific, runtime-verified entry, per [§12](#12-authorization-model). An optimization MUST NOT elide an `auth(...)` call on the grounds that "the same address was already checked earlier in this invocation" unless this specification is amended by KIP to define such elision as safe and to specify its exact conditions — until then, every `auth(...)` call in source corresponds to a real runtime check, every time.

**Every emitted event belongs to exactly one successful transaction.** No event MUST be observable externally unless the invocation that emitted it, and everything depending on its success, is part of a transaction that committed, per [§11.3](#113-persistence-and-visibility). A compiler or runtime MUST NOT emit events eagerly ahead of confirming the enclosing invocation's success.

**Failed transactions never modify persistent state.** This is [§2.4](#24-every-failure-is-atomic) and [§10.3](#103-rollback-guarantees) restated as an invariant: an implementation MUST guarantee, not merely intend, that a rolled-back invocation leaves ledger storage byte-for-byte identical to its pre-invocation state.

**The compiler never generates undefined runtime behavior.** Every construct the compiler accepts MUST have behavior fully specified by [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) and this document. A compiler MUST reject, at compile time, any construct or combination of constructs whose runtime behavior is not fully determined by these specifications — it MUST NOT silently choose an unspecified behavior and ship it, because an unspecified behavior chosen by one version of the compiler is a behavior a later version is free to change, silently breaking [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution)'s promise to already-deployed contracts.

## 14.1 Why Optimizations May Never Violate These Invariants

A deployed Kyne contract's correctness has already been reviewed — by its authors, by its reviewers, and in many cases by a paid professional audit — against the runtime behavior this document defines. An optimization that changes observable behavior, even in a way its author believes is "equivalent," silently invalidates every one of those reviews, because "equivalent" was never proven against the full space of inputs an adversarial or merely unusual transaction might supply, only assumed. The invariants in this section are therefore not performance targets to be balanced against other goals — they are the boundary within which all optimization MUST occur. An optimization that cannot be implemented without touching one of these invariants is not a valid optimization for a conforming Kyne compiler; it is a proposal for a new runtime behavior, and MUST go through the KIP process in [§16](#16-future-runtime-evolution) like any other.

---

# 15. Runtime Design Principles

The rules in this document did not fall out of Soroban's constraints alone — they reflect deliberate choices about what kind of runtime Kyne wants to be, beyond what the underlying platform strictly requires.

**Predictability over cleverness.** Wherever a more clever runtime strategy — speculative execution, aggressive caching across invocations, inferred optimizations based on usage patterns — would make behavior harder for a developer or auditor to predict from source alone, this document chooses the less clever, more predictable option. A runtime that is occasionally faster but occasionally surprising is a worse runtime for a system whose entire value proposition rests on developers being able to reason precisely about what their code does with real money.

**Explicit state transitions.** Every stage in the [Execution Pipeline](#7-execution-pipeline) and every category in the [State Model](#6-state-model) exists to make "what changed, and when did it become real" a question with one unambiguous answer. This is not a stylistic preference carried over from the language's syntax — it is the runtime's own version of [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit), applied to execution rather than to text.

**Auditability.** Every rule in this document is phrased so that a reviewer, given a piece of Kyne source and this specification, can determine its exact runtime behavior without needing to run it, profile it, or trust an implementation's internal documentation. A runtime model that requires empirical observation to understand is not auditable, no matter how well-behaved the implementation happens to be in practice.

**Small runtime surface.** This document defines exactly three data categories, one execution pipeline, three call relationships, and four failure outcomes — not because smaller numbers are inherently virtuous, but because every additional case in any of these taxonomies is a case every future contract author and every future auditor must learn, and a case the compiler must correctly implement forever. The runtime surface is held to the same budget discipline as the keyword surface in [LANGUAGE_SPEC.md's Design Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place).

**No hidden behavior.** Every guarantee in [§13](#13-runtime-guarantees) and every invariant in [§14](#14-runtime-invariants) exists specifically to close off a category of behavior that might otherwise be left to "whatever the implementation happens to do." A runtime with hidden behavior is a runtime whose actual contract with its users is smaller than the one its documentation claims.

**Blockchain transparency.** Kyne's runtime model does not abstract away the fact that code is executing on a public blockchain with real economic consequences — it makes that fact more legible, not less. Authorization is explicit because on-chain authorization is a real, adversarial concern, not a formality. Atomicity is guaranteed because financial logic without atomicity is unsafe by construction. Every principle in this section ultimately serves the same goal stated in [FOUNDATION.md](./FOUNDATION.md#the-north-star): making it simpler to write secure Soroban contracts without sacrificing clarity or correctness.

---

# 16. Future Runtime Evolution

The runtime specification is **intentionally stable**. A deployed Kyne contract's correctness depends on the runtime behaving, for the entire lifetime of that contract's deployment, exactly as it behaved when the contract was written, reviewed, and audited. This document is therefore held to a higher stability bar than ordinary language features: a change to any MUST-level rule in [§2](#2-runtime-manifesto), [§13](#13-runtime-guarantees), or [§14](#14-runtime-invariants) is, by definition, a breaking change under [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution), and MUST be treated with the corresponding degree of caution.

Future runtime changes MUST go through the Kyne Improvement Proposal (KIP) process defined in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution). A KIP proposing a runtime change MUST additionally state, explicitly, which guarantee or invariant in this document it affects, and MUST demonstrate that the change either preserves that guarantee under a clarified definition or is being proposed as a deliberate, versioned breaking change with a migration path for already-deployed contracts.

Breaking runtime behavior is expected to be extremely rare, rarer than breaking syntax changes, because a syntax change affects only code not yet written, while a runtime behavior change can silently alter the meaning of code that is already deployed, immutable, and holding real value. Backward compatibility of runtime behavior is a first-order priority: where a future capability can be added without changing any existing MUST-level guarantee (for example, exposing an additional, opt-in storage durability class alongside the existing Persistent-only model), that additive path MUST be preferred over a path that redefines existing behavior, even where the redefinition would be more elegant in isolation.

---

# Non-Goals

This document does **not** define, and explicitly defers to other documents:

- **Compiler architecture** — the lexer, parser, AST, HIR, semantic analysis passes, optimization passes, and Rust code generation strategy are the subject of a forthcoming `COMPILER_ARCHITECTURE.md`, referenced in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md#compiler-architecture).
- **Language syntax and grammar** — the complete lexical structure, EBNF grammar, and static semantics of Kyne are defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Standard library surface** — the exact set of functions available for accessing execution context, collections, and conversions is the subject of a future Standard Library Specification; this document fixes only their runtime behavior and guarantees, not their names or signatures.
- **Package management** — reserved by [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem) and unaffected by anything in this document.

Nothing in this document should be read as specifying how a compiler internally achieves the behavior described here — only that it MUST achieve exactly this behavior, observably, from the outside.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission and values Kyne exists to serve.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy and KIP governance process this document's own evolution follows.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the syntax and static semantics whose runtime meaning this document defines.

Compiler architecture, the generated-Rust memory model's implementation strategy, and the standard library surface are each reserved for their own future specification documents, referenced above and in [Non-Goals](#non-goals). This document is the sole authority on runtime behavior until and unless amended by KIP; no other document may silently redefine an execution guarantee established here.

---

# Closing

This specification is authoritative for how every Kyne program executes. A compiler engineer implementing Kyne's runtime should be able to do so from this document alone, without needing to infer behavior from examples or from Soroban's own documentation for anything this document defines explicitly. Where a future engineer finds a runtime behavior this document does not address, that is a gap to be closed by a KIP, not a decision to be made silently in an implementation.
