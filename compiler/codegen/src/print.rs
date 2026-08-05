//! RIR-to-Rust pretty-printing, per docs/COMPILER_ARCHITECTURE.md §14.
//!
//! `generate` walks a fully-lowered [`RProgram`] and emits formatted Rust
//! source text forming a `#[contract]`/`#[contractimpl]` Soroban SDK
//! crate body. Every ownership/type-mapping decision was already made by
//! `kyne_rir`; the print-time choices this module still has to make
//! (checked-arithmetic method names, the literal storage-API call shape,
//! import selection) are documented in
//! docs/adr/ADR-0012-codegen-implementation.md.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

use kyne_ast::{BinaryOp, Literal, Path, Pattern, UnaryOp, Visibility};
use kyne_rir::{
    RBlock, RConst, RContract, REnumDecl, REnumVariantShape, RErrorDecl, RExpr, RFnDecl, RMatch,
    RMatchArm, RMatchArmBody, RProgram, RStmt, RStructDecl, RType,
};

/// An internal `kyne_codegen` defect - the pretty-printer produced text
/// `rustfmt` rejected as invalid Rust, or `rustfmt` itself could not be
/// invoked. Per docs/COMPILER_ARCHITECTURE.md §14 ("no remaining semantic
/// decisions of its own"), this should never happen for RIR produced by
/// `kyne_rir`; it is not a user-facing diagnostic.
#[derive(Debug)]
pub struct CodegenError(pub String);

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CodegenError {}

struct Ctx {
    /// Every `state` field's own [`RType`], keyed by field name - needed
    /// to print an unambiguous `.get::<Symbol, T>(...)` turbofish at each
    /// `StorageGet`/`StorageMutate` call site, since a bare `.get(...)`
    /// leaves Rust nothing to infer `T` from in several of Counter/
    /// Token's own call sites.
    state_types: HashMap<String, RType>,
    /// Every contract-member function's own name - needed to distinguish
    /// a same-contract function call (`balance_of(to)`, which must
    /// become `Self::balance_of(env.clone(), to)` in the generated
    /// `impl` block, since inherent associated functions have no
    /// implicit "same object" call syntax) from a constructor call
    /// (`Ok(true)`, `Some(x)`, a `struct`/tuple-variant literal).
    local_fns: HashSet<String>,
}

/// Renders `rir` as a complete, `rustfmt`-formatted Rust source file body
/// (everything but `Cargo.toml`, deferred to issue #18 per
/// docs/adr/ADR-0012-codegen-implementation.md).
pub fn generate(rir: &RProgram) -> Result<String, CodegenError> {
    let raw = print_program(&rir.contract);
    run_rustfmt(&raw)
}

fn print_program(c: &RContract) -> String {
    let ctx = Ctx {
        state_types: c
            .state_fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect(),
        local_fns: c.functions.iter().map(|f| f.name.clone()).collect(),
    };

    let mut sdk_types: BTreeSet<&'static str> = BTreeSet::new();
    for f in &c.state_fields {
        collect_type_names(&f.ty, &mut sdk_types);
    }
    for cst in &c.consts {
        // `String`-typed consts print as `&str` (see `print_const_type`) -
        // no runtime `soroban_sdk::String` value is ever constructed for
        // them, so they must not pull in the `String` import.
        if !matches!(cst.ty, RType::String) {
            collect_type_names(&cst.ty, &mut sdk_types);
        }
    }
    for ev in &c.events {
        for p in &ev.params {
            collect_type_names(&p.ty, &mut sdk_types);
        }
    }
    for s in &c.structs {
        for f in &s.fields {
            collect_type_names(&f.ty, &mut sdk_types);
        }
    }
    for e in &c.enums {
        for v in &e.variants {
            match &v.shape {
                REnumVariantShape::Unit => {}
                REnumVariantShape::Tuple(types) => {
                    for t in types {
                        collect_type_names(t, &mut sdk_types);
                    }
                }
                REnumVariantShape::Struct(fields) => {
                    for f in fields {
                        collect_type_names(&f.ty, &mut sdk_types);
                    }
                }
            }
        }
    }
    for f in &c.functions {
        for p in &f.params {
            collect_type_names(&p.ty, &mut sdk_types);
        }
        if let Some(rt) = &f.return_type {
            collect_type_names(rt, &mut sdk_types);
        }
    }

    let mut out = String::new();
    out.push_str("#![no_std]\n\n");
    out.push_str(&print_imports(c, &sdk_types));
    out.push_str("\n\n");

    for cst in &c.consts {
        out.push_str(&print_const(cst));
        out.push('\n');
    }
    if !c.consts.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("#[contract]\npub struct {};\n", c.name));

    for s in &c.structs {
        out.push('\n');
        out.push_str(&print_struct(s));
        out.push('\n');
    }
    for e in &c.enums {
        out.push('\n');
        out.push_str(&print_enum(e));
        out.push('\n');
    }
    for e in &c.errors {
        out.push('\n');
        out.push_str(&print_error(e));
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&format!("#[contractimpl]\nimpl {} {{\n", c.name));
    for (i, f) in c.functions.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&print_fn(f, &ctx));
        out.push('\n');
    }
    out.push_str("}\n");

    out
}

