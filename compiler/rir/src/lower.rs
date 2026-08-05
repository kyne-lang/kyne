//! HIR-to-RIR lowering: the Ownership/Memory/Storage Planner work, per
//! docs/MEMORY_MODEL.md §21, all realized in this one pass.

use std::collections::{HashMap, HashSet};

use kyne_ast::{AssignOp, BinaryOp, Path, Type};
use kyne_hir::{
    HBlock, HConstDecl, HContractMember, HExpr, HFnDecl, HMatch, HMatchArm, HMatchArmBody,
    HProgram, HStmt, HTopLevelDecl,
};

use crate::nodes::*;

struct Ctx {
    state_names: HashSet<String>,
    state_defaults: HashMap<String, RExpr>,
}

/// Lowers a (desugared) [`HProgram`] into RIR, per
/// docs/COMPILER_ARCHITECTURE.md §12. Takes the first `ContractDecl`
/// found - `kyne_semantics`'s current (v0.2) scope does not yet enforce
/// "at most one contract" itself (deferred to its own Phase 2 follow-up,
/// per docs/adr/ADR-0009-semantics-implementation.md's sibling issue
/// #14), so this stage tolerates, rather than assumes, that invariant:
/// a file with no contract lowers to an empty [`RContract`] instead of
/// panicking.
pub fn lower(program: &HProgram) -> RProgram {
    let contract = program.items.iter().find_map(|item| match item {
        HTopLevelDecl::Contract(c) => Some(c),
        _ => None,
    });

    let Some(contract) = contract else {
        return RProgram {
            contract: RContract {
                name: String::new(),
                state_fields: Vec::new(),
                consts: Vec::new(),
                errors: Vec::new(),
                events: Vec::new(),
                structs: Vec::new(),
                enums: Vec::new(),
                functions: Vec::new(),
            },
        };
    };

    let state_names: HashSet<String> = contract
        .members
        .iter()
        .filter_map(|m| match m {
            HContractMember::State(s) => Some(s.name.clone()),
            _ => None,
        })
        .collect();

    // Defaults are lowered against an empty defaults map (a default
    // expression can never itself reference a `state` field's storage
    // value - LANGUAGE_SPEC.md §4.4 requires it to be a literal), then
    // folded into the real `Ctx` used for everything else.
    let bootstrap_ctx = Ctx {
        state_names: state_names.clone(),
        state_defaults: HashMap::new(),
    };
    let state_defaults: HashMap<String, RExpr> = contract
        .members
        .iter()
        .filter_map(|m| match m {
            HContractMember::State(s) => {
                let default = match &s.init {
                    Some(init) => Some(lower_expr(init, &bootstrap_ctx)),
                    None if has_implicit_default(&s.ty) => Some(empty_literal_for(&s.ty)),
                    None => None,
                };
                default.map(|d| (s.name.clone(), d))
            }
            _ => None,
        })
        .collect();

    let ctx = Ctx {
        state_names,
        state_defaults,
    };

    let mut state_fields = Vec::new();
    let mut consts = Vec::new();
    let mut errors = Vec::new();
    let mut events = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();

    for member in &contract.members {
        match member {
            HContractMember::State(s) => state_fields.push(RStateField {
                name: s.name.clone(),
                ty: lower_type(&s.ty),
                key: s.name.clone(),
            }),
            HContractMember::Const(c) => consts.push(lower_const(c, &ctx)),
            HContractMember::Error(e) => errors.push(RErrorDecl {
                name: e.name.clone(),
                variants: e.variants.clone(),
            }),
            HContractMember::Event(e) => events.push(REventDecl {
                name: e.name.clone(),
                params: e.params.iter().map(lower_param).collect(),
            }),
            HContractMember::Struct(s) => structs.push(RStructDecl {
                name: s.name.clone(),
                fields: s.fields.iter().map(lower_field).collect(),
            }),
            HContractMember::Enum(e) => enums.push(lower_enum(e)),
            HContractMember::Fn(f) => functions.push(lower_fn(f, &ctx)),
        }
    }

    RProgram {
        contract: RContract {
            name: contract.name.clone(),
            state_fields,
            consts,
            errors,
            events,
            structs,
            enums,
            functions,
        },
    }
}

fn has_implicit_default(ty: &Type) -> bool {
    matches!(ty, Type::List(_) | Type::Map(_, _))
}

fn empty_literal_for(ty: &Type) -> RExpr {
    match ty {
        Type::List(_) => RExpr::ListLiteral(Vec::new()),
        Type::Map(_, _) => RExpr::MapLiteral(Vec::new()),
        _ => RExpr::ListLiteral(Vec::new()),
    }
}

