//! The symbol table Name Resolution's collection pass builds, per
//! docs/COMPILER_ARCHITECTURE.md §8.

use std::collections::HashMap;

/// What kind of declaration a name refers to - enough to answer "is this
/// name shadowable by a `let`" (per LANGUAGE_SPEC.md §4.3/§4.5) and to
/// classify diagnostics; not a full type description, which is
/// `kyne_types`'s job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Contract,
    Struct,
    Enum,
    Error,
    Event,
    Const,
    Fn,
    State,
    /// A `let` binding, function parameter, `for`-loop variable, or
    /// pattern-bound name - introduced during the binding pass, scoped
    /// to a block, not part of the collection-pass symbol table proper.
    Local,
}

/// A flat scope: every name declared directly in one syntactic level
/// (module scope, or one contract's member scope), collected in a single
/// pass per docs/LANGUAGE_SPEC.md §2.1's order-independence guarantee -
/// every name here is visible from every position in that scope,
/// regardless of where in the source it was declared.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    names: HashMap<String, DeclKind>,
}

impl Scope {
    pub fn get(&self, name: &str) -> Option<DeclKind> {
        self.names.get(name).copied()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    /// Inserts `name`, returning the previous declaration kind if `name`
    /// was already present - the caller uses this to detect and report a
    /// duplicate declaration.
    fn insert(&mut self, name: String, kind: DeclKind) -> Option<DeclKind> {
        self.names.insert(name, kind)
    }
}

/// The two scopes Name Resolution's collection pass produces for a
/// single-file project, per docs/COMPILER_ARCHITECTURE.md §8's
/// identifier-lookup order: module-scope declarations (top-level
/// `struct`/`enum`/`error`/`event`/`const`/`fn`), and, if the file
/// contains a `contract`, that contract's own member scope.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub module_scope: Scope,
    pub contract_scope: Option<Scope>,
    /// Names introduced into module scope by `use` imports, per
    /// LANGUAGE_SPEC.md §12.2. See this crate's README for the
    /// intentional scope limitation on what "resolved" means here: this
    /// compiler has no multi-file project loader yet, so an imported
    /// name cannot actually be checked against another file's real
    /// declarations. It is recorded as a valid, resolvable module-scope
    /// name on trust, so that referencing it does not spuriously fail
    /// name resolution within the one file this crate does analyze.
    pub imported_names: Scope,
}

impl SymbolTable {
    pub fn declare_module(&mut self, name: &str, kind: DeclKind) -> Option<DeclKind> {
        self.module_scope.insert(name.to_string(), kind)
    }

    pub fn declare_contract_member(&mut self, name: &str, kind: DeclKind) -> Option<DeclKind> {
        self.contract_scope
            .get_or_insert_with(Scope::default)
            .insert(name.to_string(), kind)
    }

    pub fn declare_import(&mut self, name: &str) {
        self.imported_names
            .insert(name.to_string(), DeclKind::Local);
    }

    /// Resolves a name using the module/contract lookup order
    /// docs/COMPILER_ARCHITECTURE.md §8 specifies for the *declaration*
    /// scopes (local lexical scope is checked separately, first, by the
    /// binding pass - see `resolve::Resolver`): contract members, then
    /// module-level declarations, then imported names.
    pub fn resolve_declared(&self, name: &str) -> Option<DeclKind> {
        self.contract_scope
            .as_ref()
            .and_then(|s| s.get(name))
            .or_else(|| self.module_scope.get(name))
            .or_else(|| self.imported_names.get(name))
    }
}
