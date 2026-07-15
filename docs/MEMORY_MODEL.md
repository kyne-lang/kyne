# MEMORY_MODEL.md

# The Kyne Memory & Storage Model

**Phase:** 0 — Foundations
**Milestone:** 6 — Memory & Storage Model
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on the reference compiler implementation and on every future compiler contributor.

This document defines how data exists, moves, persists, and is managed throughout the lifetime of a Kyne program. It describes memory **from the perspective of the language**, not from the perspective of Rust: every rule below states the observable behavioral contract a conforming compiler must uphold, not the specific algorithm a compiler must use to uphold it. Where more than one implementation strategy could satisfy a given rule, this document deliberately does not choose one — it fixes the observable behavior and leaves the strategy to [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) and to the reference implementation.

This document implements — and must never redefine — [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), and [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), all of which are locked, constitutional documents with respect to this one. In particular, this document elaborates on, and must remain fully consistent with, the value-semantics rule already established in [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only), the three-category state model already established in [RUNTIME_MODEL.md §6](./RUNTIME_MODEL.md#6-state-model), the storage-staging pipeline already established in [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline) and [§10](./RUNTIME_MODEL.md#10-storage-behavior), and the Rust Intermediate Representation already established as the stage where ownership decisions are made in [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations). Where this document appears to conflict with any of those five, this document is wrong and MUST be corrected by KIP.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings.

## The Fundamental Memory Principle

**Kyne developers never manage memory.** The compiler owns memory; the developer owns business logic. This is the single organizing idea behind every rule in this document, and it is immutable: the language MUST NOT expose ownership annotations, borrow checking, lifetime annotations, manual allocation, manual deallocation, or unsafe memory primitives, in this or any future version, without that exposure being a foundational break subject to the extraordinary process in [LANGUAGE_PRINCIPLES.md's Ten Commandments](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne), not an ordinary KIP. Every one of those six things is a compiler implementation detail, and this document treats them accordingly: it specifies what the compiler must guarantee, never how a developer would express or override it, because a developer has no such expression available to them at all.

---

# 1. Memory Philosophy

Kyne's approach to memory rests on four properties, each a direct consequence of [Kyne's three immutable pillars](./LANGUAGE_PRINCIPLES.md#foundation).

## Simplicity

Developers should never think about memory. Not "should rarely need to" — should never need to, in the ordinary course of writing a contract. A Kyne developer writes `let listing = Listing { seller, price, sold: false };` and reasons about `listing` exactly as they would reason about a value in TypeScript or Go: a self-contained thing with a name, usable until it goes out of scope, with no separate mental model of who owns it, who may read it, or when it stops being valid beyond ordinary lexical scoping.

This is not a claim that memory management is unimportant — it is a claim about where that importance should be *addressed*. [FOUNDATION.md](./FOUNDATION.md#why-kyne-exists) identifies Rust's ownership and borrowing model as one of the concepts a Soroban developer must master today that has nothing to do with the business logic of a smart contract. Kyne's answer is not to make that model easier to learn — it is to remove the requirement to learn it at all, by moving every memory-management decision into the compiler, where it can be made once, correctly, by compiler engineers, rather than repeatedly, with variable success, by every contract author.

## Safety

Memory corruption MUST be impossible to express in Kyne source. This is a stronger claim than "memory corruption is checked and rejected" — it is a claim that the language has no construct capable of describing a dangling reference, a use-after-free, or a data race in the first place. [Chapter 14](#14-memory-safety) makes this guarantee precise; the philosophical point here is that Kyne achieves memory safety by **non-expressibility** rather than by verification. A verifying system (a borrow checker, a runtime bounds check) can reject an unsafe program written in a language capable of expressing unsafety. Kyne's surface language is simply incapable of expressing it, which is a stronger and simpler property: there is no proof obligation to discharge, because there is no unsafe category of program to rule out.

## Predictability

Values behave consistently, every time, regardless of what the compiler chooses to do underneath them. Assigning a value to a new binding, passing it to a function, or returning it from one always produces the same *observable* result — an independent, usable value — regardless of whether the compiler's generated Rust actually clones it, moves it, or passes a reference to it. Predictability here does not mean "the generated Rust is always the same shape" (that would forbid the compiler from ever improving its output); it means "the Kyne-level behavior a developer can observe is always the same," which is precisely the boundary [Chapter 3](#3-value-semantics) formalizes.

## Transparency

Every memory decision the compiler makes MUST remain explainable, in principle, to a developer who asks why. This is the memory-model-specific expression of [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction): Kyne is permitted to make sophisticated memory decisions on a developer's behalf, but it is never permitted to make those decisions in a way that could not, in principle, be explained by pointing at a specific rule in this document and a specific piece of generated Rust. [Chapter 19](#19-memory-transparency) develops this into a dedicated requirement; it is listed here as a philosophical commitment because it is the property that prevents "the compiler owns memory" from quietly becoming "the compiler does something to memory that no one can account for."

---

# 2. Memory Categories

Kyne exposes exactly three categories of data, restated here from the memory perspective. These are the same three categories [RUNTIME_MODEL.md §6](./RUNTIME_MODEL.md#6-state-model) defines from the execution perspective; this chapter does not redefine their runtime behavior, only elaborates their memory-specific treatment.

## Local Memory

Introduced by `let` bindings, function parameters, and every intermediate value produced during expression evaluation (see [Chapter 7](#7-temporary-objects)).

- **Lifetime.** Bounded exactly by the enclosing lexical block, per [LANGUAGE_SPEC.md §4.5](./LANGUAGE_SPEC.md#45-scope). A Local value never outlives the invocation that created it.
- **Allocation.** The compiler MAY realize a given Local value on the stack or on the heap, and MAY change that choice between compiler versions, provided the choice has no observable effect — see [Chapter 6](#6-allocation-model).
- **Cleanup.** Deterministic: every Local value is released, in full, no later than the end of the block that introduced it. There is no deferred, best-effort, or probabilistic cleanup.
- **Visibility.** Lexical only. A Local value is never visible outside its declaring function's own execution — not to a caller, not to a callee, and never to a separate invocation, per [RUNTIME_MODEL.md §8](./RUNTIME_MODEL.md#8-function-invocation-model).
- **Compiler behavior.** The compiler is free to choose the cheapest correct realization strategy for each Local value independently; a `let` holding a small `i64` and a `let` holding a large `struct` are not required to be treated identically, so long as both honor every rule in this document.

## Persistent Storage

Introduced by `state` field declarations at contract scope, per [LANGUAGE_SPEC.md §4.3](./LANGUAGE_SPEC.md#43-state-persistent-contract-storage).

- **Persistence.** Survives across separate invocations and separate transactions, backed by the Soroban ledger's persistent storage, subject to the archival lifecycle in [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle).
- **Serialization.** Every Persistent Storage value MUST be converted to and from a durable, storable byte representation at the ledger boundary — see [Chapter 8](#8-storage-mapping).
- **Visibility.** Direct-by-name within any member function of the declaring contract; never accessible from library code, never accessible from a different contract instance, per [RUNTIME_MODEL.md §6.2](./RUNTIME_MODEL.md#62-persistent).
- **Compiler mapping.** Every `state` field maps to a distinct entry in the contract instance's persistent storage, keyed by the field's declared name, per [RUNTIME_MODEL.md §9.1](./RUNTIME_MODEL.md#91-persistent-state).
- **Storage lifetime.** The contract instance's entire on-chain existence, subject to explicit lifetime extension and archival — see [Chapter 9](#9-storage-lifecycle).

## Constants

Introduced by `const` declarations, per [LANGUAGE_SPEC.md §4.2](./LANGUAGE_SPEC.md#42-const-compile-time-constants).

- Constants have **no runtime allocation at all.** A `const` is fully evaluated at compile time and embedded directly into the generated Rust at every point it is used — it is never dynamically allocated, on the stack, on the heap, or in persistent storage, because it never exists as a runtime value distinct from the literal the compiler inlines.
- **Compiler treatment.** The compiler MUST resolve every `const` to a concrete value during compilation and MUST NOT generate any runtime code path that computes a `const`'s value dynamically. A `const` that cannot be fully evaluated at compile time is not a valid Kyne program, per [LANGUAGE_SPEC.md §4.2](./LANGUAGE_SPEC.md#42-const-compile-time-constants).

## Why Exactly Three

Kyne intentionally exposes **only** these three memory categories. There is no fourth category — no "shared," no "cached," no "temporary-but-not-local" classification available to a Kyne program. This closed set is not a limitation this document apologizes for; it is the same design discipline [RUNTIME_MODEL.md §6.7](./RUNTIME_MODEL.md#67-why-exactly-three-categories) already applies to the state model, restated here at the memory level: every value in a Kyne program must answer exactly one question — does it survive the current execution, and if so, does it belong to this contract instance forever, or was it inlined away before execution ever began — and three categories are the minimum, and the maximum, needed to answer it completely.

---

# 3. Value Semantics

This is the foundational chapter of this document, and it elaborates the rule LANGUAGE_SPEC.md already establishes at the language level: **Kyne developers always reason about values, never about ownership, references, or aliasing.**

Every assignment, every function argument, and every function return in Kyne source behaves, from the developer's perspective, as though it produces or consumes a complete, independent value. Two `let` bindings are never observably the "same" storage such that mutating one could affect the other; a `struct` passed to a function is never observably shared with the caller such that the function could mutate the caller's copy through it. This is true **regardless of the type involved** — a `bool`, an `i128`, a `struct`, an `enum`, a `list<T>`, or a `map<K, V>` all obey the identical value-semantics contract, with no type-by-type exception.

The compiler is free to implement this contract using any combination of the following strategies, individually or in combination, chosen per value, per call site, or per compiler version, entirely at its own discretion:

- **Borrowing** — passing a reference to an existing value rather than a new copy of it, where the compiler can prove doing so is observably indistinguishable from a copy.
- **Moving** — transferring an existing value's underlying storage to a new binding without duplicating it, where the compiler can prove the original binding is never used again.
- **Cloning** — producing a genuine, independent duplicate of a value's underlying storage.
- **Referencing** — holding an indirect pointer to a value's storage internally, without that indirection ever being observable from Kyne source.
- **Copy elision** — eliding an intermediate copy entirely by constructing a value directly into its final destination.

**The one constraint that binds every one of these strategies:** observable behavior MUST never change as a result of which strategy the compiler chooses. "Observable" here means observable from Kyne source and from the runtime guarantees in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — it does not mean "observable by inspecting the generated Rust," since the generated Rust is explicitly permitted, and expected, to vary in its choice of strategy across compiler versions, per [Chapter 10](#10-memory-optimizations).

## Why This Abstraction Exists

Value semantics with a compiler-chosen realization strategy underneath is what lets Kyne make two claims simultaneously that would otherwise be in tension: that developers never think about memory ([Chapter 1](#1-memory-philosophy)), and that generated Rust remains efficient and idiomatic ([COMPILER_ARCHITECTURE.md §14.3](./COMPILER_ARCHITECTURE.md#143-reviewability)). A language that exposed only true, naive, always-clone-everything value semantics would be simple to reason about but would generate needlessly wasteful Rust; a language that exposed Rust's own borrowing model directly would generate efficient Rust but would reintroduce exactly the learning burden [FOUNDATION.md](./FOUNDATION.md#why-kyne-exists) identifies as the problem. Separating the *contract* (values behave like values) from the *mechanism* (however the compiler chooses to realize that) is what allows the mechanism to improve indefinitely, across every future compiler version, without ever being a language-level breaking change — because the mechanism was never part of the language to begin with.

---

# 4. Ownership Model

Kyne has no ownership model exposed to the developer. This is not an omission to be filled in by a future version; it is the permanent design, restated here as its own chapter because "ownership" is frequently assumed, by developers coming from Rust, to be a concept every systems-adjacent language must eventually surface. Kyne does not.

**The compiler owns ownership.** Every decision that a language like Rust would require a developer to annotate or reason about explicitly — whether a given value is borrowed, moved, cloned, or referenced at a given point in the program; how long a temporary allocation should live; what the effective lifetime of a value is — is made entirely by the compiler, during [RIR lowering](./COMPILER_ARCHITECTURE.md#12-intermediate-representations), with zero input from, and zero visibility to, the Kyne source that produced it.

Concretely, the compiler determines, for every value:

- **Borrow** — whether a given use of a value can be satisfied by a reference rather than a copy.
- **Move** — whether a given use of a value can consume its storage directly rather than duplicating it.
- **Clone** — whether a given use of a value requires an independent duplicate.
- **Reference** — whether internal indirection is used to avoid an otherwise-necessary copy.
- **Temporary allocation** — where and how long an intermediate value needs to exist during evaluation.
- **Lifetime** — how long a given piece of underlying storage must remain valid to satisfy every use that depends on it.

## Why Ownership Remains Invisible

Every one of these six decisions is, in a language like Rust, something the developer must at least be aware of, and frequently must annotate explicitly, because Rust's ownership model is part of *how the developer proves the program is safe* to the compiler. Kyne inverts this relationship entirely: safety is guaranteed by [Chapter 14](#14-memory-safety)'s non-expressibility argument, not by a proof the developer constructs, which means there is no safety-relevant reason for ownership to ever surface in Kyne source. Making ownership invisible is therefore not a simplification made *despite* safety concerns — it is possible only *because* Kyne's safety guarantee does not depend on the developer participating in an ownership proof at all. A future version of Kyne MUST NOT introduce ownership-relevant syntax as a means of expressing something the compiler cannot already infer on its own, because doing so would reintroduce exactly the cognitive burden this chapter exists to remove, for a safety property Kyne already has without it.

---

# 5. Lifetime Model

Kyne exposes no lifetime annotations, and none are ever inferred *from source* in the sense of appearing anywhere a developer could read or write them — lifetimes are a purely internal compiler concept, inferred entirely from the structure of the program during RIR lowering.

**Creation.** A value's lifetime begins at the point its defining expression is evaluated — a `let` binding's initializer, a function parameter's argument at the call site, or a temporary's producing sub-expression, per [Chapter 7](#7-temporary-objects).

**Reachability.** A value remains live for exactly as long as some later point in the program could still observe it — through its original binding, through a value derived from it that is itself still live, or through a `state` write that has captured it into Persistent Storage. The compiler computes reachability directly from the HIR's structure (per [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations)); it is not a runtime computation and has no runtime cost.

**Destruction.** A value's lifetime ends, and its underlying storage MUST be released, at the first point after which it is provably unreachable — no later than the end of its enclosing lexical block ([Chapter 2](#2-memory-categories)'s Local category), and, for a genuine temporary, frequently much earlier ([Chapter 7](#7-temporary-objects)).

**Temporary lifetimes.** An intermediate value with no `let` binding of its own — the result of a sub-expression inside a larger expression — has a lifetime bounded by, at most, the statement that produced it, and the compiler MAY shorten that lifetime further whenever it can prove the temporary is not observed again before that point.

**Compiler responsibilities.** The compiler MUST compute a valid lifetime for every value such that no value is ever released while a still-reachable reference to it exists (this is the safety-critical direction — see [Chapter 14](#14-memory-safety)) and MUST release every value no later than the point this document specifies (this is the leak-prevention direction — see [Chapter 16](#16-runtime-guarantees)). Both directions are compiler obligations with no developer-visible counterpart.

## Why Lifetime Syntax Does Not Exist

Lifetime annotations exist in a language like Rust because the compiler's own lifetime-inference algorithm cannot always determine a safe lifetime assignment without a hint, and because making lifetimes explicit is part of how such a language documents a function's aliasing contract to its callers. Kyne needs neither: because Kyne exposes no references or aliasing at the source level at all (per [Chapter 4](#4-ownership-model)), there is no aliasing contract for a function signature to document, and because Kyne's compiler has complete, whole-program visibility into every value's construction and use (there being no external, unannotated boundary for a value to cross, since Kyne has no dynamic linking or foreign-function boundary at the source level), lifetime inference is always fully determined by the program's own structure, with no case requiring a developer-supplied hint. Lifetime syntax would therefore be pure, unjustified overhead — a keyword that has not earned its place, in the exact sense [LANGUAGE_SPEC.md's Design Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) rejects.

---

# 6. Allocation Model

This chapter defines **allocation categories** — the implementation-level realizations available to the compiler — as distinct from the **memory categories** in [Chapter 2](#2-memory-categories), which are language-level concepts a developer actually reasons about. The two are related but not identical, and this document deliberately keeps them separate to avoid conflating "what a developer declares" with "how the compiler realizes it":

| Memory category (language-level) | Permitted allocation categories (implementation-level) |
|---|---|
| Local | Stack, Heap |
| Persistent | Persistent Storage (always) |
| Constant | None — inlined, never allocated |

**Stack.** The compiler MAY realize any Local value whose size is known at compile time and whose lifetime is provably bounded by a single, non-escaping execution frame on the stack. This is the cheapest available realization and the compiler SHOULD prefer it wherever both conditions hold.

**Heap.** The compiler MAY realize a Local value on the heap where its size is not statically bounded (for example, a `list<T>` or `map<K, V>` whose length is not known at compile time) or where its lifetime cannot be proven to stay within a single stack frame.

**Persistent Storage.** Every value written to a `state` field is, by the definition of the Persistent memory category, realized in the Soroban ledger's persistent storage — this is not optional and is the only allocation category available to Persistent values, per [Chapter 8](#8-storage-mapping).

**Allocation.** The compiler MAY allocate storage for a Local value eagerly, at the point of first use, or MAY defer allocation until it is structurally necessary (for example, deferring a heap allocation for a `list<T>` until its first `.push` call, if the compiler can prove no prior use requires storage to already exist).

**Deallocation.** Every allocation the compiler introduces MUST be deterministically released no later than the point [Chapter 5](#5-lifetime-model) establishes as that value's destruction point. There is no deferred, generational, or best-effort deallocation strategy available to a conforming implementation — see [Chapter 14](#14-memory-safety) for why probabilistic reclamation (garbage collection) is not a legitimate strategy for Kyne.

**Compiler decisions.** Every choice in this chapter — stack versus heap, eager versus deferred allocation — is made during RIR lowering ([COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations)) and MAY differ between two structurally similar values, between two compiler versions, or between two optimization settings, with no observable consequence to Kyne source.

**Developers never choose allocation strategy.** There is no Kyne syntax through which a developer could request stack allocation, request heap allocation, or otherwise influence this chapter's decisions — doing so would be exactly the kind of ownership-adjacent annotation [Chapter 4](#4-ownership-model) rules out.

---

# 7. Temporary Objects

A **temporary** is any value produced during expression evaluation that has no `let` binding of its own — the intermediate result of a sub-expression inside a larger expression, a function's return value before it is bound or consumed, or a struct literal constructed directly as an argument.

**Expression evaluation.** Kyne evaluates expressions left to right, per [LANGUAGE_SPEC.md §7.4](./LANGUAGE_SPEC.md#74-function-calls) for call arguments and the natural evaluation order implied by each expression form elsewhere in [LANGUAGE_SPEC.md §7](./LANGUAGE_SPEC.md#7-expressions). Every sub-expression that is not itself a bare identifier or literal produces a temporary value as it evaluates.

**Intermediate values.** In `let total = balance_of(from) + balance_of(to);`, each call to `balance_of` produces a temporary `i128` value that exists only long enough to be consumed by the `+` operator; neither temporary has a name, and neither is reachable after the enclosing statement completes.

**Compiler cleanup.** A temporary's underlying storage — if the compiler chose to allocate any at all, per [Chapter 6](#6-allocation-model) — MUST be released no later than the end of the statement that produced it, and the compiler MAY release it earlier whenever it can prove the temporary is not observed again before that point. This is the same destruction rule [Chapter 5](#5-lifetime-model) establishes generally, specialized to the common case of a value with no explicit binding at all.

**Lifetime.** A temporary's lifetime is always bounded by, at most, a single statement. Kyne provides no mechanism to extend a temporary's lifetime beyond the statement that produced it — a temporary that needs to outlive its producing statement MUST be given an explicit `let` binding in source, which converts it from a temporary into an ordinary Local value with the corresponding, longer lifetime.

**Optimization.** The compiler MAY eliminate a temporary's allocation entirely wherever it can construct the value directly into its final destination (copy elision, per [Chapter 3](#3-value-semantics)) — for example, `let listing = Listing { seller, price, sold: false };` MAY be realized by the compiler as a single, direct construction of `listing`'s storage, with no separate, intermediate struct value ever materialized and then copied into place.

**Example.**

```kyne
public fn total_balance(a: address, b: address) -> i128 {
    return balance_of(a) + balance_of(b);
}
```

Here, the two `balance_of(...)` calls each produce a temporary `i128`. Both temporaries are consumed immediately by the `+` operator within the same statement and MUST be released no later than the `return` statement's completion; the compiler MAY, and typically will, avoid allocating any separate storage for them at all, since `i128` values are cheaply realized without heap allocation regardless.

---

# 8. Storage Mapping

Every `state` field maps to persistent Soroban storage according to the following rules.

**Key mapping.** A `state` field's declared name is its storage key, per [RUNTIME_MODEL.md §9.1](./RUNTIME_MODEL.md#91-persistent-state) — there is no separate, developer-visible key derivation, and two `state` fields in the same contract can never collide in storage, since [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) already requires every contract member's name to be unique within its declaring contract.

**Serialization.** Writing a `state` field requires converting its in-memory, Kyne-typed value into a durable byte representation suitable for the ledger; reading it requires the reverse. The compiler MUST use a serialization format capable of round-tripping every value of every type describable in [LANGUAGE_SPEC.md §6](./LANGUAGE_SPEC.md#6-types) — primitives, the five closed intrinsic parametric types, and every user-defined `struct` and `enum` — without loss.

**Deserialization.** A `state` field's byte representation MUST deserialize back into a value structurally and observably identical to the value that was originally serialized — deserialization MUST NOT be lossy for any type Kyne permits in `state` position, and MUST fail as a Runtime Failure (per [RUNTIME_MODEL.md §9.4](./RUNTIME_MODEL.md#94-runtime-failure)), never silently substitute a default value, if the stored bytes cannot be deserialized as the field's declared type.

**Persistent layout.** This document does not fix a specific on-the-wire byte format — that is an implementation detail of the reference compiler, not a language-level guarantee, and different conforming implementations MAY use different serialization formats internally, provided each is internally consistent (an implementation always deserializes what it itself serialized) and satisfies the round-trip guarantee above.

**Compiler responsibilities.** The compiler MUST generate, for every `state` field, the serialization and deserialization logic required to move that field's value between its in-memory representation and its ledger byte representation, and MUST do so without any developer-authored serialization code — there is no Kyne syntax for a developer to customize how a `state` field is serialized, consistent with [Chapter 4](#4-ownership-model)'s "developer never manages memory-adjacent mechanism" principle extended to storage.

**Generated Rust behavior.** The generated Rust issues a typed read against the Soroban host's persistent storage API at each point a `state` field is read for the first time within an invocation, and a typed write at each point it is assigned, per the staging discipline in [Chapter 9](#9-storage-lifecycle). This document does not name the specific Soroban SDK functions involved, since that is a code-generation detail governed by [COMPILER_ARCHITECTURE.md §14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation), not a memory-model guarantee.

---

# 9. Storage Lifecycle

Every `state` access proceeds through the following seven-stage lifecycle:

```
Read
    ↓
Deserialize
    ↓
Operate
    ↓
Validate
    ↓
Serialize
    ↓
Stage
    ↓
Commit
```

**Read.** The field's current byte representation is retrieved — either from the ledger's last-committed bytes, or from bytes already staged earlier in the same transaction, per [RUNTIME_MODEL.md §10.2](./RUNTIME_MODEL.md#102-consistency-guarantees).

**Deserialize.** The retrieved bytes are converted into an in-memory, Kyne-typed value, per [Chapter 8](#8-storage-mapping).

**Operate.** Kyne source reads and computes against the deserialized value — this is the memory-model-level realization of [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline)'s **Execute Function** stage, insofar as it touches this particular field.

**Validate.** As in [RUNTIME_MODEL.md §10](./RUNTIME_MODEL.md#10-storage-behavior), static checks (types, definite assignment) have already ruled out invalid writes before runtime; the only remaining validation is that the write originates from a member function of the owning contract.

**Serialize.** The new in-memory value is converted back into its durable byte representation, per [Chapter 8](#8-storage-mapping).

**Stage.** The serialized bytes are recorded in the current invocation's staging layer — not yet written to the ledger, per [RUNTIME_MODEL.md §7.1](./RUNTIME_MODEL.md#71-why-state-is-staged-before-commit).

**Commit.** If the invocation succeeds, the staged bytes merge upward into the caller's staging layer or, at the top level, become the new last-committed ledger bytes, per [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline).

## 9.1 Relationship to RUNTIME_MODEL.md's Storage Lifecycle

[RUNTIME_MODEL.md §10](./RUNTIME_MODEL.md#10-storage-behavior) defines a five-stage lifecycle — Read, Modify, Validate, Stage, Commit — from the *execution* perspective. This chapter's seven-stage lifecycle is a memory-level elaboration of the same underlying process, not a competing definition: **Deserialize** is inserted immediately after RUNTIME_MODEL.md's **Read** (making explicit the byte-to-value conversion RUNTIME_MODEL.md abstracts away, since it is not concerned with serialization mechanics), **Operate** corresponds exactly to RUNTIME_MODEL.md's **Modify**, and **Serialize** is inserted immediately before RUNTIME_MODEL.md's **Stage** (making explicit the reverse, value-to-byte conversion). The two lifecycles describe the same five semantic moments; this chapter simply names the two additional, memory-specific boundary conversions that occur within them.

## 9.2 Rollback Interaction

If the invocation performing this lifecycle fails — a Recoverable Error caught nowhere, or a Runtime Failure — every value staged by it, at every stage from **Serialize** through **Stage**, MUST be discarded without ever reaching **Commit**, exactly as [RUNTIME_MODEL.md §10.3](./RUNTIME_MODEL.md#103-rollback-guarantees) requires at the execution level. At the memory level, this means: the deserialized, in-memory representation produced during **Deserialize** and mutated during **Operate** is simply released as an ordinary Local value (per [Chapter 2](#2-memory-categories)'s Local cleanup rule) once the invocation frame that produced it is discarded, and the serialized bytes staged during **Stage** are dropped from the staging layer without ever being written to the ledger. No special memory-level rollback mechanism is required beyond the ordinary release of Local memory and the ordinary discarding of a staging layer — this is a direct consequence of state changes being staged as data, per [RUNTIME_MODEL.md §7.1](./RUNTIME_MODEL.md#71-why-state-is-staged-before-commit), rather than being applied destructively and requiring a separate undo step.

---

# 10. Memory Optimizations

The following optimizations are permitted, in addition to those already named in [Chapter 3](#3-value-semantics):

**Borrowing.** Already covered in [Chapter 3](#3-value-semantics) — restated here as a memory-optimization technique specifically: avoiding a copy by proving a reference is observably sufficient.

**Allocation reuse.** The compiler MAY reuse a single underlying allocation across multiple Local values in sequence where it can prove no overlapping liveness — for example, reusing a loop iteration variable's storage across iterations rather than allocating fresh storage each time.

**Temporary elimination.** As in [Chapter 7](#7-temporary-objects), the compiler MAY eliminate a temporary's allocation entirely via copy elision wherever the temporary's value can be constructed directly into its final destination.

**Inlining.** As in [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization), inlining a trivial internal function call MAY eliminate the allocation of a separate stack frame for that call, folding its Local memory directly into the caller's frame.

**Stack promotion.** A value that would otherwise be heap-allocated (per [Chapter 6](#6-allocation-model)'s general rule for dynamically sized values) MAY be promoted to stack allocation wherever the compiler can prove, for a specific instance, that its size is in fact bounded and its lifetime does not escape the current frame — for example, a `list<T>` local built from a small, statically known number of `.push` calls with no further growth.

**Storage caching.** A `state` field read more than once within a single invocation, with no intervening write to that field, MAY have its deserialized value cached in Local memory after the first read, avoiding repeated deserialization. This cache MUST be treated as invalidated the instant any write to that field occurs within the same invocation, consistent with [RUNTIME_MODEL.md §10.2](./RUNTIME_MODEL.md#102-consistency-guarantees)'s requirement that every read observe the most recent write.

**Optimizations MUST NEVER change observable behavior.** This is not a chapter-specific rule — it is the same absolute precondition [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization) already establishes for every compiler optimization, restated here because memory optimizations are a category where the temptation to make an "almost always safe" exception is highest (for example, a storage cache that is invalidated on *most* writes but misses an edge case). No such exception exists: a memory optimization that cannot be proven safe for every reachable case is not eligible to ship, regardless of how rare the unsafe case would be in practice, per [Principle 3 of COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization).

---

# 11. Copy Behavior

Assignment in Kyne always behaves, from the developer's perspective, as though it produces an independent copy. `let b = a;` MUST make `b` a value entirely independent of `a` — no Kyne program can ever observe a case where a later change involving `b` affects `a`, or vice versa, regardless of `a`'s type.

The compiler decides, invisibly, whether to realize this as an actual copy, a borrow, a move, a clone, or an internal reference. This decision:

- has no Kyne-level syntax to request or inspect it,
- MAY differ between two textually identical assignments depending on surrounding context (for example, whether `a` is used again after the assignment),
- MAY differ between compiler versions for identical source, and
- MUST NEVER be observable as a behavioral difference from Kyne source.

## Examples

```kyne
let a = Transaction { to, amount, approvals: 0, executed: false };
let b = a;
```

From the developer's perspective, `b` is now a complete, independent value. Whether the compiler's generated Rust performs a genuine clone of `a`'s underlying storage, moves it (if `a` is never referenced again after this point), or elides the distinction entirely by constructing `b` directly, is an RIR-lowering decision with no bearing on what this program can be observed to do.

```kyne
public fn total(items: list<i128>) -> i128 {
    let sum = 0;
    for value in items {
        sum += value;
    }
    return sum;
}
```

`items` is passed into `total` by value, per [LANGUAGE_SPEC.md §5.2](./LANGUAGE_SPEC.md#52-parameters-and-return-types). The caller's own `list<i128>` remains fully intact and unaffected by anything `total` does, regardless of whether the compiler's generated Rust actually duplicates the list's storage at the call site or passes an internal reference — since `total` never mutates or reassigns `items`, the compiler is very likely, though not obligated, to choose a borrow here rather than a clone, purely as a performance decision invisible to this source.

---

# 12. Collections

| Type | Allocation | Persistence | Copy semantics | Compiler behavior |
|---|---|---|---|---|
| `list<T>` | Heap-backed when dynamically sized; MAY be stack-promoted per [§10](#10-memory-optimizations) when statically bounded. | As a `state` field, backed by persistent storage; see below for storage-collection realization. | Value semantics: assigning or passing a `list<T>` behaves as an independent copy, per [Chapter 11](#11-copy-behavior). | `.push`/`.get`/`.len`/`.remove` per [LANGUAGE_SPEC.md §6.2.1](./LANGUAGE_SPEC.md#621-listt-methods) — realized by the compiler with no developer-visible allocation strategy. |
| `map<K, V>` | Heap-backed. | As a `state` field, backed by persistent storage; see below. | Value semantics, identical to `list<T>`. | `.get`/`.set`/`.has`/`.remove`/`.len` per [LANGUAGE_SPEC.md §6.2.2](./LANGUAGE_SPEC.md#622-mapk-v-methods). |
| `bytes<N>` | Fixed-size, stack-eligible (no dynamic growth is possible, per [LANGUAGE_SPEC.md §6.2](./LANGUAGE_SPEC.md#62-collections)). | As a `state` field, serialized as a fixed-length byte sequence. | Value semantics. | Trivially copyable at the implementation level, since its size is always statically known. |
| `Option<T>` | Matches `T`'s own allocation category, plus a small discriminant; introduces no additional heap allocation beyond what `T` itself would require. | As a `state` field, serialized as a discriminant plus, where present, `T`'s own serialized bytes. | Value semantics, following `T`'s. | `Some`/`None` construction and `.is_some`/`.is_none`/`.unwrap_or` per [LANGUAGE_SPEC.md §6.6](./LANGUAGE_SPEC.md#66-option-and-result). |
| `Result<T, E>` | Matches whichever of `T` or `E` is present, plus a discriminant. | As above, generalized to two possible payload types. | Value semantics, following whichever of `T`/`E` is present. | `Ok`/`Err` construction and `.is_ok`/`.is_err`/`.unwrap_or` per [LANGUAGE_SPEC.md §6.6](./LANGUAGE_SPEC.md#66-option-and-result). |
| `struct` | Composed of its fields' own allocation categories. | As a `state` field (or nested within one), serialized field by field. | Value semantics; structural equality is automatic per [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only) and has no memory cost of its own. | No methods, per [LANGUAGE_SPEC.md §6.3](./LANGUAGE_SPEC.md#63-structs) — all logic is free functions operating on the value. |
| `enum` | Composed of the active variant's payload, plus a discriminant. | As a `state` field (or nested within one), serialized as discriminant plus active payload. | Value semantics. | Consumed via `match`, per [LANGUAGE_SPEC.md §7.7](./LANGUAGE_SPEC.md#77-pattern-matching). |
| Nested collections (e.g. `map<u64, map<address, bool>>`) | Each level follows its own type's rule, recursively; an outer collection's allocation does not itself allocate storage for entries it does not yet contain. | Nested collections compose the same serialization rule recursively — an outer entry's bytes fully encode its nested collection's contents. | Value semantics, applied recursively — copying an outer collection is observably equivalent to independently copying every value it contains, at every level. | No additional methods beyond composing the outer and inner types' own method tables. |

## 12.1 Storage Collections

A `list<T>`- or `map<K, V>`-typed `state` field is a **storage collection**, and its memory realization differs from an equivalent Local collection in one important respect: the compiler MAY choose between two strategies, invisibly, per field:

- **Single-entry realization** — the entire collection is serialized as one value under the field's storage key, read and rewritten in full on every access.
- **Per-element realization** — for a `map<K, V>`, each key-value pair MAY instead be realized as its own independent persistent storage entry, keyed by a derivation of the field's name and the specific key, so that a `.get`/`.set` on one key does not require reading or rewriting every other key's data.

Both realizations are permitted, and the compiler MAY choose per field, informed by expected access patterns; **the choice MUST be entirely invisible to Kyne source and MUST NOT change any behavior specified by [LANGUAGE_SPEC.md §6.2](./LANGUAGE_SPEC.md#62-collections)'s method tables.** A per-element realization is a natural fit for `map<K, V>`, whose `.get`/`.set`/`.has`/`.remove` operations are already independent per key; it is a substantially more constrained fit for `list<T>`, whose `.remove(index)` semantics (shifting every subsequent element) make a naive per-index realization costly, and an implementation choosing a per-element strategy for `list<T>` MUST still preserve `list<T>`'s exact ordering and indexing semantics regardless of the internal storage layout chosen.

---

# 13. Function Memory

**Parameter passing.** Every parameter is passed by value, per [LANGUAGE_SPEC.md §5.2](./LANGUAGE_SPEC.md#52-parameters-and-return-types) — the callee always receives what behaves as an independent copy, with the compiler free to realize that behavior via borrowing, moving, or cloning per [Chapter 3](#3-value-semantics).

**Return values.** A function's return value is likewise governed by value semantics; the compiler MAY use move semantics or copy elision internally (analogous to return-value optimization) to avoid an unnecessary duplicate when constructing the caller's binding directly from the callee's result.

**Temporary allocation.** Arguments are evaluated into temporaries before a call proceeds, per [Chapter 7](#7-temporary-objects), and those temporaries are released no later than the end of the statement containing the call, exactly as for any other temporary.

**Stack frames.** Each function invocation is associated with at least one execution frame holding its Local memory. This is related to, but distinct from, the *invocation frame* concept in [RUNTIME_MODEL.md §7](./RUNTIME_MODEL.md#7-execution-pipeline) and [§8](./RUNTIME_MODEL.md#8-function-invocation-model): RUNTIME_MODEL.md's invocation frame governs the scope of *state staging and rollback*, while this chapter's execution frame governs the scope of *Local memory*. The two coincide for every non-inlined call, but per [Chapter 10](#10-memory-optimizations)'s inlining optimization, an inlined internal function call MAY share its caller's execution frame for Local-memory purposes while still being, per [RUNTIME_MODEL.md §8.1](./RUNTIME_MODEL.md#81-internal-function-calls), the same single invocation frame from a staging perspective — the two concepts were never required to produce a new instance in lockstep, and inlining is precisely the case where they diverge, harmlessly.

**Cleanup.** All Local memory belonging to a given execution frame is released, deterministically, when that frame ends (the function returns, throws, or is subsumed by a Runtime Failure) — per [Chapter 2](#2-memory-categories), there is no possibility of a Local value outliving its frame, since there is no mechanism by which a developer could request that it do so short of writing it into `state`.

**Compiler optimization.** Subject to every rule in [Chapter 10](#10-memory-optimizations), including inlining, stack promotion, and allocation reuse across a function's execution.

**Memory visibility.** A given invocation's Local memory is never visible to any other invocation — not its caller, not its callee, not a sibling invocation elsewhere in the same transaction — per [RUNTIME_MODEL.md §6.1](./RUNTIME_MODEL.md#61-local)'s visibility rule, restated here as a memory-isolation guarantee rather than a scoping one.

**Execution lifetime.** A function's Local memory exists for exactly the duration of that function's own execution and no longer; nothing about a function's Local memory persists past its return except whatever it has explicitly written into `state` along the way.

---

# 14. Memory Safety

A conforming Kyne implementation guarantees the following, unconditionally, for every valid Kyne program:

**No dangling pointers.** Kyne exposes no pointer or reference type at the source level at all — there is no construct through which a Kyne program could hold, or attempt to dereference, a reference to storage that has already been released.

**No use-after-free.** Because release timing is entirely determined by [Chapter 5](#5-lifetime-model)'s lifetime rules, and because no Kyne construct can observe or hold a reference past a value's computed lifetime, there is no program a developer can write that accesses a value after its underlying storage has been released.

**No double free.** Because deallocation is never developer-invoked (there is no `free`, `delete`, or equivalent), and because the compiler's own release logic is derived deterministically from a single, whole-program lifetime computation, there is no path by which the same storage could be released twice.

**No manual allocation.** There is no Kyne syntax to request a specific allocation — no `new`, no explicit heap/stack directive, nothing resembling [Chapter 6](#6-allocation-model)'s categories exposed as a choice.

**No manual deallocation.** Symmetric to the above — there is no Kyne syntax to request early or explicit release of a value's storage.

**No undefined behavior.** Every memory-relevant operation Kyne's grammar permits has a fully specified outcome, defined either in this document or in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md); there is no memory-relevant construct whose behavior is left to an implementation's discretion in a way that could vary program correctness, as opposed to the explicitly permitted, behavior-preserving discretion of [Chapter 10](#10-memory-optimizations).

## Why These Guarantees Exist

Each of these six guarantees is achieved by the **absence of the primitive that would be required to violate it**, not by a runtime check or a static proof discharged against a more permissive language. A language with pointers needs a borrow checker, or a garbage collector, or manual discipline, to avoid dangling pointers; Kyne avoids the entire category by never introducing pointers into its surface language. This is the concrete realization of [Chapter 1](#1-memory-philosophy)'s Safety principle, and it is also why [Chapter 17](#17-future-memory-evolution) treats any future feature that would introduce one of these primitives — a custom allocator, a shared-memory construct — as requiring the highest level of scrutiny this document's evolution process affords, since introducing the primitive would mean introducing, for the first time, a category of defect Kyne currently cannot express at all.

---

# 15. Compiler Responsibilities

The compiler — specifically, RIR lowering, per [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations) — owns every one of the following determinations, for every value in every Kyne program it compiles:

| Responsibility | Governed by |
|---|---|
| Ownership (borrow, move, clone, reference) | [Chapter 3](#3-value-semantics), [Chapter 4](#4-ownership-model) |
| Lifetimes | [Chapter 5](#5-lifetime-model) |
| Allocation strategy (stack, heap) | [Chapter 6](#6-allocation-model) |
| Temporary value management | [Chapter 7](#7-temporary-objects) |
| Persistent storage layout and serialization | [Chapter 8](#8-storage-mapping) |
| Storage staging and commit sequencing | [Chapter 9](#9-storage-lifecycle) |
| Memory-relevant optimizations | [Chapter 10](#10-memory-optimizations) |
| Copy/move/clone/borrow/reference selection at each use site | [Chapter 11](#11-copy-behavior) |
| Collection realization strategy | [Chapter 12](#12-collections) |
| Execution frame management | [Chapter 13](#13-function-memory) |

Every entry in this table is a decision the compiler MUST make on the developer's behalf, MUST make consistently with the observable-behavior guarantees stated in this document, and MUST make with no corresponding Kyne-level syntax through which a developer could inspect, override, or influence the decision beyond the ordinary structure of their own program (for example, a developer can influence whether a value is used again after an assignment, which in turn influences whether the compiler chooses a move — but this is an effect of ordinary program structure, not a memory-management annotation).

---

# 16. Runtime Guarantees

Every execution of a conforming Kyne program guarantees:

**Deterministic allocation.** For a fixed compiler version and fixed input, the compiler's allocation decisions ([Chapter 6](#6-allocation-model)) are identical every time, per [COMPILER_ARCHITECTURE.md's Principle 5](./COMPILER_ARCHITECTURE.md#principle-5--predictable-output) extended to memory-layout decisions specifically — this is a compiler-engineering-level determinism guarantee, distinct from, but consistent with, [RUNTIME_MODEL.md §2.1](./RUNTIME_MODEL.md#21-every-execution-is-deterministic)'s contract-execution-level determinism guarantee.

**Predictable cleanup.** Every value's release timing is fully determined by [Chapter 5](#5-lifetime-model)'s rules, with no probabilistic or best-effort component — a developer (or, more precisely, a compiler engineer verifying this document's guarantees) can always determine exactly when a given value's storage will be released, purely from the program's structure.

**No memory leaks visible to developers.** Because Kyne exposes no manual allocation, there is no Kyne-level action that could cause a leak a developer would need to guard against. The compiler's own internal allocation is bounded by the deterministic lifetime rules in [Chapter 5](#5-lifetime-model) and is not permitted to accumulate unreleased storage across invocations for any Local value — a defect that caused this would be a compiler bug, not a possible outcome of any Kyne program.

**Safe persistent storage.** Every `state` read and write proceeds through the [Storage Lifecycle](#9-storage-lifecycle) in full, with the serialization round-trip guarantee from [Chapter 8](#8-storage-mapping) and the rollback guarantee from [§9.2](#92-rollback-interaction) both holding unconditionally.

**No invalid references.** As established in [Chapter 14](#14-memory-safety), there is no reference primitive in Kyne source capable of becoming invalid in the first place.

**No undefined runtime behavior.** Every memory-relevant construct's behavior is fully specified by this document and by [RUNTIME_MODEL.md](./RUNTIME_MODEL.md); a compiler MUST reject, at compile time, any program whose memory behavior these documents do not fully determine, per the same discipline [RUNTIME_MODEL.md §14](./RUNTIME_MODEL.md#14-runtime-invariants) already requires for runtime behavior generally.

---

# 17. Future Memory Evolution

The following are intentionally **outside the scope of Kyne v1** and MUST NOT be introduced without a KIP explicitly amending this document:

- **Arena allocation** — a bulk allocation strategy that could improve performance for certain access patterns but would require the compiler to reason about allocation scopes not currently modeled by this document's three memory categories.
- **Zero-copy collections** — collection types designed to avoid duplication across boundaries (for example, a genuinely shared view into a caller's collection), which would require exposing some form of aliasing at the source level, in direct tension with [Chapter 3](#3-value-semantics)'s value-semantics guarantee as currently written.
- **Shared memory** — any construct permitting two concurrently live values to observably alias the same underlying storage, which Kyne currently has no execution model for at all, per [RUNTIME_MODEL.md §1.1](./RUNTIME_MODEL.md#11-deterministic)'s single-threaded, sequential execution model.
- **Custom allocators** — any mechanism allowing a Kyne program to influence [Chapter 6](#6-allocation-model)'s allocation-strategy decisions, which would be a direct, and currently unjustified, reintroduction of memory-management concerns into Kyne source.
- **Advanced optimizations** — memory optimizations beyond those enumerated in [Chapter 10](#10-memory-optimizations), reserved for future proposal once real-world compiled contracts provide evidence of where additional optimization would meaningfully help.
- **Async execution** — already excluded at the language level for v1, per [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)'s enumerated v1 exclusions; this document notes that async execution would additionally require an entirely different lifetime and execution-frame model than [Chapter 5](#5-lifetime-model) and [Chapter 13](#13-function-memory) currently define, making it a memory-model concern as much as a language-syntax one.
- **Generics** — already excluded at the language level for v1 beyond the five closed intrinsic parametric types, per [LANGUAGE_SPEC.md §6.7](./LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature); this document notes that user-facing generics would require this document's collection-memory rules in [Chapter 12](#12-collections) to generalize beyond five hand-specified cases, which is itself a nontrivial extension of this document, not merely of the type checker.

**These are intentionally outside Kyne v1.** Their absence is not an oversight to be silently corrected in a point release — it is the same deliberate, evidence-driven deferral [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) and [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) already apply to their own future-reserved features. Any future addition to this list MUST go through the KIP process defined in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution), and MUST explicitly demonstrate that it can be added without weakening any guarantee in [Chapter 14](#14-memory-safety) or [Chapter 16](#16-runtime-guarantees).

---

# 18. Memory Invariants

The following invariants MUST always hold, for every conforming Kyne implementation, without exception.

**Values behave like values.** No Kyne program can ever observe aliasing between two logically independent bindings, regardless of type, regardless of size, and regardless of which realization strategy the compiler chooses underneath them, per [Chapter 3](#3-value-semantics). A future optimization that makes this observable — even in a narrow, hard-to-trigger case — is not a valid optimization; it is a language-semantics regression.

**Persistent storage is explicit.** Every value that survives beyond its producing invocation does so because, and only because, it was written to a `state` field by name, per [Chapter 2](#2-memory-categories)'s Persistent category and [RUNTIME_MODEL.md §2.2](./RUNTIME_MODEL.md#22-every-state-change-is-explicit). There is no implicit promotion of a Local value into persistence.

**The compiler owns allocation.** Every allocation-relevant decision in this document belongs to the compiler, per [Chapter 15](#15-compiler-responsibilities), with no Kyne-level construct capable of overriding, requesting, or inspecting a specific choice.

**Developers never manage memory.** Restated as an invariant, not merely a philosophy: a conforming Kyne grammar MUST NOT contain any construct through which a developer expresses an allocation, deallocation, ownership, borrowing, or lifetime decision, per the [Fundamental Memory Principle](#the-fundamental-memory-principle).

**Persistent state survives execution.** A `state` field's value, once successfully committed, MUST remain available to every future invocation of the same contract instance until either explicitly reassigned or affected by the archival lifecycle in [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) — it MUST NOT be lost, reset, or altered by anything other than an explicit, successfully committed assignment.

**Temporary values never survive execution.** No Local value — bound or unbound — persists past the end of the invocation that created it unless it was explicitly written into `state` along the way, per [Chapter 2](#2-memory-categories) and [Chapter 13](#13-function-memory).

**Compiler optimizations never change observable behavior.** Restated as a permanent invariant from [Chapter 10](#10-memory-optimizations) and [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization): every optimization in this document's scope is a transformation of *mechanism*, never of *meaning*.

## Why Future Contributors May Never Violate Them

Every invariant above is load-bearing for a promise this document makes on the compiler's behalf to every Kyne developer: that they can reason about their program's correctness by reading its Kyne source alone, with total confidence that no memory-level detail the compiler chooses could ever change what that source means. A future compiler change that violates one of these invariants — even one that is faster, or produces smaller generated Rust, or seems like an obviously safe special case — breaks that promise silently, for every contract compiled with the new compiler version, including contracts whose authors have no way to know the promise has changed. These invariants are therefore not a checklist to satisfy at merge time; they are the fixed boundary within which all future memory-related compiler engineering, without exception, must occur.

---

# 19. Memory Transparency

**Memory Transparency** is the requirement that, although the compiler performs sophisticated memory optimization on a developer's behalf, every optimization MUST remain explainable. A developer — or, in practice, a compiler engineer or an advanced contributor investigating a specific contract's generated Rust — should always be able to determine, for any given value, why it was borrowed, copied, moved, cached, or otherwise realized the way it was, by inspecting a documented rule in this specification and the corresponding RIR representation for that value.

This is the direct extension of [Compiler as a Teacher](./COMPILER_ARCHITECTURE.md#principle-1--the-compiler-is-a-teacher) into the memory domain: a compiler that makes memory decisions no one can account for has stopped teaching and started guessing on the developer's behalf, which is a different, and unacceptable, relationship. **Memory optimization must never become "magic."** Every decision remains, in principle, a specific, nameable application of a specific rule from [Chapter 10](#10-memory-optimizations), never an opaque heuristic whose outcome cannot be traced back to this document.

Concretely, this requirement is satisfied through the same inspectability mechanism [COMPILER_ARCHITECTURE.md's Principle 2](./COMPILER_ARCHITECTURE.md#principle-2--every-transformation-is-inspectable) already establishes for the pipeline generally: because RIR is a real, dumpable, documented data structure (per [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations)), and because RIR is specifically the stage at which every ownership and allocation decision in this document is made, a compiler engineer can always inspect a given value's RIR realization directly rather than needing to reverse-engineer it from generated Rust text. This document does not mandate a specific developer-facing tool (an "explain this value" command, for instance) — that is a future toolchain concern — but it does mandate that the underlying information such a tool would need MUST always exist, in an inspectable form, as a consequence of RIR's own required inspectability.

---

# 20. Memory Commandments

These are Kyne's permanent memory rules. Changing any of them is, by definition, a foundational break subject to the extraordinary process described in [LANGUAGE_PRINCIPLES.md's Ten Commandments](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne), not an ordinary KIP.

1. **The developer never manages memory.** Every allocation, deallocation, ownership, and lifetime decision belongs to the compiler, permanently, per the [Fundamental Memory Principle](#the-fundamental-memory-principle). This is the root commandment every other one in this list serves.

2. **Values behave like values.** Assignment, parameter passing, and return always produce independent, non-aliased results from the developer's perspective, regardless of type or size, per [Chapter 3](#3-value-semantics). This is what makes commandment 1 possible without sacrificing safety: a language where developers reason about values, not references, has no aliasing bugs to reason about in the first place.

3. **Storage is always explicit.** A value survives beyond its producing invocation if, and only if, it was written to a named `state` field, per [Chapter 2](#2-memory-categories). There is no implicit persistence, ever, at any optimization level.

4. **Allocation is deterministic.** For a fixed compiler version and input, memory decisions never vary between runs, per [Chapter 16](#16-runtime-guarantees). Nondeterministic allocation would undermine both the reproducibility [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) requires and the auditability every Kyne contract depends on.

5. **Memory safety is non-negotiable.** No dangling pointers, no use-after-free, no double free, no undefined behavior — ever, under any optimization, in any version, per [Chapter 14](#14-memory-safety). Kyne achieves this by never introducing the primitives that would make a violation possible, and no future feature may reintroduce one of those primitives without triggering the highest level of scrutiny this ecosystem's governance affords.

6. **Compiler optimizations are invisible.** A developer cannot request, inspect through source, or be required to understand a specific memory-realization strategy in order to write correct Kyne code, per [Chapter 4](#4-ownership-model) and [Chapter 15](#15-compiler-responsibilities). Invisibility is what makes commandment 1 sustainable as the compiler improves: an invisible mechanism can keep getting better without ever becoming a developer-facing breaking change.

7. **Generated Rust may optimize memory but must never change observable behavior.** Every optimization is a transformation of mechanism, never of meaning, per [Chapter 10](#10-memory-optimizations) and [Chapter 18](#18-memory-invariants). This is the commandment that makes it safe for Kyne to keep improving its compiler indefinitely: as long as this one holds, no future optimization can ever be a silent correctness regression for an already-deployed, already-audited contract recompiled with a newer toolchain.

## Why These Rules Are Permanent

Each commandment protects a promise a Kyne developer relies on the moment they write their first line of source: that they can trust values, trust that persistence is only ever what they wrote explicitly, and trust that the compiler's invisible work will never surprise them. A language that revisited these rules casually would be asking every developer who ever trusted them to re-verify every contract they had already written and audited. That cost is why this document treats these seven rules not as defaults that happen to be in place today, but as permanent commitments the language makes to everyone who has ever relied on them.

---

# 21. Memory Planning Architecture

Future compiler implementations MAY organize the memory-related decisions in this document into three conceptual planning subsystems. **These are conceptual compiler components, not language features** — they have no Kyne-level manifestation whatsoever, and they exist entirely within the [RIR lowering stage](./COMPILER_ARCHITECTURE.md#12-intermediate-representations) already fixed by [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline). This chapter does not introduce a new pipeline stage, and MUST NOT be read as amending COMPILER_ARCHITECTURE.md's locked pipeline — it describes how the work already assigned to RIR lowering may be conceptually subdivided for the reference implementation's own internal clarity.

**Memory Planner.** Responsible for allocation strategy: deciding, per [Chapter 6](#6-allocation-model), whether a given Local value is realized on the stack or the heap, and applying the allocation-relevant optimizations in [Chapter 10](#10-memory-optimizations) such as allocation reuse and stack promotion.

**Ownership Planner.** Responsible for the borrow, move, clone, and reference decisions described in [Chapter 3](#3-value-semantics) and [Chapter 4](#4-ownership-model) — determining, for each use of each value, which realization strategy satisfies the value-semantics contract at the lowest cost.

**Storage Planner.** Responsible for mapping `state` fields onto Soroban persistent storage per [Chapter 8](#8-storage-mapping) and [Chapter 9](#9-storage-lifecycle), including the single-entry-versus-per-element realization choice for storage collections described in [§12.1](#121-storage-collections).

## Responsibility Boundaries

The Memory Planner and Ownership Planner both operate over Local memory and frequently interact — a value the Ownership Planner decides to borrow rather than clone has a correspondingly different allocation footprint for the Memory Planner to realize — but their concerns remain conceptually distinct: the Ownership Planner answers "does this use require independent storage at all," and the Memory Planner answers "given that it does, where does that storage live." The Storage Planner is more sharply isolated from the other two, since it operates exclusively over the Persistent memory category, which has no allocation-strategy or ownership question in the Local-memory sense at all — every Persistent value is, definitionally, realized in ledger storage, per [Chapter 6](#6-allocation-model)'s category table.

A reference implementation MAY choose to structure its RIR-lowering code around these three conceptual subsystems, MAY choose a different internal structure entirely, or MAY merge them, provided the resulting behavior satisfies every rule in this document — this chapter constrains a *useful way to think about* the RIR-lowering stage's responsibilities, not the stage's required internal code organization, which remains implementation-defined per [COMPILER_ARCHITECTURE.md's Non-Goals](./COMPILER_ARCHITECTURE.md#non-goals).

---

# Non-Goals

This document does **not** define, and explicitly defers to other documents or to implementation discretion:

- **Compiler implementation algorithms** — the specific algorithms used to compute lifetimes, choose between stack and heap allocation, or select a borrow/move/clone strategy are implementation details of the reference compiler, not language-level guarantees; this document fixes observable behavior only.
- **Garbage collection implementation** — Kyne has no garbage collector; its deterministic, lifetime-based release model (Chapters 5, 14, 16) is not a garbage-collection strategy and MUST NOT be implemented as one.
- **Rust ownership rules** — this document does not restate or redefine Rust's own borrow-checker semantics; the generated Rust's ownership structure is a code-generation detail governed by [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations) and [§14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation).
- **Borrow checker implementation** — Kyne exposes no borrow checker to developers, per [Chapter 4](#4-ownership-model); any internal analysis the compiler performs to make ownership decisions is an implementation detail, not a specification.
- **Parser behavior** — defined in [COMPILER_ARCHITECTURE.md §5](./COMPILER_ARCHITECTURE.md#5-parsing).
- **Language syntax** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Standard library** — the exact functions for collection manipulation, storage lifetime extension, and execution-context access belong to a forthcoming `STANDARD_LIBRARY.md`.
- **Package management** — reserved per [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem).

Wherever multiple implementation strategies could satisfy a rule in this document, this document has deliberately described the required *observable behavior* rather than prescribing a specific compiler algorithm — a future reference implementation is free to choose, and improve, its own strategy indefinitely, provided every guarantee here continues to hold.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission and values this memory model exists to serve.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy and KIP governance process this document's own evolution follows.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the value-semantics rule ([§4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)) and the type system ([§6](./LANGUAGE_SPEC.md#6-types)) this document elaborates.
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — the state model ([§6](./RUNTIME_MODEL.md#6-state-model)) and storage/execution pipeline ([§7](./RUNTIME_MODEL.md#7-execution-pipeline), [§10](./RUNTIME_MODEL.md#10-storage-behavior)) this document elaborates at the memory level.
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) — the pipeline stage (RIR lowering) and inspectability guarantee this document's compiler responsibilities depend on.

The following documents are anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `STANDARD_LIBRARY.md`, `TOOLCHAIN.md`, `GOVERNANCE.md`, and `ROADMAP.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is the single authoritative specification describing how memory and storage behave in Kyne, written from the perspective of the language rather than of Rust. A compiler engineer implementing Kyne's memory model should be able to determine, from this document alone, exactly what a conforming implementation must guarantee — and should remain free to choose, and to keep improving, exactly how it guarantees it. Where a future engineer finds a memory behavior this document does not address, that is a gap to be closed by a KIP, not a decision to be made silently in an implementation.