fn lower_const(c: &HConstDecl, ctx: &Ctx) -> RConst {
    RConst {
        name: c.name.clone(),
        ty: lower_type(&c.ty),
        value: lower_expr(&c.value, ctx),
    }
}

fn lower_param(p: &kyne_ast::Param) -> RParam {
    RParam {
        name: p.name.clone(),
        ty: lower_type(&p.ty),
    }
}

fn lower_field(f: &kyne_ast::Field) -> RField {
    RField {
        name: f.name.clone(),
        ty: lower_type(&f.ty),
    }
}

fn lower_enum(e: &kyne_hir::HEnumDecl) -> REnumDecl {
    REnumDecl {
        name: e.name.clone(),
        variants: e
            .variants
            .iter()
            .map(|v| REnumVariant {
                name: v.name.clone(),
                shape: match &v.shape {
                    kyne_ast::EnumVariantShape::Unit => REnumVariantShape::Unit,
                    kyne_ast::EnumVariantShape::Tuple(types) => {
                        REnumVariantShape::Tuple(types.iter().map(lower_type).collect())
                    }
                    kyne_ast::EnumVariantShape::Struct(fields) => {
                        REnumVariantShape::Struct(fields.iter().map(lower_field).collect())
                    }
                },
            })
            .collect(),
    }
}

/// Maps every Kyne primitive and intrinsic parametric type onto its
/// Soroban SDK Rust equivalent - see
/// docs/adr/ADR-0011-rir-implementation.md for the full table.
fn lower_type(ty: &Type) -> RType {
    match ty {
        Type::Named(name) => match name.as_str() {
            "bool" => RType::Bool,
            "i32" => RType::I32,
            "i64" => RType::I64,
            "i128" => RType::I128,
            "u32" => RType::U32,
            "u64" => RType::U64,
            "u128" => RType::U128,
            "address" => RType::Address,
            "symbol" => RType::Symbol,
            "string" => RType::String,
            "bytes" => RType::Bytes,
            other => RType::Named(other.to_string()),
        },
        Type::List(inner) => RType::Vec(Box::new(lower_type(inner))),
        Type::Map(k, v) => RType::Map(Box::new(lower_type(k)), Box::new(lower_type(v))),
        Type::Bytes(text) => RType::FixedBytes(text.parse().unwrap_or(0)),
        Type::Option(inner) => RType::Option(Box::new(lower_type(inner))),
        Type::Result(t, e) => RType::Result(Box::new(lower_type(t)), Box::new(lower_type(e))),
    }
}

fn lower_fn(f: &HFnDecl, ctx: &Ctx) -> RFnDecl {
    RFnDecl {
        visibility: f.visibility,
        name: f.name.clone(),
        params: f.params.iter().map(lower_param).collect(),
        return_type: f.return_type.as_ref().map(lower_type),
        body: lower_block(&f.body, ctx),
    }
}

fn lower_block(block: &HBlock, ctx: &Ctx) -> RBlock {
    RBlock {
        statements: block
            .statements
            .iter()
            .map(|s| lower_stmt(s, ctx))
            .collect(),
    }
}

fn assign_op_to_binary(op: AssignOp) -> Option<BinaryOp> {
    match op {
        AssignOp::Assign => None,
        AssignOp::Add => Some(BinaryOp::Add),
        AssignOp::Sub => Some(BinaryOp::Sub),
        AssignOp::Mul => Some(BinaryOp::Mul),
        AssignOp::Div => Some(BinaryOp::Div),
        AssignOp::Rem => Some(BinaryOp::Rem),
    }
}

fn lower_stmt(stmt: &HStmt, ctx: &Ctx) -> RStmt {
    match stmt {
        HStmt::Let { name, ty, value } => RStmt::Let {
            name: name.clone(),
            ty: ty.as_ref().map(lower_type),
            value: lower_expr(value, ctx),
        },
        HStmt::Assign { target, op, value } => lower_assign(target, *op, value, ctx),
        HStmt::Auth(e) => RStmt::RequireAuth(lower_expr(e, ctx)),
        HStmt::Emit { event, args } => RStmt::Emit {
            event: event.clone(),
            args: args.iter().map(|a| lower_expr(a, ctx)).collect(),
        },
        HStmt::Match(m) => RStmt::Match(lower_match(m, ctx)),
        HStmt::For {
            var,
            iterable,
            body,
        } => RStmt::For {
            var: var.clone(),
            iterable: lower_expr(iterable, ctx),
            body: lower_block(body, ctx),
        },
        HStmt::While { condition, body } => RStmt::While {
            condition: lower_expr(condition, ctx),
            body: lower_block(body, ctx),
        },
        HStmt::Expr(e) => RStmt::Expr(lower_expr(e, ctx)),
    }
}

