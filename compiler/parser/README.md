# kyne_parser

## Purpose

Enforces the grammar defined in [`LANGUAGE_SPEC.md` §2](../../docs/LANGUAGE_SPEC.md#2-grammar)
against the token stream `kyne_lexer` produces, and builds a Concrete
Syntax Tree.

## Responsibilities

- Accept exactly the set of token sequences the EBNF grammar defines; reject every other sequence.
- Recover from a syntax error by synchronizing at the next safe token, per [`COMPILER_ARCHITECTURE.md` §5](../../docs/COMPILER_ARCHITECTURE.md#5-parsing).
- Build the lossless raw syntax tree `kyne_cst` wraps as the compiler's stable CST type — never an AST directly.

## Dependencies

- `kyne_lexer` — consumes its token stream.
- `kyne_diagnostics` — reports syntax errors.

## Status

Implemented. `Parser` (`src/parser.rs`) is a hand-written recursive-descent
parser with precedence climbing for expressions, per
[ADR-0005](../../docs/adr/ADR-0005-parser-implementation.md), which also
records two grammar disambiguations forced by `LANGUAGE_SPEC.md`'s own
worked examples but not spelled out in its formal EBNF (no struct
literals in `if`/`while`/`for`/`match` head expressions; bare
`return`/`throw`/`break`/`continue` as a `match` arm body). `src/tree.rs`
defines the lossless `SyntaxNode`/`NodeKind` tree the parser builds
directly via a builder-stack pattern.

Covered by unit tests per grammar production (including rejected-input
and error-recovery cases), plus integration tests
(`tests/canonical_examples.rs`) confirming all six canonical examples
parse with zero diagnostics and losslessly reconstruct their exact source
text.

## Future work

None for this crate's scope per `COMPILER_ARCHITECTURE.md` §5. `kyne_ast`
(issue #8) is the next stage, lowering this crate's tree (via `kyne_cst`)
into the AST.
