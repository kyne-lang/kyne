# LANGUAGE_SPEC.md

# The Kyne Language Specification

**Phase:** 0 — Foundations
**Milestone:** 3 — Language Specification
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on the compiler implementation.

This specification supersedes no prior document. It formalizes the philosophy in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) and the mission in [FOUNDATION.md](./FOUNDATION.md) into concrete, implementable language rules. Where this document and those documents appear to conflict, this document governs implementation behavior, and the conflict should be raised as a KIP to correct whichever document is wrong.

## Notation Conventions

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings. A conforming Kyne compiler MUST reject any program that violates a MUST/MUST NOT rule in this document.

Grammar is given in EBNF, following the convention used by the Go language specification:

```
Production  = expression .
"literal"     terminal token, written verbatim
{ x }         zero or more repetitions of x
[ x ]         x is optional
( x | y )     grouping and alternation
```

Kyne source files use the `.kyn` extension and MUST be encoded as UTF-8. There is a single, unified source file extension — a file's role as part of a contract project, a library, or a shared module is determined by project structure and content (see [§12](#12-modules)), never by its extension.

---

# Kyne Design Philosophy

[LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) explains why Kyne exists and what it values. This chapter explains something narrower and more operational: the architectural principles the language committee applies when deciding what goes *into* this specification in the first place. Every section from [§1](#1-lexical-structure) onward is downstream of the six principles below. A future KIP that cannot be justified against them should not be accepted regardless of how useful it seems in isolation, and an existing rule in this specification that no longer serves one of them is a candidate for revision, not a precedent to defend on its own inertia.

## Every Keyword Must Earn Its Place

Every keyword Kyne reserves is a permanent tax on every person who ever learns the language, reads a grammar summary, or writes a syntax highlighter. A keyword is not free simply because it is short to type — it occupies a name forever, it appears in every keyword table in every piece of documentation, and it is one more thing a newcomer must be told not to use as a variable name before they have written a single line of working code. The committee treats the keyword list in [§1.3](#13-keywords) as a budget, not a wishlist: nothing is added to it by default, and everything already on it is expected to justify its continued presence.

A keyword earns its place only by solving a problem specific to writing correct, secure smart contracts — not by making a common pattern marginally shorter, and not by matching a convention from Rust, Go, or TypeScript for its own sake. `auth` earns its place because Soroban's authorization model has no other reasonable spelling. `state` earns its place because distinguishing persistent storage from a local binding by keyword, rather than by convention, is load-bearing for the compiler's [definite-assignment analysis](#44-definite-assignment-of-state) and for an auditor's ability to `grep` a contract's entire storage surface in one pass.

This principle is not hypothetical. The removal of the `private` visibility modifier from this specification is a direct application of it: `private` and the unmarked, file-local default described in [§5.3](#53-visibility) meant the same thing, and a keyword that exists only to say, more verbosely, what silence already says has not earned its place. Two ways to spell one idea is a cost with no offsetting benefit, and the committee removed it rather than let it stand on the grounds that it was already shipped. The same scrutiny applies to every keyword still in the list, indefinitely.

## Discoverability Over Memorization

A developer should be able to guess correctly. If a container type supports lookup, its lookup method should be named `get`, not `fetch` in one type and `find` in another. If a function mutates state and returns whether it succeeded, its return type should be `Result<bool, E>` in every contract that follows the same pattern, not `Result<(), E>` in one and a bare `bool` in the next. Kyne optimizes for the developer who has read exactly one Kyne contract and is now guessing at the shape of a second, because that is the position almost every developer is in for almost their entire first month with the language.

Discoverability is why [§6.2](#62-collections)'s `list<T>` and `map<K, V>` methods are named `get`, `set`, `has`, `push`, `remove`, and `len` — the smallest, plainest vocabulary that covers the operations those types support, reused identically across both types wherever the operation is the same shape. It is also why naming conventions in [§1.7](#17-naming-conventions) are uniform across the entire language rather than negotiated per module: a developer who has internalized "types are `PascalCase`, everything else is `snake_case`, constants shout" never needs to look up a name's casing, only its spelling.

Discoverability is deliberately placed above cleverness in naming. A shorter or more elegant name that a developer would not have guessed on their own is a worse name than a slightly longer one they would have. This is a standing instruction to future contributors proposing standard library or compiler-diagnostic names: optimize for the name a competent developer would type before checking the documentation, not for the name that reads best in isolation.

## Auditability Over Cleverness

Kyne code is written once and read many times — by its author days later, by a co-author during review, and, for any contract handling real value, by a professional auditor who has never seen the codebase before and is being paid to find what is wrong with it. Every one of those readings is more expensive, and more consequential, than the few seconds a clever construct might have saved the original author at write time. Kyne's syntax and standard library are chosen for the audit, not for the demo.

This is why the [canonical member order](#32-canonical-member-order) is a compiler-enforced hard error rather than a formatter suggestion: an auditor should never have to search a contract to find its storage layout or its error surface, because both are always in the same relative position. It is why arithmetic is checked by default rather than wrapping silently ([§7.2](#72-arithmetic)) — a reviewer should not have to independently verify that every addition in a token contract cannot overflow; the language already guarantees it, or the code says explicitly, in the name of the function called, that it does not. Cleverness that requires a reader to hold more context in their head at once is a cost this specification consistently declines to pay, even when the clever version would be shorter.

## Secure by Default

Nothing in Kyne is unsafe unless a developer goes out of their way to make it so, and Kyne v1 provides no such way at all — there is no `unsafe` block, no escape hatch that disables a compiler guarantee, and no convenience flag that trades a security property for a shorter function. Where a behavior could reasonably be either strict or permissive, this specification chooses strict, and requires the permissive behavior — if it is offered at all — to be reached through a differently named, clearly labeled function rather than through a modifier on the default one.

The compiler's obligation under this principle goes beyond refusing unsafe programs; it extends to steering developers toward the safe pattern before they have written the unsafe one. This is why [§14](#14-error-philosophy) requires every diagnostic to suggest a fix, and why the default arithmetic, default state-initialization checking, and default exhaustive `match` requirement ([§7.7](#77-pattern-matching)) all exist as defaults rather than as opt-in flags a developer would need to already know to reach for. A secure-by-default language is one where the easiest program to write is also the safe one, not one where the safe program merely exists as an alternative to the easy one.

## Transparent Abstraction

Every construct in this specification MUST have a stable, documented, human-followable path to the Rust it produces. This is not a preference about code style — it is the property that lets a Rust-literate auditor treat Kyne source as a readable summary of the Rust program that will actually execute on Soroban, rather than as a black box they must compile and inspect before trusting. A Kyne feature that cannot be explained by pointing at the specific Rust it generates is not ready for this specification, no matter how much boilerplate it would remove.

Transparent abstraction is also why this specification is exact about what it does *not* hide: [§4.0](#40-memory-model-value-semantics-only) is explicit that Kyne's value semantics describe guarantees visible at the source level, not a claim that the generated Rust avoids borrowing or references — the compiler remains free to generate whatever idiomatic Rust a skilled engineer would write underneath. Abstraction in Kyne removes repetition; it never removes the reader's ability to answer, precisely, "what does the chain actually do when this runs?"

## Consistency Above Preference

Kyne deliberately has fewer stylistic degrees of freedom than most languages its influences draw from. There is one canonical member order, one canonical formatting style enforced by a single formatter with no configuration file, one casing convention per kind of name, and one obvious way to express a conditional value, a loop, or a fallible operation. None of this is an accident of an unfinished style guide — it is the intended end state. Every choice this specification takes away from an individual author is a choice a future reader no longer has to reconstruct.

The goal is that a developer moving from one Kyne contract to another, written by a different team, on a different project, should feel almost no friction from the change of authorship. The ecosystem should read as though every contract were written by the same disciplined engineering team, because in every way this specification can enforce, it effectively was. Where a future KIP proposes to add a second acceptable way to express something Kyne can already express, the burden of proof is on the proposal to show that the existing way is being *replaced*, not merely supplemented — per [Commandment 6](#15-language-commandments), one obvious way is not a starting position to be negotiated away as the language grows.

---

# 1. Lexical Structure

## 1.1 Source Encoding and Unicode

Kyne source files are UTF-8 text. Comments and string literals MAY contain arbitrary Unicode. **Identifiers MUST NOT contain non-ASCII characters.** This is a deliberate security decision, not an oversight: Unicode identifiers permit homoglyph attacks, where a variable, function, or type name is visually indistinguishable from another but resolves to a different symbol (for example, a Cyrillic `а` substituted for a Latin `a`). In a language whose programs move real value, an auditor's ability to trust that two identical-looking names are the same name is not negotiable. This restriction may be revisited only if a confusable-resistant subset of Unicode identifiers is formally specified in a future KIP; it MUST NOT be relaxed casually.

## 1.2 Identifiers

```
identifier = letter { letter | digit } .
letter     = "A" … "Z" | "a" … "z" | "_" .
digit      = "0" … "9" .
```

An identifier MUST begin with an ASCII letter or underscore and MUST NOT begin with a digit. Identifiers are case-sensitive.

## 1.3 Keywords

The following are the **primary keywords**, fixed by the Milestone 2 design decisions. They are reserved and MUST NOT be used as identifiers:

```
contract  fn       state    let      const
event     emit      use      public   internal
struct    enum      trait    match    if
else      for       while    return   break
continue  error     throw    true     false
auth      in
```

`trait` is reserved for a future version and has no grammar production in v1. `auth` and `in` are grammar-load-bearing keywords required to express authorization checks and `for` loops respectively, and are treated as part of the Milestone 2 decision set rather than a deviation from it.

### 1.3.1 Additional Reserved Words

The following identifiers are reserved for future language versions. They have **no meaning in v1** and MUST NOT be used as identifiers. Reserving them now, ahead of need, is itself an application of [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution) — it lets Kyne add method syntax, casting, or an escape hatch in a later version without that addition becoming a breaking change to existing programs:

```
self  Self  impl  mut  async  await  unsafe  dyn  as  move
```

Notably absent from this list: Kyne does not reserve a macro-introducing token (such as `macro` or a `!` suffix convention), because macros are permanently excluded — see [Non-Goals](./LANGUAGE_PRINCIPLES.md#non-goals) — not deferred.

### 1.3.2 Prelude Bindings

`Some`, `None`, `Ok`, and `Err` are not keywords. They are ordinary identifiers pre-bound in every module's scope as the variant constructors of the intrinsic `Option<T>` and `Result<T, E>` types (see [§6.6](#66-option-and-result)). A top-level declaration MUST NOT shadow a prelude binding; the compiler MUST reject such a declaration with a name-collision error.

## 1.4 Comments

```
line_comment  = "//" { any character except newline } .
block_comment = "/*" { any character } "*/" .
doc_comment   = "///" { any character except newline } .
```

Block comments MAY be nested. `///` doc comments MUST immediately precede a top-level or contract-member declaration; the documentation generator attaches the comment to that declaration. A doc comment separated from its declaration by a blank line is not attached and SHOULD be flagged by the formatter.

## 1.5 Whitespace

Spaces, tabs, and newlines are insignificant except as token separators. Kyne has no significant indentation. Every block is delimited by `{` and `}`. This is a direct application of [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit): block structure MUST be visible independent of formatting.

## 1.6 Literals

```
int_literal    = decimal_lit | hex_lit .
decimal_lit    = digit { digit | "_" } .
hex_lit        = "0x" hex_digit { hex_digit | "_" } .
hex_digit      = digit | "A" … "F" | "a" … "f" .
bool_literal   = "true" | "false" .
string_literal = `"` { string_char } `"` .
string_char    = any Unicode character except `"` or unescaped backslash
               | escape_sequence .
escape_sequence = "\\n" | "\\t" | "\\\\" | "\\\"" | "\\u{" hex_digit { hex_digit } "}" .
```

Underscores in numeric literals are purely visual separators and carry no semantic meaning (`1_000_000` and `1000000` are the same literal). Kyne v1 has no character/rune literal and no raw byte-string literal; `bytes` values are produced through explicit standard-library conversion functions, not literal syntax, keeping the lexer small.

There is no dedicated `symbol` literal syntax. A `string_literal` used where a `symbol` is expected (see [§6.1](#61-primitive-types)) is coerced by the compiler at that use site, since a symbol is a restricted, short, interned string and every valid symbol is already a valid string literal.

## 1.7 Naming Conventions

| Construct | Convention | Example |
|---|---|---|
| Contract, struct, enum, event, error names | `PascalCase` | `Marketplace`, `TokenError` |
| Functions, parameters, variables, state fields | `snake_case` | `transfer`, `total_supply` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SUPPLY` |
| Enum variants | `PascalCase` | `Ok`, `InsufficientBalance` |

These conventions are enforced by `kyne fmt` diagnostics (warnings) rather than by the compiler as hard errors, keeping the boundary between "does not compile" and "does not match house style" clean — see [§13](#13-formatting-rules).

---

# 2. Grammar

This section defines the complete syntactic grammar of Kyne v1. Section-specific subsections below (§3–§12) restate the relevant productions inline with explanation; this section is the authoritative consolidated reference.

```
Program        = { Import } { TopLevelDecl } .

Import         = "use" ImportPath [ "." "{" IdentifierList "}" ] ";" .
ImportPath     = identifier { "." identifier } .
IdentifierList = identifier { "," identifier } [ "," ] .

TopLevelDecl   = ContractDecl
               | StructDecl
               | EnumDecl
               | ErrorDecl
               | EventDecl
               | ConstDecl
               | FnDecl .

ContractDecl   = "contract" identifier "{" { ContractMember } "}" .
ContractMember = StateDecl | ConstDecl | ErrorDecl | EventDecl
               | StructDecl | EnumDecl | FnDecl .

StateDecl      = "state" identifier ":" Type [ "=" Expression ] ";" .
ConstDecl      = "const" identifier ":" Type "=" Expression ";" .

ErrorDecl      = "error" identifier ( "{" ErrorVariants "}" | ";" ) .
ErrorVariants  = identifier { "," identifier } [ "," ] .

EventDecl      = "event" identifier "(" [ ParamList ] ")" ";" .

StructDecl     = "struct" identifier "{" [ FieldList ] "}" .
FieldList      = Field { "," Field } [ "," ] .
Field          = identifier ":" Type .

EnumDecl       = "enum" identifier "{" EnumVariant { "," EnumVariant } [ "," ] "}" .
EnumVariant    = identifier [ "(" TypeList ")" | "{" FieldList "}" ] .
TypeList       = Type { "," Type } .

FnDecl         = [ Visibility ] "fn" identifier "(" [ ParamList ] ")" [ "->" Type ] Block .
Visibility     = "public" | "internal" .
ParamList      = Param { "," Param } .
Param          = identifier ":" Type .

Type           = PrimitiveType | ParametricType | identifier .
PrimitiveType  = "bool" | "i32" | "i64" | "i128"
               | "u32" | "u64" | "u128"
               | "address" | "symbol" | "string" | "bytes" .
ParametricType = "list" "<" Type ">"
               | "map" "<" Type "," Type ">"
               | "bytes" "<" int_literal ">"
               | "Option" "<" Type ">"
               | "Result" "<" Type "," Type ">" .

Block          = "{" { Statement } "}" .

Statement      = LetStmt | AssignStmt | AuthStmt | EmitStmt | ThrowStmt
               | IfStmt | MatchStmt | ForStmt | WhileStmt
               | ReturnStmt | BreakStmt | ContinueStmt | ExprStmt .

LetStmt        = "let" identifier [ ":" Type ] "=" Expression ";" .
AssignStmt     = Expression AssignOp Expression ";" .
AssignOp       = "=" | "+=" | "-=" | "*=" | "/=" | "%=" .
AuthStmt       = "auth" "(" Expression ")" ";" .
EmitStmt       = "emit" identifier "(" [ ArgList ] ")" ";" .
ThrowStmt      = "throw" Expression ";" .

IfStmt         = "if" Expression Block [ "else" ( IfStmt | Block ) ] .
MatchStmt      = "match" Expression "{" { MatchArm } "}" .
MatchArm       = Pattern [ "if" Expression ] "=>" ( Expression "," | Block ) .
Pattern        = "_"
               | Literal
               | identifier
               | identifier "(" PatternList ")"
               | identifier "{" FieldPatternList "}" .
PatternList    = Pattern { "," Pattern } .
FieldPatternList = identifier { "," identifier } [ "," ] .

ForStmt        = "for" identifier "in" Expression Block .
WhileStmt      = "while" Expression Block .

ReturnStmt     = "return" [ Expression ] ";" .
BreakStmt      = "break" ";" .
ContinueStmt   = "continue" ";" .
ExprStmt       = Expression ";" .

ArgList        = Expression { "," Expression } .
```

Expression grammar is given as a precedence table rather than a nested production chain — see [§7](#7-expressions).

### 2.1 Name Resolution Is Order-Independent

The compiler MUST resolve all top-level and contract-member names in a single pass before type-checking. Declaration order within a file or contract body has no effect on whether a program compiles — a `state` field MAY reference a `struct` defined later in the same contract, and a `fn` MAY call another `fn` declared below it. Declaration order is a purely stylistic concern, governed entirely by [§3.2](#32-canonical-member-order) and [§13](#13-formatting-rules), not a semantic one. This mirrors Go and Rust, and exists so that the canonical ordering rule can be about readability and audit consistency without also becoming a forward-declaration burden on the author.

---

# 3. Contracts

## 3.1 Declaration and Members

```
ContractDecl   = "contract" identifier "{" { ContractMember } "}" .
```

A contract is the sole unit of deployment in Kyne. A `ContractMember` MAY be a `state` field, a `const`, an `error`, an `event`, a local `struct`/`enum`, or a `fn`.

A project MUST contain **at most one** `ContractDecl` across all of its files. A project containing zero `ContractDecl`s is a **library** (see [§12](#12-modules)); a project containing exactly one is a **contract project**, whose sole contract is the deployment artifact. A project containing more than one `ContractDecl` MUST be rejected by the compiler with a hard error — there is no such thing, at the language level, as an ambiguous deployment target.

## 3.2 Canonical Member Order

Every contract body MUST declare its members in the following order:

1. `state` declarations
2. `const` declarations
3. `error` declarations
4. `event` declarations
5. local `struct` / `enum` declarations
6. `fn init` (the constructor, if present)
7. `public fn` declarations
8. `internal fn` declarations
9. unmarked (file-private) `fn` declarations

This ordering is enforced by the compiler as a **hard error**, not merely a formatter convention. This is a deliberate departure from languages that leave member order to taste: a Kyne auditor reviewing an unfamiliar contract for the first time MUST be able to find "what state exists," "what can go wrong," and "what can be observed externally" in the same relative position in every Kyne contract that has ever been written, before reading a single function body. Consistency of this kind is cheap for the compiler to enforce and valuable for every reader forever afterward.

## 3.3 The Public API

A `public fn` on a contract becomes part of that contract's on-chain interface: it is compiled to an exported entry point callable by any external transaction, cross-contract call, or client SDK. `public` MUST only be used on contract members. Using `public` on a library-level `fn` (see [§12](#12-modules)) is a compiler error, because a library is never deployable and therefore cannot have an ABI — enforcing at the type-checking level a rule that would otherwise be a matter of convention.

## 3.4 Internal and File-Private Helpers

An `internal fn` is callable from anywhere within the same project — including from library code and from other functions in the same contract — but does not appear in the ABI. A `fn` with no visibility modifier is file-private: callable only from within the same file, and not appearing in the ABI. Functions without an explicit visibility modifier are file-private. This is the narrowest scope available (see [§5.3](#53-visibility)).

## 3.5 The Constructor

A contract MAY define exactly one `fn` named `init`. If present, `init` MUST be `public` and is invoked exactly once, at deployment time, by the Soroban constructor mechanism the compiler generates. The compiler MUST reject any attempt to call `init` from within any other function. `init` is the canonical place to perform [definite assignment](#44-definite-assignment-of-state) of `state` fields that have no default initializer.

---

# 4. Variables

Kyne has three variable-introducing constructs — `let`, `const`, and `state` — distinguished by lifetime and mutability, not by an orthogonal `mut` modifier.

## 4.0 Memory Model: Value Semantics Only

**Kyne has no borrow checker, no reference types, and no lifetimes in its surface syntax.** Every value — primitive, struct, enum, or collection — is passed to functions, returned from functions, and assigned to other variables **by value**. There is no `&`, no `&mut`, and no concept of a value being "moved" out from under its owner as observable Kyne-level behavior.

This is one of the most consequential decisions in this specification, and it is made deliberately rather than by omission. Rust's ownership and borrowing model is the single largest source of the learning curve Kyne exists to remove — see [Why Kyne Exists](./FOUNDATION.md#why-kyne-exists). A TypeScript or Go developer already has a value-and-reference intuition that does not include compiler-enforced exclusive borrows, and asking them to learn one in order to write a token contract is exactly the kind of avoidable tax [Design Philosophy](./LANGUAGE_PRINCIPLES.md#why-familiarity-matters-and-where-it-stops) rejects.

This does **not** constrain the Rust the compiler generates. The compiler MAY, and in most cases SHOULD, generate idiomatic Rust that borrows, references, or clones exactly where a skilled Rust engineer would — [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) requires the *output* to be idiomatic Rust, not that Kyne source mirror the *absence* of ownership syntax in its own grammar. Kyne's value semantics describe the guarantees visible at the source level; the compiler is free to choose the most efficient correct implementation underneath them.

A direct consequence: structural equality (`==`, `!=`) is automatically defined for every `struct` and `enum` type, with no `derive` mechanism required, because macros are excluded and value semantics make structural comparison unambiguous by default.

## 4.1 `let` — Local Variables

```
LetStmt = "let" identifier [ ":" Type ] "=" Expression ";" .
```

A `let` binding is scoped to the enclosing `Block` and exists only for the duration of the current function invocation; it never touches contract storage. A `let` binding MUST be initialized at declaration — Kyne has no uninitialized-variable state, eliminating an entire class of undefined-behavior-adjacent bugs common in languages that allow declare-then-assign-later. A `let` binding is mutable and MAY be reassigned with `=` or a compound assignment operator; there is no `mut` keyword because there is no immutable-by-default `let` to distinguish it from — mutability of locals is not a security-relevant axis in a language with no aliasing to reason about (per [§4.0](#40-memory-model-value-semantics-only)).

## 4.2 `const` — Compile-Time Constants

```
ConstDecl = "const" identifier ":" Type "=" Expression ";" .
```

A `const` is an immutable, compile-time-evaluated value. Its initializer MUST be a constant expression (a literal or an expression composed entirely of literals and other `const`s). A `const` MAY be declared at contract scope or at module (file) scope. `const`s are inlined at compile time and have zero on-chain storage cost — they are a property of the compiled program, not of any deployed contract instance.

## 4.3 `state` — Persistent Contract Storage

```
StateDecl = "state" identifier ":" Type [ "=" Expression ] ";" .
```

A `state` field is a named, typed, persistent value that survives across separate invocations of the contract on-chain. It is compiled to a Soroban persistent storage entry keyed by the field's name within the contract instance. `state` fields MUST have an explicit type — they are never inferred, because storage layout is part of a contract's audited surface and MUST be visible without running type inference over the whole file.

`state` is accessed **directly by name** — `balance`, not `self.balance` or `this.balance`. The compiler resolves any bare identifier that matches a declared `state` field, within any function belonging to that contract, as a storage read; assigning to it is a storage write. This is a deliberate [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) trade: the mechanism (a storage `get`/`set` call in the generated Rust) is hidden, but the concept (this is contract state, not a local) remains visible because `state`-declared names are visually distinguished at their declaration site and MUST NOT collide with `let` names in the same function (shadowing a `state` field with a `let` of the same name is a compiler error, to keep "is this a local or persistent?" always unambiguous by grep-ing the contract's `state` block).

## 4.4 Definite Assignment of State

Every `state` field MUST be **definitely assigned** before any `public` or `internal` function other than `init` can be reached. A field satisfies this by either:

1. carrying a literal initializer at its declaration (`state count: i64 = 0;`), or
2. being unconditionally assigned somewhere in `init` before `init` returns.

The compiler MUST perform a definite-assignment analysis over `init` and reject the contract if any `state` field could be read while still unset. This eliminates an entire class of "read of unset storage" bugs at compile time rather than deferring them to a runtime default-value guess, which is a common source of subtle accounting bugs in other smart-contract ecosystems.

## 4.5 Scope

Kyne uses standard lexical block scoping: an identifier introduced by `let` or a function parameter is visible from its declaration to the end of the innermost enclosing `Block`. Shadowing a `let` with another `let` of the same name in a nested block is permitted; shadowing a `state` field or a `const` with a `let` is not (per [§4.3](#43-state-persistent-contract-storage)).

---

# 5. Functions

## 5.1 Declaration

```
FnDecl = [ Visibility ] "fn" identifier "(" [ ParamList ] ")" [ "->" Type ] Block .
```

```kyne
public fn transfer(from: address, to: address, amount: i128) -> Result<bool, TokenError> {
    // ...
}
```

A function with no `-> Type` returns the unit type (equivalent to Go's no-return-value functions); there is no explicit `void` or `unit` keyword to write.

## 5.2 Parameters and Return Types

Parameters are passed by value (see [§4.0](#40-memory-model-value-semantics-only)) and MUST each carry an explicit type — Kyne does not infer parameter types from call sites. A function MAY declare at most one return type; multiple return values (as in Go) are not supported in v1 — a function needing to return more than one logical value MUST return a `struct`, which is one obvious way to do it rather than two competing conventions (bare tuples vs. structs).

## 5.3 Visibility

| Modifier | Callable from | Appears in ABI |
|---|---|---|
| `public` | anywhere (external calls, other contracts, client SDKs) | Yes — contract members only |
| `internal` | anywhere within the same project | No |
| *(none)* | the same file only | No |

`public` MUST only appear on a `ContractMember` (see [§3.3](#33-the-public-api)). Kyne has only two explicit visibility modifiers, `public` and `internal`. There is no `private` keyword. **Functions without an explicit visibility modifier are file-private.** This keeps the visibility surface to two deliberate keywords plus one implicit default, rather than three keywords where two would otherwise mean the same thing — a direct application of [One Obvious Way](./LANGUAGE_PRINCIPLES.md#small-language-philosophy).

## 5.4 Recursion

Recursion is permitted. Kyne does not provide, and does not guarantee, tail-call optimization — the compiler MUST NOT be relied upon to convert recursive calls into loops. Because Soroban meters execution resources, unbounded or deep recursion is a genuine correctness risk, not merely a style concern; a `fn` whose recursion depth cannot be statically bounded SHOULD be flagged by the security analyzer (see [Future Vision](./LANGUAGE_PRINCIPLES.md#future-vision)) and authors SHOULD prefer `for`/`while` for any loop whose bound is a runtime collection size.

## 5.5 No Overloading

A function name MUST be unique within its declaring scope (contract or file). Kyne does not support overloading by parameter type or count. This is a direct application of [One Obvious Way](./LANGUAGE_PRINCIPLES.md#small-language-philosophy): a reader seeing a call to `transfer(...)` should never need to perform overload resolution to know which `transfer` is meant.

## 5.6 Naming

Functions use `snake_case`. State-mutating functions SHOULD be named with an imperative verb (`transfer`, `mint`, `approve`); pure queries that read but do not mutate `state` SHOULD be named as a noun phrase or `get_`-prefixed phrase (`balance_of`, `get_owner`) so that read/write intent is legible from the call site alone, without needing to open the function body.

---

# 6. Types

## 6.1 Primitive Types

| Type | Description |
|---|---|
| `bool` | Boolean, `true` or `false` |
| `i32`, `i64`, `i128` | Signed integers |
| `u32`, `u64`, `u128` | Unsigned integers |
| `address` | A Soroban account or contract address. No literal form; obtained only from parameters, `state`, or SDK-provided values. |
| `symbol` | A short, interned string (used for storage keys, event topics); coercible from a `string_literal` at the use site. |
| `string` | A dynamically sized UTF-8 string. |
| `bytes` | A dynamically sized byte sequence. |

There are no implicit conversions between numeric types, including between signed and unsigned variants of the same width. This is [Explicit Over Implicit](./LANGUAGE_PRINCIPLES.md#explicit-over-implicit) applied to arithmetic: silent numeric coercion is a well-documented source of accounting bugs. Conversion is performed with a **call-style conversion function** named after the target type — `i64(some_i32)`, `u32(some_u64)` — mirroring Go's conversion syntax rather than introducing an `as` operator. A conversion that cannot preserve the value's magnitude (for example, converting a negative `i64` to `u32`) MUST panic at runtime and SHOULD be flagged by the compiler at compile time when the source is a constant expression.

## 6.2 Collections

Kyne v1 excludes user-defined generics (see [§6.7](#67-generics-a-closed-not-an-open-feature)) but ships a small, fixed set of **intrinsic parametric types** — compiler-recognized, not expressible through any general mechanism a user could invoke to build their own:

| Type | Description |
|---|---|
| `list<T>` | An ordered, growable sequence of `T`. |
| `map<K, V>` | An associative container from `K` to `V`. |
| `bytes<N>` | A fixed-length byte array of exactly `N` bytes (`N` MUST be a constant expression). |

### 6.2.1 `list<T>` methods

| Method | Signature | Description |
|---|---|---|
| `push` | `(value: T) -> unit` | Appends `value`. |
| `get` | `(index: u32) -> Option<T>` | Returns `None` if `index` is out of bounds. |
| `len` | `() -> u32` | Current element count. |
| `remove` | `(index: u32) -> unit` | Removes the element at `index`. |

A `list<T>` is iterable with `for value in list_expr { ... }` (see [§8.5](#85-for)).

### 6.2.2 `map<K, V>` methods

| Method | Signature | Description |
|---|---|---|
| `get` | `(key: K) -> Option<V>` | Returns `None` if `key` is absent. |
| `set` | `(key: K, value: V) -> unit` | Inserts or overwrites. |
| `has` | `(key: K) -> bool` | Membership test. |
| `remove` | `(key: K) -> unit` | Removes the entry, if present. |
| `len` | `() -> u32` | Current entry count. |

Iterating a `map<K, V>`'s keys and values directly (e.g., `for (k, v) in map_expr`) is **not supported in v1** and is reserved for a future version behind an explicit `.entries()`-style API, keeping the v1 iteration model to the single, unambiguous `list<T>` case.

## 6.3 Structs

```
StructDecl = "struct" identifier "{" [ FieldList ] "}" .
```

```kyne
struct Listing {
    seller: address,
    price: i128,
    sold: bool,
}
```

A `struct` is a plain, named product type. **Kyne v1 has no `impl` blocks and no methods attached to `struct` or `enum` types** — this follows directly from the keyword set in [§1.3](#13-keywords), which deliberately omits `impl` and `self`. All logic lives in `fn` declarations (contract members or library free functions) that take the struct as a parameter. This is a decisive small-language choice: it removes an entire axis of design (where does behavior live — the type or a free function?) by giving only one answer, and it keeps every contract's logic reachable by reading its `fn` list rather than also auditing methods scattered across type definitions.

Every field MUST be supplied when constructing a struct literal — there are no default field values and no partial construction, since there is no `Default`-like mechanism in a language without generics or traits.

## 6.4 Enums

```
EnumDecl    = "enum" identifier "{" EnumVariant { "," EnumVariant } [ "," ] "}" .
EnumVariant = identifier [ "(" TypeList ")" | "{" FieldList "}" ] .
```

```kyne
enum Status {
    Pending,
    Approved(address),
    Rejected { reason: string },
}
```

An enum variant MAY carry no data (`Pending`), positional data (`Approved(address)`), or named fields (`Rejected { reason: string }`). Enums are consumed with `match` (see [§7.7](#77-pattern-matching) and [§8.2](#82-match)).

## 6.5 `error`

```
ErrorDecl     = "error" identifier ( "{" ErrorVariants "}" | ";" ) .
ErrorVariants = identifier { "," identifier } [ "," ] .
```

```kyne
error TokenError {
    InsufficientBalance,
    InvalidAmount,
}
```

An `error` declaration defines a closed, data-less enumeration of failure reasons for a contract, used exclusively as the `E` in a function's `Result<T, E>` return type. `error` exists as a distinct declaration from `enum` — rather than requiring authors to write `enum TokenError { ... }` and separately promise to only use it in `Result` position — so that a reader scanning a contract's [canonical member order](#32-canonical-member-order) can find every way that contract can fail in one place, without distinguishing "regular" enums from error enums by convention.

## 6.6 `Option` and `Result`

`Option<T>` and `Result<T, E>` are intrinsic enums, always available, with prelude-bound constructors ([§1.3.2](#132-prelude-bindings)):

```
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

(shown here in enum notation for exposition; both are compiler intrinsics, not user-writable declarations.)

| Type | Method | Signature | Description |
|---|---|---|---|
| `Option<T>` | `is_some` | `() -> bool` | |
| `Option<T>` | `is_none` | `() -> bool` | |
| `Option<T>` | `unwrap_or` | `(default: T) -> T` | Returns the contained value, or `default`. |
| `Result<T, E>` | `is_ok` | `() -> bool` | |
| `Result<T, E>` | `is_err` | `() -> bool` | |
| `Result<T, E>` | `unwrap_or` | `(default: T) -> T` | Returns the `Ok` value, or `default`. |

`Result<T, E>` additionally supports the postfix `?` operator — see [§7.8](#78-error-propagation).

## 6.7 Generics: A Closed, Not an Open, Feature

Kyne v1 excludes user-defined generics as a language feature (per the Milestone 2 decisions). `list<T>`, `map<K, V>`, `bytes<N>`, `Option<T>`, and `Result<T, E>` are not evidence of a general generics mechanism — they are a **fixed, closed set of five compiler-intrinsic parametric types**, each hand-implemented in the compiler, with no syntax available for a Kyne program to define a sixth. This mirrors pre-1.18 Go, which had built-in parametric `map` and `slice` types for years before user-facing generics existed, and it is a deliberate application of [Small Language Philosophy](./LANGUAGE_PRINCIPLES.md#small-language-philosophy): the ergonomic benefit of parametric containers is captured in full, while the much larger design and complexity surface of a general generics system — variance, trait bounds, monomorphization visible to the author — is deferred until it can be justified on its own KIP, independent of these five container types.

## 6.8 Future Reserved Types

The following are not part of v1 but are anticipated and MUST NOT be assumed available: user-defined generic types, `trait`-based interfaces and trait objects (`dyn`), a `Duration`/`Timestamp` wrapper distinct from raw integer ledger time, borrowed/reference types, and any bytes-string literal syntax beyond the escape sequences in [§1.6](#16-literals).

---

# 7. Expressions

## 7.1 Precedence

From highest to lowest precedence:

| Level | Operators | Associativity |
|---|---|---|
| 1 (highest) | `.field`, `.method()`, `(call)`, `?` | left |
| 2 | unary `-`, `!` | right |
| 3 | `*`, `/`, `%` | left |
| 4 | `+`, `-` | left |
| 5 | `<`, `>`, `<=`, `>=` | left |
| 6 | `==`, `!=` | left |
| 7 | `&&` | left |
| 8 (lowest) | `\|\|` | left |

Assignment (`=`, `+=`, `-=`, `*=`, `/=`, `%=`) is **not an expression** — it is exclusively a `Statement` production ([§8](#8-statements)). Kyne deliberately excludes assignment-as-expression, which is the class of language design that permits `if (x = y)` to compile when `x == y` was meant. This is [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience) applied to a single, narrow, historically expensive footgun.

## 7.2 Arithmetic

`+ - * / %` on integer types are **checked** by default. An operation that would overflow or underflow the operand type's range MUST panic (abort the current invocation) rather than silently wrap. There is no bare-operator wrapping or saturating arithmetic; a function that intentionally needs wrapping or saturating behavior MUST call an explicit standard-library function (`wrapping_add`, `saturating_sub`, etc.) whose name states the non-default behavior at the call site. This guarantees that a bare `a + b` in Kyne source can never silently misrepresent a balance — the single most common category of exploitable arithmetic bug in smart contracts.

## 7.3 Comparison, Boolean, and Logical Expressions

`==` and `!=` perform structural equality, automatically defined for every `struct` and `enum` (see [§4.0](#40-memory-model-value-semantics-only)). `< > <= >=` are defined for numeric types only. `&& ` and `||` short-circuit, left to right, and operate only on `bool`.

## 7.4 Function Calls

```
Call = Expression "(" [ ArgList ] ")" .
```

Arguments are evaluated left to right and bound to parameters by position, matching the declared `ParamList`.

## 7.5 Construction

**Struct literals:**

```kyne
Listing { seller: seller, price: price, sold: false }
```

**Field-init shorthand** is permitted when the field name and the bound variable name are identical:

```kyne
Listing { seller, price, sold: false }
```

**Enum construction:**

```kyne
Status::Approved(admin)
Status::Rejected { reason: "insufficient collateral" }
```

**List and map literals:**

```kyne
let ids: list<u64> = [1, 2, 3];
let empty: list<u64> = [];
let scores: map<address, i128> = { alice_addr: 100, bob_addr: 50 };
let empty_map: map<address, i128> = {};
```

An empty `{}` is unambiguous as a map literal in expression position: Kyne has no free-standing block-as-expression form other than `if`/`match`, which have their own leading keyword, so a bare `{` beginning an expression is always a map literal.

## 7.6 `if` as an Expression

```kyne
let fee = if amount > 1000 { amount / 100 } else { 10 };
```

When `if` is used in expression position (its value is bound, returned, or passed), the `else` branch is **mandatory** and both branches MUST produce the same type. When used as a `Statement` ([§8.1](#81-if)), `else` is optional, since the (non-)result is discarded. This dual role mirrors Rust and keeps Kyne from needing a separate ternary operator — one obvious way to express a conditional value.

## 7.7 Pattern Matching

`match` is usable as either a `Statement` or an `Expression`, following the same rule as `if`: in expression position every arm MUST produce a value of the same type. Patterns support enum variant destructuring, wildcard (`_`), literal matching, and an optional trailing guard (`if <expr>`):

```kyne
match status {
    Status::Approved(by) if by == admin => return true,
    Status::Approved(_) => return false,
    Status::Rejected { reason } => throw ProcessError::Rejected,
    _ => return false,
}
```

A `match` MUST be **exhaustive** — the compiler MUST reject a `match` over an enum that does not cover every variant, unless a wildcard `_` arm is present. Exhaustiveness checking is a compiler feature, not a lint, because a missing variant in a `match` is a common source of "the new enum case silently falls through to nothing" bugs when an enum is extended later.

`throw` and `return` used as a match arm's body have the bottom type — the compiler treats them as compatible with whatever type the surrounding `match` expression expects, since they never produce a value at all.

## 7.8 Error Propagation

The postfix `?` operator is valid only inside a function whose return type is `Result<T, E>`. Applied to a `Result<T, E>`-typed expression, `?` evaluates to the contained `T` if the value is `Ok`, or immediately returns the `Err(e)` from the enclosing function if it is `Err` — provided the enclosing function's error type is exactly `E`. This maps directly onto Rust's own `?` operator in the generated code, making it one of the purest expressions of [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) in the language: the sugar and the target are the same operator.

---

# 8. Statements

## 8.1 `if`

```kyne
if amount <= 0 {
    throw TokenError::InvalidAmount;
}
```

Braces are **always mandatory**, even for a single statement, and there is no bare (unbraced) `if` form. This eliminates dangling-`else` ambiguity and closes off the exact class of defect behind the historical "goto fail" bug, where a missing brace silently changed which statement was conditional.

## 8.2 `match`

See [§7.7](#77-pattern-matching). As a statement, arm bodies MAY be either a single `Expression` followed by `,` or a `Block`.

## 8.3 `for`

```kyne
for owner in owners {
    if owner == caller {
        return Ok(true);
    }
}
```

`for` iterates a `list<T>`, binding each element to the loop variable in order. There is no C-style three-clause `for`; a bounded counting loop uses `while` with an explicit counter, keeping exactly one loop-with-a-condition form and one loop-over-a-collection form.

## 8.4 `while`

```kyne
let mut_i = 0;
while mut_i < 10 {
    mut_i += 1;
}
```

(Note: `mut_i` here is an ordinary identifier chosen for clarity, not a language construct — recall `mut` is a reserved word with no meaning in v1, per [§1.3.1](#131-additional-reserved-words).)

## 8.5 `for` Details

Iteration order for `list<T>` is insertion order, matching Soroban's underlying `Vec` semantics. There is no v1 iteration form for `map<K, V>` (see [§6.2.2](#622-mapk-v-methods)).

## 8.6 `return`, `break`, `continue`

`return` exits the current function, optionally with a value matching the declared return type. `break` and `continue` apply to the innermost enclosing `for` or `while` loop and MUST NOT be used outside one.

## 8.7 `throw`

```kyne
throw TokenError::InsufficientBalance;
```

`throw` is valid only inside a function returning `Result<T, E>`, where the thrown expression's type MUST be exactly `E`. `throw <expr>;` is defined to desugar to `return Err(<expr>);` and exists as distinct syntax specifically so that failure exits are visually distinct from success exits when scanning a function body — an application of [Readability First](./LANGUAGE_PRINCIPLES.md#readability-first): the reader should be able to tell "this function can fail here" from the keyword alone, without inspecting the wrapped value's constructor.

## 8.8 `emit`

```kyne
emit Transfer(from, to, amount);
```

See [§11](#11-events) for full validation rules. `emit` is valid only inside a contract member function.

## 8.9 `auth`

```kyne
auth(caller);
```

See [§10](#10-authentication) for full semantics.

---

# 9. Storage

## 9.1 Persistent State

`state`-declared fields (see [§4.3](#43-state-persistent-contract-storage)) are the only values that persist across separate contract invocations. Each is compiled to a distinct Soroban persistent storage entry, keyed within the contract instance by the field's declared name. There is no way to express Soroban's Temporary or Instance storage durability classes directly in Kyne v1 — every `state` field uses Persistent storage. Exposing the other durability classes is reserved for a future version once real-world usage shows which contracts need the more advanced, and more error-prone, tradeoffs those classes offer.

## 9.2 Temporary (In-Memory) Variables

`let` bindings (see [§4.1](#41-let-local-variables)) never touch the ledger. They exist only in the executing invocation's call frame and are discarded when the function that declared them returns.

## 9.3 Immutable Values

`const` values (see [§4.2](#42-const-compile-time-constants)) have no runtime storage representation at all — they are inlined at compile time into the generated Rust, and therefore carry zero read or write cost against a contract's resource budget.

## 9.4 Compiler Rules

- `state` fields MUST have an explicit type; type inference is never applied to a `StateDecl`.
- Every `state` field MUST satisfy [definite assignment](#44-definite-assignment-of-state) before any non-`init` entry point can execute.
- `state` MUST NOT be mutated from a library free function directly. Libraries operate purely on the values passed to them and return new values; only a contract member function may assign to its own contract's `state`. This keeps every state mutation grep-able from within the one file that declares that state, and it is a direct consequence of [Composition Over Inheritance](./LANGUAGE_PRINCIPLES.md#composition-over-inheritance): shared logic is reused as pure functions over explicit data, never as ambient access to another module's mutable storage.
- A `state` field MUST NOT share a name with a `let` binding or parameter in any function of the same contract ([§4.3](#43-state-persistent-contract-storage)).

---

# 10. Authentication

## 10.1 The `auth` Statement

```
AuthStmt = "auth" "(" Expression ")" ";" .
```

`auth(addr)` asserts that the transaction currently being processed carries a valid authorization from `addr`, compiling directly to Soroban's `require_auth()` call on that address. If authorization is absent or invalid, execution MUST abort before any subsequent statement runs.

## 10.2 No Implicit Sender

**Kyne has no implicit `sender`, `msg.sender`, or equivalent global.** Every function that needs to know which party authorized an action MUST receive that party's `address` as an explicit parameter, and MUST explicitly call `auth` on it. This mirrors Soroban's own multi-party authorization model, where a single invocation can legitimately carry authorizations from several distinct addresses, and it is a deliberate security decision: implicit-sender globals in other smart-contract ecosystems are a documented source of confused-deputy vulnerabilities in relayed or delegated calls, where code written against an assumed single caller misbehaves once a contract calls it on someone else's behalf. Requiring the address explicitly at every call site makes "who is this authorization for?" a question answerable by reading the function signature alone.

## 10.3 Canonical Function Shape

Contract functions that mutate `state` on behalf of a specific party SHOULD follow this order: **validate inputs → `auth` the responsible party → mutate `state` → `emit` events.** `auth` itself has no side effects and is not a storage mutation, so it is not part of the checks-effects-interactions ordering in the strict sense — but placing it after basic input validation and before any state mutation ensures a malformed call fails cheaply, before an authorization check is even attempted, while still guaranteeing no state changes ahead of a caller being proven authorized to cause them.

```kyne
public fn transfer(from: address, to: address, amount: i128) -> Result<bool, TokenError> {
    if amount <= 0 {
        throw TokenError::InvalidAmount;
    }

    auth(from);

    let from_balance = balance_of(from);
    if from_balance < amount {
        throw TokenError::InsufficientBalance;
    }

    balances.set(from, from_balance - amount);
    balances.set(to, balance_of(to) + amount);

    emit Transfer(from, to, amount);
    return Ok(true);
}
```

## 10.4 Compiler Expectations

The core compiler does not — and cannot, in general — determine which functions *ought* to require authorization; that is a question of contract-specific intent, not syntax. The compiler's role is limited to correctly translating `auth(...)` into `require_auth()` and to rejecting `auth` outside a contract member function. Detecting a *missing* `auth` call on a function that looks privileged (for example, one that unconditionally mutates another party's balance) is the responsibility of the **Security Analyzer** tool (see [Future Vision](./LANGUAGE_PRINCIPLES.md#future-vision)), which operates as a lint layer on top of, not inside, the core compiler — keeping the compiler's job (is this program well-formed?) cleanly separated from the analyzer's job (is this program probably safe?).

---

# 11. Events

## 11.1 Declaration

```
EventDecl = "event" identifier "(" [ ParamList ] ")" ";" .
```

```kyne
event Transfer(from: address, to: address, amount: i128);
```

An event declares a fixed, named, positional schema. Event declarations are contract members and follow [§3.2](#32-canonical-member-order)'s ordering.

## 11.2 Emission

```kyne
emit Transfer(from, to, amount);
```

`emit` MUST reference an `event` declared in the same contract, and MUST supply arguments matching the declared parameter list exactly in count, order, and type. The compiler MUST reject an `emit` whose arguments do not match — there is no partial or keyword-based emission in v1. `emit` is valid only inside a contract member function; a library free function cannot emit an event directly, for the same reason it cannot mutate `state` directly (see [§9.4](#94-compiler-rules)) — observable on-chain effects belong exclusively to the contract that owns them.

## 11.3 Naming

Event names use `PascalCase` and SHOULD read as a past-tense or noun description of what occurred (`Transfer`, `Mint`, `Approved`, `Executed`) rather than an imperative verb, distinguishing an event's "this happened" role from a function's "do this" role at a glance.

---

# 12. Modules

## 12.1 Contracts and Libraries

A `.kyn` file that contains a `ContractDecl` is part of a **contract project**; a project with no `ContractDecl` anywhere is a **library**, importable by contract projects but never itself deployable (see [Contract-Oriented Design](./LANGUAGE_PRINCIPLES.md#why-contracts-are-the-unit-of-everything)). This is a project-level, not file-level, classification: a contract project MAY still contain many files that hold only `struct`, `enum`, `error`, `fn`, and `const` declarations supporting the one file that declares its single `contract`.

## 12.2 Imports

```
Import     = "use" ImportPath [ "." "{" IdentifierList "}" ] ";" .
ImportPath = identifier { "." identifier } .
```

```kyne
use math.safe_math;
use collections.{List, Map};
```

Import paths use `.`-separated segments mirroring TypeScript- and Go-familiar module access, rather than Rust's `::`, since import syntax is surface-level developer experience and carries no implication about the generated Rust's own module paths.

## 12.3 Visibility Across Modules

Type declarations (`struct`, `enum`, `error`, `event`) carry no visibility modifier in v1 — every type is visible throughout the project once its declaring file is `use`-imported. Restricting type visibility below whole-project scope (for cross-project package distribution) is deferred until a package distribution model exists to make that restriction meaningful; adding it later is additive and non-breaking. Function visibility follows [§5.3](#53-visibility): `public` is contract-only and ABI-producing, `internal` is project-wide, and unmodified (file-private) is file-local.

## 12.4 Namespaces

Kyne has no explicit `mod`-style in-file namespace declaration. One `.kyn` file is one implicit module, and its module path mirrors its position in the project's directory structure — matching Go's package-per-directory model rather than Rust's explicit `mod` tree, consistent with [Go's simplicity](./LANGUAGE_PRINCIPLES.md#language-identity) as a stated influence.

---

# 13. Formatting Rules

`kyne fmt` produces exactly one canonical rendering of any syntactically valid program. Formatting is enforced by tooling and CI convention, not by the compiler itself — `kyne build` MUST NOT refuse to compile correctly formatted-differently source. This keeps "is this program well-formed" (the compiler's job) and "does this program match house style" (the formatter's job) as two separate, separately testable concerns, even though projects are expected to gate merges on `kyne fmt --check` passing.

| Rule | Value |
|---|---|
| Indentation | 4 spaces; tabs are rejected by the formatter |
| Brace style | Opening brace on the same line as its introducing construct |
| Max line length | 100 columns |
| Blank lines | Exactly one blank line between contract members of different [canonical order](#32-canonical-member-order) categories; none required within a category |
| Trailing commas | Required in any struct literal, enum variant, list literal, map literal, or parameter list that spans multiple lines |
| Import ordering | Grouped as: standard library imports, then project-local imports; alphabetized within each group; exactly one blank line between groups |
| Contract member order | As specified in [§3.2](#32-canonical-member-order) (also a compiler-enforced hard error, not just a formatting rule) |

---

# 14. Error Philosophy

Kyne compiler diagnostics exist to teach, not merely to reject — see [Compiler as a Teacher](./LANGUAGE_PRINCIPLES.md#compiler-as-a-teacher). Every diagnostic MUST include: the specific problem, its location, an explanation of *why* it matters in a smart-contract-specific context, and — wherever a fix is mechanical — a suggested correction. The following are illustrative examples of the expected quality bar.

**Reading `state` before it is definitely assigned:**

```
error[KY0104]: state `balance` may be read before it is initialized
  --> src/token.kyn:14:15
   |
14 |     let x = balance + 1;
   |             ^^^^^^^ possibly-uninitialized persistent state
   |
   = note: `balance` has no default value and is not assigned in `init`
   = help: give `balance` a default value at its declaration:
   |
   |     state balance: i128 = 0;
   |
   = help: or assign it unconditionally inside `init`:
   |
   |     public fn init() {
   |         balance = 0;
   |     }
```

**Arithmetic that provably overflows a constant expression:**

```
error[KY0210]: this operation always overflows `u32`
  --> src/token.kyn:9:18
   |
 9 |     const MAX: u32 = 4_294_967_295 + 1;
   |                      ^^^^^^^^^^^^^^^^^ `u32` cannot represent this value
   |
   = note: Kyne arithmetic is checked by default and never wraps silently — see LANGUAGE_SPEC.md §7.2
   = help: if you intended wraparound, call it explicitly: `MAX_U32.wrapping_add(1)`
```

**Event emission with mismatched arguments:**

```
error[KY0311]: `emit Transfer` does not match its declared event
  --> src/token.kyn:47:5
   |
47 |     emit Transfer(from, to);
   |          ^^^^^^^^^^^^^^^^^ expected 3 arguments, found 2
   |
   = note: `event Transfer(from: address, to: address, amount: i128);` declared at src/token.kyn:6
   = help: did you forget to pass `amount`?
```

Diagnostic codes are namespaced by section of this specification (`KY01xx` = storage, `KY02xx` = types/arithmetic, `KY03xx` = events, and so on), so that a diagnostic code alone is enough for a contributor to know which section of this document governs the rule being enforced.

---

# 15. Language Commandments

These restate, in normative form, the principles established at length in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#the-ten-commandments-of-kyne). Where that document explains *why*, this section states *what the compiler and its contributors MUST guarantee*.

1. **Security before convenience.** A compiler change MUST NOT be accepted if it weakens a safety guarantee in this specification for the sake of ergonomics, regardless of the ergonomic gain.
2. **Readability first.** Any construct whose generated Rust an experienced Rust engineer cannot read without special tooling MUST be reworked before it ships.
3. **Transparent abstraction.** Every Kyne construct MUST have a documented, stable mapping to the specific Rust it produces.
4. **Explicit over implicit.** Anything with an on-chain effect — state mutation, authorization, external calls, arithmetic that can fail — MUST be visible in source at the point it happens.
5. **Composition over inheritance.** Kyne MUST NOT gain an inheritance mechanism; reuse is expressed through composition, free functions, and explicit data passed by value.
6. **One obvious way.** A KIP proposing a second syntax for something Kyne can already express MUST be rejected unless the existing syntax is being replaced, not merely supplemented.
7. **Compiler as teacher.** Every new diagnostic MUST ship with an explanation and, where mechanical, a suggested fix, per [§14](#14-error-philosophy).
8. **Contracts first.** Every language feature MUST be justifiable in terms of a real contract-authoring need; general-purpose usefulness alone is not sufficient justification, per [Non-Goals](./LANGUAGE_PRINCIPLES.md#non-goals).

---

# 16. Examples

The following six contracts exercise every construct defined in this specification and are intended as canonical reference implementations for the compiler's own test suite.

## 16.1 Counter

```kyne
contract Counter {
    state count: i64 = 0;

    event Incremented(by: address, new_value: i64);

    public fn init() {
        count = 0;
    }

    public fn increment(by: address) -> i64 {
        auth(by);

        count += 1;
        emit Incremented(by, count);
        return count;
    }

    public fn get() -> i64 {
        return count;
    }
}
```

## 16.2 Token

```kyne
contract Token {
    state total_supply: i128 = 0;
    state balances: map<address, i128>;

    const NAME: string = "Kyne Example Token";
    const SYMBOL: string = "KYN";
    const DECIMALS: u32 = 7;

    error TokenError {
        InsufficientBalance,
        InvalidAmount,
    }

    event Transfer(from: address, to: address, amount: i128);
    event Mint(to: address, amount: i128);

    public fn init(admin: address) {
        auth(admin);
        total_supply = 0;
    }

    public fn mint(admin: address, to: address, amount: i128) -> Result<bool, TokenError> {
        if amount <= 0 {
            throw TokenError::InvalidAmount;
        }

        auth(admin);

        balances.set(to, balance_of(to) + amount);
        total_supply += amount;

        emit Mint(to, amount);
        return Ok(true);
    }

    public fn transfer(from: address, to: address, amount: i128) -> Result<bool, TokenError> {
        if amount <= 0 {
            throw TokenError::InvalidAmount;
        }

        auth(from);

        let from_balance = balance_of(from);
        if from_balance < amount {
            throw TokenError::InsufficientBalance;
        }

        balances.set(from, from_balance - amount);
        balances.set(to, balance_of(to) + amount);

        emit Transfer(from, to, amount);
        return Ok(true);
    }

    public fn balance_of(account: address) -> i128 {
        return balances.get(account).unwrap_or(0);
    }
}
```

## 16.3 Escrow

```kyne
contract Escrow {
    state buyer: address;
    state seller: address;
    state arbiter: address;
    state amount: i128;
    state released: bool = false;

    error EscrowError {
        AlreadyReleased,
        Unauthorized,
    }

    event Released(to: address, amount: i128);

    public fn init(buyer_addr: address, seller_addr: address, arbiter_addr: address, deposit: i128) {
        auth(buyer_addr);

        buyer = buyer_addr;
        seller = seller_addr;
        arbiter = arbiter_addr;
        amount = deposit;
        released = false;
    }

    public fn release(caller: address) -> Result<bool, EscrowError> {
        if released {
            throw EscrowError::AlreadyReleased;
        }

        if caller != arbiter && caller != buyer {
            throw EscrowError::Unauthorized;
        }

        auth(caller);

        released = true;
        emit Released(seller, amount);
        return Ok(true);
    }
}
```

## 16.4 Marketplace

```kyne
contract Marketplace {
    state listings: map<u64, Listing>;
    state next_id: u64 = 0;

    error MarketError {
        NotFound,
        AlreadySold,
    }

    event Listed(id: u64, seller: address, price: i128);
    event Sold(id: u64, buyer: address);

    struct Listing {
        seller: address,
        price: i128,
        sold: bool,
    }

    public fn init() {
        next_id = 0;
    }

    public fn list_item(seller: address, price: i128) -> u64 {
        auth(seller);

        let id = next_id;
        listings.set(id, Listing { seller, price, sold: false });
        next_id += 1;

        emit Listed(id, seller, price);
        return id;
    }

    public fn buy(buyer: address, id: u64) -> Result<bool, MarketError> {
        auth(buyer);

        let listing = match listings.get(id) {
            Some(value) => value,
            None => throw MarketError::NotFound,
        };

        if listing.sold {
            throw MarketError::AlreadySold;
        }

        listings.set(id, Listing { seller: listing.seller, price: listing.price, sold: true });

        emit Sold(id, buyer);
        return Ok(true);
    }
}
```

## 16.5 Voting

```kyne
contract Voting {
    state admin: address;
    state proposals: map<u64, Proposal>;
    state voted: map<address, bool>;
    state next_id: u64 = 0;

    error VoteError {
        AlreadyVoted,
        NotFound,
        Unauthorized,
    }

    event ProposalCreated(id: u64, description: string);
    event Voted(id: u64, voter: address, approve: bool);

    struct Proposal {
        description: string,
        yes_votes: u64,
        no_votes: u64,
    }

    public fn init(admin_addr: address) {
        auth(admin_addr);
        admin = admin_addr;
    }

    public fn create_proposal(caller: address, description: string) -> Result<u64, VoteError> {
        auth(caller);

        if caller != admin {
            throw VoteError::Unauthorized;
        }

        let id = next_id;
        proposals.set(id, Proposal { description, yes_votes: 0, no_votes: 0 });
        next_id += 1;

        emit ProposalCreated(id, description);
        return Ok(id);
    }

    public fn vote(voter: address, id: u64, approve: bool) -> Result<bool, VoteError> {
        auth(voter);

        if voted.has(voter) {
            throw VoteError::AlreadyVoted;
        }

        let proposal = match proposals.get(id) {
            Some(value) => value,
            None => throw VoteError::NotFound,
        };

        let updated = if approve {
            Proposal {
                description: proposal.description,
                yes_votes: proposal.yes_votes + 1,
                no_votes: proposal.no_votes,
            }
        } else {
            Proposal {
                description: proposal.description,
                yes_votes: proposal.yes_votes,
                no_votes: proposal.no_votes + 1,
            }
        };

        proposals.set(id, updated);
        voted.set(voter, true);

        emit Voted(id, voter, approve);
        return Ok(true);
    }
}
```

## 16.6 Multi-Signature Wallet

```kyne
contract MultisigWallet {
    state owners: list<address>;
    state threshold: u32;
    state transactions: map<u64, Transaction>;
    state approvals_index: map<u64, map<address, bool>>;
    state next_id: u64 = 0;

    error WalletError {
        NotOwner,
        NotFound,
        AlreadyApproved,
        AlreadyExecuted,
        ThresholdNotMet,
    }

    event Submitted(id: u64, to: address, amount: i128);
    event Approved(id: u64, owner: address, total_approvals: u32);
    event Executed(id: u64, to: address, amount: i128);

    struct Transaction {
        to: address,
        amount: i128,
        approvals: u32,
        executed: bool,
    }

    public fn init(initial_owners: list<address>, required: u32) {
        owners = initial_owners;
        threshold = required;
    }

    public fn submit(caller: address, to: address, amount: i128) -> Result<u64, WalletError> {
        auth(caller);
        require_owner(caller)?;

        let id = next_id;
        transactions.set(id, Transaction { to, amount, approvals: 0, executed: false });
        next_id += 1;

        emit Submitted(id, to, amount);
        return Ok(id);
    }

    public fn approve(caller: address, id: u64) -> Result<bool, WalletError> {
        auth(caller);
        require_owner(caller)?;

        let tx = match transactions.get(id) {
            Some(value) => value,
            None => throw WalletError::NotFound,
        };

        if tx.executed {
            throw WalletError::AlreadyExecuted;
        }

        let signers = approvals_index.get(id).unwrap_or({});
        if signers.has(caller) {
            throw WalletError::AlreadyApproved;
        }

        signers.set(caller, true);
        approvals_index.set(id, signers);

        let updated = Transaction {
            to: tx.to,
            amount: tx.amount,
            approvals: tx.approvals + 1,
            executed: tx.executed,
        };
        transactions.set(id, updated);

        emit Approved(id, caller, updated.approvals);
        return Ok(true);
    }

    public fn execute(caller: address, id: u64) -> Result<bool, WalletError> {
        auth(caller);
        require_owner(caller)?;

        let tx = match transactions.get(id) {
            Some(value) => value,
            None => throw WalletError::NotFound,
        };

        if tx.executed {
            throw WalletError::AlreadyExecuted;
        }

        if tx.approvals < threshold {
            throw WalletError::ThresholdNotMet;
        }

        transactions.set(id, Transaction {
            to: tx.to,
            amount: tx.amount,
            approvals: tx.approvals,
            executed: true,
        });

        emit Executed(id, tx.to, tx.amount);
        return Ok(true);
    }

    internal fn require_owner(caller: address) -> Result<bool, WalletError> {
        for owner in owners {
            if owner == caller {
                return Ok(true);
            }
        }
        throw WalletError::NotOwner;
    }
}
```

---

# Compiler Architecture

Compiler implementation details are intentionally **not** specified in this document. This specification defines what a conforming Kyne program means; it does not define how a compiler is internally structured to arrive at that meaning. Keeping the two separate lets the compiler's internal architecture evolve — new intermediate representations, alternative optimization strategies, a rewritten diagnostic engine — without those changes ever becoming a language-level breaking change subject to [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution), which properly governs observable language behavior, not implementation strategy.

The compiler pipeline — lexing, parsing, AST construction, semantic analysis, definite-assignment and exhaustiveness checking, optimization, Rust code generation, and diagnostic reporting — is the subject of a separate, forthcoming document: `COMPILER_ARCHITECTURE.md`. That document, not this one, is where future compiler contributors should look for how the rules in this specification are implemented, and it is where implementation-level proposals belong. This specification remains the source of truth for *what* must hold; `COMPILER_ARCHITECTURE.md` will be the source of truth for *how* the reference compiler makes it hold.

# Future Ecosystem

Package management — a registry, dependency resolution, versioning, package signing, and the security model around distributing and consuming third-party Kyne libraries — is intentionally **out of scope for Kyne v1**. Nothing in this specification should be read as precluding a package system; nothing in it should be read as specifying one either. [§12](#12-modules) defines how a single project resolves its own files and imports, which is the full extent of what v1 needs to compile contracts that exist today.

A package ecosystem raises questions — how a registry authenticates publishers, how version resolution interacts with [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution)'s breaking-change guarantees, how a security review treats a dependency the reviewer does not control — that deserve their own dedicated KIPs once real multi-project usage exists to inform the answers, rather than a design guessed at ahead of that experience. This section reserves the space in the specification for that future work; it does not attempt to fill it.

---

# Closing

This specification is authoritative for Kyne v1. It should change only through the KIP process described in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution), and any change to a MUST-level rule in this document is, by definition, a breaking change subject to [Stable Evolution](./LANGUAGE_PRINCIPLES.md#stable-evolution). Future compiler contributors should treat disagreement with this document as a signal to open a KIP, not to quietly diverge in an implementation.