/// A state-field assignment target has no Rust place to apply `+=`/etc.
/// to directly, so a compound op is expanded into an explicit
/// read-modify-write here: `count += 1` becomes a storage write of
/// `<storage read of count> + 1`. A non-`state` target keeps its
/// compound op expanded the same way for uniformity (still valid, just
/// marginally less idiomatic than a native `+=` would be) - this
/// crate's scope (Counter/Token) never actually exercises a non-`state`
/// assignment target, so that path is implemented for correctness but
/// untested against the canonical examples specifically.
fn lower_assign(target: &HExpr, op: AssignOp, value: &HExpr, ctx: &Ctx) -> RStmt {
    let is_state =
        matches!(target, HExpr::Path(Path::Ident(name)) if ctx.state_names.contains(name));
    let lowered_value = lower_expr(value, ctx);
    let final_value = match assign_op_to_binary(op) {
        None => lowered_value,
        Some(bin_op) => RExpr::Binary {
            left: Box::new(lower_expr(target, ctx)),
            op: bin_op,
            right: Box::new(lowered_value),
        },
    };
    if is_state {
        let HExpr::Path(Path::Ident(name)) = target else {
            unreachable!("checked above")
        };
        RStmt::StorageSet {
            key: name.clone(),
            value: final_value,
        }
    } else {
        RStmt::Assign {
            target: lower_expr(target, ctx),
            value: final_value,
        }
    }
}

fn lower_match(m: &HMatch, ctx: &Ctx) -> RMatch {
    RMatch {
        subject: lower_expr(&m.subject, ctx),
        arms: m.arms.iter().map(|a| lower_match_arm(a, ctx)).collect(),
    }
}

fn lower_match_arm(arm: &HMatchArm, ctx: &Ctx) -> RMatchArm {
    RMatchArm {
        pattern: arm.pattern.clone(),
        guard: arm.guard.as_ref().map(|g| lower_expr(g, ctx)),
        body: match &arm.body {
            HMatchArmBody::Block(b) => RMatchArmBody::Block(lower_block(b, ctx)),
            HMatchArmBody::Expr(e) => RMatchArmBody::Expr(lower_expr(e, ctx)),
        },
    }
}

fn lower_expr(expr: &HExpr, ctx: &Ctx) -> RExpr {
    match expr {
        HExpr::Literal(l) => RExpr::Literal(l.clone()),
        HExpr::Path(Path::Ident(name)) if ctx.state_names.contains(name) => RExpr::StorageGet {
            key: name.clone(),
            default: ctx.state_defaults.get(name).cloned().map(Box::new),
        },
        HExpr::Path(p) => RExpr::Path(p.clone()),
        HExpr::Binary { left, op, right } => RExpr::Binary {
            left: Box::new(lower_expr(left, ctx)),
            op: *op,
            right: Box::new(lower_expr(right, ctx)),
        },
        HExpr::Unary { op, operand } => RExpr::Unary {
            op: *op,
            operand: Box::new(lower_expr(operand, ctx)),
        },
        HExpr::Call { callee, args } => RExpr::Call {
            callee: Box::new(lower_expr(callee, ctx)),
            args: args.iter().map(|a| lower_expr(a, ctx)).collect(),
        },
        HExpr::Field { base, name } => RExpr::Field {
            base: Box::new(lower_expr(base, ctx)),
            name: name.clone(),
        },
        HExpr::MethodCall { base, method, args } => RExpr::MethodCall {
            base: Box::new(lower_expr(base, ctx)),
            method: method.clone(),
            args: args.iter().map(|a| lower_expr(a, ctx)).collect(),
        },
        HExpr::StructLiteral { path, fields } => RExpr::StructLiteral {
            path: path.clone(),
            fields: fields
                .iter()
                .map(|f| RFieldInit {
                    name: f.name.clone(),
                    value: lower_expr(&f.value, ctx),
                })
                .collect(),
        },
        HExpr::ListLiteral(items) => {
            RExpr::ListLiteral(items.iter().map(|i| lower_expr(i, ctx)).collect())
        }
        HExpr::MapLiteral(entries) => RExpr::MapLiteral(
            entries
                .iter()
                .map(|e| RMapEntry {
                    key: lower_expr(&e.key, ctx),
                    value: lower_expr(&e.value, ctx),
                })
                .collect(),
        ),
        HExpr::Match(m) => RExpr::Match(Box::new(lower_match(m, ctx))),
        HExpr::Return(e) => RExpr::Return(e.as_ref().map(|e| Box::new(lower_expr(e, ctx)))),
        HExpr::Throw(e) => RExpr::Throw(Box::new(lower_expr(e, ctx))),
        HExpr::Break => RExpr::Break,
        HExpr::Continue => RExpr::Continue,
    }
}
