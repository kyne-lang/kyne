# COMPILER_ARCHITECTURE.md

# The Kyne Compiler Architecture

**Phase:** 0 — Foundations
**Milestone:** 5 — Compiler Architecture
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on the reference compiler implementation and on every future compiler contributor.

This document defines the engineering architecture of the Kyne compiler: how source becomes generated Rust, how the compiler is organized internally, and how every future compiler contributor should extend it. It implements — and must never redefine — [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), and [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), each of which is a locked, constitutional document with respect to this one. Where this document appears to conflict with any of those four, this document is wrong and MUST be corrected by KIP; it has no authority to redefine language syntax or runtime behavior, only to specify how the compiler realizes them.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings. A conforming Kyne compiler implementation MUST NOT deviate from a MUST/MUST NOT rule in this document without that deviation being resolved through a KIP amending this document first.

---

# 1. Compiler Philosophy

The Kyne compiler is not merely a translator from `.kyn` source to Rust. A translator's only obligation is to produce output that means the same thing as its input; the Kyne compiler has four additional obligations that follow directly from [Kyne's three immutable pillars](./LANGUAGE_PRINCIPLES.md#foundation): it is responsible for **correctness** (a program it accepts must behave exactly as [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) and [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) define), for **education** (a program it rejects must leave the developer better equipped to fix it than a bare pass/fail verdict would), for **security** (a program that is syntactically and semantically valid but exhibits a known-dangerous smart-contract pattern must be flagged before it ever reaches a network), for **determinism** (the same input must produce the same output, every time, forever, for a given compiler version), and for **consistency** (every Kyne contract that compiles successfully looks, at the Rust level, like it was produced by the same disciplined process, because it was).

These five obligations are formalized as the compiler's five governing principles below. They apply to every stage described in this document, and a proposed compiler change that satisfies its immediate goal while violating one of these principles is not an acceptable change, regardless of how useful that goal is in isolation.

## Principle 1 — The Compiler Is a Teacher

Every diagnostic the compiler produces MUST be written as though it is teaching the reader something they did not already know, not merely informing them that they made a mistake. This is the compiler-engineering expression of [Compiler as a Teacher](./LANGUAGE_PRINCIPLES.md#compiler-as-a-teacher): a diagnostic that states only "expected `;`" has done the minimum a compiler is required to do; a diagnostic that states where the `;` was expected, why the parser was looking for it there, and shows the corrected line has done what Kyne requires.

This principle exists because Kyne's primary audience is developers who are often new to Rust, to Soroban, and to blockchain development generally, per [FOUNDATION.md's Target Audience](./FOUNDATION.md#target-audience). For this audience, the compiler's diagnostic output is frequently the single most-read piece of Kyne documentation they will encounter in a given week — a compiler that treats diagnostics as an afterthought is, in practice, treating documentation as an afterthought. The [Diagnostics Engine](#15-diagnostics-engine) chapter of this document exists specifically to make this principle enforceable rather than aspirational: every diagnostic has a required shape, and a diagnostic that does not fit that shape is an incomplete implementation, not a stylistic choice.

## Principle 2 — Every Transformation Is Inspectable

Every stage of the [Compiler Pipeline](#3-compiler-pipeline) MUST be independently observable: its input and output MUST be representable in a form a compiler engineer, a tooling author, or a sufficiently motivated contract author can inspect directly, without needing to run the entire pipeline end to end or to instrument the compiler's internals ad hoc. A compiler engineer debugging a Rust-code-generation defect MUST be able to dump the HIR and RIR for the offending contract and read them directly, exactly as they will be consumed by the next stage.

This principle exists for two reasons. First, it is what makes the compiler debuggable at all — a pipeline whose intermediate states are opaque can only be debugged by guessing, which is precisely the kind of unpredictability [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) rejects at the language level and which this document rejects at the compiler-engineering level. Second, it is what makes the [Library-First Compiler Architecture](#22-library-first-compiler-architecture) possible: a stage's output can only be reused by a second tool (a language server, a formatter, a documentation generator) if that output is a real, inspectable data structure with a stable shape, not an implementation detail buried inside a monolithic compile function.

## Principle 3 — Compiler Correctness Before Optimization

Correct generated Rust MUST always be preferred over more aggressively optimized generated Rust. Every optimization pass described in [§13](#13-optimization) is subordinate to this principle without exception: an optimization that produces smaller, faster, or more idiomatic Rust at the cost of any observable behavioral difference — however small, however unlikely to matter in practice — is not a valid optimization for the Kyne compiler to ship.

This ordering exists because a Kyne contract, once deployed, is immutable and frequently holds real economic value, per [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle). A miscompilation caused by an overzealous optimization is not a performance regression a later release can quietly improve upon — it is a security incident in an already-deployed, unpatchable artifact. The Kyne compiler is permitted to be slow to adopt an optimization; it is never permitted to be wrong in service of adopting one sooner.

## Principle 4 — Fast Incremental Compilation

The compiler MUST avoid unnecessary recompilation. A change to one file in a multi-file contract project SHOULD NOT require re-lexing, re-parsing, or re-analyzing files the change could not possibly have affected. [§16](#16-incremental-compilation) and [§17](#17-compiler-cache) define the mechanism by which this is achieved.

This principle exists because compiler speed is a direct input to [Developer Experience](./LANGUAGE_PRINCIPLES.md#design-goals), and because the [Compiler as a Teacher](#principle-1--the-compiler-is-a-teacher) principle is only valuable if diagnostics arrive quickly enough to stay attached to the developer's current train of thought. A compiler that is correct and pedagogically excellent but takes minutes to report a one-line typo has still failed its users; fast, incremental feedback is not a secondary concern layered on top of correctness, it is a requirement the compiler's architecture must be designed around from the outset, not retrofitted later.

## Principle 5 — Predictable Output

The same `.kyn` source, compiled by the same compiler version with the same settings, MUST always produce byte-identical generated Rust. This is a stronger guarantee than the language-level determinism [RUNTIME_MODEL.md §2.1](./RUNTIME_MODEL.md#21-every-execution-is-deterministic) makes about contract *execution* — this principle is about the *compiler's own behavior* as a piece of software, and it MUST hold even for aspects of compilation that have no runtime-observable consequence, such as the exact formatting or internal ordering of generated Rust items.

This principle exists because predictable output is what makes generated Rust reviewable in the first place: an auditor comparing two versions of a contract's generated Rust needs to trust that any difference they see reflects an actual change in the `.kyn` source or the compiler version, not incidental variation from a non-deterministic internal ordering (for example, iterating a hash map without a defined order when emitting struct fields). [§14.4](#144-deterministic-generation) specifies the concrete engineering discipline — canonical ordering, no reliance on non-deterministic collection iteration — required to uphold this principle.

---

# 2. Compiler Overview

The Kyne compiler is architected as a pipeline of independent stages, each satisfying three properties: **one responsibility**, **one input type**, and **one output type**. A stage MUST NOT reach backward into an earlier stage's internal state, and MUST NOT anticipate or special-case behavior belonging to a later stage. Each stage receives exactly the data structure the previous stage produces, and produces exactly the data structure the next stage expects — nothing more, nothing implicit.

This discipline is chosen deliberately over a more traditional, tightly coupled compiler design (where, for example, type checking and code generation might be interleaved for implementation convenience) for four concrete engineering reasons:

**Maintainability.** A contributor fixing a defect in name resolution needs to understand only the [Name Resolution](#8-name-resolution) stage's input (an AST) and output (a resolved symbol table plus an annotated AST) — not the type checker's internals, not the code generator's internals, and not the lexer's internals. Small, sharply bounded stages keep the amount of context required to make a correct change small and stable as the compiler grows.

**Testability.** Each stage can be tested in complete isolation by constructing its input directly (rather than deriving it from real source through every preceding stage) and asserting on its output — the methodology formalized in [§19](#19-testing-strategy). A pipeline of independent stages is a pipeline of independently verifiable units; a monolithic compile function is not.

**Reusability.** Because each stage's output is a real, stable, documented data structure (per [Principle 2](#principle-2--every-transformation-is-inspectable)), other tools in the Kyne ecosystem — a language server, a formatter, a documentation generator — can consume the output of exactly the stages they need without running the whole pipeline, per [§22](#22-library-first-compiler-architecture).

**Correctness under change.** A pipeline with sharply bounded stage responsibilities makes it structurally difficult for a change to one concern (say, an optimization) to accidentally affect an unrelated concern (say, diagnostic wording), because the two live in different stages with different input/output contracts. This is an architectural enforcement of [Principle 3](#principle-3--compiler-correctness-before-optimization): it is harder to accidentally break correctness when correctness-relevant stages (parsing, type checking, semantic analysis) are structurally isolated from stages whose entire purpose is to *not* change meaning (optimization, code generation).

---

# 3. Compiler Pipeline

The complete Kyne compiler pipeline, in order, is:

```
.kyn Source
    ↓
Lexer
    ↓
Parser
    ↓
Concrete Syntax Tree (CST)
    ↓
Abstract Syntax Tree (AST)
    ↓
Name Resolution
    ↓
Type Checker
    ↓
Semantic Analyzer
    ↓
Security Analyzer
    ↓
High-Level Intermediate Representation (HIR)
    ↓
Optimization Passes
    ↓
Rust Intermediate Representation (RIR)
    ↓
Rust Code Generator
    ↓
Cargo Build
    ↓
Soroban Build
    ↓
WASM
```

| Stage | Responsibility | Input | Output |
|---|---|---|---|
| Lexer | Convert raw UTF-8 text into a stream of tokens. | `.kyn` source text | Token stream |
| Parser | Enforce the grammar and build a lossless tree of the source. | Token stream | CST |
| CST → AST lowering | Discard trivia and syntax sugar not needed for semantic analysis. | CST | AST |
| Name Resolution | Bind every identifier reference to the declaration it refers to. | AST | AST annotated with resolved bindings; symbol table |
| Type Checker | Assign and validate a type for every typed construct. | Resolved AST | AST annotated with types |
| Semantic Analyzer | Enforce every MUST-level static rule in LANGUAGE_SPEC.md not already covered by parsing, resolution, or typing. | Typed AST | Typed AST (unchanged) + diagnostics |
| Security Analyzer | Detect known-dangerous smart-contract patterns per [§11](#11-security-analyzer). | Typed AST | Typed AST (unchanged) + diagnostics |
| HIR lowering | Desugar all syntax sugar into a small, canonical, fully-typed core representation. | Typed, analyzed AST | HIR |
| Optimization Passes | Apply behavior-preserving transformations. | HIR | Optimized HIR (same shape, smaller/simpler) |
| RIR lowering | Make every Rust-specific decision (ownership, SDK type mapping, storage calls). | Optimized HIR | RIR |
| Rust Code Generator | Pretty-print RIR as formatted, idiomatic Rust source text. | RIR | Rust source files (a Cargo crate) |
| Cargo Build | Compile the generated crate as ordinary Rust. | Generated Cargo crate | Native/WASM-target Rust build artifacts |
| Soroban Build | Apply Soroban-specific build steps (WASM optimization, contract metadata embedding) on top of the Cargo build. | Rust build artifacts | Soroban-ready WASM |
| WASM | The final deployable artifact. | — | Deployable contract binary |

Each stage is described in full in the sections that follow. The **Cargo Build**, **Soroban Build**, and **WASM** stages are, by design, ordinary invocations of the unmodified standard Rust and Soroban toolchains — per [RUNTIME_MODEL.md §3](./RUNTIME_MODEL.md#3-project-lifecycle), Kyne introduces no fork of, or special case in, this part of the pipeline, and this document does not further specify their internals, since they are external tools Kyne depends on rather than components Kyne implements.

---

# 4. Lexical Analysis

The lexer's sole responsibility is **tokenization**: converting a `.kyn` file's raw UTF-8 text into a flat, ordered stream of tokens, each carrying its kind, its literal text, and its exact source span (byte offset, line, and column) for use by every later diagnostic-producing stage.

**Unicode.** Per [LANGUAGE_SPEC.md §1.1](./LANGUAGE_SPEC.md#11-source-encoding-and-unicode), source files are UTF-8, and the lexer MUST accept arbitrary Unicode within comments and string literals while rejecting any non-ASCII byte sequence that would otherwise form part of an identifier. The lexer, not a later stage, is the enforcement point for this rule, because identifier validity is a lexical-category concern (what characters are permitted to compose a token of kind `identifier`) rather than a semantic one.

**Comments.** The lexer recognizes `//` line comments, `/* */` block comments (nestable, per [LANGUAGE_SPEC.md §1.4](./LANGUAGE_SPEC.md#14-comments)), and `///` doc comments as distinct token kinds. Regular and block comments are retained in the token stream (rather than being discarded during lexing) specifically so the CST, built directly from this stream, can preserve them — see [§6](#6-concrete-syntax-tree). Doc comments are additionally tagged with the declaration they precede once the CST is built, for consumption by a future documentation generator.

**Whitespace.** Whitespace is tokenized (not silently skipped) for the same reason as comments: the CST needs it to support exact-fidelity source reconstruction. The lexer MUST NOT assign any semantic significance to whitespace beyond its role as a token separator, per [LANGUAGE_SPEC.md §1.5](./LANGUAGE_SPEC.md#15-whitespace).

**Error recovery.** On encountering a byte sequence that cannot begin any valid token (for example, a non-ASCII character outside a string literal or comment, or an unterminated string literal), the lexer MUST NOT abort the entire lexing pass. It MUST emit an `Error` token spanning the offending bytes, record a diagnostic, and resume lexing immediately afterward. This is a deliberate application of [Principle 1](#principle-1--the-compiler-is-a-teacher): a developer who made two unrelated typos in one file should see two diagnostics from a single `kyne build` invocation, not one diagnostic followed by silence, forcing them to fix issues one at a time across repeated invocations.

**Token stream.** The lexer's output is a flat sequence of tokens with no tree structure at all — grouping and nesting are entirely the parser's responsibility. This strict separation is why lexical analysis is isolated as its own stage rather than folded into parsing: tokenization is a small, self-contained, easily-tested problem (recognizing keywords, literals, punctuation, and identifiers character by character) with no knowledge of grammar, and keeping it isolated means the parser's grammar-level logic never has to reason about character-level concerns like escape-sequence decoding or comment nesting depth.

---

# 5. Parsing

The parser's responsibility is to enforce the grammar defined in [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar) against the token stream produced by the lexer, and to produce a **Concrete Syntax Tree** as its output.

**Grammar enforcement.** The parser MUST accept exactly the set of token sequences the EBNF grammar in LANGUAGE_SPEC.md §2 defines, and MUST reject every other sequence. The parser is the sole authority for whether a program is syntactically valid; no later stage is permitted to reject a program on syntactic grounds, and no later stage is permitted to accept a tree shape the parser's grammar could not have produced.

**Syntax validation.** Where the grammar is ambiguous without additional context the parser can resolve locally (for example, disambiguating an empty map literal `{}` from a block, per [LANGUAGE_SPEC.md §7.5](./LANGUAGE_SPEC.md#75-construction)), the parser MUST apply exactly the disambiguation rule LANGUAGE_SPEC.md specifies, with no implementation-specific fallback.

**Recovery strategy.** On encountering a token sequence that does not match any valid production at the current parse position, the parser MUST NOT abort parsing the entire file. It MUST record a diagnostic, then synchronize by discarding tokens up to the next token that can safely resume parsing at the same nesting depth — typically the next `;` or the next `}` that closes the current block. This bounded, block-scoped recovery strategy is chosen specifically because Kyne's grammar mandates braces around every block (per [LANGUAGE_SPEC.md §8.1](./LANGUAGE_SPEC.md#81-if)), which gives the parser a reliable, unambiguous synchronization point that a brace-optional grammar would not provide. As with lexer recovery, this exists to maximize the number of real diagnostics a single `kyne build` invocation can report, per [Principle 1](#principle-1--the-compiler-is-a-teacher).

**Creation of the CST.** The parser's direct output is a CST: a tree that records every token from the input stream — including whitespace and comment tokens — attached to the grammar node it syntactically belongs to. The CST is, by construction, a lossless representation: reprinting a CST's tokens in order reproduces the original source text exactly, byte for byte.

**Transition into the AST.** The parser itself does not produce an AST. A separate, subsequent lowering step (described in [§7](#7-abstract-syntax-tree)) consumes the CST and produces the AST by discarding trivia and normalizing syntax sugar. Keeping this lowering step distinct from parsing itself — rather than having the parser produce an AST directly — is what makes the CST available as a first-class, independently useful artifact for the tooling use cases in [§6](#6-concrete-syntax-tree), rather than a discarded intermediate the parser happens to pass through internally.

---

# 6. Concrete Syntax Tree

The CST exists because the AST, by design, is not sufficient for every consumer of parsed Kyne source. The AST (§7) exists to serve semantic analysis and code generation, both of which have no use for whitespace, comments, or redundant parenthesization — but several essential tools in the Kyne ecosystem need exactly that information, and would be unable to function correctly without it:

**Preserving original source layout.** The formatter ([LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules)) needs to know precisely how the original source was written — including its comments and their exact attachment points — in order to reformat it correctly. A formatter built on the AST alone would be unable to preserve comments at all, since the AST discards them.

**Supporting IDE tooling.** A language server needs to answer questions like "what token is under the cursor" and "what is the exact span of this expression, including its enclosing whitespace" — questions that require a lossless tree, not a semantically normalized one.

**Supporting diagnostics.** Precise diagnostic spans (used throughout [§15](#15-diagnostics-engine)) are computed against the CST's token positions, since the CST is the tree that retains the exact source-text correspondence the parser observed, including for constructs (like redundant parentheses) the AST will have already discarded.

**Supporting code navigation.** "Go to definition" and "find all references" — both language-server features — need to map a source position back to a specific token in a specific file, which is a CST-level operation; the AST, having discarded exact positional and trivia information for some constructs during lowering, is not sufficient on its own.

**Why the CST is separate from the AST.** Folding both roles into a single tree would force a choice: either the "semantic" tree carries comments, whitespace, and redundant syntax it has no use for, complicating every semantic pass with irrelevant trivia-handling logic, or the "lossless" tree is discarded after parsing, making formatting, exact-fidelity tooling, and precise diagnostics impossible to implement correctly. Keeping the two trees separate, with an explicit, one-directional lowering step between them (CST → AST, never the reverse), lets each tree be exactly as simple as its consumers need and no simpler — a direct application of [§2](#2-compiler-overview)'s "one responsibility" discipline applied to data structures rather than only to pipeline stages.

---

# 7. Abstract Syntax Tree

The AST is the compiler's canonical representation of a Kyne program's **language semantics** — what the program means, independent of exactly how it was formatted or commented.

**Removal of syntax noise.** The CST→AST lowering step discards whitespace and comment tokens entirely, collapses redundant parenthesization (since the AST's tree structure already encodes operator precedence unambiguously, per [LANGUAGE_SPEC.md §7.1](./LANGUAGE_SPEC.md#71-precedence)), and normalizes trailing-comma variation (a struct literal written with or without a trailing comma produces an identical AST node). None of this information is needed by any stage from Name Resolution onward, and retaining it would only add irrelevant complexity to every one of those stages.

**Tree structure.** The AST's node types mirror the grammar's productions in [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar) directly and by design: a `ContractDecl` grammar production corresponds to a `ContractDeclNode` AST type, a `MatchArm` production corresponds to a `MatchArmNode` type, and so on. This one-to-one correspondence is a deliberate simplicity choice — a compiler engineer who knows the grammar already knows the AST's shape, with no separate mental model to learn.

**Transformation rules.** The CST→AST lowering step applies a small, fixed set of normalization rules (trivia removal, redundant-parenthesization collapse, trailing-comma normalization, struct-literal field-init shorthand expansion into explicit `field: field` form). These rules MUST be total and MUST NOT be able to fail on any CST the parser produced successfully — if a CST parsed without error, its corresponding AST MUST always be constructible. Any failure during lowering indicates a parser defect (having accepted something it should not have), not a legitimate AST-construction error, and MUST be treated as an internal compiler error rather than a user-facing diagnostic.

**Ownership.** AST nodes for a given compilation are allocated in a single arena owned by that compilation's pipeline invocation and are immutable once constructed. No later stage mutates the AST in place; stages that need to record additional information about AST nodes (resolved bindings, inferred types) do so in **side tables** keyed by node identity, keeping the AST itself a stable, append-only-annotated structure throughout the remainder of the pipeline. This immutability is a direct application of [Predictability Over Cleverness](./RUNTIME_MODEL.md#15-runtime-design-principles): a tree that never changes shape after construction is trivially safe to share across concurrent analyses (for example, running the Type Checker and an independent lint pass over the same tree without synchronization) and trivially safe to cache, per [§17](#17-compiler-cache).

**Traversal.** Every pass that operates on the AST (Name Resolution, the Type Checker, the Semantic Analyzer, the Security Analyzer) does so through a shared visitor abstraction rather than each pass implementing its own tree-walking logic. This keeps each pass's code focused on what it uniquely contributes and ensures that a new AST node type added in a future language version needs its traversal behavior defined in exactly one place, not once per pass.

---

# 8. Name Resolution

Name Resolution's responsibility is to bind every identifier reference in the AST to the specific declaration it refers to, producing a symbol table and an AST annotated with those bindings.

**Scope resolution and order-independence.** Per [LANGUAGE_SPEC.md §2.1](./LANGUAGE_SPEC.md#21-name-resolution-is-order-independent), declaration order MUST NOT affect whether a program resolves successfully. Name Resolution therefore operates in two passes over each module: a **collection pass** that registers every top-level and contract-member declaration into the symbol table regardless of the order in which they appear in source, followed by a **binding pass** that walks every identifier reference in the AST and resolves it against the now-complete symbol table. A single-pass, order-dependent design would violate LANGUAGE_SPEC.md's explicit guarantee and is therefore not a conforming implementation strategy.

**Identifier lookup.** For a given identifier reference, resolution proceeds from the innermost enclosing lexical scope outward: local `let` bindings and parameters in the current and enclosing blocks, then contract members (`state`, `const`, `error`, `event`, local `struct`/`enum`, `fn`) of the enclosing contract, then module-level declarations of the current file, then declarations brought into scope by `use` imports.

**Imports.** A `use` statement (per [LANGUAGE_SPEC.md §12.2](./LANGUAGE_SPEC.md#122-imports)) is resolved by mapping its dot-separated `ImportPath` onto the project's directory structure ([LANGUAGE_SPEC.md §12.4](./LANGUAGE_SPEC.md#124-namespaces)) to locate the target module, then registering the imported names into the importing file's scope. An import that names a module or identifier that cannot be located is a Name Resolution error.

**Visibility.** Name Resolution determines what is *reachable* through an import (a type declaration is reachable project-wide once its file is imported, per [LANGUAGE_SPEC.md §12.3](./LANGUAGE_SPEC.md#123-visibility-across-modules)); it does **not** determine what is *legal* to reference given a function's `public`/`internal`/file-private modifier — that legality check belongs to the [Semantic Analyzer](#10-semantic-analysis), which runs after types are known and can therefore give a more precise diagnostic (for example, distinguishing "not found" from "found, but not visible from here"). This split exists so that Name Resolution's job stays narrowly about *which declaration does this name refer to*, never about *is this reference allowed*.

**Namespace resolution.** A module's namespace corresponds exactly to its file's position in the project's directory structure, per [LANGUAGE_SPEC.md §12.4](./LANGUAGE_SPEC.md#124-namespaces) — Name Resolution performs no additional namespace mapping beyond mirroring the filesystem layout the compiler was invoked against.

**Shadowing rules.** A nested `let` MAY shadow an outer `let` or a parameter, per [LANGUAGE_SPEC.md §4.5](./LANGUAGE_SPEC.md#45-scope). A `let` MUST NOT shadow a `state` field or a `const` of the same contract, per [LANGUAGE_SPEC.md §4.3](./LANGUAGE_SPEC.md#43-state-persistent-contract-storage) — Name Resolution enforces this by rejecting a `let` declaration whose name collides with an already-registered `state` or `const` name in the enclosing contract's collection-pass results, producing a diagnostic at the point of the colliding `let`, not at the point of the `state`/`const` declaration it collides with, since the later declaration is the one a reader is most likely to be actively editing.

---

# 9. Type Checker

The Type Checker assigns and validates a type for every typed construct in the resolved AST, producing a fully typed AST as its output.

**Primitive types.** Every occurrence of `bool`, `i32`/`i64`/`i128`, `u32`/`u64`/`u128`, `address`, `symbol`, `string`, and `bytes` (per [LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types)) is checked against this fixed, closed set; there is no mechanism for a Kyne program to introduce a new primitive type.

**Contract types.** A `contract` declaration does not introduce a first-class value type that can be held in a variable, passed as a parameter, or stored in `state` — per [Contract-Oriented Design](./LANGUAGE_PRINCIPLES.md#why-contracts-are-the-unit-of-everything), a contract is a deployment unit, not a data type. The Type Checker therefore never needs to reason about "values of contract type"; a reference to another deployed contract instance is represented, at the type level, as an ordinary `address`, consistent with [RUNTIME_MODEL.md §8.3](./RUNTIME_MODEL.md#83-cross-contract-calls). The precise standard-library mechanism for invoking a function on that address is a Standard Library Specification concern (see [Cross References](#cross-references)) and is not further specified here.

**`Option` and `Result`.** These two intrinsic parametric types (per [LANGUAGE_SPEC.md §6.6](./LANGUAGE_SPEC.md#66-option-and-result)) are handled by dedicated, hand-written type-checking rules rather than by a general generic-instantiation mechanism, since no such general mechanism exists in Kyne v1 (per [§6.7 below](#67-generics-a-closed-not-an-open-feature)). The Type Checker MUST recognize `Some`/`None`/`Ok`/`Err` as the fixed constructors of these two types and MUST reject any attempt to redeclare them, per [LANGUAGE_SPEC.md §1.3.2](./LANGUAGE_SPEC.md#132-prelude-bindings).

**Collections.** `list<T>`, `map<K, V>`, and `bytes<N>` are likewise checked as a fixed set of intrinsic parametric types (per [LANGUAGE_SPEC.md §6.2](./LANGUAGE_SPEC.md#62-collections)), each with its own hand-written method-resolution table (`get`, `set`, `has`, `push`, `remove`, `len`, as applicable) rather than a general trait- or interface-based method-resolution mechanism, since Kyne v1 has neither traits nor methods on user-defined types (per [LANGUAGE_SPEC.md §6.3](./LANGUAGE_SPEC.md#63-structs)).

**Type inference policy.** A `let` binding's type MAY be inferred from its initializer expression when no explicit annotation is given, per [LANGUAGE_SPEC.md §4.1](./LANGUAGE_SPEC.md#41-let-local-variables). No other construct is ever inferred: `state` fields, `const` declarations, function parameters, and function return types MUST always carry an explicit type annotation in source, and the Type Checker MUST reject a program relying on inference in any of those positions — there is no fallback inference attempt for them at all, since accepting one would silently create an inference feature LANGUAGE_SPEC.md never defined.

**Why Kyne intentionally limits inference.** A `let` binding's type is almost always obvious from its initializer and is pure local implementation detail — requiring an annotation there would be verbosity with no comprehension benefit, which [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first) explicitly rejects. A `state` field's type, by contrast, is part of a contract's audited storage surface; a function's parameter and return types are part of its audited public contract. Both belong to what a reader — especially an auditor who is not running the compiler's inference engine while reviewing — needs to see explicitly and immediately, per [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit). The Type Checker's inference boundary is therefore drawn exactly at the boundary between "private implementation detail" and "audited surface," not at an arbitrary syntactic convenience line.

**Type compatibility.** Two values are type-compatible if they share the identical declared type; Kyne performs no implicit widening or narrowing between numeric types (per [LANGUAGE_SPEC.md §6.1](./LANGUAGE_SPEC.md#61-primitive-types)) and no structural subtyping between distinct `struct` or `enum` declarations, even where their fields happen to coincide. `==`/`!=` structural equality (per [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)) is automatically defined for every `struct` and `enum` and is checked by the Type Checker without requiring any user-authored equality implementation, since Kyne has no mechanism for one.

**Assignment validation.** An `AssignStmt`'s right-hand side type MUST exactly match its left-hand side's declared type; a compound assignment operator (`+=`, `-=`, etc.) MUST only be used where the corresponding binary operator is itself defined for the target's type, per [LANGUAGE_SPEC.md §7.1](./LANGUAGE_SPEC.md#71-precedence).

**Return validation.** A `return`'s expression type MUST exactly match the enclosing function's declared return type; a function declared with no `-> Type` returns unit, and a bare `return;` (or falling off the end of the function body) is the only legal way to exit it.

## 9.1 Generics: A Closed, Not an Open, Feature

Because Kyne v1 has no user-facing generics mechanism (per [LANGUAGE_SPEC.md §6.7](./LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature)), the Type Checker's handling of `Option<T>`, `Result<T, E>`, `list<T>`, `map<K, V>`, and `bytes<N>` is implemented as five separate, hand-written special cases rather than as five instantiations of one general parametric-type-checking algorithm. This is a deliberate engineering simplification, not an oversight: building a general instantiation mechanism to serve exactly five fixed, closed types would be strictly more implementation complexity for zero additional expressive power, since no sixth parametric type can be introduced by any Kyne program in v1. Should a future KIP introduce user-facing generics, that KIP's implementation MAY choose to generalize these five special cases into instances of the new mechanism; until then, the special-cased implementation is the correct and complete one.

---

# 10. Semantic Analysis

The Semantic Analyzer enforces every MUST-level static rule in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) that is not already enforced by parsing, Name Resolution, or the Type Checker. It operates on the fully typed AST and MUST run to completion (collecting every violation it finds) rather than stopping at the first violation, consistent with [Principle 1](#principle-1--the-compiler-is-a-teacher).

**Control-flow validation.** `break` and `continue` MUST occur only within an enclosing `for` or `while` loop, per [LANGUAGE_SPEC.md §8.6](./LANGUAGE_SPEC.md#86-return-break-continue). A function with a non-unit return type MUST have a `return` (or `throw`, for `Result`-returning functions) on every control-flow path — the Semantic Analyzer performs this definite-return analysis by walking the typed AST's control-flow structure and verifying that every terminal position of every branch (`if`/`else`, every `match` arm) ends in a diverging statement or an explicit `return`/`throw`.

**Contract validation.** The Semantic Analyzer enforces that a project contains at most one `ContractDecl` across all of its files, per [LANGUAGE_SPEC.md §3.1](./LANGUAGE_SPEC.md#31-declaration-and-members); that a contract's members appear in the [canonical order](./LANGUAGE_SPEC.md#32-canonical-member-order) — a hard compiler error, exactly as LANGUAGE_SPEC.md requires, not a lint; and that at most one `fn` named `init` exists per contract, that it is `public`, and that it is never called from anywhere other than the deployment mechanism, per [LANGUAGE_SPEC.md §3.5](./LANGUAGE_SPEC.md#35-the-constructor).

**Visibility validation.** `public` MUST only appear on a `ContractMember`, per [LANGUAGE_SPEC.md §3.3](./LANGUAGE_SPEC.md#33-the-public-api) — a `public fn` on a library-level declaration is rejected here, at the semantic level, since it requires knowing whether the enclosing file belongs to a contract project or a library project, information Name Resolution's scoping pass does not itself need to track.

**State rules.** The Semantic Analyzer performs the [definite-assignment analysis](./LANGUAGE_SPEC.md#44-definite-assignment-of-state) required of every `state` field before any non-`init` entry point can be reached, and rejects any attempt by library code to assign to a `state` field directly, per [LANGUAGE_SPEC.md §9.4](./LANGUAGE_SPEC.md#94-compiler-rules).

**Event rules.** Every `emit` statement's argument list MUST match its target `event`'s declared parameter list exactly in count, order, and type, per [LANGUAGE_SPEC.md §11.2](./LANGUAGE_SPEC.md#112-emission); `emit` MUST only appear within a contract member function.

**Authentication rules.** `auth(...)` MUST only appear within a contract member function, per [LANGUAGE_SPEC.md §10.1](./LANGUAGE_SPEC.md#101-the-auth-statement); the Semantic Analyzer additionally validates that the `auth(...)` argument expression is of type `address`.

**Runtime rule validation.** The Semantic Analyzer is the compiler's checkpoint for [RUNTIME_MODEL.md's invariant](./RUNTIME_MODEL.md#the-compiler-never-generates-undefined-runtime-behavior) that the compiler never accepts a construct whose runtime behavior is undefined: any AST shape the Semantic Analyzer cannot map to a rule explicitly stated in RUNTIME_MODEL.md MUST be rejected here rather than allowed to reach HIR lowering and code generation with an implementation-chosen, undocumented behavior.

**Syntax errors versus semantic errors.** A syntax error means the token sequence could not be assembled into any valid CST at all — the Parser's grammar, defined purely in terms of token shapes, was violated, independent of what any identifier resolves to or what type anything has. A semantic error means a well-formed AST — syntactically valid, every identifier resolved, every type checked — nonetheless violates a rule about what the program is allowed to *mean*: a canonical-order violation, a definite-assignment violation, an illegal `emit`. This distinction is why the pipeline separates Parsing, the Type Checker, and the Semantic Analyzer into three stages rather than one: a syntax error can be detected with no knowledge of names or types at all, a type error requires a fully resolved AST but is itself a large, self-contained, well-understood algorithmic problem (type inference and unification) best isolated in its own stage, and the long tail of remaining MUST-level rules in LANGUAGE_SPEC.md — mostly independent, mostly local, rule-based checks that do not require the type system's machinery — are better served by a dedicated stage than by being scattered across the Type Checker's unification logic.

---

# 11. Security Analyzer

Security analysis is a **compiler stage**, not an external linter a developer must remember to run separately. It executes as part of every standard `kyne build` and `kyne check` invocation, immediately after the Semantic Analyzer, on the same fully typed, semantically valid AST.

This is a deliberate architectural choice, and it refines — without contradicting — [LANGUAGE_SPEC.md §10.4](./LANGUAGE_SPEC.md#104-compiler-expectations), which states that the core compiler cannot, in general, determine which functions *ought* to require authorization, and that this determination belongs to "the Security Analyzer tool... which operates as a lint layer on top of, not inside, the core compiler." That statement remains true at the architectural boundary that matters: the Security Analyzer is implemented as its own library crate (`kyne_security`, per [§22](#22-library-first-compiler-architecture)) with its own well-defined input and output, entirely separate from the crates responsible for deciding whether a program is *well-formed* (the Parser, Name Resolution, Type Checker, and Semantic Analyzer). What this document specifies is that the compiler **driver** composes `kyne_security` into the default pipeline invoked by `kyne build`, so that these checks run automatically on every build rather than requiring a separately remembered command — the architectural separation LANGUAGE_SPEC.md describes is preserved at the library-boundary level; only the driver-level default invocation changes.

## 11.1 Analyses

The Security Analyzer performs the following analyses over the typed AST. Each is heuristic by nature — capable of both false positives and false negatives — except where noted; none of them determines whether a program is well-formed, only whether it exhibits a pattern worth a developer's attention.

**Missing `auth()`.** A `public fn` that assigns to a `state` field but contains no `auth(...)` call anywhere in its own body is flagged. This is intentionally intra-procedural and conservative — it does not attempt interprocedural analysis through internal helper calls — and it is expected to produce false positives on legitimately unauthenticated mutating functions (a public counter anyone may increment, for example). Severity: **Warning**.

**Unused authorization.** An `auth(addr)` call whose `addr` argument does not otherwise influence any subsequent `state` mutation or return value in the same function is flagged as a likely-wasted resource cost (every `auth` check consumes metered execution resources, per [RUNTIME_MODEL.md §12](./RUNTIME_MODEL.md#12-authorization-model)) or a sign that an intended access-control check was left incomplete. Severity: **Hint**.

**Unauthorized state mutation.** A `state`-mutating statement reachable, via the project's call graph, from more than one `public fn` entry point, where at least one reaching call path contains no `auth(...)` call anywhere along it, is flagged — this is a broader, interprocedural refinement of the "Missing `auth()`" check, aimed at catching the specific case where one entry point to shared mutation logic was correctly gated and a sibling entry point was not. Severity: **Warning**.

**Arithmetic safety.** Beyond the language's default checked-arithmetic guarantee ([LANGUAGE_SPEC.md §7.2](./LANGUAGE_SPEC.md#72-arithmetic)), the analyzer flags a subtraction between two values, at least one of which is state-derived, where no prior conditional in the same function establishes that the minuend is not smaller than the subtrahend — a common precursor to an avoidable runtime panic. It also flags every call to an explicit wrapping or saturating standard-library arithmetic function as worth double-checking, since such a call is, by construction, an opt-out of Kyne's default overflow protection. Severity: **Warning** for the unguarded-subtraction case; **Hint** for the explicit wrapping/saturating call.

**Unreachable code.** Code following an unconditional `return`, `throw`, `break`, or `continue` within the same block, and a `match` arm whose pattern is fully shadowed by an earlier arm, are both flagged as unreachable. (`match` non-exhaustiveness itself remains a Type Checker hard error, per [LANGUAGE_SPEC.md §7.7](./LANGUAGE_SPEC.md#77-pattern-matching) — this analysis instead targets an arm the Type Checker's exhaustiveness proof allows but that can, in fact, never be selected.) Severity: **Warning**.

**Dead code.** A file-private or `internal` declaration (function, `struct`, `enum`, `const`) that is never referenced anywhere in the project is flagged, since unused code widens a contract's audit surface with no corresponding functionality, contrary to [Auditability](./LANGUAGE_PRINCIPLES.md#readability-first). Severity: **Hint**.

**Infinite recursion.** A function that calls itself directly and unconditionally, with no intervening `if`, `match`, or early `return`/`throw` on any path to the recursive call, is flagged. This check is deliberately conservative — it catches only the statically obvious case, since termination in general is undecidable — and MUST NOT be read as a claim that a function passing this check is guaranteed to terminate. Severity: **Warning**.

**Dangerous storage growth.** A `state` field of type `list<T>` or `map<K, V>` that is written to only via `.push`/`.set` across the whole contract, with no reachable `.remove` call for that field anywhere, is flagged as a candidate for unbounded growth, which carries long-term resource and archival cost per [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle). Severity: **Hint**.

**Invalid state transitions.** An `enum`-typed `state` field is flagged when it is reassigned after having been set to a variant whose name matches a common terminal-state naming convention (for example, `Finalized`, `Closed`, `Executed`, `Sold`). This is a low-confidence, naming-heuristic check, explicitly offered as a prompt to double-check intent rather than as a claim of a real defect. Severity: **Hint**.

**Unsafe runtime assumptions.** The analyzer flags `for` loops iterating a `list<T>` whose length is influenced by caller-supplied input with no preceding bound check, since an unbounded iteration risks exhausting the Soroban host's metered resources and becoming a Runtime Failure, per [RUNTIME_MODEL.md §9.4](./RUNTIME_MODEL.md#94-runtime-failure). Severity: **Warning**.

## 11.2 Severity Levels

| Severity | Blocks compilation | Meaning |
|---|---|---|
| **Compiler Error** | Yes | A MUST-level rule from LANGUAGE_SPEC.md or RUNTIME_MODEL.md is violated. Never produced by the Security Analyzer in v1 — reserved for the fully decidable checks in [§9](#9-type-checker) and [§10](#10-semantic-analysis). |
| **Warning** | No | A pattern statistically associated with real smart-contract defects was found. SHOULD be surfaced prominently in every build; a project's CI pipeline MAY be configured to treat Warnings as blocking, but the compiler itself MUST NOT block on them by default. |
| **Hint** | No | A lower-confidence or purely stylistic/efficiency observation, best surfaced in an IDE rather than a terminal build log. |

The Security Analyzer MUST NOT emit a Compiler Error in Kyne v1: every check in [§11.1](#111-analyses) is heuristic by design, and promoting a heuristic finding to a build-blocking error would mean a false positive — which every heuristic check here can produce — halts a legitimate, correct contract from compiling at all. This is why these checks are architecturally distinct from the Semantic Analyzer's rules despite running immediately after them in the pipeline: Semantic Analyzer violations are always real; Security Analyzer findings are informed suspicions.

**Future extensibility.** A future KIP MAY define a project-level configuration mechanism allowing specific Warning-class checks to be escalated to build-blocking status for a given project (for instance, a project handling unusually high-value transfers might reasonably want "missing `auth()`" to block its own CI). Designing that configuration surface is a toolchain concern deferred to a future specification (see [Non-Goals](#non-goals)); this document only reserves the possibility.

## 11.3 Why Security by Construction Requires Compiler Integration

If security analysis were only available as a separate, optional tool, a developer could simply never invoke it, and a CI pipeline could be misconfigured to omit it — reducing "security by construction" to "security by remembering to run an extra command," which is exactly the discipline-dependent failure mode [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience) exists to eliminate: "telling developers to be careful does not work at scale." Making `kyne_security` a default stage of `kyne build` means every build a developer runs — including the very first one, run before they have configured anything — surfaces these findings, with no separate step to forget.

---

# 12. Intermediate Representations

Kyne's compiler pipeline defines three distinct tree representations between source text and generated Rust: the **AST**, the **HIR**, and the **RIR**. Each exists to answer a different question, and each is a real, documented, inspectable data structure per [Principle 2](#principle-2--every-transformation-is-inspectable).

**AST — "What did the developer write?"** The AST (§7) is syntax-shaped: its structure mirrors the grammar directly, it still contains Kyne-level sugar (the `?` operator, field-init shorthand already expanded during CST→AST lowering, but constructs like `if`-as-expression and `match`-as-expression still in their surface form), and by the time it reaches HIR lowering it has been fully annotated with resolved names and checked types by Name Resolution and the Type Checker. It answers "what did the developer write, and what does every part of it mean" — it is not yet concerned with how that meaning will be realized in Rust.

**HIR — "What does this program unambiguously do?"** The High-Level Intermediate Representation is produced by desugaring every remaining piece of Kyne-level syntax sugar into a small, canonical core: the `?` operator is expanded into its explicit match-and-early-return form (per [LANGUAGE_SPEC.md §7.8](./LANGUAGE_SPEC.md#78-error-propagation)), `if`- and `match`-as-expressions are normalized into a single canonical conditional-value construct, and every node carries its fully resolved type. HIR is still Kyne-semantics-shaped, not yet Rust-shaped — it has no notion of Rust ownership, borrowing, or the Soroban SDK's specific types. This is the representation the [Optimization Passes](#13-optimization) operate on, precisely because it is small and canonical enough that an optimization only needs to handle a handful of node kinds, not the full breadth of surface syntax.

**RIR — "How is this realized in Rust?"** The Rust Intermediate Representation is produced by lowering HIR into a tree that directly models the Rust constructs the code generator will emit: Kyne's `address` becomes `soroban_sdk::Address`, a `state` read/write becomes an explicit storage-API call, an `auth(addr)` statement becomes an explicit `require_auth()` call, and — critically — this is the stage at which the compiler makes every ownership and borrowing decision that [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only) explicitly reserves to the compiler's discretion: whether a given value is cloned, borrowed, or moved in the generated Rust is decided here, invisibly to Kyne source, chosen to produce the most idiomatic Rust a skilled engineer would write by hand.

**Why three representations, not two.** Collapsing HIR and RIR into one stage would force every optimization pass to also understand Rust-specific concerns (ownership, SDK type mapping) it has no reason to care about, and would force the Rust-specific lowering logic to also handle Kyne-level sugar it has no reason to re-derive. Keeping HIR "Kyne-shaped" and RIR "Rust-shaped," with the boundary between them exactly at the optimization stage, means optimizations reason about a small, stable, language-semantics-only tree, and Rust-specific lowering is a separate, later concern that operates on an already-optimized program — reducing the amount of Rust-shaped tree the code generator ever has to redundantly re-simplify.

**Why RIR is separate from Rust source text itself.** Making RIR lowering and Rust Code Generation two distinct stages — rather than having HIR lowering emit Rust text directly — means the actual text-printing step ([§14](#14-rust-code-generation)) is a nearly mechanical pretty-printer with no remaining semantic decisions of its own. This keeps code generation itself "dumb," in service of [Principle 5](#principle-5--predictable-output): a stage with no decisions left to make cannot introduce nondeterminism, and a stage this simple is easy to verify produces byte-identical output for identical RIR input.

---

# 13. Optimization

**Optimization MUST NEVER change observable behavior.** This is not a target to be balanced against performance — it is an absolute precondition every optimization pass MUST satisfy before it is eligible to run at all, per [Principle 3](#principle-3--compiler-correctness-before-optimization) and per [RUNTIME_MODEL.md §14.1](./RUNTIME_MODEL.md#141-why-optimizations-may-never-violate-these-invariants): an optimization that cannot be implemented without touching one of RUNTIME_MODEL.md's invariants is not a valid optimization, it is a proposal for new runtime behavior, and MUST go through the KIP process like any other.

Optimization passes operate exclusively on HIR ([§12](#12-intermediate-representations)) and produce HIR of the same shape, only smaller or simpler. The following passes are specified for v1:

**Dead code elimination.** HIR nodes proven, by the same reachability analysis underlying the Security Analyzer's [unreachable-code check](#111-analyses), to be unreachable are removed from the HIR before RIR lowering. This is safe by construction: code that can never execute has, by definition, no observable effect to preserve, so removing it changes nothing an external observer could detect. The Security Analyzer's reachability results MAY be reused directly by this pass rather than recomputed, since both consume the same underlying analysis; the two remain architecturally distinct because one reports its findings to the developer and the other silently acts on them in the generated output.

**Constant folding.** Expressions composed entirely of `const`s and literals are evaluated at compile time and replaced with their resulting literal value. Constant folding MUST preserve Kyne's checked-arithmetic semantics exactly: a constant expression that would overflow at runtime MUST be rejected as a compile-time error during folding (per the diagnostic example in [LANGUAGE_SPEC.md §14](./LANGUAGE_SPEC.md#14-error-philosophy)), never silently folded to a wrapped value.

**Inline expansion.** Trivial file-private or `internal` helper functions MAY be inlined into their call sites in HIR, reducing generated-Rust call overhead and, in many cases, improving generated-Rust readability by flattening a one-line helper directly into its caller. This is safe with respect to [RUNTIME_MODEL.md §8.1](./RUNTIME_MODEL.md#81-internal-function-calls): because an internal function call's staged state changes already merge directly into its caller's staging layer with no independent commit point, inlining changes nothing about a program's runtime-observable behavior — it only changes the shape of the generated Rust.

**Future optimizations.** Additional passes — for example, eliminating a provably-redundant repeated `auth(addr)` check on the same address within one function — are anticipated but explicitly **not** specified by this document, because such a pass would need to be checked against [RUNTIME_MODEL.md §14](./RUNTIME_MODEL.md#14-runtime-invariants)'s invariant that "every `auth(...)` call in source corresponds to a real runtime check, every time," and MUST NOT be implemented until a KIP has either demonstrated the optimization is compatible with that invariant as currently written or has amended the invariant itself.

**Optimization philosophy: correctness before speed.** The optimizer MUST be fully disableable via a compiler flag, and the compiler's correctness test suite ([§19](#19-testing-strategy)) MUST pass identically whether optimizations are enabled or disabled — the only permissible difference between an optimized and unoptimized build's behavior is speed and generated-code size, never outcome. This is what makes it possible to trust an optimization pass incrementally: a suspected optimizer defect can always be isolated by disabling optimization and confirming the defect disappears, without that disabling itself changing what "correct" means for the contract under test.

---

# 14. Rust Code Generation

The Rust Code Generator's sole responsibility is to render RIR ([§12](#12-intermediate-representations)) as formatted Rust source text, forming a complete, buildable Cargo crate.

**RIR to Rust.** Because every ownership, borrowing, and SDK-type-mapping decision was already made during RIR lowering, this stage is a pretty-printer: it walks the RIR tree and emits the corresponding Rust syntax, with no remaining semantic decisions of its own to make.

**Readable Rust.** The generator's raw text output MUST be passed through a deterministic Rust formatter (rustfmt or an equivalent) as a mandatory final step — generated Rust MUST NOT ever be presented to a developer or auditor in an unformatted or minified form. This is a direct, non-negotiable requirement of [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction): generated code that requires manual reformatting before a human can review it has not actually satisfied "generated Rust must remain readable."

**Soroban compatibility.** The generated crate applies the standard Soroban SDK attribute macros (`#[contract]`, `#[contractimpl]`, `#[contracttype]`, `#[contracterror]`) to the appropriate generated items, and its `Cargo.toml` declares the same dependencies any hand-written Soroban contract crate would. This is what makes the **Cargo Build** and **Soroban Build** pipeline stages ([§3](#3-compiler-pipeline)) ordinary, unmodified invocations of the standard toolchain, per [RUNTIME_MODEL.md §3](./RUNTIME_MODEL.md#3-project-lifecycle).

## 14.1 Code Organization

The generated crate's file layout mirrors the `.kyn` project's own file layout one-to-one: each `.kyn` file produces a corresponding Rust module at the equivalent path. A reader who knows which `.kyn` file declares a given `struct` or `fn` MUST be able to locate its generated Rust counterpart without needing to search the whole crate — this is a concrete, checkable expression of [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) rather than an abstract aspiration.

## 14.2 Compiler Guarantees

Every construct the code generator emits MUST correspond to a mapping documented either in this chapter or in a future, more granular code-generation reference; the generator MUST NOT introduce Rust behavior that [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) has not already defined at the language level. Where a genuinely new mapping is needed (for example, to support a new Soroban SDK feature), that mapping MUST be specified in a KIP before the code generator is changed to produce it, not introduced silently as an implementation detail.

## 14.3 Reviewability

Generated Rust MUST be reviewable by an experienced Rust developer without Kyne-specific tooling or training: it MUST use ordinary, idiomatic Rust patterns throughout — standard control flow, standard error handling via `Result` and `?`, standard Soroban SDK calls — and MUST NOT rely on macro-generated code whose expansion is not itself part of the emitted, readable output. A Rust developer unfamiliar with Kyne who is asked to audit a generated contract should be able to do so exactly as they would audit any other Soroban contract they did not write themselves.

## 14.4 Deterministic Generation

Identical RIR, compiled by an identical compiler version, MUST produce byte-identical Rust output. This requires specific engineering discipline within the code generator: item ordering within a generated file MUST follow a canonical, source-derived order (mirroring the canonical contract member order from [LANGUAGE_SPEC.md §3.2](./LANGUAGE_SPEC.md#32-canonical-member-order) where applicable, and declaration order elsewhere), and the generator MUST NOT rely on the iteration order of any non-deterministically-ordered internal collection (for example, an unordered hash map keyed by identifier) when deciding the order in which to emit items. A generator that incidentally produces different output across two runs of the identical compiler on identical input — even where that difference has no runtime-behavioral consequence — is a defect under [Principle 5](#principle-5--predictable-output), because it breaks an auditor's ability to trust that an observed diff in generated Rust reflects an actual source or compiler-version change.

---

# 15. Diagnostics Engine

Every diagnostic the compiler produces, regardless of which stage produced it, MUST contain the following fields:

| Field | Purpose |
|---|---|
| **Error Code** | A stable identifier (see [§15.1](#151-diagnostic-code-namespace)) allowing the diagnostic to be looked up, searched for, and referenced independent of its message text, which MAY be reworded across compiler versions. |
| **Title** | A one-line summary of the problem. |
| **Explanation** | The specific defect, shown in the context of the offending source span. |
| **Reason** | Why this is a problem in Kyne's domain specifically — not merely "this is invalid syntax," but the smart-contract-relevant consequence, where one exists. |
| **Suggested Fix** | A concrete correction, shown as a code snippet wherever the fix is mechanical. |
| **Future Documentation Link** | A stable reference (by code) into the language documentation site, allowing a developer to look up more detail than fits in a terminal diagnostic. This field is reserved for a future documentation-site integration and is not further specified here — see [Non-Goals](#non-goals). |

## 15.1 Diagnostic Code Namespace

Diagnostic codes are prefixed to make their severity and blocking status legible from the code alone, without needing to read the surrounding message:

- **`KYxxxx`** — Compiler Errors. Blocking. Produced by the Parser, Type Checker, and Semantic Analyzer.
- **`KSxxxx`** — Security Analyzer findings. Non-blocking (Warning or Hint). Produced only by the Security Analyzer.

Within the `KY` namespace, codes are grouped by concern:

| Range | Concern |
|---|---|
| `KY00xx` | Lexical |
| `KY01xx` | Storage / state |
| `KY02xx` | Types / arithmetic |
| `KY03xx` | Events |
| `KY04xx` | Contract structure / canonical ordering |
| `KY05xx` | Authorization |
| `KY06xx` | Visibility / modules |
| `KY07xx` | Control flow / semantic |

This extends the numbering scheme already established by [LANGUAGE_SPEC.md §14](./LANGUAGE_SPEC.md#14-error-philosophy)'s `KY01xx`/`KY02xx`/`KY03xx` examples; this document adds the remaining ranges required to cover every stage described here, and reserves the `KS` prefix entirely for the Security Analyzer.

## 15.2 Examples

**A Compiler Error** (Semantic Analyzer, canonical order violation):

```
error[KY0401]: contract members are out of canonical order
  --> src/token.kyn:22:5
   |
22 |     public fn transfer(...) { ... }
   |     ^^^^^^^^^^^^^^^^^^^^^^^ this `public fn` appears before an `event` declaration
   |
   = note: LANGUAGE_SPEC.md §3.2 requires: state, const, error, event, struct/enum, init, public fn, internal fn, unmarked fn
   = help: move `event Transfer(...)` above this function, or move this function below all `event` declarations
```

**A Warning** (Security Analyzer, missing `auth()`):

```
warning[KS0101]: this function mutates `state` but calls no `auth(...)`
  --> src/token.kyn:31:1
   |
31 | public fn set_admin(new_admin: address) {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutates `state.admin` with no authorization check found in this function
   |
   = note: this is a heuristic finding — some public mutating functions are intentionally unauthenticated
   = help: if `new_admin` should require the current admin's approval, add: auth(admin);
```

## 15.3 Diagnostic Categories

Diagnostics are categorized by the stage that produced them — Lexical, Syntax, Name Resolution, Type, Semantic, and Security — matching the pipeline stages in [§3](#3-compiler-pipeline). A diagnostic's category is always determinable from its code prefix and range per [§15.1](#151-diagnostic-code-namespace), giving a developer or a tooling integration a stable way to filter, group, or specially render diagnostics by origin without needing to parse message text.

## 15.4 Why the Compiler Teaches Developers

The diagnostic shape required here exists to make [Principle 1](#principle-1--the-compiler-is-a-teacher) an enforceable engineering requirement rather than a matter of individual contributor taste: a diagnostic missing its Reason or Suggested Fix field is an incomplete implementation of a compiler check, in the same sense that a function missing its error handling is an incomplete implementation of that function — both are defects to be fixed, not stylistic gaps to be tolerated.

---

# 16. Incremental Compilation

The compiler MUST avoid re-running pipeline stages whose input has not changed since the last successful compilation, per [Principle 4](#principle-4--fast-incremental-compilation).

**Dependency graph.** Kyne v1 tracks dependencies at **file granularity**: each `.kyn` file is a node, and each `use` import is a directed edge from the importing file to the imported file's module. This is a deliberate simplification over declaration-level granularity — tracking dependencies per individual `struct`, `fn`, or `const` would allow more precise invalidation (a change to one function need not invalidate files that only depend on an unrelated function in the same file), but at meaningfully higher bookkeeping complexity. File-granularity tracking is chosen for v1 because typical Kyne contract projects are small (one contract file plus a handful of supporting library files, per [Project Model](./LANGUAGE_PRINCIPLES.md#project-model)), making the precision gap rarely consequential in practice; declaration-level granularity is reserved as a future KIP once large, multi-file Kyne projects are common enough to justify the added complexity.

**Change detection.** Each `.kyn` file is content-hashed. A file is considered changed if its current hash differs from the hash recorded the last time it was successfully compiled.

**Partial recompilation.** On a subsequent build, the compiler re-runs the pipeline only for files whose hash has changed, plus every file that transitively depends on a changed file via the dependency graph (since a change to an imported `struct`, `fn`, or `const` signature could affect the importer's Name Resolution or Type Checking results). Files unaffected by the change reuse their cached results from [§17](#17-compiler-cache) for every stage up to and including the last stage whose result remains valid.

**Cache invalidation.** A change to the compiler version itself MUST invalidate the entire cache unconditionally, regardless of whether any source file changed, since [Principle 5](#principle-5--predictable-output)'s byte-identical-output guarantee is only promised for a fixed compiler version — reusing a cache entry produced by a different compiler version would risk silently mixing output from two versions that make different codegen decisions.

**Philosophy: only compile what changed.** This principle governs every design choice in this section: the dependency graph exists to know what "changed" transitively means, change detection exists to know what literally changed, and the cache in [§17](#17-compiler-cache) exists to make "not recompiling" actually mean "reusing a stored result" rather than "silently skipping work and hoping the old result is still valid."

---

# 17. Compiler Cache

The compiler cache is a persistent, on-disk store of every pipeline stage's output, keyed so that a subsequent build can determine, without redoing the work, whether a given stage's cached output is still valid for the current input.

## 17.1 Layout

```
.kyn/
  cache/
    <compiler-version>/
      cst/
      ast/
      hir/
      rir/
      diagnostics/
  manifest.json
```

The cache root is named `.kyn/`, matching the project's own `.kyn` source file convention established in [LANGUAGE_SPEC.md §1](./LANGUAGE_SPEC.md#1-lexical-structure) — a project's tooling directory MUST share its source file's brand name for the same reason every other naming convention in this ecosystem is kept consistent, per [Consistency Above Preference](./LANGUAGE_SPEC.md#consistency-above-preference).

Cache entries are partitioned by `<compiler-version>` at the top level, so that switching compiler versions never requires manually clearing a stale cache — an old version's cache subdirectory simply goes unused rather than being consulted and potentially misapplied.

**`cst/`** — Cached Concrete Syntax Trees, keyed by file content hash. Consumed primarily by the formatter and language-server tooling described in [§22](#22-library-first-compiler-architecture), which frequently need only this stage's output.

**`ast/`** — Cached Abstract Syntax Trees, keyed by file content hash. Represents the post-parse, pre-semantic state of a file.

**`hir/`** — Cached HIR, keyed by a composite of the file's own content hash and the content hashes of every file it transitively depends on (per the dependency graph in [§16](#16-incremental-compilation)), since HIR construction depends on fully resolved, fully typed information that can be affected by a dependency's change even when the file itself is untouched.

**`rir/`** — Cached RIR, keyed identically to `hir/`. Because Rust code generation typically assembles a whole crate from the full project's RIR, this cache is most useful for supporting fast repeated `kyne check` invocations that stop short of full code generation while still allowing a subsequent `kyne build` to reuse already-computed RIR for unaffected files.

**`diagnostics/`** — Cached diagnostic results per file, keyed identically to the stage that produced them. Allows a `kyne check` invocation with no changed files to return its previous diagnostic output instantly with no recomputation at all.

**`manifest.json`** — Records the mapping from file paths to their last-known content hashes, the compiler version used to produce the current cache, and the cache format version, allowing the compiler to determine cache validity without needing to inspect every cached artifact individually.

## 17.2 Invalidation

A cache entry for a given file and stage is invalid, and MUST be recomputed, whenever: the file's own content hash has changed; any file it transitively depends on has a changed content hash (for `hir/` and `rir/` entries specifically, per their composite keys); the compiler version differs from the version recorded for that cache entry; or the cache format version itself has changed (reserved for future compiler releases that alter a stage's output data structure).

## 17.3 Cleanup

Because every cache entry is a pure, derived function of its key (file hashes plus compiler version), the cache is, architecturally, a strictly disposable artifact: **deleting any subset of `.kyn/cache/`, up to and including the entire directory, MUST NEVER change the correctness of a subsequent build — only its speed.** A future `kyne clean` command MAY prune cache entries that have not been accessed within some retention window without any risk of corrupting a later build, precisely because of this guarantee. A conforming implementation MUST NOT introduce any cache entry that is not a pure function of its stated key, since doing so would violate this cleanup guarantee.

## 17.4 Reproducibility

Because cache keys are derived entirely from content hashes and compiler version — never from file paths, timestamps, or any other machine-specific detail — two different machines compiling the same `.kyn` source with the same compiler version MUST compute identical cache keys and, per [Principle 5](#principle-5--predictable-output), MUST produce identical final Rust output. The cache itself introduces no source of cross-machine variance.

---

# 18. Plugin Architecture

**Kyne v1 does not support compiler plugins** — arbitrary third-party code executing inside the compiler process during compilation. This is a deliberate security decision, not a missing feature: allowing arbitrary plugin code to run during the compilation of a smart contract would introduce a trust and auditability surface directly contrary to [Kyne's exclusion of runtime magic and unsafe code](./LANGUAGE_PRINCIPLES.md#language-paradigm) — a malicious or merely buggy plugin could silently alter the generated Rust for a contract an author believes they have fully reviewed in `.kyn` form, defeating [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) at its root.

**The architecture nonetheless reserves extension points**, as a direct consequence of the [Library-First Compiler Architecture](#22-library-first-compiler-architecture) described in the next chapter: because every stage — `kyne_lexer`, `kyne_parser`, `kyne_cst`, `kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_semantics`, `kyne_security`, `kyne_hir`, `kyne_optimizer`, `kyne_rir`, `kyne_codegen` — is already a separately consumable library crate with a well-defined, stable input/output boundary, a future plugin system, should one ever be proposed by KIP, would not require restructuring the compiler. It would only require defining a hook API around boundaries that already exist as real architectural seams, not retrofitting seams into a previously monolithic implementation.

**Future possibilities**, none of which are designed or promised by this document:

- **Static analyzers** — additional passes consuming HIR alongside the built-in Security Analyzer, potentially project-specific rather than part of the core language distribution.
- **Custom diagnostics** — a project-defined lint rule set layered on top of the fixed `KS`-namespace checks in [§11](#11-security-analyzer).
- **Alternative generators** — a hypothetical backend targeting something other than Rust/Soroban, consuming HIR (or an optimized HIR) as its input, exactly as `kyne_codegen` does today. This is explicitly speculative and not a roadmap commitment.
- **Documentation generators** — notably, the Documentation Generator described in [LANGUAGE_PRINCIPLES.md's Future Vision](./LANGUAGE_PRINCIPLES.md#future-vision) does not actually require a plugin mechanism at all: it can be built as an ordinary consumer of `kyne_cst` (for doc comments) and `kyne_ast` (for declaration shapes), exactly like any other tool described in [§22](#22-library-first-compiler-architecture). This is worth stating explicitly because it demonstrates that the library-first architecture already satisfies most of what a plugin system would otherwise need to provide.

---

# 19. Testing Strategy

Compiler testing occurs **per stage**, mirroring the pipeline's own decomposition, so that a defect can be localized to the specific stage responsible for it without needing to reproduce it through the entire pipeline first.

**Lexer tests.** Golden token-stream tests: given a source text fixture, assert the exact resulting token sequence, including span information. Include fixtures specifically exercising error recovery — malformed input that should produce an `Error` token and a diagnostic, followed by successful resumption.

**Parser tests.** Golden CST/AST tests per grammar production in [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar), covering both accepted inputs and rejected inputs with their expected diagnostics, including recovery-and-resynchronization cases.

**AST tests.** Tests targeting the CST→AST lowering rules specifically ([§7](#7-abstract-syntax-tree)) — trivia removal, redundant-parenthesization collapse, and field-init shorthand expansion — verified independent of any later semantic concern.

**Type checker tests.** Positive and negative type-compatibility cases spanning every rule in [§9](#9-type-checker): primitive type mismatches, `Option`/`Result` handling, collection method signatures, assignment and return validation, and the inference boundary between `let` (inferred) and every other construct (not inferred).

**Semantic tests.** Cases covering canonical member ordering, definite assignment, visibility legality, single-contract-per-project enforcement, event argument validation, and every other rule in [§10](#10-semantic-analysis) — both violating and non-violating fixtures for each rule.

**Security tests.** Each Security Analyzer check in [§11.1](#111-analyses) requires both a true-positive fixture (a pattern the check is meant to catch) and a documented, intentional false-negative or accepted-pattern fixture (a case the heuristic is known not to catch, or a legitimate pattern it must not flag) — since these checks are explicitly imperfect by design, their test suite MUST document the heuristic's known boundaries rather than only demonstrating its successes.

**Code generation tests.** Golden-file tests comparing generated Rust text against checked-in expected output. The six canonical example contracts in [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) — Counter, Token, Escrow, Marketplace, Voting, and Multisig Wallet — MUST be maintained as the canonical end-to-end golden corpus for this test category, since they are already the language's own normative usage examples.

**Golden file tests.** A general methodology used across several of the categories above: an input fixture and its expected output fixture are both checked into the repository; the test suite diffs actual output against the checked-in expectation on every run. Any intentional change to output requires updating the golden file in the same commit as the compiler change that caused it, which forces a reviewer to see the exact output diff during code review rather than trusting a change description alone.

**End-to-end tests.** The strongest test category: full pipeline from `.kyn` source through generated Rust, and from there through an actual invocation of the real Soroban/Rust toolchain, confirming the generated crate both compiles and, where a local Soroban test environment is available, executes with the exact runtime behavior [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) specifies — for example, an end-to-end test asserting that a `throw`ed Recoverable Error inside a cross-contract call rolls back only the callee's staged changes while the caller's own subsequent state changes still commit, per [RUNTIME_MODEL.md §9.5](./RUNTIME_MODEL.md#95-scoped-rollback-and-catching).

---

# 20. Repository Architecture

```
compiler/
  lexer/
  parser/
  cst/
  ast/
  resolver/
  types/
  semantics/
  security/
  hir/
  optimizer/
  rir/
  codegen/
  diagnostics/
  cache/
  driver/
  tests/
```

| Directory | Responsibility |
|---|---|
| `lexer/` | Tokenization, per [§4](#4-lexical-analysis). |
| `parser/` | Grammar enforcement and CST construction, per [§5](#5-parsing). |
| `cst/` | The CST data structure and CST→AST lowering, per [§6](#6-concrete-syntax-tree) and [§7](#7-abstract-syntax-tree). |
| `ast/` | The AST data structure and its shared visitor abstraction, per [§7](#7-abstract-syntax-tree). |
| `resolver/` | Name Resolution, per [§8](#8-name-resolution). |
| `types/` | The Type Checker, per [§9](#9-type-checker). |
| `semantics/` | The Semantic Analyzer, per [§10](#10-semantic-analysis). |
| `security/` | The Security Analyzer, per [§11](#11-security-analyzer). |
| `hir/` | HIR construction and its shared visitor abstraction, per [§12](#12-intermediate-representations). |
| `optimizer/` | Optimization passes, per [§13](#13-optimization). |
| `rir/` | RIR construction, per [§12](#12-intermediate-representations). |
| `codegen/` | Rust code generation, per [§14](#14-rust-code-generation). |
| `diagnostics/` | The shared diagnostic type, formatting, and code namespace, per [§15](#15-diagnostics-engine). |
| `cache/` | The compiler cache, per [§17](#17-compiler-cache). |
| `driver/` | Pipeline orchestration: the only crate that sequences every other stage into a full compilation. |
| `tests/` | Cross-stage integration and end-to-end tests, per [§19](#19-testing-strategy). |

**Ownership and boundaries.** Each directory corresponds to exactly one pipeline stage or one clearly bounded cross-cutting concern, exposes a single public API surface, and is consumed only by `driver/` and by any later stage with a legitimate reason to depend on it. The dependency graph among these directories MUST be **acyclic and strictly forward**, matching the pipeline order in [§3](#3-compiler-pipeline) exactly: `resolver/` MUST NOT depend on `types/`, `semantics/` MUST NOT depend on `security/`, and no stage crate may depend on `driver/`, since `driver/` is the only crate permitted to depend on every stage. `diagnostics/` is a foundation crate: every stage MAY depend on it (since every stage emits diagnostics), but it MUST NOT depend on any stage crate itself, keeping it a leaf in the dependency graph. `cache/` is consumed by `driver/` to wrap each stage invocation transparently and MUST NOT be depended upon by any individual stage crate directly, since caching is an orchestration concern, not a per-stage one. `tests/` sits outside the pipeline dependency graph entirely, as an integration-test crate permitted to depend on everything.

This strict, acyclic, forward-only dependency structure is what makes the "one responsibility, one input, one output" discipline from [§2](#2-compiler-overview) enforceable by the build system itself, not merely by convention: a change that attempts to introduce a backward dependency (for example, `resolver/` reaching into `types/` for convenience) MUST fail to compile the compiler's own codebase, catching the architectural violation at the earliest possible point.

---

# 21. Compiler Invariants

The following invariants MUST always hold for every conforming Kyne compiler implementation. They are restated here as an explicit checklist against which every future change to the compiler — including every new pipeline stage, every optimization, and every refactor — MUST be validated before merging.

**The compiler never changes program meaning.** Every stage from the Parser through the Rust Code Generator is, in aggregate, a meaning-preserving translation from `.kyn` source to Rust source, per [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)'s definition of what a Kyne program means. No stage — including Optimization — is permitted to alter what a program does; each is permitted only to change how that unchanged meaning is represented or realized.

**Every transformation is deterministic.** Restated from [Principle 5](#principle-5--predictable-output) and enforced concretely by [§14.4](#144-deterministic-generation)'s code-generation discipline and [§17.4](#174-reproducibility)'s cache-key discipline: identical input, at any pipeline stage, produces identical output, on any machine, for a fixed compiler version.

**Every optimization preserves runtime guarantees.** Restated from [§13](#13-optimization) and grounded directly in [RUNTIME_MODEL.md §14](./RUNTIME_MODEL.md#14-runtime-invariants): an optimization is validated not only against "does this preserve behavior" in the abstract, but specifically against every named invariant in RUNTIME_MODEL.md §14 — determinism, atomicity, explicit authorization, event-transaction binding, and rollback correctness.

**Generated Rust remains readable.** Restated from [§14.3](#143-reviewability): this is not a best-effort aspiration but a MUST-level requirement checked, in practice, by the golden-file code-generation tests in [§19](#19-testing-strategy) and by mandatory formatting via [§14](#14-rust-code-generation)'s required rustfmt pass.

**Diagnostics never guess.** A diagnostic MUST NEVER assert more certainty than its producing stage actually has. A `KY`-prefixed Compiler Error MUST be unconditionally correct — since it blocks compilation, being wrong is never acceptable — while a `KS`-prefixed Security Analyzer finding is explicitly permitted to be probabilistic, provided its message is honestly phrased as a heuristic observation (per the example in [§15.2](#152-examples), "this is a heuristic finding") rather than as a settled accusation. "Never guess" means never overstating confidence, not never expressing an informed suspicion — the Security Analyzer's entire purpose is to express informed suspicions, honestly labeled as such.

**Security analysis occurs before code generation.** Restated from the pipeline order in [§3](#3-compiler-pipeline): the Security Analyzer runs, and its findings are surfaced to the developer, before HIR lowering and Rust Code Generation ever begin. Note that because Security Analyzer findings are non-blocking by default (per [§11.2](#112-severity-levels)), "before code generation" governs the order in which findings become *visible* to the developer, not whether code generation is permitted to proceed afterward — it always is, absent a project-level configuration escalating a specific finding to blocking status.

**Compiler stages remain independently testable.** Restated from [§2](#2-compiler-overview) and [§19](#19-testing-strategy), and enforced structurally by the acyclic, forward-only dependency graph in [§20](#20-repository-architecture): a stage's tests MUST be constructible from that stage's documented input type alone, with no requirement to run any earlier stage to produce a valid test fixture.

**Why future contributors may never violate these invariants.** Every invariant in this section exists because a deployed Kyne contract's correctness has already been reviewed by its authors, its reviewers, and often a paid audit, against exactly the guarantees this document and RUNTIME_MODEL.md describe. A compiler change that violates one of these invariants — even one its author believes is a strict improvement — silently invalidates every prior review that relied on the invariant holding, because none of those reviews could have anticipated a violation that had not yet been introduced. These invariants are therefore not a checklist to satisfy once at merge time; they are the permanent boundary within which all future compiler engineering must occur.

---

# 22. Library-First Compiler Architecture

Kyne is architected as a set of reusable libraries, not a monolithic compiler executable. Each pipeline stage from [§3](#3-compiler-pipeline) corresponds to its own published crate:

```
kyne_lexer
kyne_parser
kyne_cst
kyne_ast
kyne_resolver
kyne_types
kyne_semantics
kyne_security
kyne_hir
kyne_optimizer
kyne_rir
kyne_codegen
kyne_diagnostics
kyne_cache
```

A thin `kyne_driver` crate composes these into the full pipeline described in [§3](#3-compiler-pipeline), and the `kyne` CLI binary is itself a thin wrapper over `kyne_driver`. This mirrors the repository layout in [§20](#20-repository-architecture) directly — each directory there is, correspondingly, one of these published crates.

**Why this benefits every tool in the ecosystem**, per [LANGUAGE_PRINCIPLES.md's Future Vision](./LANGUAGE_PRINCIPLES.md#future-vision):

**CLI.** `kyne build`, `kyne check`, and every other CLI subcommand are thin compositions of these libraries, invoked through `kyne_driver` — the CLI itself contains no compiler logic of its own.

**Formatter.** `kyne fmt` needs only `kyne_lexer`, `kyne_parser`, and `kyne_cst` — formatting does not require semantic analysis to succeed, matching how mature formatters for other languages operate on syntactically valid but not-yet-type-checked code. Depending on only these three crates keeps the formatter fast and keeps it functional even on a file with type errors elsewhere in the project.

**Language Server.** An LSP implementation consumes `kyne_cst`, `kyne_ast`, `kyne_resolver`, and `kyne_types` incrementally, per keystroke, without ever needing `kyne_optimizer` or `kyne_codegen` — an editor needs diagnostics and navigation information, never generated Rust, making this a meaningfully lighter-weight consumer than a full build.

**Documentation Generator.** Needs only `kyne_cst` (for doc-comment text and attachment) and `kyne_ast` (for declaration shapes) — no semantic analysis, optimization, or code generation dependency at all.

**Playground.** Consumes the full pipeline through `kyne_codegen`, identically to the CLI's `kyne build`, just invoked within a sandboxed or web-hosted context rather than a local shell.

**Testing.** A future `kyne_test` framework can consume `kyne_ast` and `kyne_hir` directly to instrument or introspect contract code for test-generation purposes, without shelling out to the CLI binary and parsing its text output.

**Future tooling.** A hypothetical, narrowly scoped security-audit tool could depend on `kyne_security` alone, without pulling in `kyne_optimizer` or `kyne_codegen` at all, since those crates are irrelevant to its purpose.

**Why code reuse is critical.** Without this architecture, every consuming tool faces one of two unacceptable choices: shell out to the `kyne` CLI binary and parse its human-oriented text output (fragile, slow, and lossy — structured information gets flattened to text and must be re-parsed), or independently reimplement lexing, parsing, and type checking (guaranteed to drift out of sync with the real compiler over time, producing an IDE or formatter that silently disagrees with what an actual `kyne build` would report). Neither outcome is acceptable for a language whose IDE tooling and CLI must agree exactly on what is and is not a valid program — a guarantee only achievable when they are, literally, built from the same code, not two independent approximations of it.

---

# 23. Future Compiler Evolution

**Architectural changes require a KIP.** A change to any MUST-level rule in this document — a new pipeline stage, a change to the boundary between two existing stages, a change to the repository's dependency structure — MUST go through the Kyne Improvement Proposal process defined in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution), exactly as a language or runtime change would. Unlike [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), whose rules are externally binding on every conforming implementation, this document's rules bind the reference implementation's internal structure specifically — but the same KIP discipline applies, because this document is the reference every future contributor is expected to follow, and an implementation that silently diverges from it without a corresponding KIP has made this document inaccurate rather than merely outdated.

**Backward compatibility is preferred.** Because the libraries in [§22](#22-library-first-compiler-architecture) are consumed by external tooling — formatters, language servers, documentation generators, and whatever future tools the ecosystem produces — a breaking change to one of their public APIs carries the same "don't break what depends on you" obligation that [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution) applies to deployed contracts, just directed at tool authors rather than contract authors. A KIP proposing a breaking change to a `kyne_*` library's public API MUST identify which downstream tools it is expected to affect and MUST prefer an additive path over a breaking one wherever one exists.

**Large rewrites should be avoided.** The library-first, strictly bounded, acyclic pipeline architecture described in this document exists specifically to make incremental, stage-by-stage evolution possible: a single stage's internals can be substantially rewritten — a new optimization algorithm, a faster parser implementation — without requiring changes to any other stage, provided its documented input/output contract is preserved. A full compiler rewrite should almost never be necessary precisely because well-maintained boundaries let the project replace one piece at a time; a proposal for a large rewrite SHOULD be treated as a signal that the existing boundaries have degraded and are themselves due for a targeted KIP, not as a routine engineering decision.

---

# Non-Goals

This document does **not** define, and explicitly defers to other documents:

- **Language syntax and grammar** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Memory model implementation strategy** — the compiler-internal strategy for realizing [LANGUAGE_SPEC.md §4.0](./LANGUAGE_SPEC.md#40-memory-model-value-semantics-only)'s value-semantics guarantee efficiently in generated Rust (when to clone, when to borrow) is the subject of a forthcoming `MEMORY_MODEL.md`; this document establishes only that RIR lowering ([§12](#12-intermediate-representations)) is the stage where those decisions are made, not what the decisions themselves should be.
- **Standard library** — the exact function names, signatures, and behavior for execution-context access, collection operations, and type conversions belong to a forthcoming `STANDARD_LIBRARY.md`.
- **Package management** — reserved, and explicitly out of scope, per [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem).
- **IDE implementation** — the specific behavior of any language server or editor extension belongs to a forthcoming `TOOLCHAIN.md`; this document defines only the libraries such an implementation would consume, per [§22](#22-library-first-compiler-architecture).
- **CLI implementation** — the specific commands, flags, and output format of the `kyne` CLI belong to a forthcoming `TOOLCHAIN.md`.

Nothing in this document should be read as specifying developer-facing tool behavior — only the internal architecture that makes correct, consistent tool behavior possible.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission and values the compiler exists to serve.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy and KIP governance process this document's own evolution follows.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the syntax and static semantics this compiler must accept and reject exactly as specified.
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — the runtime behavior every stage of this pipeline, from Semantic Analysis through Rust Code Generation, must faithfully preserve.

The following documents are anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `MEMORY_MODEL.md`, `STANDARD_LIBRARY.md`, `TOOLCHAIN.md`, and `ROADMAP.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is the single engineering blueprint for the Kyne compiler. A contributor joining the project for the first time should be able to determine, from this document alone: how the compiler is organized into stages, how information flows between them, which directory and which crate owns a given responsibility, where a new contribution belongs, and how a new compiler stage would integrate with the existing pipeline without restructuring it. Where a future contributor finds an architectural question this document does not answer, that is a gap to be closed by a KIP amending this document — not a decision to be made silently in the implementation.
