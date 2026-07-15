# GOVERNANCE.md

# The Kyne Governance & Language Evolution Specification

**Phase:** 0 — Foundations
**Milestone:** 9 — Governance & Language Evolution Specification
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative and constitutional — this document is binding on every future maintainer, contributor, and language steward, and governs the process by which every other constitutional document, including itself, may change.

This document defines **how** Kyne evolves — not **what** Kyne is. It exists to protect [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md), and [TOOLCHAIN.md](./TOOLCHAIN.md) — collectively, alongside this document itself, Kyne's **constitutional documents**, per [§25](#25-constitutional-documents) — from uncoordinated, unreviewed, or silently inconsistent change. This document MUST NOT redefine language, runtime, compiler, memory, toolchain, or standard library behavior; where it references those documents, it defers to them entirely.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings.

---

# 1. Governance Philosophy

**Purpose.** Governance exists to answer one question consistently, for as long as Kyne exists: when someone wants to change something about this language, who decides, and how. Without an answer fixed in advance, that question is answered ad hoc, by whoever is loudest or most present in a given moment — which is precisely how coherent, well-designed languages accumulate incoherent, poorly-integrated features over time. Governance is the mechanism that keeps [LANGUAGE_PRINCIPLES.md's Ten Commandments](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne) enforceable a decade from now, not merely aspirational today.

**Mission.** To let Kyne evolve — because a language that cannot evolve at all will eventually be abandoned for one that can — while guaranteeing that every evolution is deliberate, evidenced, documented, and consistent with everything Kyne has already promised its users. Evolution and stability are not opposing goals this document trades off against each other; stability is the *precondition* for evolution being worth doing carefully.

**Values.** Governance inherits, without modification, [Kyne's three immutable pillars](./LANGUAGE_PRINCIPLES.md#foundation): Transparent Abstraction means every governance decision is understandable and every accepted change ships with its reasoning, never decided behind closed doors. Security by Construction means security concerns receive priority over convenience at the governance level exactly as they do at the language level, with an emergency process ([§11](#11-security-governance)) built into the constitution itself rather than improvised when first needed. Contract-Oriented Design means governance exists to preserve Kyne's long-term integrity as a language for writing smart contracts specifically — short-term convenience for a contributor or a vocal user base must never compromise the long-term stability every deployed, immutable Kyne contract depends on.

**Why governance exists.** [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) establishes that a deployed Kyne contract's code is immutable and frequently holds real value for its entire on-chain lifetime. Every promise this document's seven constitutional predecessors make to a contract author — determinism, atomicity, explicit authorization, readable generated Rust — is only real if there is a durable mechanism preventing those promises from being casually revised later. Governance is that mechanism.

**Relationship to the language.** Governance does not write language, runtime, compiler, memory, toolchain, or standard library rules — the eight documents this one exists to protect already do that, exhaustively, within their own scope. Governance writes the *process* by which those rules may ever change.

**Relationship to contributors.** Governance defines a legible path from first-time contributor to trusted steward ([§12](#12-contributor-journey)), so that influence over Kyne's direction is a function of demonstrated judgment and sustained contribution, never of employer, seniority elsewhere, or volume of opinion alone.

**Relationship to constitutional documents.** Every constitutional document, including this one, is protected by the same elevated process this document defines in [§25](#25-constitutional-documents) — governance is not exempt from itself.

---

# 2. Project Principles

**Stability over novelty.** A change is not justified merely by being a good idea — it is justified by evidence that the problem it solves is real, common, and not already solvable within Kyne's existing design. This exists because every mature language accumulates far more good ideas than it can accept without becoming an incoherent accumulation of good ideas, per [Small Language Philosophy](./LANGUAGE_PRINCIPLES.md#small-language-philosophy) — the discipline required to reject a good idea for lack of evidence is precisely what keeps Kyne small on purpose rather than small by accident.

**Clarity over cleverness.** A governance decision — like a language decision, per [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first) — is judged on whether a future maintainer with no memory of the original discussion can understand why it was made from the record alone. A clever resolution that only the people in the room at the time understand is not an acceptable resolution, regardless of how elegant it seemed.

**Security before convenience.** Restated at the governance level from [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#security-before-convenience): a governance process that is faster because it skips security review is not a more efficient process, it is a defective one. [§11](#11-security-governance)'s emergency path exists precisely so that "convenience" is never the reason a security concern is under-examined — the fast path is fast because it is *pre-designed* for security specifically, not because it cuts corners.

**Simplicity scales.** A governance structure with six roles ([§3](#3-governance-structure)), one primary decision mechanism ([§4](#4-decision-making)), and one proposal process ([§6](#6-kip-lifecycle)) remains legible whether Kyne has five contributors or five hundred. A governance structure that grows more elaborate in direct proportion to project size will eventually become a larger burden to navigate than the technical work it exists to coordinate — Kyne's governance is designed to stay simple as it scales, not to grow in step with the community it serves.

**Contributors serve developers.** Every role in [§3](#3-governance-structure), every KIP in [§6](#6-kip-lifecycle), and every ADR in [§19](#19-architecture-decision-records-adr) ultimately exists to serve the Kyne contract author who will never read any of them — per [FOUNDATION.md's North Star](./FOUNDATION.md#the-north-star), the standing question behind every governance decision is the same one behind every language decision: does this make writing secure Soroban smart contracts simpler without sacrificing clarity or correctness. A governance process that serves its own participants' convenience over that question has lost sight of its purpose.

---

# 3. Governance Structure

| Role | Responsibilities | Authority | Promotion | Limitations | Succession |
|---|---|---|---|---|---|
| **Founder** | Sets Kyne's overall technical direction; exercises final tie-breaking authority per [§4](#4-decision-making); is the primary steward of the constitutional documents. | Final authority when consensus cannot be reached, per the [Governance Model](#governance-model); may veto a KIP that conflicts with [FOUNDATION.md](./FOUNDATION.md)'s mission even where consensus otherwise exists, though SHOULD do so only with a written rationale added to the KIP's record. | Not a promoted role — see [§15](#15-leadership) for how this role is established and, eventually, transitioned. | Bound by every constitutional document exactly as any other role is; tie-breaking authority is a last resort, per [§4](#4-decision-making), not a substitute for discussion. | [§15](#15-leadership) defines succession explicitly. |
| **Core Maintainer** | Reviews and votes on KIPs within their area of expertise; mentors Maintainers and Reviewers; participates in [§14](#14-voting)'s Core Maintainer promotion votes. | Approval authority sufficient, alongside other Core Maintainers' consensus, to accept a KIP without Founder involvement, per [§4](#4-decision-making). | Promoted from Maintainer, per [§12](#12-contributor-journey), via a Core Maintainer vote. | Authority is scoped to sound technical judgment within [§2](#2-project-principles)'s constraints, not personal preference — a Core Maintainer's approval does not override a KIP's own acceptance criteria, per [§8](#8-kip-acceptance-criteria). | Inactive-maintainer handling per [§15](#15-leadership). |
| **Maintainer** | Owns a specific area (a `kyne_*` crate, a toolchain component, a standard library module) per [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s own boundary discipline; reviews and merges ordinary pull requests within that area. | Merge authority within their owned area for changes that are not KIP-requiring, per [§26](#26-governance-scope). | Promoted from Reviewer, per [§12](#12-contributor-journey). | Cannot unilaterally accept a KIP-requiring change, even within their own owned area. | As above. |
| **Reviewer** | Reviews pull requests for correctness, style, and consistency per [§13](#13-code-review-philosophy); may approve routine changes but relies on a Maintainer for merge authority. | Review authority; no merge authority. | Promoted from Contributor. | Approval is advisory until a Maintainer's merge; a Reviewer's approval does not itself authorize a change to ship. | N/A — a Reviewer who becomes inactive simply stops reviewing; no formal process is required to revoke a non-authoritative role. |
| **Contributor** | Submits pull requests, participates in KIP Discussion, reports issues. | None beyond ordinary participation. | Anyone submitting a merged, non-trivial contribution is, by that fact, a Contributor — this is a recognition, not a granted status requiring a decision. | None beyond ordinary community expectations, which are outside this document's scope per [Non-Goals](#non-goals). | Not applicable. |
| **Community Member** | Uses Kyne, asks questions, reports bugs, participates in public discussion. | None. | The default status of anyone engaging with the project at all. | None. | Not applicable. |

**Why six roles, not more.** Each role in this table corresponds to a distinct, meaningful increment of trust and authority — Community Member (no commitment assumed), Contributor (has contributed), Reviewer (trusted to evaluate others' work), Maintainer (trusted to merge within a bounded area), Core Maintainer (trusted to accept language-level change), Founder (trusted to break a deadlock). A seventh role would need to describe a genuinely distinct trust increment this table does not already capture; per [Project Principle: Simplicity Scales](#2-project-principles), the burden of proof for adding one is on the proposal, not on this document to have anticipated it.

---

# 4. Decision Making

**Consensus is the default and preferred mechanism.** For an ordinary KIP, per [§6](#6-kip-lifecycle), acceptance is reached when every participating Core Maintainer with a stated position in Discussion either supports the proposal or has had their objections addressed — silence from a Core Maintainer who was given reasonable opportunity to object is not itself an objection, and consensus does not require unanimity of enthusiasm, only the absence of an unresolved, substantive objection from someone with the standing to raise one.

**Evidence outweighs opinion.** A Core Maintainer's objection during Discussion MUST be substantive — grounded in a conflict with a Core Principle ([§2](#2-project-principles)), a Constitutional Document, or a concrete, demonstrable technical concern — not a bare preference. [§8](#8-kip-acceptance-criteria)'s required KIP content exists specifically so that Discussion has evidence to evaluate rather than competing assertions to adjudicate.

**Discussion.** Every KIP MUST have a Discussion period of fixed minimum duration, per [§6](#6-kip-lifecycle), during which any Core Maintainer, Maintainer, Reviewer, Contributor, or Community Member MAY comment; only Core Maintainers' objections are binding on the consensus calculation, per [Governance Structure](#3-governance-structure), but every comment becomes part of the KIP's permanent, transparent record, per [Transparent Abstraction](#1-governance-philosophy).

**Founder tie-breaking authority.** Where genuine disagreement among Core Maintainers persists past a reasonable, defined discussion period with no consensus emerging, the Founder MAY exercise final tie-breaking authority to accept, reject, or return the KIP to Draft with specific requested changes. This authority is a **last resort**, exercised only after discussion has genuinely been given the chance to resolve the disagreement on its own — it is not a shortcut available to bypass Discussion, and a Founder decision under this authority MUST include a written rationale added to the KIP's permanent record, per [Transparent Abstraction](#1-governance-philosophy).

**Emergency decisions.** [§11](#11-security-governance) defines a separate, purpose-built expedited path for security-critical fixes; it is not an instance of ordinary tie-breaking, and this section's discussion-first requirement does not apply to it.

**Deadlock resolution.** A deadlock — genuine, sustained disagreement with no consensus forming — is resolved exclusively through Founder tie-breaking authority as described above; there is no separate deadlock-specific process, since inventing one would create two different escalation paths for what is, functionally, the same situation.

**Escalation.** A Contributor or Maintainer who believes a KIP was wrongly rejected, or believes a legitimate proposal is stalled without genuine engagement, MAY request Founder review directly, citing the specific unaddressed evidence or reasoning. This is distinct from re-submitting the same KIP unchanged — escalation is for process failures, not for disagreement with an outcome reached through the process working correctly.

---

# 5. Language Evolution

**Every architectural change to Kyne — language, runtime, compiler, memory, toolchain, standard library, or governance itself — requires a KIP.** There is no other path by which such a change may enter Kyne, regardless of who proposes it or how small it initially appears.

**Language evolution is specification-first.** A KIP's primary output is an accepted amendment to a constitutional document — [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), or another, per [§7](#7-kip-categories) — not a merged pull request against the compiler. Implementation follows an accepted specification; it never precedes or substitutes for one. [§23](#23-specification-first-development) expands this into its own dedicated chapter, since it is one of the most consequential rules this document establishes.

**Why this rule exists.** A change implemented first and specified afterward inverts the entire relationship this document exists to protect: instead of the constitutional documents constraining what the compiler may do, the compiler's accidental behavior becomes the de facto specification, and the written documents become a description of an implementation rather than a constraint on one. [COMPILER_ARCHITECTURE.md's own invariant](./COMPILER_ARCHITECTURE.md#the-compiler-never-generates-undefined-runtime-behavior) — that the compiler never accepts a construct whose behavior these documents do not already define — is only enforceable if "these documents" are always written before the behavior they describe exists at all.

---

# 6. KIP Lifecycle

```
Idea
    ↓
Draft
    ↓
Discussion
    ↓
Accepted
    ↓
Implemented
    ↓
Released
    ↓
Archived
```

This lifecycle **refines**, rather than replaces, the five-stage lifecycle already sketched in [LANGUAGE_PRINCIPLES.md's Language Evolution section](./LANGUAGE_PRINCIPLES.md#language-evolution) — that document was written before Kyne had a compiler, a toolchain, or a release process to distinguish "accepted" from "implemented" from "released" in a meaningful way. This document makes that distinction explicit now that all three exist. The correspondence is direct: LANGUAGE_PRINCIPLES.md's *Draft* splits here into **Idea** and **Draft**; its *Discussion* and *Decision* correspond to this lifecycle's **Discussion** and **Accepted**; its *Final* splits into **Implemented** and **Released**; and its *Withdrawn*, together with every KIP's eventual permanent resting state — accepted or not — is unified here as **Archived**.

**Idea.** An informal stage: a problem statement, posted for early, low-stakes community reaction, before the effort of writing a full Draft is invested. Entry criteria: none — any Contributor or Community Member may raise one. Exit criteria: the proposer chooses to write a Draft, or the idea is abandoned with no formal record required. Responsibilities: none beyond ordinary participation.

**Draft.** The proposer writes the full KIP document, satisfying every required content section in [§8](#8-kip-acceptance-criteria). Entry criteria: a written document meeting the required structure. Exit criteria: the proposer submits it for Discussion. Responsibilities: the proposer owns the document's completeness and accuracy; a Draft missing a required section per [§8](#8-kip-acceptance-criteria) MUST NOT proceed to Discussion.

**Discussion.** Open review per [§4](#4-decision-making). Entry criteria: a complete Draft. Exit criteria: consensus is reached (proceed to Accepted), a Core Maintainer or the Founder identifies unresolved issues (return to Draft), or the proposer withdraws (proceed to Archived). Responsibilities: Core Maintainers owe the proposal substantive engagement within a reasonable period, per [§4](#4-decision-making).

**Accepted.** The KIP's specification content is merged into the relevant constitutional document(s). Entry criteria: consensus or Founder tie-break, per [§4](#4-decision-making). Exit criteria: implementation begins. Responsibilities: the amended constitutional document is now binding, per [§25](#25-constitutional-documents), even before any compiler change ships — an Accepted KIP changes what conformance means immediately, per [§23](#23-specification-first-development).

**Implemented.** The compiler, runtime, or toolchain change realizing the Accepted specification is built and merged, per the relevant repository's own ordinary code review, informed by [§13](#13-code-review-philosophy). Entry criteria: an Accepted KIP. Exit criteria: the implementation passes the test strategy [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy) requires and merges. Responsibilities: implementers MUST conform to the Accepted specification exactly — an implementation detail not dictated by the KIP belongs in an ADR, per [§24](#24-kips-vs-adrs), never as a silent deviation from what was accepted.

**Released.** The implementation ships in a numbered Kyne release, per [§9](#9-release-policy). Entry criteria: Implemented. Exit criteria: the release is published. Responsibilities: from this point, the change is subject to [§10](#10-compatibility-policy)'s compatibility guarantees.

**Archived.** The KIP's document becomes permanent, immutable historical record — whether it was ultimately accepted and released, or rejected, or withdrawn. Entry criteria: any terminal outcome. Exit criteria: none — Archived is permanent. Responsibilities: an Archived KIP MUST NOT be edited after archival; a later change of mind is a new KIP, which SHOULD reference the earlier one, not a reopening of it.

---

# 7. KIP Categories

| Category | Amends |
|---|---|
| Language | [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) |
| Compiler | [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) |
| Runtime | [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) |
| Memory | [MEMORY_MODEL.md](./MEMORY_MODEL.md) |
| Toolchain | [TOOLCHAIN.md](./TOOLCHAIN.md) |
| Standard Library | [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) |
| Documentation | Informative (non-constitutional) documentation, per [§17](#17-documentation-governance) — notably, a Documentation-category KIP is the *exception* in this table in that it does not amend a constitutional document, and therefore follows [§17](#17-documentation-governance)'s lighter process rather than [§25](#25-constitutional-documents)'s elevated one. |
| Governance | This document, [GOVERNANCE.md](./GOVERNANCE.md). |
| Ecosystem | A future `ECOSYSTEM.md`, per [§18](#18-ecosystem-governance). |
| Package Manager | A future `PACKAGE_MANAGER.md`, per [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-future-package-manager). |

A KIP MAY span more than one category where a change genuinely affects more than one constitutional document (a language change frequently has runtime consequences, for example) — such a KIP MUST identify every document it amends and satisfies this document's process with respect to each.

---

# 8. KIP Acceptance Criteria

Every KIP MUST include the following sections before it may leave Draft:

| Required section | Purpose |
|---|---|
| Motivation | Why this problem is worth solving at all. |
| Problem statement | The specific, concrete problem, ideally with a real or realistic example. |
| Alternatives | Other approaches considered, and why they were not chosen — a KIP with no considered alternatives has not demonstrated the proposed approach is the right one, only that it is *an* approach. |
| Trade-offs | What the proposal gives up in exchange for what it gains — per [Design Goals](./LANGUAGE_PRINCIPLES.md#design-goals), every design choice trades against another, and a KIP that does not name its trade-off has not actually evaluated it against that ranking. |
| Backward compatibility | Whether, and how, the proposal affects already-shipped behavior, per [§10](#10-compatibility-policy). |
| Migration strategy | If the proposal is not purely additive, how an existing project adapts — required even for a proposal the author believes is unlikely to affect real projects, since that belief is exactly what Discussion exists to test. |
| Security analysis | What new attack surface, if any, the proposal introduces, and how it was considered — required for every KIP, not only ones that appear security-relevant on their face, since [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience) applies to the review process, not only to features that announce themselves as security features. |
| Performance impact | The expected effect on compile time, generated code size, or runtime resource cost, where applicable. |
| Benchmarks | Where the proposal makes a performance claim, evidence for it — a KIP MAY state "not applicable" here where no performance claim is made, but MUST NOT assert a performance benefit without supporting evidence. |

This required structure elaborates [LANGUAGE_PRINCIPLES.md's own acceptance criteria](./LANGUAGE_PRINCIPLES.md#acceptance-criteria) rather than replacing it: LANGUAGE_PRINCIPLES.md fixes *what must be true* for a KIP to be accepted — it serves a Core Principle, violates no Commandment, does not regress the Design Goals ranking; this chapter fixes *what the document must contain* so that Discussion has the material needed to evaluate those questions. A KIP satisfying this chapter's structure has not thereby earned acceptance — it has earned a properly informed Discussion.

**Review expectations.** A Core Maintainer reviewing a KIP MUST evaluate every required section present, not only the ones relevant to their own area of expertise — a Runtime-category KIP's Security Analysis section is everyone's concern, not only a security specialist's, precisely because Kyne has no separate security-specialist role in [§3](#3-governance-structure) distinct from the ordinary reviewing responsibility every Core Maintainer already carries.

---

# 9. Release Policy

**Major releases** correspond to a new edition, per [TOOLCHAIN.md §19](./TOOLCHAIN.md#19-version-management), or to a constitutional change [§25](#25-constitutional-documents) classifies as breaking. A major release is rare by design, per [§10](#10-compatibility-policy).

**Minor releases** ship additive, non-breaking Accepted-and-Implemented KIPs — new standard library functions, new diagnostics, new toolchain commands — within the current edition.

**Patch releases** ship compiler and toolchain defect fixes that do not change any constitutional document's specified behavior; a patch release MUST NOT alter what a conforming program means, only correct an implementation's deviation from what the specification already required.

**Support windows.** The current major version and the immediately preceding major version both receive patch releases; a major version older than that receives patches only for a security issue meeting [§11](#11-security-governance)'s emergency criteria. Within a major version, only the latest minor version is supported — patch fixes roll forward, they are not independently backported to every prior minor version.

**Release cadence.** Minor and patch releases follow a regular, predictable cadence, the specific interval for which is a toolchain-operational detail outside this document's scope, per [Governance Scope](#26-governance-scope); major releases have no fixed cadence at all, since a new edition or constitutional change occurs only when [§25](#25-constitutional-documents)'s elevated process actually produces one, never on a calendar-driven schedule.

**Emergency releases.** A release addressing a [§11](#11-security-governance) security finding MAY ship outside the regular cadence, at any time, following that section's expedited process.

**Compatibility expectations.** Restated from [§10](#10-compatibility-policy): a minor or patch release MUST NOT change the meaning of a program that compiled successfully under an earlier release of the same major version.

---

# 10. Compatibility Policy

**Backward compatibility is the default, not an aspiration.** A change that would alter the meaning of an already-valid Kyne program is a breaking change, full stop, regardless of how narrow the affected case is believed to be.

**Deprecation.** A construct or API MAY be marked deprecated — flagged with a compiler or standard library warning, per [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine) and [STANDARD_LIBRARY.md §20](./STANDARD_LIBRARY.md#20-stability-guarantees) — without being removed, and MUST remain functional for at least one full major version cycle after deprecation before removal is even eligible for KIP consideration.

**Edition system.** This document does not redefine [TOOLCHAIN.md §19](./TOOLCHAIN.md#19-version-management)'s edition mechanism; it states only that a new edition is itself a Language- or Runtime-category KIP subject to every rule in this document, including [§25](#25-constitutional-documents)'s elevated constitutional review where the edition changes a MUST-level rule.

**Migration tooling.** Where a KIP's migration strategy ([§8](#8-kip-acceptance-criteria)) identifies that existing projects require mechanical adaptation, the KIP SHOULD identify whether `kyne fix` ([TOOLCHAIN.md §26](./TOOLCHAIN.md#26-developer-assistance-commands)) can safely automate that adaptation, subject to that command's own "provably safe transformations only" boundary — a KIP MUST NOT assume automated migration is possible without that boundary being satisfied.

**Breaking change policy.** A breaking change requires: a Language-, Runtime-, Compiler-, Memory-, Toolchain-, or Standard-Library-category KIP; explicit identification as breaking in that KIP's Backward Compatibility section; a documented migration strategy; and association with a major release, per [§9](#9-release-policy). A KIP that is breaking but does not satisfy every one of these MUST NOT be accepted.

**Compatibility guarantees.** Once Released, per [§6](#6-kip-lifecycle), a KIP's guarantees hold for the remainder of its major version's support window, per [§9](#9-release-policy), without exception.

---

# 11. Security Governance

**Responsible disclosure.** A security issue in the language, compiler, runtime, or standard library MUST be reported privately rather than through ordinary public issue tracking, so that a fix can be prepared before the issue is public knowledge.

**Private reporting.** A dedicated, non-public reporting channel MUST exist and MUST be reachable by anyone, including a Community Member with no prior standing in [§3](#3-governance-structure) — a security reporter's credibility is judged on the report's substance, never on their role.

**Security advisories.** Once a fix is ready, per the Security Embargoes provision below, Kyne MUST publish a public advisory describing the issue, its severity, the affected versions, and the fix — satisfying [Transparent Abstraction](#1-governance-philosophy) even for security matters, once the embargo that necessarily delayed that transparency has served its purpose.

**Emergency patches.** A security fix MAY bypass the ordinary [§6](#6-kip-lifecycle) Discussion period entirely and MAY be Implemented and Released before a corresponding KIP completes its own process — this is the one case in which implementation is permitted to precede acceptance, a deliberate, narrow exception to [§23](#23-specification-first-development)'s specification-first rule, justified because the alternative (waiting for ordinary Discussion while a known vulnerability remains exploitable) is a worse outcome than the exception. A retroactive KIP documenting the fix and its rationale MUST still be filed and MUST reach Accepted status after the fact, ensuring the permanent record [§6](#6-kip-lifecycle) requires is never permanently skipped, only ever delayed.

**Security embargoes.** Between private report and public advisory, discussion of the issue MUST be restricted to those actively working on the fix — Core Maintainers by default, extended only as narrowly as the fix requires. The embargo period SHOULD be as short as responsible remediation allows and MUST NOT be extended for reasons unrelated to remediation readiness (for example, to align with an unrelated marketing or release timeline).

**Security review expectations.** Every KIP's required Security Analysis section, per [§8](#8-kip-acceptance-criteria), is the ordinary-track counterpart to this section's emergency track — most security consideration in Kyne's governance happens during ordinary KIP review, not during incident response, precisely so that incident response remains rare.

**Relationship to the Security Analyzer.** [COMPILER_ARCHITECTURE.md §11](./COMPILER_ARCHITECTURE.md#11-security-analyzer)'s Security Analyzer is a compiler feature that finds patterns in *contract source code*; this section governs vulnerabilities in *Kyne itself* — the compiler, runtime, or standard library. The two are not substitutes for one another: a Security Analyzer improvement is an ordinary Compiler-category KIP, while a defect in the Security Analyzer's own implementation that caused it to miss a real vulnerability class is itself eligible for this section's process if the miss is severe enough to constitute a security issue in Kyne's tooling.

---

# 12. Contributor Journey

```
Community Member
    ↓
Contributor
    ↓
Reviewer
    ↓
Maintainer
    ↓
Core Maintainer
```

**Promotion philosophy.** Every promotion in this chain is earned through demonstrated judgment in the role below it, observed over time by those already holding the role being granted — per [Meritocracy](#2-project-principles), there is no fixed tenure requirement, no application process, and no self-nomination pathway; a Community Member becomes a Contributor by contributing, a Contributor becomes a Reviewer by demonstrating reviewing judgment on their own and others' work, a Reviewer becomes a Maintainer by demonstrating ownership-worthy judgment within a bounded area, and a Maintainer becomes a Core Maintainer, per [§14](#14-voting), by demonstrating judgment at the level [§7](#7-kip-categories) requires: correctly weighing a proposal against the constitutional documents as a whole, not only a bounded area.

**Trust.** Each step up this chain grants a strictly larger scope of unilateral authority, per [§3](#3-governance-structure)'s table — the journey is, functionally, a journey of expanding trust, and every expansion is reversible: a role granted for demonstrated judgment can be reduced where that judgment is no longer being exercised, per [§15](#15-leadership)'s inactive-maintainer provisions, without that reduction being read as a punitive act.

**Responsibility.** Authority in this document is never granted without a corresponding responsibility — a Reviewer's authority to approve is paired with the responsibility to review thoroughly per [§13](#13-code-review-philosophy); a Core Maintainer's authority to accept a KIP is paired with the responsibility to have actually evaluated it against [§8](#8-kip-acceptance-criteria) in full, not merely skimmed it.

**Expectations.** No role in this chain requires a specific time commitment measured in hours — it requires sustained, reliable engagement of *some* cadence sufficient that the role continues to reflect real, current judgment about Kyne's present state, not judgment frozen at the moment of promotion.

---

# 13. Code Review Philosophy

Every review, at every level of [§3](#3-governance-structure)'s structure, evaluates a change against the following, in no particular priority beyond correctness always being non-negotiable:

**Correctness.** Does the change do what it claims, and does it do nothing it does not claim — the latter being at least as important as the former, since an unintended side effect is a defect even where the intended behavior is right.

**Documentation.** Does the change include the documentation [§17](#17-documentation-governance) and the relevant constitutional document require — an undocumented behavior is, per [COMPILER_ARCHITECTURE.md's own invariant](./COMPILER_ARCHITECTURE.md#the-compiler-never-generates-undefined-runtime-behavior), not eligible to exist in Kyne at all.

**Testing.** Does the change include tests at the appropriate level of [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy)'s strategy — a change without a test asserting its own correctness is a change the next contributor might silently break without anyone noticing.

**Performance.** Does the change meet the performance expectations its KIP, if any, committed to in [§8](#8-kip-acceptance-criteria)'s Performance Impact section.

**Security.** Does the change introduce a new attack surface not already accounted for in its KIP's Security Analysis section, or, for a non-KIP change, is it free of security implications entirely.

**Maintainability.** Does the change fit [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s existing boundaries, or does it quietly introduce a dependency the acyclic pipeline discipline forbids.

**Consistency.** Does the change match the naming, structure, and style conventions already established in the area it touches — per [STANDARD_LIBRARY.md §18](./STANDARD_LIBRARY.md#18-api-design-rules)'s own consistency rule, applied here to code review generally rather than only to API naming.

**Review standards.** A review approving a change MUST address every dimension above that is applicable — an approval that only addresses correctness while ignoring an obviously missing test is an incomplete review, and the reviewer, not only the author, bears responsibility for that gap.

---

# 14. Voting

Kyne's default decision mechanism is consensus, per [§4](#4-decision-making) — voting is reserved for the narrow set of matters where a discrete, countable outcome is genuinely needed rather than a negotiated resolution:

**Who votes.** Core Maintainer promotions and Founder succession matters ([§15](#15-leadership)) are decided by a vote of sitting Core Maintainers. No other governance matter in this document is decided by vote — an ordinary KIP is decided by consensus-or-tie-break, per [§4](#4-decision-making), never by a headcount.

**When votes occur.** Only for the two matters named above.

**Quorum.** A vote requires participation from a simple majority of sitting Core Maintainers to be valid; a vote falling short of quorum is void and MUST be re-opened rather than decided on a partial count.

**Tie breaking.** A tied vote is broken by the Founder, consistent with [§4](#4-decision-making)'s general tie-breaking authority — this is not a separate mechanism, only that authority's application to a vote specifically rather than to a KIP Discussion.

**Abstentions.** A Core Maintainer MAY abstain; an abstention counts toward quorum but not toward the vote's outcome in either direction.

**When votes are unnecessary.** Every other governance decision in this document — KIP acceptance, release timing within [§9](#9-release-policy)'s policy, code review outcomes — is explicitly not a voting matter, because a vote produces a winner and a loser on a question that consensus-seeking is better suited to actually resolving, per [§4](#4-decision-making)'s reasoning.

---

# 15. Leadership

**Founder responsibilities.** Set and protect Kyne's overall technical direction; hold final tie-breaking authority, per [§4](#4-decision-making); serve as the primary steward of the constitutional documents, per [§25](#25-constitutional-documents); and, per the [Governance Model](#governance-model), hold this authority for Kyne's formative years specifically, not as a permanent, unconditional grant.

**Core team.** The body of sitting Core Maintainers collectively carries the day-to-day weight of KIP review across every category in [§7](#7-kip-categories); the Founder is one participant in that body's Discussion, exercising tie-breaking authority only where the body's own consensus-seeking has been given a genuine chance and has not succeeded.

**Succession.** Should the Founder become permanently unavailable or choose to step down, sitting Core Maintainers MUST select a successor by vote, per [§14](#14-voting), within a bounded period — this document does not fix the exact period, leaving it to be set by the Core Maintainer body at the time, but requires that a vacancy in this role MUST NOT be permitted to persist indefinitely, since [§4](#4-decision-making)'s tie-breaking mechanism depends on the role being filled.

**Inactive maintainers.** A Core Maintainer, Maintainer, or Reviewer who has not exercised their role's responsibilities over a sustained period MAY have that role reduced by Core Maintainer vote — this is a routine maintenance action on the governance structure's own accuracy, not a disciplinary one, and SHOULD be communicated plainly as such.

**Transfer of responsibility.** Any owned area under [§3](#3-governance-structure)'s Maintainer role MAY be transferred to a new Maintainer by the outgoing Maintainer's own designation, subject to Core Maintainer confirmation that the incoming Maintainer has demonstrated the judgment [§12](#12-contributor-journey) requires.

**Long-term stewardship.** This chapter, together with [§21](#21-future-governance), is what allows this document's opening claim — that it remains authoritative for the lifetime of the Kyne project — to be more than aspirational: a governance document with no succession plan for its own most powerful role is not, in fact, built to outlast any single person holding that role.

---

# 16. Conflict Resolution

**Technical disagreements.** Resolved through [§4](#4-decision-making)'s consensus-then-tie-break mechanism — a technical disagreement is, definitionally, a KIP Discussion that has not yet reached consensus, and this document provides exactly one path through that situation, not a separate conflict-resolution track running alongside it.

**Community disagreements** not tied to a specific KIP or pull request (interpersonal conflict, disagreement about project direction expressed outside a formal proposal) are outside this document's scope — such matters belong to a project's community conduct process, per [Non-Goals](#non-goals), which this document explicitly does not define.

**Code ownership.** Disputed ownership of an area under [§3](#3-governance-structure)'s Maintainer role is resolved by Core Maintainer vote, per [§14](#14-voting), treated as equivalent in weight to a Maintainer-promotion decision.

**Appeals.** [§4](#4-decision-making)'s escalation provision is this document's sole appeals mechanism — a Contributor who believes a decision was reached incorrectly requests Founder review directly, citing specific unaddressed evidence.

**Mediation.** Where a technical disagreement has an interpersonal dimension neither [§4](#4-decision-making) nor a community conduct process cleanly resolves alone, the Founder MAY informally mediate, but this is a discretionary, not a required, intervention, and does not create a third resolution track beyond the two named above.

**Final authority.** In every case this section addresses, final authority traces back to either Core Maintainer consensus, a Core Maintainer vote, or Founder tie-break — this document deliberately provides no fourth mechanism, since a fourth mechanism would be, by definition, an escape hatch from the first three, undermining the predictability [Project Principles](#2-project-principles) requires.

**Philosophy.** Conflict resolution in Kyne is not a separate system layered on top of ordinary decision-making — it is what ordinary decision-making already is, applied to a situation where agreement has not yet emerged. Inventing a distinct "conflict resolution process" would imply that ordinary governance is only for agreeable cases, which defeats the purpose of having a governance document at all.

---

# 17. Documentation Governance

**Normative documents** are the nine constitutional documents named in [§25](#25-constitutional-documents), plus every Accepted KIP's content once merged into one of them. A normative document's content is binding on every conforming implementation and every future contributor.

**Informative documents** — tutorials, guides, blog posts, a project README, [TOOLCHAIN.md §4](./TOOLCHAIN.md#4-project-layout)'s hand-written project-level `docs/` — describe or explain Kyne's behavior without themselves being a source of truth for it. An informative document that disagrees with a normative one is simply wrong and MUST be corrected to match; the reverse is never true.

**Versioning.** Normative documents are versioned alongside the Kyne release they describe, per [§9](#9-release-policy); informative documents have no independent versioning requirement beyond ordinary revision history.

**Approval process.** A change to an informative document follows ordinary [§13](#13-code-review-philosophy) code review, requiring only a Maintainer's or Reviewer's approval within the relevant owned area. A change to a normative document follows [§25](#25-constitutional-documents)'s elevated process without exception.

**Review expectations.** Every documentation change, normative or informative, is reviewed for accuracy against the current state of the constitutional documents — an informative guide describing a deprecated API is a defect exactly as a code defect would be.

**Relationship to constitutional documents.** This chapter governs the *process* for changing documentation; [§25](#25-constitutional-documents) governs which documents that process treats as constitutional. The two are complementary, not overlapping: this chapter would be incomplete without [§25](#25-constitutional-documents)'s classification, and [§25](#25-constitutional-documents) would be incomplete without this chapter's account of what happens to everything it does not classify as constitutional.

---

# 18. Ecosystem Governance

**Official projects** — the compiler, the toolchain, the standard library, and any project explicitly adopted under the `kyne-lang` (or equivalent) organizational namespace — are governed entirely by this document, exactly as the constitutional documents themselves are.

**Community projects** — third-party libraries, tools, and integrations built on Kyne but not adopted as official — are governed by their own maintainers, entirely outside this document's authority. Kyne governance MUST NOT impose requirements on a community project's internal process.

**Future package registry.** Once [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-future-package-manager)'s package manager exists, the registry it depends on is an official project under this section's first provision, and its own operational policies (publishing requirements, abuse handling) belong to a future `PACKAGE_MANAGER.md`, not to this document.

**Package ownership.** Ownership of an individual community package is a matter between that package's author and the future registry's own policies — this document takes no position on it, beyond noting that registry-level policy is itself subject to [§25](#25-constitutional-documents)'s process once `PACKAGE_MANAGER.md` is itself constitutional, per that future document's own eventual designation.

**Certification.** Kyne governance MAY, in the future, define a certification or "official" marking for community packages meeting some quality bar — this document reserves the possibility without designing the mechanism, consistent with [§21](#21-future-governance)'s general deferral pattern.

**Recommended tooling.** Kyne governance MAY publish a list of recommended (but not official, and not exclusively endorsed) third-party tools — for example, a specific CI integration — without that recommendation constituting adoption as an official project under this section.

**Boundaries.** The single organizing rule of this chapter is: **governance authority extends exactly as far as "official" extends, and no further.** A community project remains entirely outside Kyne governance's reach regardless of its popularity, its quality, or how central it becomes to common Kyne workflows, unless and until it is explicitly adopted as official through this document's own process.

---

# 19. Architecture Decision Records (ADR)

**Purpose.** An ADR records an *implementation* decision — a choice made in the course of building the compiler, runtime, or toolchain that does not change any constitutional document's specified behavior, but that future contributors need to understand the reasoning behind in order to avoid re-litigating it or accidentally reversing it without realizing a deliberate choice is being undone.

**Relationship to KIPs.** A KIP changes *what is specified*; an ADR records *why a particular conforming implementation was built the way it was*. [§24](#24-kips-vs-adrs) develops this distinction fully; this chapter defines the ADR mechanism itself.

**ADR lifecycle.** An ADR is proposed, discussed among the Maintainers and Core Maintainers of the affected area, and accepted by that area's own Maintainer(s) — an ADR does not require Core Maintainer consensus at the project-wide level [§4](#4-decision-making) describes for KIPs, since it does not touch a constitutional document. Once accepted, an ADR is numbered sequentially (`ADR-001`, `ADR-002`, ...) and becomes part of the permanent record, alongside, but distinct from, the KIP archive.

**Example.** `ADR-001: Incremental Compilation Framework Selection` — recording, for illustration, the reasoning behind choosing a specific incremental-computation framework (of the general kind real-world compiler projects such as rust-analyzer have adopted) to realize [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation)'s file-granularity dependency graph and cache invalidation rules. This example is illustrative of the *kind* of decision an ADR records, not a binding commitment of this document to any specific framework — [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) deliberately specifies incremental compilation's observable behavior without prescribing an implementation strategy, per its own Non-Goals, and this document does not silently narrow that choice by naming one here.

**Why implementation decisions belong in ADRs, not KIPs.** A KIP's acceptance criteria, per [§8](#8-kip-acceptance-criteria), require a Security Analysis, a Migration Strategy, and Backward Compatibility considerations — questions that make sense for a specification change and do not meaningfully apply to, for example, a choice of internal caching data structure. Forcing every implementation decision through the KIP process would either dilute that process's rigor (by accepting implementation decisions that trivially satisfy sections irrelevant to them) or overburden implementers (by requiring specification-grade review for decisions that do not touch the specification at all). ADRs exist to give implementation decisions a real, permanent, discoverable record without either cost.

---

# 20. Governance Invariants

**Every breaking change requires a KIP.** No exception exists outside [§11](#11-security-governance)'s emergency path, and even that path requires a retroactive KIP — there is no permanently ungoverned way for Kyne's specified behavior to change.

**Every accepted KIP produces an ADR where implementation architecture is affected.** A KIP that changes what is specified frequently also requires a corresponding implementation choice not itself dictated by the specification; per [§19](#19-architecture-decision-records-adr), that choice MUST be recorded, not left as an unrecorded artifact of whoever happened to implement it.

**Security fixes may bypass normal governance.** Restated permanently from [§11](#11-security-governance): this is the one deliberate, narrow exception to specification-first development, justified by the asymmetric cost of delay in a genuine security incident, and bounded strictly by that section's retroactive-KIP requirement so the exception never becomes a permanent gap in the record.

**Constitutional documents require elevated review.** Restated permanently from [§25](#25-constitutional-documents): no constitutional document, including this one, is ever amended by an ordinary pull request.

**Consensus is preferred.** Restated permanently from [§4](#4-decision-making): tie-breaking authority exists to resolve deadlock, not to make consensus-seeking optional or perfunctory.

**Evidence outweighs opinion.** Restated permanently from [§4](#4-decision-making) and [§8](#8-kip-acceptance-criteria): a KIP's acceptance turns on its Motivation, Problem Statement, and Trade-offs sections being substantively engaged with, never on the proposer's standing or the loudness of support.

**Why future contributors must preserve them.** Every invariant above closes off a specific, foreseeable way Kyne's governance could erode without anyone deciding, in a single identifiable moment, that it should: a security exception quietly becoming the normal path, a constitutional document quietly amended by an unreviewed pull request, a KIP quietly accepted on the strength of who proposed it rather than what it demonstrated. None of these erosions typically happen through a deliberate decision to abandon good governance — they happen through an accumulation of individually reasonable-seeming exceptions. These invariants exist to make each such exception visibly a violation, not a plausible reading of an ambiguous rule.

---

# 21. Future Governance

Kyne's governance structure is not fixed for all time — [§6](#6-kip-lifecycle) applies to Governance-category KIPs exactly as it applies to every other category, meaning governance itself evolves through the same disciplined, evidenced process it applies to the language. This chapter names, without designing, several directions that evolution may plausibly take:

**Foundation.** A future non-profit or foundation entity MAY be established to hold Kyne's assets (trademark, infrastructure, potentially funding) independent of any individual, providing continuity beyond any single maintainer's involvement.

**Steering committee.** [Governance Model](#governance-model) below defines the transition philosophy from the current Founder-led model toward a Steering Committee in detail; this chapter notes only that such a transition, when it occurs, is itself a Governance-category KIP.

**Working groups.** As KIP volume within a category ([§7](#7-kip-categories)) grows, a dedicated working group with delegated review authority within that category MAY be established, without that delegation removing the category's KIPs from this document's overall process.

**Corporate sponsorship.** Kyne MAY, in the future, accept sponsorship or dedicated engineering contribution from a company or organization, provided doing so does not grant that sponsor governance authority beyond what [§3](#3-governance-structure)'s roles already define — sponsorship funds or staffs contribution, it does not purchase decision-making power.

**Trademark ownership.** The Kyne name and any associated marks SHOULD eventually be held by a Foundation, per this chapter's first provision, rather than by an individual, once one exists — until then, this document does not specify an interim arrangement, leaving that operational detail outside its scope per [Governance Scope](#26-governance-scope).

**Future governance transitions.** Every direction named in this chapter requires its own Governance-category KIP when actually proposed, subject to every rule this document already establishes, including [§25](#25-constitutional-documents)'s elevated review, since a change to this document's own structure is, definitionally, a constitutional change.

---

# 22. Governance Commandments

1. **The language belongs to its users.** Every governance decision is ultimately answerable to the Kyne contract author who will never read this document, per [Project Principles](#2-project-principles)'s "Contributors serve developers" — governance authority is held in trust for that developer's interests, not as an end in itself.

2. **Contributors earn influence.** Restated permanently from [Meritocracy](#2-project-principles): [§3](#3-governance-structure)'s roles are never granted by appointment from outside the project, by seniority elsewhere, or by employer — only by demonstrated judgment within Kyne itself, per [§12](#12-contributor-journey).

3. **No silent breaking changes.** Restated permanently from [§10](#10-compatibility-policy): every breaking change is identified as such in its KIP, associated with a major release, and accompanied by a migration strategy — there is no such thing as an accidental or unannounced breaking change in a conforming release.

4. **No undocumented behavior.** Restated permanently from [COMPILER_ARCHITECTURE.md's own invariant](./COMPILER_ARCHITECTURE.md#the-compiler-never-generates-undefined-runtime-behavior): a construct with no defined behavior in a constitutional document is not a valid part of Kyne, regardless of what any particular compiler build happens to do with it.

5. **No implementation before specification.** Restated permanently from [§23](#23-specification-first-development): the one narrow, bounded exception is [§11](#11-security-governance)'s emergency path, which itself requires a retroactive specification, never a permanently unspecified state.

6. **Consensus first.** Restated permanently from [§4](#4-decision-making): tie-breaking authority is exercised only after consensus-seeking has genuinely been attempted, never as a substitute for it.

7. **Evidence before opinion.** Restated permanently from [§4](#4-decision-making) and [§8](#8-kip-acceptance-criteria): a KIP is evaluated on its Motivation, Alternatives, and Trade-offs, never on the proposer's identity or the volume of support expressed without substantive reasoning behind it.

8. **Security is never optional.** Restated permanently from [§1](#1-governance-philosophy) and [§11](#11-security-governance): a governance shortcut that trades security review for speed is not a valid shortcut anywhere in this document, and the one process that is genuinely faster — the security emergency path — is faster because it was built for exactly this case, not because it skips scrutiny.

**Why these principles are permanent.** Each commandment closes off a specific way a governance system can decay while still appearing, on the surface, to be functioning — a project can still hold votes, still merge pull requests, still call its process "open," while having quietly abandoned every one of these eight commitments one at a time. They are declared permanent, in the same sense [LANGUAGE_PRINCIPLES.md's Ten Commandments](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne) are permanent, because a governance document that could casually revise its own foundational commitments would not actually be constraining anyone — it would only be describing whatever the current holders of authority currently prefer.

---

# 23. Specification-First Development

**No implementation begins before a specification exists.** Every constitutional document this project has produced — [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) before a parser, [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) before an execution engine, [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) before a compiler, [MEMORY_MODEL.md](./MEMORY_MODEL.md) before a memory strategy, [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) before a single stdlib function, [TOOLCHAIN.md](./TOOLCHAIN.md) before a CLI — was written in exactly this order, deliberately, and this document formalizes that ordering as a permanent requirement rather than a historical accident of how Kyne happened to be designed.

**Specifications define expected behavior.** A constitutional document states, in RFC 2119 terms, what MUST, MUST NOT, SHOULD, and MAY happen. This is the complete, authoritative description of correct behavior — not a summary of it, not a guide to it, the thing itself.

**Implementations conform to specifications.** A compiler, runtime, or toolchain change's correctness is judged entirely by whether it conforms to the relevant constitutional document, per [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy)'s testing strategy — an implementation choice made where the specification is silent belongs in an ADR, per [§19](#19-architecture-decision-records-adr), and an implementation behavior that contradicts the specification is a bug, full stop, regardless of how long that behavior has shipped or how many contracts happen to depend on it, subject only to [§10](#10-compatibility-policy)'s ordinary compatibility rules if fixing it would itself be a breaking change.

**Compiler code never becomes the specification.** This is the rule every other sentence in this chapter exists to protect: it MUST NEVER be the case that "what the compiler currently does" is treated as the authoritative answer to a question the constitutional documents do not already settle. Where such a gap is found, the correct response is a KIP closing the gap in the specification — never a decision, however reasonable, to let the existing implementation's behavior stand as the de facto answer by default.

**Why this philosophy exists.** A specification a reader can trust is one where reading the document is sufficient — no inspection of a particular compiler version's source code required, no experimentation needed to discover what actually happens. The moment "read the specification" and "read the compiler source, since that's what actually happens" diverge as answers to the same question, every constitutional document this project has produced becomes, at best, aspirational documentation rather than a binding contract. Specification-first development is the discipline that keeps them the same answer, permanently.

---

# 24. KIPs vs ADRs

| | KIPs | ADRs |
|---|---|---|
| Govern | Architectural evolution — what Kyne *is*. | Implementation decisions — how a specific conformant behavior is *realized*. |
| Examples | Language syntax, runtime behavior, memory model, compiler architecture, toolchain behavior, governance itself, standard library APIs. | Choice of parser implementation strategy, choice of incremental-compilation framework, choice of internal crate layout beyond what [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture) already fixes, choice of caching algorithm. |
| Changes a constitutional document | Yes, always. | No, never — an ADR that would require changing a constitutional document is not an ADR, it is evidence a KIP is actually needed. |
| Review body | Core Maintainer consensus (or Founder tie-break), per [§4](#4-decision-making). | The relevant area's own Maintainer(s), per [§19](#19-architecture-decision-records-adr). |
| Required sections | [§8](#8-kip-acceptance-criteria)'s full list. | Context, decision, and consequences — a lighter structure, since an ADR does not carry [§8](#8-kip-acceptance-criteria)'s specification-level stakes. |

**Why separating these documents improves long-term maintainability.** Collapsing the two into a single process would force every implementation detail through specification-grade scrutiny (slowing ordinary engineering work to a crawl for decisions that do not affect what Kyne promises anyone) or force every specification change through implementation-grade lightness (letting the language's actual promises change with the same low ceremony as an internal refactor). Keeping them separate lets each move at the pace appropriate to its actual stakes: KIPs move deliberately, because they bind every future implementation and every user's expectations; ADRs move quickly, because they bind only the specific codebase they document, and can be revisited by a future ADR without touching what Kyne, as a specification, actually is.

---

# 25. Constitutional Documents

The following documents are, collectively, Kyne's constitution:

- [FOUNDATION.md](./FOUNDATION.md)
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md)
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)
- [MEMORY_MODEL.md](./MEMORY_MODEL.md)
- [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md)
- [TOOLCHAIN.md](./TOOLCHAIN.md)
- **This document, [GOVERNANCE.md](./GOVERNANCE.md).**

This document's own inclusion in this list is deliberate: **GOVERNANCE.md is amendable only through the process it itself defines.** There is no meta-process outside this document's own KIP mechanism by which this document changes — a self-referential rule, in the same sense a constitution typically specifies its own amendment procedure, precisely so that no future authority can bypass this document's protections by first "reforming" the document that supplies them.

**Changing a constitutional document requires:**

- A KIP, per [§6](#6-kip-lifecycle), in the category matching the document being changed, per [§7](#7-kip-categories).
- **Extended review** — a longer minimum Discussion period than an ordinary, non-constitutional change, reflecting the elevated stakes of amending a document every conforming implementation and every user relies on.
- **Migration analysis**, per [§8](#8-kip-acceptance-criteria)'s required Migration Strategy section, where the change is not purely additive.
- **Explicit approval** from Core Maintainer consensus or Founder tie-break, per [§4](#4-decision-making) — never from a Maintainer's or Reviewer's authority alone, regardless of how narrow the change appears.

**Ordinary pull requests are insufficient** for any change to a constitutional document's normative content, full stop — even a change that looks like a typo fix, if it alters meaning rather than correcting a rendering error, requires this process. A pull request correcting an actual, meaning-preserving typo (a misspelled word, a broken link) is not itself a constitutional change and MAY proceed through ordinary [§17](#17-documentation-governance) review; the test is whether the change could, even slightly, alter what a conforming implementation is required to do — if yes, it is constitutional, regardless of how small it looks.

**Why constitutional stability is essential.** [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) establishes that a deployed Kyne contract's code is immutable and frequently holds real value for its entire on-chain existence. Every guarantee that contract's author relied on when they wrote and audited it traces back to a specific sentence in one of these nine documents. If any of those documents could be casually revised, every such guarantee would be, in practice, only as durable as the next ordinary pull request — which is to say, not durable at all. This chapter is what makes "constitutional" mean something more than "currently true."

---

# 26. Governance Scope

Governance, per this document, does **not** cover:

- **Bug fixes** — a change correcting an implementation's deviation from an already-specified behavior, per [§23](#23-specification-first-development), requires ordinary [§13](#13-code-review-philosophy) review only, never a KIP, since it changes nothing about what Kyne is specified to do.
- **Refactoring** — a change to internal code structure that preserves every observable behavior [COMPILER_ARCHITECTURE.md §21](./COMPILER_ARCHITECTURE.md#21-compiler-invariants) and [TOOLCHAIN.md §21](./TOOLCHAIN.md#21-toolchain-invariants) already guarantee requires only the relevant area's own Maintainer review.
- **Performance improvements** that satisfy [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)'s "never change observable behavior" rule and [MEMORY_MODEL.md's equivalent](./MEMORY_MODEL.md#10-memory-optimizations) require ordinary review, not a KIP — the constraint that makes them safe to review this lightly is precisely that they cannot, by the definition of "optimization" those documents already fix, change what a program means.
- **Compiler optimizations** within the bounds those same two documents already establish — an optimization is only exempt from KIP review because its safety boundary was itself already fixed by a constitutional document; an optimization proposal that would require *loosening* that boundary is not exempt, and is a Compiler-category KIP.
- **Internal crate organization** beyond what [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture) fixes as a MUST-level boundary — reorganizing code within a crate, or splitting one internal module into two, is an ordinary engineering decision, potentially worth an ADR per [§19](#19-architecture-decision-records-adr) but never a KIP.
- **Documentation typos** and other meaning-preserving corrections, per [§25](#25-constitutional-documents)'s own test.
- **Routine maintenance** — dependency updates, CI configuration, and other operational upkeep with no bearing on specified behavior.

**Why these should not require governance.** [Project Principle: Simplicity Scales](#2-project-principles) is directly at stake here: a governance process that required KIP-level ceremony for a one-line bug fix would not make Kyne more stable, it would make ordinary maintenance prohibitively slow, creating pressure to route routine work around the process entirely — which is a far worse outcome than simply scoping the process correctly from the start. Every item in this chapter is excluded not because it is unimportant, but because [§13](#13-code-review-philosophy)'s ordinary review is already the right-sized process for it, and applying a heavier one would not catch more defects, only slow down the catching of them.

---

# Governance Model

Kyne adopts a **Benevolent Dictator for Life (BDFL)** model for its formative years, as fixed by [§3](#3-governance-structure)'s Founder role and [§4](#4-decision-making)'s tie-breaking authority.

**Discussion is open.** Every KIP Discussion, per [§6](#6-kip-lifecycle), is public and open to comment from any Community Member, regardless of role.

**Evidence is encouraged.** [§8](#8-kip-acceptance-criteria)'s required sections exist to make evidenced argument the currency of Discussion, not standing or seniority.

**Consensus is preferred.** [§4](#4-decision-making) states, and this section restates for emphasis, that consensus among Core Maintainers is Kyne's default and preferred path to a KIP's acceptance.

**However, when consensus cannot be reached, the Founder has final tie-breaking authority.** This is not a formality — it is the specific mechanism that prevents a single sustained disagreement from permanently blocking Kyne's evolution, and it exists because a language with no mechanism to break a deadlock is a language that, in practice, cannot make a genuinely contested decision at all.

**Why this model exists for Kyne's formative years.** A young language's architecture is at its most fragile precisely when it has the fewest established precedents to reason from — every early decision sets a pattern later decisions will be judged for consistency against, per [Consistency Above Preference](./LANGUAGE_SPEC.md#consistency-above-preference). A single point of final architectural accountability during this period is what keeps those early, precedent-setting decisions coherent with one another, in a way a larger, more distributed body — still building its own shared judgment about what "feels like Kyne" — is not yet positioned to guarantee.

**Future transition path.** If Kyne reaches sufficient maturity and community size, governance MAY evolve into a Steering Committee model, through an accepted Governance-category KIP satisfying [§25](#25-constitutional-documents)'s elevated constitutional process in full. This document does not define that future Steering Committee's composition, authority, or election mechanism in any detail — doing so now, before the community and precedent base that would make such a body meaningful actually exist, would be exactly the kind of premature, evidence-free design [Project Principle: Stability over Novelty](#2-project-principles) warns against. What this document fixes is only the transition's *precondition*: it happens through the ordinary constitutional amendment process, deliberately, evidenced, and openly discussed — never through informal drift, and never through a unilateral declaration by anyone, including the Founder.

---

# Non-Goals

This document does **not** define:

- **Compiler implementation** — defined in [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md).
- **Language syntax** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Memory model** — defined in [MEMORY_MODEL.md](./MEMORY_MODEL.md).
- **Toolchain implementation** — defined in [TOOLCHAIN.md](./TOOLCHAIN.md).
- **Standard library implementation** — defined in [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md).
- **Community code of conduct** — governs interpersonal conduct, distinct from the technical and structural governance this document defines; belongs to a separate document this specification does not produce.
- **GitHub workflows, CI/CD, issue templates, repository layout** — operational tooling detail, per [§26](#26-governance-scope)'s "routine maintenance" exclusion, that this document deliberately leaves to ordinary engineering judgment rather than constitutional fixing.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission every governance decision is ultimately answerable to.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the original KIP concept and acceptance criteria this document refines rather than replaces, per [§6](#6-kip-lifecycle) and [§8](#8-kip-acceptance-criteria).
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md), [TOOLCHAIN.md](./TOOLCHAIN.md) — the six technical constitutional documents this one exists to protect, per [§25](#25-constitutional-documents).

The following documents are anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `ROADMAP.md`, `CONTRIBUTING.md`, `PACKAGE_MANAGER.md`, and `ECOSYSTEM.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is Kyne's constitution. It does not describe what Kyne is — the eight documents it protects already do that, completely — it describes how Kyne is permitted to change, by whom, and under what scrutiny. A future maintainer, contributor, or steward encountering a governance question this document does not answer should treat that gap exactly as [§23](#23-specification-first-development) requires any gap in the technical constitution to be treated: as a Governance-category KIP to be filed, discussed, and resolved through the process this document itself defines — never as a decision to be made silently, and never as evidence that the process no longer applies.