fn print_imports(c: &RContract, sdk_types: &BTreeSet<&'static str>) -> String {
    let mut names: BTreeSet<&str> = BTreeSet::new();
    names.insert("contract");
    names.insert("contractimpl");
    if !c.structs.is_empty() || !c.enums.is_empty() {
        names.insert("contracttype");
    }
    if !c.errors.is_empty() {
        names.insert("contracterror");
    }
    names.insert("Env");
    names.insert("Symbol");
    for t in sdk_types {
        names.insert(t);
    }
    let joined: Vec<&str> = names.into_iter().collect();
    format!("use soroban_sdk::{{{}}};", joined.join(", "))
}

/// Kyne's primitive/parametric types, already Soroban-SDK-shaped per
/// [`kyne_rir::RType`] - this is a direct, mechanical token substitution,
/// not a mapping decision (that decision was `kyne_rir`'s, per
/// docs/adr/ADR-0011-rir-implementation.md).
fn print_type(ty: &RType) -> String {
    match ty {
        RType::Bool => "bool".to_string(),
        RType::I32 => "i32".to_string(),
        RType::I64 => "i64".to_string(),
        RType::I128 => "i128".to_string(),
        RType::U32 => "u32".to_string(),
        RType::U64 => "u64".to_string(),
        RType::U128 => "u128".to_string(),
        RType::Address => "Address".to_string(),
        RType::Symbol => "Symbol".to_string(),
        RType::String => "String".to_string(),
        RType::Bytes => "Bytes".to_string(),
        RType::FixedBytes(n) => format!("BytesN<{n}>"),
        RType::Vec(inner) => format!("Vec<{}>", print_type(inner)),
        RType::Map(k, v) => format!("Map<{}, {}>", print_type(k), print_type(v)),
        RType::Option(inner) => format!("Option<{}>", print_type(inner)),
        RType::Result(t, e) => format!("Result<{}, {}>", print_type(t), print_type(e)),
        RType::Named(name) => name.clone(),
        RType::Unit => "()".to_string(),
    }
}

/// `const` items have no `Env` to construct a runtime SDK value with, so
/// a `string`-typed `const` prints as a plain compile-time `&'static
/// str` instead of `soroban_sdk::String` - see
/// docs/adr/ADR-0012-codegen-implementation.md. Every other type used in
/// a `const` in Counter/Token's scope (`u32`, ...) is already a real
/// Rust primitive with no such gap.
fn print_const_type(ty: &RType) -> String {
    match ty {
        RType::String => "&str".to_string(),
        other => print_type(other),
    }
}

fn collect_type_names(ty: &RType, out: &mut BTreeSet<&'static str>) {
    match ty {
        RType::Address => {
            out.insert("Address");
        }
        RType::Symbol => {
            out.insert("Symbol");
        }
        RType::String => {
            out.insert("String");
        }
        RType::Bytes => {
            out.insert("Bytes");
        }
        RType::FixedBytes(_) => {
            out.insert("BytesN");
        }
        RType::Vec(inner) => {
            out.insert("Vec");
            collect_type_names(inner, out);
        }
        RType::Map(k, v) => {
            out.insert("Map");
            collect_type_names(k, out);
            collect_type_names(v, out);
        }
        RType::Option(inner) => collect_type_names(inner, out),
        RType::Result(t, e) => {
            collect_type_names(t, out);
            collect_type_names(e, out);
        }
        RType::Bool
        | RType::I32
        | RType::I64
        | RType::I128
        | RType::U32
        | RType::U64
        | RType::U128
        | RType::Named(_)
        | RType::Unit => {}
    }
}

