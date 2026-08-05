//! The two-pass resolver itself, per docs/COMPILER_ARCHITECTURE.md §8.

use std::collections::HashMap;

use kyne_ast::{
    ContractMember, Expr, FnDecl, IfStmt, MatchArmBody, MatchStmt, Path, Pattern, Program,
    Statement, TopLevelDecl,
};
use kyne_diagnostics::{Diagnostic, Span as DiagSpan};

use crate::symbol_table::{DeclKind, SymbolTable};

/// Identifiers pre-bound in every module's scope as the variant
/// constructors of `Option<T>`/`Result<T, E>`, per LANGUAGE_SPEC.md
/// §1.3.2. Never flagged as unresolved.
const PRELUDE: &[&str] = &["Some", "None", "Ok", "Err"];

pub struct ResolveResult {
    pub symbols: SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
}

/// Resolves every identifier reference in `program` against its
/// declarations, per docs/COMPILER_ARCHITECTURE.md §8: a collection pass
/// registers every top-level and contract-member declaration regardless
/// of source order (honoring LANGUAGE_SPEC.md §2.1), then a binding pass
/// walks every function body resolving each reference against local
/// lexical scope first, then contract members, then module-level
/// declarations, then imports.
pub fn resolve(program: &Program, file: &str) -> ResolveResult {
    let mut diagnostics = Vec::new();
    let symbols = collect(program, file, &mut diagnostics);
    let binding_diagnostics = {
        let mut resolver = Resolver {
            symbols: &symbols,
            file: file.to_string(),
            diagnostics,
            scopes: Vec::new(),
        };
        resolver.walk_program(program);
        resolver.diagnostics
    };
    ResolveResult {
        symbols,
        diagnostics: binding_diagnostics,
    }
}

// ---- Collection pass ----

fn collect(program: &Program, file: &str, diagnostics: &mut Vec<Diagnostic>) -> SymbolTable {
    let mut symbols = SymbolTable::default();

    for import in &program.imports {
        for name in &import.names {
            symbols.declare_import(name);
        }
        if import.names.is_empty() {
            if let Some(last) = import.path.last() {
                symbols.declare_import(last);
            }
        }
    }

    for item in &program.items {
        match item {
            TopLevelDecl::Contract(c) => {
                declare_module(&mut symbols, diagnostics, file, &c.name, DeclKind::Contract);
                for member in &c.members {
                    collect_contract_member(member, file, &mut symbols, diagnostics);
                }
            }
            TopLevelDecl::Struct(s) => {
                declare_module(&mut symbols, diagnostics, file, &s.name, DeclKind::Struct)
            }
            TopLevelDecl::Enum(e) => {
                declare_module(&mut symbols, diagnostics, file, &e.name, DeclKind::Enum)
            }
            TopLevelDecl::Error(e) => {
                declare_module(&mut symbols, diagnostics, file, &e.name, DeclKind::Error)
            }
            TopLevelDecl::Event(e) => {
                declare_module(&mut symbols, diagnostics, file, &e.name, DeclKind::Event)
            }
            TopLevelDecl::Const(c) => {
                declare_module(&mut symbols, diagnostics, file, &c.name, DeclKind::Const)
            }
            TopLevelDecl::Fn(f) => {
                declare_module(&mut symbols, diagnostics, file, &f.name, DeclKind::Fn)
            }
        }
    }

    symbols
}

fn collect_contract_member(
    member: &ContractMember,
    file: &str,
    symbols: &mut SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (name, kind) = match member {
        ContractMember::State(s) => (&s.name, DeclKind::State),
        ContractMember::Const(c) => (&c.name, DeclKind::Const),
        ContractMember::Error(e) => (&e.name, DeclKind::Error),
        ContractMember::Event(e) => (&e.name, DeclKind::Event),
        ContractMember::Struct(s) => (&s.name, DeclKind::Struct),
        ContractMember::Enum(e) => (&e.name, DeclKind::Enum),
        ContractMember::Fn(f) => (&f.name, DeclKind::Fn),
    };
    if symbols.declare_contract_member(name, kind).is_some() {
        diagnostics.push(duplicate_declaration_diagnostic(file, name));
    }
}

fn declare_module(
    symbols: &mut SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
    file: &str,
    name: &str,
    kind: DeclKind,
) {
    if symbols.declare_module(name, kind).is_some() {
        diagnostics.push(duplicate_declaration_diagnostic(file, name));
    }
}

