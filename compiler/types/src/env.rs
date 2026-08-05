//! Builds the type environment every declared name checks against, by
//! scanning the resolved `Program` once up front. Kyne allows at most one
//! `contract` per project, so module-level and contract-member
//! declarations are merged into one flat environment - equivalent, for a
//! single-file, single-contract project, to `kyne_resolver`'s own
//! contract-then-module lookup order.

use std::collections::HashMap;

use kyne_ast::{ContractMember, EnumVariantShape, FnDecl, Program, TopLevelDecl};

use crate::ty::{lower_type, Ty};

#[derive(Debug, Clone)]
pub struct FnSig {
    pub params: Vec<Ty>,
    pub return_ty: Ty,
}

#[derive(Debug, Clone)]
pub enum EnumVariantTys {
    Unit,
    Tuple(Vec<Ty>),
    Struct(Vec<(String, Ty)>),
}

#[derive(Debug, Default)]
pub struct TypeEnv {
    pub structs: HashMap<String, Vec<(String, Ty)>>,
    pub enums: HashMap<String, HashMap<String, EnumVariantTys>>,
    /// Error name -> its data-less variant names.
    pub errors: HashMap<String, Vec<String>>,
    /// Event name -> its positional parameter types.
    pub events: HashMap<String, Vec<Ty>>,
    pub consts: HashMap<String, Ty>,
    pub state: HashMap<String, Ty>,
    pub functions: HashMap<String, FnSig>,
}

impl TypeEnv {
    pub fn build(program: &Program, on_error: &mut impl FnMut(String)) -> TypeEnv {
        let mut env = TypeEnv::default();
        for item in &program.items {
            match item {
                TopLevelDecl::Contract(c) => {
                    for member in &c.members {
                        env.add_contract_member(member, on_error);
                    }
                }
                TopLevelDecl::Struct(s) => {
                    env.structs
                        .insert(s.name.clone(), fields(&s.fields, on_error));
                }
                TopLevelDecl::Enum(e) => {
                    env.enums
                        .insert(e.name.clone(), enum_variants(&e.variants, on_error));
                }
                TopLevelDecl::Error(e) => {
                    env.errors.insert(e.name.clone(), e.variants.clone());
                }
                TopLevelDecl::Event(e) => {
                    env.events.insert(
                        e.name.clone(),
                        e.params
                            .iter()
                            .map(|p| lower_type(&p.ty, on_error))
                            .collect(),
                    );
                }
                TopLevelDecl::Const(c) => {
                    env.consts
                        .insert(c.name.clone(), lower_type(&c.ty, on_error));
                }
                TopLevelDecl::Fn(f) => {
                    env.functions.insert(f.name.clone(), fn_sig(f, on_error));
                }
            }
        }
        env
    }

    fn add_contract_member(&mut self, member: &ContractMember, on_error: &mut impl FnMut(String)) {
        match member {
            ContractMember::State(s) => {
                self.state
                    .insert(s.name.clone(), lower_type(&s.ty, on_error));
            }
            ContractMember::Const(c) => {
                self.consts
                    .insert(c.name.clone(), lower_type(&c.ty, on_error));
            }
            ContractMember::Error(e) => {
                self.errors.insert(e.name.clone(), e.variants.clone());
            }
            ContractMember::Event(e) => {
                self.events.insert(
                    e.name.clone(),
                    e.params
                        .iter()
                        .map(|p| lower_type(&p.ty, on_error))
                        .collect(),
                );
            }
            ContractMember::Struct(s) => {
                self.structs
                    .insert(s.name.clone(), fields(&s.fields, on_error));
            }
            ContractMember::Enum(e) => {
                self.enums
                    .insert(e.name.clone(), enum_variants(&e.variants, on_error));
            }
            ContractMember::Fn(f) => {
                self.functions.insert(f.name.clone(), fn_sig(f, on_error));
            }
        }
    }
}

fn fields(fields: &[kyne_ast::Field], on_error: &mut impl FnMut(String)) -> Vec<(String, Ty)> {
    fields
        .iter()
        .map(|f| (f.name.clone(), lower_type(&f.ty, on_error)))
        .collect()
}

fn enum_variants(
    variants: &[kyne_ast::EnumVariant],
    on_error: &mut impl FnMut(String),
) -> HashMap<String, EnumVariantTys> {
    variants
        .iter()
        .map(|v| {
            let shape = match &v.shape {
                EnumVariantShape::Unit => EnumVariantTys::Unit,
                EnumVariantShape::Tuple(types) => {
                    EnumVariantTys::Tuple(types.iter().map(|t| lower_type(t, on_error)).collect())
                }
                EnumVariantShape::Struct(fs) => EnumVariantTys::Struct(fields(fs, on_error)),
            };
            (v.name.clone(), shape)
        })
        .collect()
}

fn fn_sig(f: &FnDecl, on_error: &mut impl FnMut(String)) -> FnSig {
    FnSig {
        params: f
            .params
            .iter()
            .map(|p| lower_type(&p.ty, on_error))
            .collect(),
        return_ty: f
            .return_type
            .as_ref()
            .map(|t| lower_type(t, on_error))
            .unwrap_or(Ty::Unit),
    }
}
