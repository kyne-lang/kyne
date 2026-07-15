# LANGUAGE_PRINCIPLES.md

# Kyne — Language Principles

**Phase:** 0 — Foundations
**Milestone:** 2 — Language Principles
**Version:** 0.1 (Draft)
**Status:** Foundational Document — supersedes no prior document, formalizes [FOUNDATION.md](./FOUNDATION.md)

---

## Purpose of This Document

[FOUNDATION.md](./FOUNDATION.md) establishes *why Kyne exists*. This document establishes *what Kyne is, as a language* — its identity, its design philosophy, the principles that constrain every future decision, and the governance model that lets those principles survive contact with real-world pressure.

Every future KIP (Kyne Improvement Proposal), every compiler feature, every standard library API, and every line of the specification must be traceable back to a principle in this document. If a proposed feature cannot be justified by anything written here, it does not belong in Kyne — regardless of how popular, convenient, or technically impressive it is.

This document is written for maintainers, contributors, and language designers, not for end users learning to write their first contract. It is deliberately opinionated. A language without opinions is not a language — it is a syntax.

---

# Language Identity

## What Kyne Is

Kyne is a statically typed, contract-oriented programming language that compiles to readable, idiomatic, Soroban-compatible Rust. It is purpose-built for one job: writing smart contracts on the Stellar network. It is not a general-purpose language wearing a smart-contract costume, and it is not a thin templating layer over the Soroban SDK. Kyne has its own grammar, its own type system, its own compiler diagnostics, and its own toolchain, all shaped by the constraints and dangers unique to on-chain code.

Kyne is a **source-to-source compiler target**, not a virtual machine and not a new runtime. The Rust it emits is the artifact that actually gets compiled to WASM and deployed to Soroban. This is a deliberate architectural choice: Kyne inherits Soroban's security properties, its execution guarantees, and its ecosystem compatibility for free, while contributing an entirely new authoring layer on top. Kyne developers are, whether they realize it or not, always writing Rust — Kyne simply gives them a better sentence to write it in.

Kyne is also a **complete platform**, not merely a grammar. From its first public release, the intention is for Kyne to include a compiler, a CLI, a formatter, a language server, a documentation generator, a testing framework, and a static security analyzer. A language without tooling is an academic exercise; a language with tooling is a place developers can build a career.

## What Kyne Is Not

Kyne is not a Rust replacement. Rust remains the substrate. Kyne developers who want to drop to raw Rust, read the generated output, or hand-tune a hot path retain that ability at all times — the generated code is not obfuscated, minified, or treated as a build artifact to be ignored. It is meant to be opened, read, and understood.

