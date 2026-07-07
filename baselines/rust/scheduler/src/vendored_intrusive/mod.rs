// Vendored subset of the `intrusive-collections` crate.
//
// # Provenance
//
// Vendored per the adjudication ruling of 2026-07-07 (measured-artifact
// self-containment): the spec-mandated core mechanics (the intrusive
// doubly-linked list, its embedded link, the non-owning handle, and O(1)
// cursor removal) must live in-source so the comparison target (a port in a
// language with no crate ecosystem) is measured against the same in-source
// machinery, not against code hidden in a cargo dependency.
//
// - Crate:    intrusive-collections, version 0.10.2 (crates.io).
// - Upstream:  https://github.com/Amanieu/intrusive-rs
// - crates.io checksum:
//             4b719c59241cfaac1042a6d26787e28ed7ee4a4e21a5a907786f54222d1b0062
//             (pinned in Cargo.lock at freeze).
// - Authors:  Amanieu d'Antras, Amari Robinson.
// - License:  MIT OR Apache-2.0 (dual). Original MIT copyright notice:
//             `Copyright (c) 2016 Amanieu d'Antras`.
//
// # What is vendored
//
// The used subset only: `LinkedList` and its `Cursor`/`CursorMut` (with O(1)
// `cursor_mut_from_ptr(...).remove()`), the `LinkedListLink` embedded link,
// the `Adapter`/`LinkOps`/`PointerOps` machinery and the `intrusive_adapter!`
// / `container_of!` macros, `UnsafeRef`, and the `DefaultPointerOps`
// specialization for `UnsafeRef`. Unused crate surface (red-black trees,
// singly-/xor-linked lists, the atomic link variant, `CursorOwning`,
// `IntoIter`, `KeyAdapter`, the `&T`/`Pin`/`Box`/`Rc`/`Arc` pointer-op
// specializations, and every unused cursor/list method) is removed as dead
// code per the ruling. The retained code is upstream's, unchanged except for
// that dead-code removal and the minimal edits required to compile it here:
// module paths (`crate::` -> `super::`/`$crate::vendored_intrusive::`) and the
// `offset_of!` source (upstream's `memoffset` re-export -> `core::mem`,
// stable since Rust 1.77). Its idiomaticity is upstream's, which is the point.

// Upstream crate-level suppression, retained verbatim as vendoring-required
// (not a memory-model relaxation): the `const NEW` associated constants that
// `intrusive_adapter!` generates would otherwise trip this lint.
#![allow(clippy::declare_interior_mutable_const)]
// Vendoring-required clippy suppressions (upstream API/idiom, not memory-model
// relaxations; the ruling forbids restyling the vendored code):
//   - missing_safety_doc: upstream's `unsafe trait`s document safety per-method,
//     not in a trait-level `# Safety` section.
//   - manual_dangling_ptr: the `1 as *mut Link` unlinked-node sentinel is a
//     never-dereferenced tag value; `NonNull::dangling()` would change its bits.
//   - wrong_self_convention: `PointerOps::{from_raw,into_raw}` take `&self`
//     because the conversion is defined on the (stateful) pointer-ops object.
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_dangling_ptr)]
#![allow(clippy::wrong_self_convention)]
// Vendoring-required (edition bridge, not a memory-model relaxation): the
// upstream crate is edition 2018, where an `unsafe fn` body is implicitly an
// unsafe context. This baseline is edition 2024, which lints that. The
// retained bodies are upstream's verbatim; wrapping each in an `unsafe {}`
// block would be exactly the restyling the ruling forbids, so the lint is
// allowed instead.
#![allow(unsafe_op_in_unsafe_fn)]

pub mod adapter;
pub mod link_ops;
pub mod linked_list;
pub mod pointer_ops;
pub mod unsafe_ref;

pub use self::adapter::Adapter;
pub use self::link_ops::{DefaultLinkOps, LinkOps};
pub use self::linked_list::Link as LinkedListLink;
pub use self::linked_list::LinkedList;
pub use self::pointer_ops::{DefaultPointerOps, PointerOps};
pub use self::unsafe_ref::UnsafeRef;