fn print_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(text) => text.clone(),
        Literal::Bool(b) => b.to_string(),
        // `{:?}` on a `&str` produces a validly-escaped Rust string
        // literal - `Literal::Str` already holds decoded content (escape
        // sequences resolved during AST lowering), so this re-escapes it
        // for Rust rather than assuming Kyne's and Rust's escape syntax
        // already agree.
        Literal::Str(s) => format!("{s:?}"),
    }
}

fn print_path(p: &Path) -> String {
    match p {
        Path::Ident(name) => name.clone(),
        Path::Qualified(ty, variant) => format!("{ty}::{variant}"),
    }
}

fn print_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Literal(l) => print_literal(l),
        // A bare identifier is ambiguous between "bind a new variable"
        // and "match a prelude/enum unit constructor" (`None`) at the
        // `kyne_ast::Pattern` level - Kyne resolves this the same way
        // Rust itself does (a name that resolves to an in-scope unit
        // constant/variant is a constructor pattern, anything else is a
        // binding), so printing the identifier verbatim is correct for
        // every case Counter/Token exercise (`None`) - see
        // docs/adr/ADR-0012-codegen-implementation.md for the one case
        // this does not generalize to (an unqualified user-declared
        // fieldless enum variant), out of this issue's scope.
        Pattern::Ident(name) => name.clone(),
        Pattern::Tuple { path, patterns } => {
            let inner = patterns
                .iter()
                .map(print_pattern)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({inner})", print_path(path))
        }
        Pattern::Struct { path, fields } => {
            format!("{} {{ {} }}", print_path(path), fields.join(", "))
        }
    }
}

fn print_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        // Arithmetic ops are handled by `print_binary`'s checked-method
        // form, never printed as a bare infix operator - listed here
        // only so this function stays exhaustive if a caller ever needs
        // the raw token.
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
    }
}

/// `+ - * / %` MUST panic on overflow/underflow rather than silently
/// wrap, per LANGUAGE_SPEC.md §7.2 - a bare Rust `+` wraps silently in a
/// release build (`overflow-checks` defaults to off), so every
/// arithmetic operator prints as an explicit `checked_*` call instead of
/// relying on a Cargo profile setting a downstream build could omit or
/// override. Comparison/logical operators have no such concern and print
/// as ordinary Rust operators. See
/// docs/adr/ADR-0012-codegen-implementation.md.
fn print_binary(left: &RExpr, op: BinaryOp, right: &RExpr, ctx: &Ctx) -> String {
    let l = print_expr(left, ctx);
    let r = print_expr(right, ctx);
    match op {
        BinaryOp::Add => format!("({l}).checked_add({r}).expect(\"arithmetic overflow\")"),
        BinaryOp::Sub => format!("({l}).checked_sub({r}).expect(\"arithmetic overflow\")"),
        BinaryOp::Mul => format!("({l}).checked_mul({r}).expect(\"arithmetic overflow\")"),
        BinaryOp::Div => {
            format!("({l}).checked_div({r}).expect(\"division overflow or division by zero\")")
        }
        BinaryOp::Rem => {
            format!("({l}).checked_rem({r}).expect(\"division overflow or division by zero\")")
        }
        _ => format!("({l} {} {r})", print_binary_op(op)),
    }
}

/// Unary negation can overflow too (`-i32::MIN`), so it gets the same
/// `checked_*` treatment as binary arithmetic; boolean `!` cannot
/// overflow and prints as the ordinary Rust operator.
fn print_unary(op: UnaryOp, operand: &RExpr, ctx: &Ctx) -> String {
    let v = print_expr(operand, ctx);
    match op {
        UnaryOp::Neg => format!("({v}).checked_neg().expect(\"arithmetic overflow\")"),
        UnaryOp::Not => format!("(!{v})"),
    }
}

fn print_call(callee: &RExpr, args: &[RExpr], ctx: &Ctx) -> String {
    let args_str: Vec<String> = args.iter().map(|a| print_expr(a, ctx)).collect();
    if let RExpr::Path(Path::Ident(name)) = callee {
        if ctx.local_fns.contains(name) {
            let mut full_args = vec!["env.clone()".to_string()];
            full_args.extend(args_str);
            return format!("Self::{name}({})", full_args.join(", "));
        }
    }
    format!("{}({})", print_expr(callee, ctx), args_str.join(", "))
}

fn print_list_literal(items: &[RExpr], ctx: &Ctx) -> String {
    if items.is_empty() {
        return "Vec::new(&env)".to_string();
    }
    let items_str: Vec<String> = items.iter().map(|i| print_expr(i, ctx)).collect();
    format!("Vec::from_array(&env, [{}])", items_str.join(", "))
}