// ---- Binding pass ----

struct Resolver<'a> {
    symbols: &'a SymbolTable,
    file: String,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, DeclKind>>,
}

impl Resolver<'_> {
    fn walk_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevelDecl::Contract(c) => {
                    for member in &c.members {
                        match member {
                            ContractMember::Fn(f) => self.walk_fn(f),
                            ContractMember::State(s) => {
                                if let Some(init) = &s.init {
                                    self.push_scope();
                                    self.walk_expr(init);
                                    self.pop_scope();
                                }
                            }
                            ContractMember::Const(c) => {
                                self.push_scope();
                                self.walk_expr(&c.value);
                                self.pop_scope();
                            }
                            _ => {}
                        }
                    }
                }
                TopLevelDecl::Fn(f) => self.walk_fn(f),
                TopLevelDecl::Const(c) => {
                    self.push_scope();
                    self.walk_expr(&c.value);
                    self.pop_scope();
                }
                _ => {}
            }
        }
    }

    fn walk_fn(&mut self, f: &FnDecl) {
        self.push_scope();
        for p in &f.params {
            self.declare_local(&p.name, DeclKind::Local);
        }
        self.walk_block_statements(&f.body.statements);
        self.pop_scope();
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_local(&mut self, name: &str, kind: DeclKind) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), kind);
        }
    }

    /// Registers a `let` binding, first checking LANGUAGE_SPEC.md
    /// §4.3/§4.5's shadowing rule: a `let` MUST NOT share a name with a
    /// `state` field or `const` of the enclosing contract. Shadowing
    /// another `let` or a parameter remains permitted.
    fn declare_let(&mut self, name: &str) {
        if let Some(kind @ (DeclKind::State | DeclKind::Const)) = self
            .symbols
            .contract_scope
            .as_ref()
            .and_then(|s| s.get(name))
        {
            self.diagnostics
                .push(illegal_shadow_diagnostic(&self.file, name, kind));
        }
        self.declare_local(name, DeclKind::Local);
    }

    fn resolve_name(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains_key(name))
            || self.symbols.resolve_declared(name).is_some()
    }

    fn check_unresolved(&mut self, name: &str) {
        if PRELUDE.contains(&name) || name == "_" || self.resolve_name(name) {
            return;
        }
        self.diagnostics
            .push(unresolved_name_diagnostic(&self.file, name));
    }

    fn walk_block_statements(&mut self, statements: &[Statement]) {
        self.push_scope();
        for stmt in statements {
            self.walk_stmt(stmt);
        }
        self.pop_scope();
    }

    fn walk_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(s) => {
                self.walk_expr(&s.value);
                self.declare_let(&s.name);
            }
            Statement::Assign(s) => {
                self.walk_expr(&s.target);
                self.walk_expr(&s.value);
            }
            Statement::Auth(e) | Statement::Throw(e) | Statement::Expr(e) => self.walk_expr(e),
            Statement::Emit(s) => {
                self.check_unresolved(&s.event);
                for arg in &s.args {
                    self.walk_expr(arg);
                }
            }
            Statement::If(if_stmt) => self.walk_if(if_stmt),
            Statement::Match(match_stmt) => self.walk_match(match_stmt),
            Statement::For(s) => {
                self.walk_expr(&s.iterable);
                self.push_scope();
                self.declare_local(&s.var, DeclKind::Local);
                self.walk_block_statements(&s.body.statements);
                self.pop_scope();
            }
            Statement::While(s) => {
                self.walk_expr(&s.condition);
                self.walk_block_statements(&s.body.statements);
            }
            Statement::Return(Some(e)) => self.walk_expr(e),
            Statement::Return(None) | Statement::Break | Statement::Continue => {}
        }
    }

    fn walk_if(&mut self, if_stmt: &IfStmt) {
        self.walk_expr(&if_stmt.condition);
        self.walk_block_statements(&if_stmt.then_block.statements);
        match &if_stmt.else_branch {
            Some(kyne_ast::ElseBranch::If(nested)) => self.walk_if(nested),
            Some(kyne_ast::ElseBranch::Block(block)) => {
                self.walk_block_statements(&block.statements)
            }
            None => {}
        }
    }

    fn walk_match(&mut self, match_stmt: &MatchStmt) {
        self.walk_expr(&match_stmt.subject);
        for arm in &match_stmt.arms {
            self.push_scope();
            self.declare_pattern(&arm.pattern);
            if let Some(guard) = &arm.guard {
                self.walk_expr(guard);
            }
            match &arm.body {
                MatchArmBody::Block(block) => self.walk_block_statements(&block.statements),
                MatchArmBody::Expr(e) | MatchArmBody::Throw(e) => self.walk_expr(e),
                MatchArmBody::Return(Some(e)) => self.walk_expr(e),
                MatchArmBody::Return(None) | MatchArmBody::Break | MatchArmBody::Continue => {}
            }
            self.pop_scope();
        }
    }

    /// Registers the bindings a pattern introduces. Does not itself
    /// resolve the pattern's leading path (e.g. the `Status` in
    /// `Status::Approved(by)`) against a real enum's variant shape -
    /// verifying a pattern actually matches its subject's type is
    /// `kyne_types`'s job, not Name Resolution's.
    fn declare_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => {}
            Pattern::Ident(name) => self.declare_local(name, DeclKind::Local),
            Pattern::Tuple { path, patterns } => {
                self.check_path(path);
                for p in patterns {
                    self.declare_pattern(p);
                }
            }
            Pattern::Struct { path, fields } => {
                self.check_path(path);
                for field in fields {
                    self.declare_local(field, DeclKind::Local);
                }
            }
        }
    }

    fn check_path(&mut self, path: &Path) {
        match path {
            Path::Ident(name) => self.check_unresolved(name),
            Path::Qualified(ty, _variant) => self.check_unresolved(ty),
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(_) => {}
            Expr::Path(path) => self.check_path(path),
            Expr::Binary { left, right, .. } => {
                self.walk_expr(left);
                self.walk_expr(right);
            }
            Expr::Unary { operand, .. } | Expr::Try { operand } => self.walk_expr(operand),
            Expr::Call { callee, args } => {
                self.walk_expr(callee);
                for arg in args {
                    self.walk_expr(arg);
                }
            }
            // A field/method NAME is not resolved here - it requires
            // knowing the base expression's type, which is kyne_types'
            // job; only the base expression is a name-resolution concern.
            Expr::Field { base, .. } => self.walk_expr(base),
            Expr::MethodCall { base, args, .. } => {
                self.walk_expr(base);
                for arg in args {
                    self.walk_expr(arg);
                }
            }
            Expr::StructLiteral { path, fields } => {
                self.check_path(path);
                for field in fields {
                    self.walk_expr(&field.value);
                }
            }
            Expr::ListLiteral(items) => {
                for item in items {
                    self.walk_expr(item);
                }
            }
            Expr::MapLiteral(entries) => {
                for entry in entries {
                    self.walk_expr(&entry.key);
                    self.walk_expr(&entry.value);
                }
            }
            Expr::If(if_stmt) => self.walk_if(if_stmt),
            Expr::Match(match_stmt) => self.walk_match(match_stmt),
        }
    }
}