Kyne is not a blockchain, a virtual machine, or a competing execution environment. It has no opinions about consensus, no opinions about validator sets, and no ambition to abstract over multiple chains. It is written for Soroban, and Soroban only. Multi-chain portability is explicitly out of scope — see [Non-Goals](#non-goals).

Kyne is not a dynamically typed scripting language, despite drawing surface-level inspiration from TypeScript's approachability. Every Kyne value has a statically known type at compile time. There is no `any`, no implicit coercion between incompatible types, and no runtime type discovery. The familiarity Kyne borrows from TypeScript is about *reading comfort*, not about *type discipline* — on that axis, Kyne sits far closer to Rust.

Kyne is not a framework, a package manager, or an application platform. It does not manage dependencies for a whole software company, it does not orchestrate microservices, and it does not have opinions about what happens off-chain. It has exactly one product: contracts that are correct, secure, and easy to read, deployed to Soroban.

---

# Design Philosophy

Every decision in Kyne's design — syntax, type system, standard library shape, compiler error format, even the CLI's verb choices — is downstream of a small number of philosophical commitments. This section explains those commitments and, critically, *why* they exist, because the "why" is what lets future contributors extend the language correctly in situations this document didn't anticipate.

## Why Contracts Are the Unit of Everything

Traditional general-purpose languages are organized around functions, modules, or classes as the primary unit of composition. Kyne is organized around the **contract**. This is not a stylistic preference — it reflects the actual deployment reality of Soroban. A Soroban contract is compiled to a single WASM binary, uploaded once, and then lives on-chain, immutable, interacting with real value, for as long as the network exists. Every abstraction Kyne offers has to answer to that reality.

This is why Kyne enforces a **one project → one contract → one deployment artifact** model rather than allowing a project to emit an arbitrary number of loosely related contracts. Ambiguity about "what actually gets deployed" is a source of real-world incidents in other ecosystems, where a build pipeline silently produces the wrong artifact, or a project accretes multiple contracts whose interactions are not obvious from reading any single file. Kyne closes that ambiguity at the language level: a project has exactly one deployable contract, and everything else in that project — libraries, types, helper modules — exists in service of that one contract. Libraries can be shared, versioned, and imported across projects, but they are never independently deployable. There is no such thing as an accidental deployment in Kyne, because there is no such thing as an ambiguous deployment target.

## Why Rust Is the Compilation Target, Not a Foreign Backend

Some language designers treat their compilation target as an implementation detail to be hidden from the user. Kyne treats Rust as a **first-class part of the developer experience**. The generated Rust is expected to be read by contributors during code review, audited by security professionals during a formal audit, and debugged by developers when something goes wrong at a layer Kyne's abstractions don't reach.

This single decision — that generated code must remain human-legible — eliminates entire categories of design that would otherwise be attractive. It rules out aggressive macro-driven code generation that produces unreadable expansions. It rules out naming schemes that mangle identifiers for internal bookkeeping. It rules out any optimization that would make the emitted Rust diverge structurally from what an experienced Rust developer would have written by hand. Transparent abstraction is not just a principle Kyne aspires to — it is a hard constraint on the compiler's output format.

## Why Security Is Structural, Not Cultural

Every ecosystem with real value moving through smart contracts has, at some point, learned the same lesson: telling developers to "be careful" does not work at scale. Reentrancy, unchecked arithmetic overflow, unauthorized access to privileged functions, and unvalidated cross-contract calls are not rare mistakes made by careless developers — they are the *default outcome* of languages that make the safe path optional. Kyne's response is to make the safe path the only path wherever the compiler can enforce it, and to make deviations from that path visible, explicit, and require deliberate developer action rather than being possible by omission.

This has downstream consequences for syntax. Where another language might let an integer silently overflow, Kyne requires the developer to either use a type that cannot overflow within its domain or to explicitly opt into checked arithmetic with a visible operation. Where another language might let any public function be called by any caller, Kyne's contract model treats authorization as a first-class, checkable property of a function's signature rather than something buried in the function body. These are not add-on security features bolted onto a general-purpose core — they are the reason the language's core exists in its particular shape.

## Why Familiarity Matters, and Where It Stops

Kyne borrows surface syntax and structural conventions from TypeScript and Go specifically because on-chain development already asks developers to absorb an enormous amount of unfamiliar domain knowledge — persistent storage models, authorization primitives, cross-contract invocation, gas-like resource metering. Asking them to *also* learn an unfamiliar syntax on top of that is an avoidable tax. Familiar syntax lowers the activation energy required to start being productive, which directly serves the mission stated in [FOUNDATION.md](./FOUNDATION.md): a developer should be able to write their first Soroban contract within an hour.

But familiarity is a tool, not a goal. Kyne does not adopt a TypeScript or Go convention because it is popular — it adopts it because it improves readability, reduces boilerplate, or matches a mental model developers already have that also happens to be *correct* for the smart contract domain. Where a familiar convention would be actively misleading in a smart-contract context — for instance, TypeScript's permissive type coercion, or dynamic property access — Kyne rejects it outright, even at the cost of some initial unfamiliarity. Correctness always outranks familiarity when the two are in tension.

---

# Core Principles

The following principles are not marketing language. They are engineering constraints. Every accepted KIP must be checked against each of these, and any KIP that regresses one of them without an extraordinary justification should be rejected regardless of its other merits.

## Security Before Convenience

Security before convenience means that when a proposed feature would make code shorter, faster to write, or more ergonomic, but would also introduce a new way for a contract to be exploited, misused, or silently miscompiled, the feature is rejected or reworked until the security cost is eliminated. This ordering is absolute, not a "usually" — Kyne does not have a convenience escape hatch that quietly disables safety checks the way some languages offer an `unsafe` or `any` keyword as a pressure release valve for impatient developers.

This principle exists because smart contracts are unusual among software artifacts: once deployed, they are frequently immutable, they directly control real economic value, and their failure modes are public, permanent, and often irreversible. A convenience bug in a web application produces a support ticket. A convenience bug in a smart contract produces a headline. Kyne's design process must weigh every ergonomic improvement against this asymmetry.

In practice, this means the compiler is willing to reject code that would compile fine in Rust, C, or TypeScript, if that code expresses a pattern statistically associated with real-world exploits — unchecked external calls before state updates, unvalidated authorization on privileged entry points, integer operations that can silently wrap. Convenience features are welcomed enthusiastically when they *reduce* the surface area for mistakes — such as syntax that makes the checks-effects-interactions pattern the default rather than something the developer must remember — but never when they merely reduce keystrokes at the expense of a guarantee.

## Transparent Abstraction

Transparent abstraction is the principle that Kyne may remove *repetition* but must never remove *understanding*. A developer reading Kyne source and its generated Rust side by side should always be able to answer, "what actually happens on-chain when this runs?" without needing to reverse-engineer a macro expansion or trust a black box.

This principle directly shapes the compiler's architecture: Kyne is not permitted to rely on procedural macros, code generation tricks, or reflection-like mechanisms that obscure the relationship between source and output. Every Kyne construct must have a well-defined, ideally one-to-one or one-to-few, mapping to a Rust pattern that a Rust developer would recognize as idiomatic. If a Kyne feature cannot be explained by pointing at the specific Rust it produces, the feature is not ready for inclusion.

Transparency also has a pedagogical dimension. Because Kyne compiles to Rust rather than to bytecode or an opaque IR, a Kyne developer who wants to grow into a Rust and Soroban SDK expert has a built-in learning path: read the code your own contracts generate. This is a deliberate on-ramp from Kyne into the broader Rust and Stellar ecosystem, not a wall between them.

## Readability First

Readability first means code is optimized for the person reading it — often a future maintainer, an auditor, or the original author six months later — over the person originally writing it. Write-time convenience is a real value, but it is subordinate to read-time clarity, because contract code is read far more often than it is written, and it is read under higher stakes than almost any other kind of software: audits, disputes, and incident postmortems all begin with someone reading the source.

This principle governs syntax decisions at a granular level. Kyne prefers explicit keywords over cryptic symbols where the symbol would save characters but cost comprehension. It prefers named, structured error types over ambiguous sentinel values. It prefers contract state to be declared in one visibly demarcated place rather than scattered implicitly across the file. None of this is about verbosity for its own sake — verbosity that doesn't aid comprehension is just noise, and Kyne rejects that just as strongly as it rejects opaque terseness. The standard is always: does this help the next reader build an accurate mental model faster?

Readability first also means Kyne treats formatting as a solved, non-negotiable problem. A single canonical formatter, applied by default, removes an entire category of bikeshedding and ensures that every Kyne codebase in the wild looks like it was written by the same disciplined team, which in turn makes cross-project auditing and onboarding faster industry-wide.

## Explicit Over Implicit

Explicit over implicit means that anything with a meaningful effect on program behavior — state mutation, external calls, authorization checks, storage reads and writes, arithmetic that could fail — must be visible in the source at the point where it happens, not inferred from context, defaults, or hidden control flow. Implicit behavior is a common source of two different failure modes: developers who don't realize something dangerous is happening, and auditors who miss it during review because it isn't textually present in the function they're reading.

This principle rules out several patterns that are common and beloved in other languages: implicit type coercion, silently inherited behavior from a parent construct, ambient global mutable state, and side effects hidden inside seemingly pure-looking expressions. It also shapes Kyne's stance on defaults — a default is acceptable only when the implicit behavior it provides is the *safe* behavior, and even then, the generated Rust must make that default's effect visible to a reader who inspects the output.

Explicitness is not absolute — Kyne is not interested in forcing developers to spell out things that carry no real ambiguity, such as requiring type annotations on every local variable when inference is unambiguous and safe. The line is drawn specifically at *effects*, not at *syntax*. Type inference is a convenience with no security cost. A silently retried external call is a convenience with a real one. Kyne accepts the former freely and rejects the latter categorically.

## Composition Over Inheritance

Composition over inheritance means Kyne contracts and libraries are built by assembling small, well-defined pieces — traits/interfaces, modules, and explicit delegation — rather than by extending a base contract and inheriting its behavior, storage layout, and mutable state. Kyne has no `extends` keyword and no concept of a contract subclassing another contract.

This principle exists because inheritance in a smart-contract context is unusually dangerous. Inherited storage layouts create upgrade and audit hazards; inherited method resolution order creates ambiguity about which code path actually executes; and diamond-shaped inheritance hierarchies are a well-documented source of subtle bugs in other smart-contract ecosystems, where the specific set of parent contracts and their linearization order changes a function's real behavior in ways that are hard to see from the leaf contract alone. Auditors need to be able to read one contract's logic in one place; inheritance actively works against that.

Composition achieves the genuine reuse benefits developers want from inheritance — shared logic, shared interfaces, pluggable behavior — without the hidden-control-flow cost. A Kyne contract can implement multiple small interfaces, delegate to library functions, and hold composed sub-structures, but the call graph for any given entry point remains flat and traceable directly from the function that defines it. There is exactly one place to look to understand what a function does: the function itself and the things it explicitly calls.

## Small Language Philosophy

The small language philosophy holds that Kyne's grammar and feature set should be as small as they can be while still fully serving the contract-oriented domain — and that every addition to that surface area is a permanent cost, not a one-time cost. A small language is faster to learn, faster to formally reason about, faster to audit for compiler bugs, and far less likely to contain a corner-case interaction between two rarely-used features that nobody anticipated.

This is why [FOUNDATION.md](./FOUNDATION.md)'s design principle — that every new feature must improve readability, security, boilerplate reduction, diagnostics, onboarding, or maintainability, or it does not get added — is treated as a hard gate rather than a guideline. "It would be convenient" and "other languages have it" are explicitly insufficient justifications on their own. The bar is not "can we justify adding this," it is "can we justify the fact that this will exist in the language forever," because in practice, mainstream languages almost never remove a feature once shipped.

The small language philosophy is also why Kyne deliberately excludes entire categories of feature — macros, reflection, runtime metaprogramming, operator overloading without clear rules, and inheritance — rather than including a restricted version of each. A restricted macro system is still a macro system: it still requires a second mental model for "code that writes code," and it still produces output that has to be transparently mapped back through that model. Kyne prefers zero of a dangerous feature to a "safe" fraction of it, because the "safe" fraction has a way of growing over time under feature-request pressure.

## Compiler as a Teacher

Compiler as a teacher means the Kyne compiler is a design partner in the developer's learning process, not merely a gatekeeper that emits pass/fail verdicts. Every diagnostic the compiler produces should explain not just *that* something is wrong, but *why* it's wrong in this specific domain, and — wherever possible — *what to do instead*, ideally referencing the actual security or correctness concern the rule protects against.

This principle is a direct consequence of Kyne's target audience, described in [FOUNDATION.md](./FOUNDATION.md): developers who are often new to blockchain development, coming from TypeScript, Go, or backend contexts, without a Rust or cryptography background. For this audience, a diagnostic that reads `error[E0382]: borrow of moved value` is a wall; a diagnostic that reads "this value was already consumed on line 12, and Soroban storage values can only be moved once — did you mean to `.clone()` it, or restructure the function to read it once?" is a lesson. Compiler errors are, for many developers, the single most-read piece of documentation the language produces, and Kyne treats them with that level of editorial care.

This also means diagnostic quality is a first-class item in the language's roadmap and review process, not an afterthought bolted on after the type checker is "done." A KIP that introduces a new class of compile error is expected to also specify the message, the explanation, and the suggested fix that will accompany it, in the same way an API proposal is expected to specify its types.

## Stable Evolution

Stable evolution means that once a Kyne feature ships, changing or removing it is treated as a serious event requiring public justification, a migration path, and — for genuinely breaking changes — a major version boundary, because deployed contracts are frequently immutable and audits are expensive, one-time investments that a language should not casually invalidate. A developer who wrote and audited a contract against Kyne 1.2 should have strong confidence that Kyne 1.9 has not quietly changed what that contract means.

This principle does not mean Kyne freezes early or refuses to fix real mistakes. Phase 0 and the early public proposal period are precisely the time to get the hard-to-change decisions — grammar shape, core type system, the contract model — right, because the cost of changing them rises sharply after real contracts depend on them. Stability is a promise Kyne makes increasingly strongly over time, not a constraint that applies with equal force on day one and year five.

Stable evolution also shapes how the standard library grows. New APIs are additive by default; existing APIs are not repurposed to mean something new; and any deprecation carries a visible, compiler-enforced warning period before removal. The goal is that a contract author should never need to fear that adopting the latest Kyne toolchain to get a bugfix will silently change the semantics of code they already shipped.

---

# Language Personality

If Kyne were a person, they would be the calm, unflappable senior engineer on the team who has personally seen a production incident caused by a missed edge case, and who has therefore become quietly, permanently allergic to ambiguity — not out of anxiety, but out of respect for the people who will be paged at 3 a.m. if something goes wrong. They are not arrogant about this. They do not lecture. They simply default to asking, "what happens if this fails halfway through?" before anyone else in the room thinks to.

**Tone.** Kyne's voice is direct, warm, and unpretentious. It does not use hype language — no "blazingly fast," no "revolutionary," no exclamation points doing the work that evidence should be doing. It respects the reader's time and intelligence. Where confidence is warranted, Kyne states things plainly ("this cannot overflow"); where nuance exists, Kyne shows the nuance rather than hiding it behind marketing polish.

**Documentation style.** Kyne's documentation reads like a well-run onboarding session from a senior engineer who has done it many times and has stopped assuming things are obvious. Every non-trivial claim is backed by a runnable example. Every warning explains the failure mode it prevents, not just the rule. Reference material is exhaustive and dry by design; guides and tutorials are warm and example-driven by design — the two are kept in separate, clearly labeled channels, because conflating "how it works" with "what it's for" is one of the most common documentation failures in open source.

**Compiler messages.** Kyne's compiler talks the way a good mentor talks during code review: it names the specific problem, it explains the specific risk in domain terms ("this storage key can be overwritten by any caller"), and it suggests a specific fix, usually with a code snippet. It never says merely "invalid syntax" when it can say what token was expected and why. It never shames the developer for the mistake — every error message is written as if the person reading it is competent but simply hasn't encountered this particular Soroban-specific gotcha yet, because in most cases, that's exactly the situation.

**CLI style.** The `kyne` CLI favors a small number of memorable, verb-first commands — `kyne build`, `kyne test`, `kyne check`, `kyne fmt`, `kyne deploy` — over a sprawling set of flags and subcommand trees. Output is quiet on success and precise on failure. The CLI assumes the developer wants to get back to writing code as quickly as possible, not to spend time learning a command-line grammar as complex as the language itself.

**API philosophy.** Kyne's standard library and generated APIs favor a small number of well-named, hard-to-misuse functions over a large surface area of overlapping convenience methods. Function and type names describe what they do in plain domain language rather than abbreviating for brevity. Where an API has a footgun — an operation that behaves differently depending on contract state, for instance — the name of the function itself carries that warning rather than relegating it to a doc comment nobody reads on the happy path.

---

# Design Goals

Kyne's design process constantly trades one desirable property against another. Rather than pretend these trade-offs don't exist, this section ranks them explicitly, so that when two goals conflict during a KIP review, there is a documented tiebreaker instead of an ad hoc argument.

| Rank | Goal | 
|------|------|
| 1 | Safety |
| 2 | Readability |
| 3 | Simplicity |
| 4 | Developer Experience |
| 5 | Learnability |
| 6 | Performance |
| 7 | Flexibility |
| 8 | Metaprogramming |

**1. Safety.** Safety ranks first without exception because a smart contract failure is frequently a financial and reputational failure with no rollback. No other goal on this list is permitted to compromise safety. When a proposal improves ergonomics but weakens a safety guarantee, the proposal is rejected or reworked, full stop.

**2. Readability.** Readability ranks second because it is the property that makes safety *auditable*. A safe language whose contracts are unreadable is only safe until the first bug an automated check doesn't catch — human review is still the deepest line of defense, and human review requires readable code. Readability is ranked above simplicity because a slightly more complex construct that produces dramatically clearer code is worth its complexity cost; the reverse trade is not automatically true.

**3. Simplicity.** Simplicity ranks third because a small, orthogonal feature set is what keeps the language auditable at the compiler level and learnable at the developer level over the long run. Simplicity is placed above developer experience specifically because short-term convenience features are the most common source of long-term complexity debt — a simple language that asks a little more of the developer today is choosing a better trade than a convenient language that owes a complexity bill tomorrow.

**4. Developer Experience.** Developer experience — fast builds, good error messages, a pleasant CLI, minimal ceremony — ranks fourth because it is what determines whether people actually adopt and stay with Kyne, but it is explicitly downstream of the first three: a delightful language that is unsafe, unreadable, or bloated is not actually a good developer experience in the medium term, just a good demo.

**5. Learnability.** Learnability ranks fifth because it directly serves the mission of lowering the barrier to Soroban development, but it is a consequence of the goals above it more than an independent target: a language that is safe, readable, and simple is already most of the way to being learnable. Kyne invests directly in learnability primarily through documentation and compiler diagnostics rather than through language features that would compromise the higher-ranked goals.

**6. Performance.** Performance ranks sixth — present, but not primary. Because Kyne compiles to Rust and ultimately to WASM on Soroban, most of the raw execution performance ceiling is inherited from that pipeline rather than determined by Kyne's own design. Kyne's job is to not throw performance away carelessly (for example, by forcing unnecessary allocations or clones as a side effect of a convenience feature), not to actively chase performance at the cost of the goals ranked above it.

**7. Flexibility.** Flexibility — the ability to express many different patterns and architectures — ranks seventh deliberately. Kyne is a domain-specific tool by design, and excess flexibility is frequently in direct tension with simplicity and safety, since more expressive power generally means more ways to misuse that power. Kyne would rather do the contract-oriented domain extremely well than be a flexible general-purpose language that does many domains adequately.

**8. Metaprogramming.** Metaprogramming ranks last, on the boundary of being a non-goal outright. Compile-time code generation, reflection, and macro-like mechanisms are the single most common source of the exact problems Kyne is designed to avoid: hidden control flow, unreadable generated output, and compiler behavior that is difficult to reason about locally. Kyne does not rule out a narrow, fully transparent form of compile-time generics or templates in the future if one can be designed without violating transparent abstraction — but it will never be prioritized over the goals above it, and it will never take the form of an open-ended macro system.

---

# Non-Goals

Stating what Kyne refuses to become is as important as stating what it aims to be. A non-goal is not a feature Kyne hasn't gotten to yet — it is a feature Kyne has deliberately decided against, and a future KIP proposing it should be expected to argue against this document directly, not simply request the feature in isolation.

**Kyne will not become a general-purpose programming language.** It will never target web servers, CLIs unrelated to its own tooling, mobile apps, or desktop software as first-class use cases. Every feature added to Kyne must serve contract-oriented Soroban development specifically.

**Kyne will not support multi-chain compilation.** Kyne does not abstract over "smart contract platforms" as a category. It has one compilation target — Soroban-compatible Rust — and broadening that target to other chains or VMs would force the language to generalize away from the Soroban-specific guarantees that justify its existence.

**Kyne will not adopt macros, reflection, or runtime metaprogramming.** These mechanisms are rejected categorically, not provisionally, because they are structurally in tension with transparent abstraction and compiler-as-teacher — both because they make generated behavior harder to predict from source, and because they make compiler diagnostics dramatically harder to make good.

**Kyne will not support class-based inheritance.** Composition, interfaces, and explicit delegation cover the legitimate reuse cases; inheritance's remaining benefits do not outweigh the audit and comprehension costs described under [Composition Over Inheritance](#composition-over-inheritance).

**Kyne will not expose an `unsafe` escape hatch as ordinary API surface.** Where the underlying Rust or Soroban layer requires something outside Kyne's safe subset, that boundary will be made maximally visible and rare — not offered as a routine convenience keyword a developer reaches for under time pressure.

**Kyne will not chase syntactic novelty for its own sake.** Kyne does not invent new symbols or unconventional grammar purely to feel distinctive. Where a widely understood convention from TypeScript, Go, or Rust already communicates an idea clearly, Kyne uses it, reserving genuine novelty for the handful of places where the contract-oriented domain actually demands something the mainstream languages don't have.

**Kyne will not prioritize backward compatibility with Rust idioms that conflict with its own principles.** Kyne compiles to Rust, but it is not obligated to expose every Rust pattern or convention if that pattern would violate explicitness, readability, or safety. Kyne borrows from Rust selectively, not automatically.

**Kyne will not grow through unreviewed accretion.** No feature enters the language, standard library, or tooling without going through the KIP process described below, regardless of how small it seems or who proposes it.

---

# The Ten Commandments of Kyne

These are the immutable design rules of the language. A KIP that violates one of these is not merely disfavored — it is out of scope for the KIP process entirely, and would instead require a public, extraordinary re-founding discussion at the governance level, not a routine proposal.

1. **Security is never optional.** No feature, flag, or mode may disable a security guarantee for the sake of convenience or performance.
2. **Generated Rust must always remain readable.** If the compiler's output cannot be understood by an experienced Rust developer without special tooling, the compiler is wrong, not the reader.
3. **A project produces exactly one deployable contract.** Libraries compose into contracts; they are never independently deployable.
4. **Explicit beats implicit for anything with an effect.** State mutation, external calls, and authorization must be visible in source, never inferred.
5. **There is one obvious way to do a common thing.** Kyne does not offer multiple competing idioms for the same everyday task.
6. **No macros. No reflection. No runtime magic.** Code that writes code, code that inspects itself, and behavior that cannot be traced statically are all permanently out of scope.
7. **Composition replaces inheritance.** Contracts and libraries are assembled from small, explicit parts — never extended from a mutable base.
8. **The compiler teaches; it does not just reject.** Every diagnostic explains the risk and suggests the fix, in domain language a newcomer can use.
9. **Breaking changes are rare, public, and justified.** Deployed contracts and completed audits deserve a language that does not move the ground beneath them without warning.
10. **Kyne strengthens Soroban; it does not compete with it.** Kyne has no ambition to become a blockchain, a VM, or a platform of its own — only the best possible way to write for the one that already exists.

---

# Language Evolution

## KIPs — Kyne Improvement Proposals

Kyne evolves through **KIPs (Kyne Improvement Proposals)**, the formal mechanism by which any change to the language grammar, type system, standard library, compiler diagnostics, or tooling is proposed, debated, and either accepted or rejected in public. KIPs play the role that RFCs play in other language ecosystems, but the name is deliberate: a KIP is expected to read as a concrete *improvement* argument grounded in Kyne's stated principles, not merely a request or a wishlist item. Where earlier foundational drafts referred to this process informally as a KEP (Kyne Enhancement Proposal), KIP is the formalized, canonical term going forward, and all future governance documents should use it.

### Proposal Lifecycle

A KIP moves through five stages, each with a clear exit condition:

**Draft.** The proposer writes the KIP as a structured document: the problem being solved, the specific principle(s) from this document it serves, the proposed syntax or API, the generated Rust it would produce (if applicable), alternatives considered, and — critically — an explicit section on what security or correctness risk it might introduce and how that risk is mitigated. A KIP without this last section is not ready for submission.

**Discussion.** The KIP is opened for public comment. Discussion is scoped to the proposal's merits against Kyne's principles, not to unrelated feature requests. The discussion period has a minimum duration to ensure the community — not just the core maintainers — has had a real opportunity to weigh in, particularly from developers who would be affected by the change in production.

**Decision.** Core maintainers evaluate the KIP explicitly against the [Core Principles](#core-principles), the [Design Goals](#design-goals) ranking, and the [Ten Commandments](#the-ten-commandments-of-kyne). A KIP is accepted, rejected, or returned to Draft with specific requested changes. Rejections are recorded publicly with reasoning, not silently closed, so that the same proposal is not re-litigated from scratch by someone who didn't see why it failed the first time.

**Final.** Accepted KIPs are merged into the specification and scheduled for implementation. A KIP that changes existing behavior in a breaking way is tagged for the next major version boundary and paired with a migration guide before it can ship, per [Stable Evolution](#stable-evolution).

**Withdrawn.** A proposer may withdraw a KIP at any stage. Withdrawn KIPs remain in the public record as part of the language's design history — a rejected or withdrawn idea is valuable prior art for whoever proposes something adjacent later.

### Acceptance Criteria

A KIP can only be accepted if it satisfies all of the following:

- It serves at least one Core Principle and violates none.
- It does not contradict any of the Ten Commandments.
- It does not regress the Design Goals ranking — in particular, it cannot trade safety or readability for a lower-ranked goal.
- It includes a concrete explanation of the generated Rust output, where applicable, sufficient to evaluate transparency.
- It includes an honest accounting of new complexity introduced, per the Small Language Philosophy — "it would be nice to have" is not sufficient justification on its own.

### Governance Philosophy

During Phase 0 and the early public life of the language, KIP decisions are made by the founding maintainers, who bear direct responsibility for protecting the principles in this document against well-intentioned but principle-eroding feature pressure. This is a deliberate, temporary concentration of authority, not a permanent one: as the contributor base and production usage grow, governance is expected to evolve toward a broader core team model with documented voting or consensus rules, the same way most successful open language projects have.

What does not change as governance evolves is the standard of evaluation. A KIP is judged against this document, not against the preferences of whoever happens to be reviewing it that week. Technical excellence and fidelity to Kyne's stated principles take precedence over popularity, contributor seniority, or how much existing code a feature would need to migrate. This document is the constitution; governance is the judiciary that interprets it, not a body empowered to rewrite it casually.

---

# Future Vision

Kyne's long-term shape is a complete development ecosystem, not merely a compiler. Each of the following pieces exists to serve the same north star articulated in [FOUNDATION.md](./FOUNDATION.md): does this make writing secure Soroban smart contracts simpler without sacrificing clarity or correctness?

**Compiler.** The core of the project: parses Kyne source, performs static analysis and security checks, and emits readable, idiomatic Soroban-compatible Rust. The compiler is the enforcement mechanism for every principle in this document — it is where "security before convenience" and "explicit over implicit" become executable rules rather than aspirations.

**CLI.** The single entry point developers use day to day — `kyne build`, `kyne test`, `kyne check`, `kyne fmt`, `kyne deploy` — unifying what would otherwise be a fragmented set of separate tools into one coherent, memorable interface, consistent with the CLI style described under [Language Personality](#language-personality).

**Formatter.** A single canonical code style, applied automatically, removing formatting debates from code review entirely and ensuring any two Kyne codebases in the wild are immediately legible to each other's contributors.

**Language Server.** Real-time diagnostics, autocomplete, go-to-definition, and inline documentation inside the developer's editor of choice, bringing the compiler-as-teacher philosophy into the moment of writing code rather than only the moment of compiling it.

**Playground.** A zero-install, browser-based environment where a newcomer can write, compile, and inspect the generated Rust for a Kyne contract within minutes of hearing about the language — the fastest possible expression of the "first contract within an hour" success metric.

**Documentation Generator.** Tooling that produces reference documentation directly from contract and library source, keeping documentation and code from drifting apart, and reinforcing the principle that documentation is a language feature, not an afterthought.

**Testing Framework.** First-class, built-in support for writing and running contract tests — including scenario-style tests that exercise cross-contract calls and authorization edge cases — because a language whose primary claim is security must make verifying that security effortless, not something bolted on via a third-party library.

**Security Analyzer.** A static analysis layer beyond the base compiler's checks, purpose-built to catch smart-contract-specific vulnerability patterns before deployment — the concrete, tooling-level expression of "security by construction" as a promise developers can actually verify before they ship.

Together, these components form the ecosystem diagrammed in [FOUNDATION.md](./FOUNDATION.md):

```text
Kyne
├── Language
├── Compiler
├── CLI
├── Formatter
├── Language Server
├── Playground
├── Documentation Generator
├── Testing Framework
├── Static Security Analyzer
├── Standard Library
└── Community Ecosystem
```

No component on this list is optional in the long run, and no component is permitted to violate the principles established here in the name of shipping faster. A future maintainer facing pressure to cut a corner on any of these should return to this document first — it exists precisely so that pressure has somewhere firm to break against.

---

# Closing

This document is a foundational artifact of the Kyne project. It should be revisited when it is wrong, extended when the language grows into situations it didn't anticipate, and defended when a proposal would quietly erode what it protects. It is not expected to be perfect on day one — it is expected to be the thing every future decision has to answer to.