fn print_map_literal(entries: &[kyne_rir::RMapEntry], ctx: &Ctx) -> String {
    if entries.is_empty() {
        return "Map::new(&env)".to_string();
    }
    let entries_str: Vec<String> = entries
        .iter()
        .map(|e| {
            format!(
                "({}, {})",
                print_expr(&e.key, ctx),
                print_expr(&e.value, ctx)
            )
        })
        .collect();
    format!("Map::from_array(&env, [{}])", entries_str.join(", "))
}

/// The literal Rust realization of [`RExpr::StorageGet`] /
/// [`RStmt::StorageMutate`]'s read half - `Symbol::new`, not the shorter
/// `symbol_short!` macro, and an explicit `::<Symbol, T>` turbofish so
/// `T` never needs inferring from a possibly-ambiguous call site. See
/// docs/adr/ADR-0011-rir-implementation.md.
fn print_storage_get(key: &str, default: Option<&RExpr>, ctx: &Ctx) -> String {
    let ty = ctx
        .state_types
        .get(key)
        .map(print_type)
        .unwrap_or_else(|| "_".to_string());
    let base =
        format!("env.storage().persistent().get::<Symbol, {ty}>(&Symbol::new(&env, \"{key}\"))");
    match default {
        Some(d) => format!("{base}.unwrap_or({})", print_expr(d, ctx)),
        // No default means kyne_semantics's definite-assignment analysis
        // guarantees this field is never read before `init` writes it -
        // `.unwrap()` is a runtime safety net for that guarantee, not a
        // place a `None` is ever expected to actually occur.
        None => format!("{base}.unwrap()"),
    }
}

fn print_expr(expr: &RExpr, ctx: &Ctx) -> String {
    match expr {
        RExpr::Literal(l) => print_literal(l),
        RExpr::Var(name) => name.clone(),
        RExpr::StorageGet { key, default } => print_storage_get(key, default.as_deref(), ctx),
        RExpr::Path(p) => print_path(p),
        RExpr::Binary { left, op, right } => print_binary(left, *op, right, ctx),
        RExpr::Unary { op, operand } => print_unary(*op, operand, ctx),
        RExpr::Call { callee, args } => print_call(callee, args, ctx),
        RExpr::Field { base, name } => format!("{}.{name}", print_expr(base, ctx)),
        RExpr::MethodCall { base, method, args } => {
            let args_str: Vec<String> = args.iter().map(|a| print_expr(a, ctx)).collect();
            format!(
                "{}.{method}({})",
                print_expr(base, ctx),
                args_str.join(", ")
            )
        }
        RExpr::StructLiteral { path, fields } => {
            let fields_str: Vec<String> = fields
                .iter()
                .map(|f| format!("{}: {}", f.name, print_expr(&f.value, ctx)))
                .collect();
            format!("{} {{ {} }}", print_path(path), fields_str.join(", "))
        }
        RExpr::ListLiteral(items) => print_list_literal(items, ctx),
        RExpr::MapLiteral(entries) => print_map_literal(entries, ctx),
        RExpr::Match(m) => print_match_construct(m, ctx),
        RExpr::Return(None) => "return".to_string(),
        RExpr::Return(Some(e)) => format!("return {}", print_expr(e, ctx)),
        // `throw <expr>;` is specified to desugar to `return Err(<expr>);`
        // verbatim, per LANGUAGE_SPEC.md §8.7 - not a mapping decision
        // left open to this crate, an already-specified 1:1 substitution.
        RExpr::Throw(e) => format!("return Err({})", print_expr(e, ctx)),
        RExpr::Break => "break".to_string(),
        RExpr::Continue => "continue".to_string(),
    }
}