// ---- Diagnostics ----
//
// Every diagnostic here uses a placeholder span (line 1, column 1) - see
// this crate's README for why: kyne_ast carries no source-span
// information at all (a scope decision made while implementing issue
// #8), so this crate cannot yet point a diagnostic at the exact
// offending location. This is a real, tracked limitation, not a
// disguised bug - see docs/adr/ADR-0007-resolver-implementation.md.

fn placeholder_span(len: usize) -> DiagSpan {
    DiagSpan::new(1, 1, len.max(1))
}

fn unresolved_name_diagnostic(file: &str, name: &str) -> Diagnostic {
    Diagnostic::error(
        "KY0601",
        format!("cannot find `{name}` in this scope"),
        file,
        placeholder_span(name.chars().count()),
        "not found",
    )
    .note("kyne_ast does not yet carry source-span information, so this diagnostic cannot point at the exact offending location in this file - see this crate's README")
    .help(format!(
        "check that `{name}` is spelled correctly and is a parameter, `let` binding, `state`/`const` field, declared type/function, or imported name"
    ))
}

fn duplicate_declaration_diagnostic(file: &str, name: &str) -> Diagnostic {
    Diagnostic::error(
        "KY0602",
        format!("the name `{name}` is declared more than once"),
        file,
        placeholder_span(name.chars().count()),
        "duplicate declaration",
    )
    .note("every declared name in the same scope must be unique")
    .help(format!("rename one of the `{name}` declarations"))
}

