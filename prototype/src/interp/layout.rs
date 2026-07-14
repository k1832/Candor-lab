//! Memory layout (design 0001 §4.2 model). Sizes, alignments and field offsets
//! for every runtime type. Field order is declared order; alignment is natural;
//! nothing is reordered. See the module docs on `interp/mod.rs`.

use std::collections::HashMap;

use crate::resolve::Items;
use crate::token::ScalarTy;
use crate::types::{ArrayLen, ItemEnv, Type};

use super::mem::round_up;

/// `Box T` is `{ ptr: u64 @0, ctx: u64 @8, vt: u64 @16 }` (§6.2).
pub const BOX_SIZE: u64 = 24;
/// Tag width prefixing every enum payload.
pub const ENUM_TAG: u64 = 8;

pub struct Layout<'a> {
    pub items: &'a Items,
    /// Integer-valued `static`s, for named array lengths.
    pub consts: &'a HashMap<String, u64>,
}

impl<'a> Layout<'a> {
    pub fn scalar_size(s: ScalarTy) -> u64 {
        match s {
            ScalarTy::Unit => 0,
            ScalarTy::Bool | ScalarTy::I8 | ScalarTy::U8 => 1,
            ScalarTy::I16 | ScalarTy::U16 => 2,
            ScalarTy::I32 | ScalarTy::U32 | ScalarTy::F32 => 4,
            ScalarTy::I64
            | ScalarTy::U64
            | ScalarTy::Isize
            | ScalarTy::Usize
            | ScalarTy::F64 => 8,
        }
    }

    pub fn array_len(&self, len: &ArrayLen) -> u64 {
        match len {
            ArrayLen::Lit(n) => *n,
            ArrayLen::Named(n) => *self.consts.get(n).unwrap_or(&0),
            ArrayLen::Unknown => 0,
        }
    }

    pub fn align_of(&self, ty: &Type) -> u64 {
        match ty {
            Type::Scalar(s) => Self::scalar_size(*s).max(1),
            Type::IntLit => 8,
            Type::RawPtr(_) | Type::FnPtr(_) | Type::Borrow(_) | Type::BorrowMut(_) => 8,
            Type::Slice(_) | Type::SliceMut(_) | Type::Str => 8,
            Type::Box(_) | Type::BoxResult(_) => 8,
            Type::Array(e, _) => self.align_of(e),
            Type::Named(n) => {
                if let Some(s) = self.items.lookup_struct(n) {
                    s.fields.iter().map(|(_, t)| self.align_of(t)).max().unwrap_or(1)
                } else {
                    8 // enum: tag is u64-aligned
                }
            }
            Type::Never | Type::Error => 1,
            // Compiler-known std `Vec[T]` is `{ buf, len, cap, ctx, vt }` — five
            // `u64` words, 8-aligned, independent of the element type.
            Type::App(n, _) if n == "Vec" || n == "Map" => 8,
            Type::Param(_) | Type::App(_, _) | Type::Proj(_, _) => {
                unreachable!("generic types are monomorphized before interpretation")
            }
        }
    }

    pub fn size_of(&self, ty: &Type) -> u64 {
        match ty {
            Type::Scalar(s) => Self::scalar_size(*s),
            Type::IntLit => 8,
            Type::RawPtr(_) | Type::FnPtr(_) | Type::Borrow(_) | Type::BorrowMut(_) => 8,
            Type::Slice(_) | Type::SliceMut(_) | Type::Str => 16,
            Type::Box(_) => BOX_SIZE,
            Type::BoxResult(t) => self.enum_size(&self.box_result_variants(t)),
            Type::Array(e, len) => {
                let stride = round_up(self.size_of(e), self.align_of(e));
                stride * self.array_len(len)
            }
            Type::Named(n) => {
                if self.items.lookup_struct(n).is_some() {
                    let (_, size, _) = self.struct_layout(n);
                    size
                } else if let Some(e) = self.items.lookup_enum(n) {
                    let vs: Vec<Vec<Type>> =
                        e.variants.iter().map(|v| v.payload.clone()).collect();
                    self.enum_size(&vs)
                } else {
                    0
                }
            }
            Type::Never | Type::Error => 0,
            // Compiler-known std `Vec[T]`: `buf`+`len`+`cap`+`ctx`+`vt` = 5×8 = 40.
            Type::App(n, _) if n == "Vec" || n == "Map" => 40,
            Type::Param(_) | Type::App(_, _) | Type::Proj(_, _) => {
                unreachable!("generic types are monomorphized before interpretation")
            }
        }
    }

    /// Struct field offsets, total size (padded to align), and alignment.
    pub fn struct_layout(&self, name: &str) -> (Vec<(String, Type, u64)>, u64, u64) {
        let s = match self.items.lookup_struct(name) {
            Some(s) => s,
            None => return (Vec::new(), 0, 1),
        };
        self.lay_fields(&s.fields)
    }

    /// Lay out an ordered field list (also used for enum payloads).
    pub fn lay_fields(&self, fields: &[(String, Type)]) -> (Vec<(String, Type, u64)>, u64, u64) {
        let mut off = 0u64;
        let mut align = 1u64;
        let mut placed = Vec::new();
        for (name, ty) in fields {
            let a = self.align_of(ty);
            align = align.max(a);
            off = round_up(off, a);
            placed.push((name.clone(), ty.clone(), off));
            off += self.size_of(ty);
        }
        let size = round_up(off, align);
        (placed, size, align)
    }

    pub fn field_offset(&self, struct_name: &str, field: &str) -> Option<(Type, u64)> {
        let (fields, _, _) = self.struct_layout(struct_name);
        fields
            .into_iter()
            .find(|(n, _, _)| n == field)
            .map(|(_, t, o)| (t, o))
    }

    // ---- enums ----

    fn box_result_variants(&self, t: &Type) -> Vec<Vec<Type>> {
        vec![vec![Type::Box(Box::new(t.clone()))], vec![]]
    }

    /// Payload struct-size of the largest variant plus the tag, padded to 8.
    fn enum_size(&self, variants: &[Vec<Type>]) -> u64 {
        let max_payload = variants
            .iter()
            .map(|p| {
                let (_, size, _) = self.lay_fields(&payload_named(p));
                size
            })
            .max()
            .unwrap_or(0);
        round_up(ENUM_TAG + max_payload, 8)
    }

    /// The `(name, payloads)` list for any enum type (user or `BoxResult`).
    pub fn enum_info(&self, ty: &Type) -> Option<Vec<(String, Vec<Type>)>> {
        match ty {
            Type::Named(n) => self.items.lookup_enum(n).map(|e| {
                e.variants
                    .iter()
                    .map(|v| (v.name.clone(), v.payload.clone()))
                    .collect()
            }),
            Type::BoxResult(t) => Some(vec![
                ("boxed".to_string(), vec![Type::Box(Box::new((**t).clone()))]),
                ("oom".to_string(), vec![]),
            ]),
            _ => None,
        }
    }

    /// Offset (from the enum base) of payload field `i` of a variant.
    pub fn payload_offset(&self, payloads: &[Type], i: usize) -> (Type, u64) {
        let (fields, _, _) = self.lay_fields(&payload_named(payloads));
        let (_, ty, off) = fields[i].clone();
        (ty, ENUM_TAG + off)
    }
}

fn payload_named(payloads: &[Type]) -> Vec<(String, Type)> {
    payloads
        .iter()
        .enumerate()
        .map(|(i, t)| (format!("_{i}"), t.clone()))
        .collect()
}