fn print_stmt(stmt: &RStmt, ctx: &Ctx) -> String {
    match stmt {
        RStmt::Let { name, ty, value } => match ty {
            Some(t) => format!(
                "let {name}: {} = {};",
                print_type(t),
                print_expr(value, ctx)
            ),
            None => format!("let {name} = {};", print_expr(value, ctx)),
        },
        RStmt::Assign { target, value } => {
            format!("{} = {};", print_expr(target, ctx), print_expr(value, ctx))
        }
        RStmt::StorageSet { key, value } => format!(
            "env.storage().persistent().set(&Symbol::new(&env, \"{key}\"), &({}));",
            print_expr(value, ctx)
        ),
        RStmt::StorageMutate {
            key,
            default,
            method,
            args,
        } => {
            let read = print_storage_get(key, default.as_deref(), ctx);
            let args_str: Vec<String> = args.iter().map(|a| print_expr(a, ctx)).collect();
            format!(
                "let mut {key} = {read};\n{key}.{method}({});\nenv.storage().persistent().set(&Symbol::new(&env, \"{key}\"), &{key});",
                args_str.join(", ")
            )
        }
        RStmt::RequireAuth(e) => format!("{}.require_auth();", print_expr(e, ctx)),
        RStmt::Emit { event, args } => {
            let args_str: Vec<String> = args.iter().map(|a| print_expr(a, ctx)).collect();
            let data = match args_str.len() {
                0 => "()".to_string(),
                1 => format!("({},)", args_str[0]),
                _ => format!("({})", args_str.join(", ")),
            };
            format!("env.events().publish((Symbol::new(&env, \"{event}\"),), {data});")
        }
        RStmt::Match(m) => print_match_construct(m, ctx),
        RStmt::For {
            var,
            iterable,
            body,
        } => format!(
            "for {var} in {} {{\n{}\n}}",
            print_expr(iterable, ctx),
            print_block(body, ctx)
        ),
        RStmt::While { condition, body } => format!(
            "while {} {{\n{}\n}}",
            print_expr(condition, ctx),
            print_block(body, ctx)
        ),
        RStmt::Expr(e) => format!("{};", print_expr(e, ctx)),
    }
}

fn print_block(b: &RBlock, ctx: &Ctx) -> String {
    b.statements
        .iter()
        .map(|s| print_stmt(s, ctx))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A `match` whose shape is exactly `HExpr`/`RExpr`'s canonical `if`
/// desugaring (`kyne_hir::lower::lower_if_to_match`: a `true` arm and a
/// wildcard arm, no guards) is reconstructed as an `if`/`else` here
/// rather than printed as a literal `match cond { true => ..., _ => ...
/// }` - both are equally correct Rust, but the latter reads as unusual
/// and would draw a `clippy::match_bool` complaint from any reviewer
/// running clippy on the output. This is a pure print-time syntactic
/// simplification (same runtime meaning), not a new semantic decision -
/// see docs/adr/ADR-0012-codegen-implementation.md.
fn print_match_construct(m: &RMatch, ctx: &Ctx) -> String {
    if let Some(if_text) = try_print_as_if(m, ctx) {
        return if_text;
    }
    let subject = print_expr(&m.subject, ctx);
    let arms: Vec<String> = m.arms.iter().map(|arm| print_arm(arm, ctx)).collect();
    format!("match {subject} {{\n{}\n}}", arms.join("\n"))
}

fn try_print_as_if(m: &RMatch, ctx: &Ctx) -> Option<String> {
    let [then_arm, else_arm] = m.arms.as_slice() else {
        return None;
    };
    if then_arm.guard.is_some() || else_arm.guard.is_some() {
        return None;
    }
    if !matches!(then_arm.pattern, Pattern::Literal(Literal::Bool(true))) {
        return None;
    }
    if !matches!(else_arm.pattern, Pattern::Wildcard) {
        return None;
    }
    let cond = print_expr(&m.subject, ctx);
    let then_body = print_arm_body(&then_arm.body, ctx);
    let else_is_empty =
        matches!(&else_arm.body, RMatchArmBody::Block(b) if b.statements.is_empty());
    if else_is_empty {
        Some(format!("if {cond} {{\n{then_body}\n}}"))
    } else {
        let else_body = print_arm_body(&else_arm.body, ctx);
        Some(format!(
            "if {cond} {{\n{then_body}\n}} else {{\n{else_body}\n}}"
        ))
    }
}

fn print_arm_body(body: &RMatchArmBody, ctx: &Ctx) -> String {
    match body {
        RMatchArmBody::Block(b) => print_block(b, ctx),
        RMatchArmBody::Expr(e) => print_expr(e, ctx),
    }
}

fn print_arm(arm: &RMatchArm, ctx: &Ctx) -> String {
    let pat = print_pattern(&arm.pattern);
    let guard = arm
        .guard
        .as_ref()
        .map(|g| format!(" if {}", print_expr(g, ctx)))
        .unwrap_or_default();
    match &arm.body {
        RMatchArmBody::Block(b) => format!("{pat}{guard} => {{\n{}\n}}", print_block(b, ctx)),
        RMatchArmBody::Expr(e) => format!("{pat}{guard} => {},", print_expr(e, ctx)),
    }
}

fn print_const(c: &RConst) -> String {
    format!(
        "pub const {}: {} = {};",
        c.name,
        print_const_type(&c.ty),
        // A `const` initializer can never reference `state`/`env` (see
        // LANGUAGE_SPEC.md §9.3), so the ordinary expression printer is
        // safe here without a `Ctx` - `Ctx` only exists to carry
        // `state`/local-function information a `const` value can never
        // observe.
        print_expr(
            &c.value,
            &Ctx {
                state_types: HashMap::new(),
                local_fns: HashSet::new()
            }
        )
    )
}

fn print_error(e: &RErrorDecl) -> String {
    let variants: Vec<String> = e
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| format!("    {v} = {},", i + 1))
        .collect();
    format!(
        "#[contracterror]\n#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]\n#[repr(u32)]\npub enum {} {{\n{}\n}}",
        e.name,
        variants.join("\n")
    )
}