fn illegal_shadow_diagnostic(file: &str, name: &str, kind: DeclKind) -> Diagnostic {
    let what = match kind {
        DeclKind::State => "state field",
        DeclKind::Const => "const",
        _ => "declaration",
    };
    Diagnostic::error(
        "KY0603",
        format!("`let {name}` shadows the {what} `{name}`"),
        file,
        placeholder_span(name.chars().count()),
        "illegal shadowing",
    )
    .note("a `let` binding must not share a name with a `state` field or `const` in the same contract, per LANGUAGE_SPEC.md §4.3/§4.5")
    .help(format!("rename this local binding to something other than `{name}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyne_cst::Cst;

    fn resolve_source(source: &str) -> ResolveResult {
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected a clean parse, got: {diagnostics:?}"
        );
        let program = kyne_ast::lower(&cst);
        resolve(&program, "test.kyn")
    }

    fn codes(result: &ResolveResult) -> Vec<&str> {
        result.diagnostics.iter().map(|d| d.code()).collect()
    }

    #[test]
    fn order_independent_forward_reference_resolves() {
        // A state field referencing a struct declared *later* in the
        // same contract, and a fn calling another fn declared *below*
        // it - both legal per LANGUAGE_SPEC.md §2.1.
        let source = "\
contract C {
    state listing: Listing;

    public fn a() -> i64 {
        return b();
    }

    public fn b() -> i64 {
        return 1;
    }

    struct Listing { price: i64 }
}
";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn unresolved_name_is_reported() {
        let source = "contract C { fn f() { let x = totally_unknown_name; } }\n";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0601"]);
    }

    #[test]
    fn prelude_constructors_never_flagged_unresolved() {
        let source = "\
error E {
    X,
}

contract C {
    fn f() -> Result<bool, E> {
        let a = Some(1);
        let b = None;
        let c = Ok(true);
        return Err(E::X);
    }
}
";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn let_may_shadow_let_and_params() {
        let source = "contract C { fn f(x: i64) { let x = x + 1; let x = x + 1; } }\n";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn let_shadowing_state_is_illegal() {
        let source = "contract C { state balance: i64 = 0; fn f() { let balance = 1; } }\n";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0603"]);
    }

    #[test]
    fn let_shadowing_const_is_illegal() {
        let source = "contract C { const MAX: i64 = 100; fn f() { let MAX = 1; } }\n";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0603"]);
    }

    #[test]
    fn duplicate_top_level_declaration_is_reported() {
        let source = "struct Foo { x: i64 }\nstruct Foo { y: i64 }\n";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0602"]);
    }

    #[test]
    fn duplicate_contract_member_is_reported() {
        let source = "contract C { const A: i64 = 1; const A: i64 = 2; }\n";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0602"]);
    }

    #[test]
    fn for_loop_variable_is_scoped_to_the_loop_body() {
        let source = "\
contract C {
    fn f(owners: list<address>) {
        for owner in owners {
            let x = owner;
        }
    }
}
";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn match_pattern_bindings_are_visible_in_the_arm_body() {
        let source = "\
enum Status {
    Approved(address),
    Rejected { reason: string },
}

contract C {
    fn f(status: Status) {
        match status {
            Status::Approved(by) => { let x = by; }
            Status::Rejected { reason } => { let y = reason; }
            _ => {}
        }
    }
}
";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn pattern_binding_does_not_leak_outside_its_arm() {
        let source = "\
enum Status {
    Approved(address),
}

contract C {
    fn f(status: Status) {
        match status {
            Status::Approved(by) => {}
            _ => {}
        }
        let leaked = by;
    }
}
";
        let result = resolve_source(source);
        assert_eq!(codes(&result), vec!["KY0601"]);
    }

    #[test]
    fn imported_names_are_treated_as_resolved() {
        let source = "use collections.{List, Map};\nfn f() { let x = List; }\n";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn state_field_visible_across_every_contract_function() {
        let source = "\
contract C {
    state count: i64 = 0;

    public fn get() -> i64 {
        return count;
    }

    public fn increment() {
        count += 1;
    }
}
";
        let result = resolve_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }
}