fn print_struct(s: &RStructDecl) -> String {
    let fields: Vec<String> = s
        .fields
        .iter()
        .map(|f| format!("    pub {}: {},", f.name, print_type(&f.ty)))
        .collect();
    format!(
        "#[contracttype]\n#[derive(Clone, Debug, Eq, PartialEq)]\npub struct {} {{\n{}\n}}",
        s.name,
        fields.join("\n")
    )
}

fn print_enum(e: &REnumDecl) -> String {
    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| match &v.shape {
            REnumVariantShape::Unit => format!("    {},", v.name),
            REnumVariantShape::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(print_type).collect();
                format!("    {}({}),", v.name, inner.join(", "))
            }
            REnumVariantShape::Struct(fields) => {
                let inner: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, print_type(&f.ty)))
                    .collect();
                format!("    {} {{ {} }},", v.name, inner.join(", "))
            }
        })
        .collect();
    format!(
        "#[contracttype]\n#[derive(Clone, Debug, Eq, PartialEq)]\npub enum {} {{\n{}\n}}",
        e.name,
        variants.join("\n")
    )
}

fn print_fn(f: &RFnDecl, ctx: &Ctx) -> String {
    let vis = match f.visibility {
        Visibility::Public => "pub ",
        Visibility::Internal => "pub(crate) ",
        Visibility::Private => "",
    };
    // Every real Soroban contract function needs an `Env` handle but
    // Kyne source never declares one - `kyne_rir` deliberately leaves
    // prepending it to this stage, per
    // docs/adr/ADR-0011-rir-implementation.md.
    let mut params = vec!["env: Env".to_string()];
    params.extend(
        f.params
            .iter()
            .map(|p| format!("{}: {}", p.name, print_type(&p.ty))),
    );
    let ret = match &f.return_type {
        Some(t) if !matches!(t, RType::Unit) => format!(" -> {}", print_type(t)),
        _ => String::new(),
    };
    format!(
        "{vis}fn {}({}){ret} {{\n{}\n}}",
        f.name,
        params.join(", "),
        print_block(&f.body, ctx)
    )
}

/// Runs raw generated Rust text through the real `rustfmt` binary - per
/// docs/COMPILER_ARCHITECTURE.md §14, generated Rust MUST NOT ever be
/// presented in unformatted form. This shells out to the system
/// `rustfmt` rather than depending on a formatting crate, consistent
/// with this workspace having no dependencies outside its own members
/// (see the root `Cargo.toml`) - `rustfmt` is a system tool invocation,
/// the same way `kyne build`'s later stages invoke `cargo`/Soroban CLI
/// tooling directly rather than linking against them.
fn run_rustfmt(source: &str) -> Result<String, CodegenError> {
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg("--emit")
        .arg("stdout")
        .arg("--quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CodegenError(format!("failed to spawn rustfmt: {e}")))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| CodegenError("failed to open rustfmt stdin".to_string()))?;
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| CodegenError(format!("failed to write to rustfmt stdin: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| CodegenError(format!("failed to read rustfmt output: {e}")))?;
    if !output.status.success() {
        return Err(CodegenError(format!(
            "rustfmt rejected generated Rust (this is a kyne_codegen bug, not a user error):\n{}\n--- unformatted source ---\n{source}",
            String::from_utf8_lossy(&output.stderr),
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| CodegenError(format!("rustfmt produced non-UTF-8 output: {e}")))
}
