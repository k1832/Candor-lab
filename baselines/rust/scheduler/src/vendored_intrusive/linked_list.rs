// Copyright 2016 Amanieu d'Antras
// Copyright 2020 Amari Robinson
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// Vendored subset of `intrusive-collections` 0.10.2; see `mod.rs` for
// provenance. Retained: `LinkedList`, its non-atomic `Link`/`LinkOps`, the
// `Cursor`/`CursorMut` methods exercised by the scheduler (read walk,
// `insert_before`, and O(1) `remove`), and the `Iter` forward iterator.
// Removed as dead code per the ruling: `AtomicLink`/`AtomicLinkOps`, the
// singly-/xor-linked-list op impls, `CursorOwning`, `IntoIter`, and every
// unused cursor/list method (push_front, pop_back, split/splice/replace,
// take, back*, etc.). The only edit to retained code is `crate::` -> `super::`
// module paths.

//! Intrusive doubly-linked list.

use core::cell::Cell;
use core::ptr::NonNull;

use super::Adapter;
use super::link_ops::{self, DefaultLinkOps};
use super::pointer_ops::PointerOps;

// =============================================================================
// LinkedListOps
// =============================================================================

/// Link operations for `LinkedList`.
pub unsafe trait LinkedListOps: link_ops::LinkOps {
    /// Returns the "next" link pointer of `ptr`.
    ///
    /// # Safety
    /// An implementation of `next` must not panic.
    unsafe fn next(&self, ptr: Self::LinkPtr) -> Option<Self::LinkPtr>;

    /// Returns the "prev" link pointer of `ptr`.
    ///
    /// # Safety
    /// An implementation of `prev` must not panic.
    unsafe fn prev(&self, ptr: Self::LinkPtr) -> Option<Self::LinkPtr>;

    /// Sets the "next" link pointer of `ptr`.
    ///
    /// # Safety
    /// An implementation of `set_next` must not panic.
    unsafe fn set_next(&mut self, ptr: Self::LinkPtr, next: Option<Self::LinkPtr>);

    /// Sets the "prev" link pointer of `ptr`.
    ///
    /// # Safety
    /// An implementation of `set_prev` must not panic.
    unsafe fn set_prev(&mut self, ptr: Self::LinkPtr, prev: Option<Self::LinkPtr>);
}

// =============================================================================
// Link
// =============================================================================

/// Intrusive link that allows an object to be inserted into a
/// `LinkedList`.
#[repr(align(2))]
pub struct Link {
    next: Cell<Option<NonNull<Link>>>,
    prev: Cell<Option<NonNull<Link>>>,
}

// Use a special value to indicate an unlinked node
const UNLINKED_MARKER: Option<NonNull<Link>> =
    unsafe { Some(NonNull::new_unchecked(1 as *mut Link)) };

impl Link {
    /// Creates a new `Link`.
    #[inline]
    pub const fn new() -> Link {
        Link {
            next: Cell::new(UNLINKED_MARKER),
            prev: Cell::new(UNLINKED_MARKER),
        }
    }

    /// Checks whether the `Link` is linked into a `LinkedList`.
    #[inline]
    pub fn is_linked(&self) -> bool {
        self.next.get() != UNLINKED_MARKER
    }
}

impl DefaultLinkOps for Link {
    type Ops = LinkOps;

    const NEW: Self::Ops = LinkOps;
}

// An object containing a link can be sent to another thread if it is unlinked.
unsafe impl Send for Link {}

// Provide an implementation of Default which simply initializes the new link as
// unlinked. This allows structs containing a link to derive Default.
impl Default for Link {
    #[inline]
    fn default() -> Link {
        Link::new()
    }
}

// =============================================================================
// LinkOps
// =============================================================================

/// Default `LinkOps` implementation for `LinkedList`.
#[derive(Clone, Copy, Default)]
pub struct LinkOps;

unsafe impl link_ops::LinkOps for LinkOps {
    type LinkPtr = NonNull<Link>;

    #[inline]
    unsafe fn acquire_link(&mut self, ptr: Self::LinkPtr) -> bool {
        if ptr.as_ref().is_linked() {
            false
        } else {
            ptr.as_ref().next.set(None);
            true
        }
    }

    #[inline]
    unsafe fn release_link(&mut self, ptr: Self::LinkPtr) {
        ptr.as_ref().next.set(UNLINKED_MARKER);
    }
}

unsafe impl LinkedListOps for LinkOps {
    #[inline]
    unsafe fn next(&self, ptr: Self::LinkPtr) -> Option<Self::LinkPtr> {
        ptr.as_ref().next.get()
    }

    #[inline]
    unsafe fn prev(&self, ptr: Self::LinkPtr) -> Option<Self::LinkPtr> {
        ptr.as_ref().prev.get()
    }

    #[inline]
    unsafe fn set_next(&mut self, ptr: Self::LinkPtr, next: Option<Self::LinkPtr>) {
        ptr.as_ref().next.set(next);
    }

    #[inline]
    unsafe fn set_prev(&mut self, ptr: Self::LinkPtr, prev: Option<Self::LinkPtr>) {
        ptr.as_ref().prev.set(prev);
    }
}

#[inline]
unsafe fn link_between<T: LinkedListOps>(
    link_ops: &mut T,
    ptr: T::LinkPtr,
    prev: Option<T::LinkPtr>,
    next: Option<T::LinkPtr>,
) {
    if let Some(prev) = prev {
        link_ops.set_next(prev, Some(ptr));
    }
    if let Some(next) = next {
        link_ops.set_prev(next, Some(ptr));
    }
    link_ops.set_next(ptr, next);
    link_ops.set_prev(ptr, prev);
}

#[inline]
unsafe fn link_before<T: LinkedListOps>(link_ops: &mut T, ptr: T::LinkPtr, next: T::LinkPtr) {
    link_between(link_ops, ptr, link_ops.prev(next), Some(next));
}

#[inline]
unsafe fn remove<T: LinkedListOps>(link_ops: &mut T, ptr: T::LinkPtr) {
    let prev = link_ops.prev(ptr);
    let next = link_ops.next(ptr);

    if let Some(next) = next {
        link_ops.set_prev(next, prev);
    }
    if let Some(prev) = prev {
        link_ops.set_next(prev, next);
    }
    link_ops.release_link(ptr);
}

// =============================================================================
// Cursor, CursorMut
// =============================================================================

/// A cursor which provides read-only access to a `LinkedList`.
pub struct Cursor<'a, A: Adapter>
where
    A::LinkOps: LinkedListOps,
{
    current: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    list: &'a LinkedList<A>,
}

impl<'a, A: Adapter> Cursor<'a, A>
where
    A::LinkOps: LinkedListOps,
{
    /// Returns a reference to the object that the cursor is currently
    /// pointing to.
    ///
    /// This returns `None` if the cursor is currently pointing to the null
    /// object.
    #[inline]
    pub fn get(&self) -> Option<&'a <A::PointerOps as PointerOps>::Value> {
        Some(unsafe { &*self.list.adapter.get_value(self.current?) })
    }

    /// Moves the cursor to the previous element of the `LinkedList`.
    ///
    /// If the cursor is pointer to the null object then this will move it to
    /// the last element of the `LinkedList`. If it is pointing to the first
    /// element of the `LinkedList` then this will move it to the null object.
    #[inline]
    pub fn move_prev(&mut self) {
        if let Some(current) = self.current {
            self.current = unsafe { self.list.adapter.link_ops().prev(current) };
        } else {
            self.current = self.list.tail;
        }
    }
}

/// A cursor which provides mutable access to a `LinkedList`.
pub struct CursorMut<'a, A: Adapter>
where
    A::LinkOps: LinkedListOps,
{
    current: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    list: &'a mut LinkedList<A>,
}

impl<'a, A: Adapter> CursorMut<'a, A>
where
    A::LinkOps: LinkedListOps,
{
    /// Moves the cursor to the next element of the `LinkedList`.
    ///
    /// If the cursor is pointer to the null object then this will move it to
    /// the first element of the `LinkedList`. If it is pointing to the
    /// last element of the `LinkedList` then this will move it to the
    /// null object.
    #[inline]
    pub fn move_next(&mut self) {
        if let Some(current) = self.current {
            self.current = unsafe { self.list.adapter.link_ops().next(current) };
        } else {
            self.current = self.list.head;
        }
    }

    /// Removes the current element from the `LinkedList`.
    ///
    /// A pointer to the element that was removed is returned, and the cursor is
    /// moved to point to the next element in the `LinkedList`.
    ///
    /// If the cursor is currently pointing to the null object then no element
    /// is removed and `None` is returned.
    #[inline]
    pub fn remove(&mut self) -> Option<<A::PointerOps as PointerOps>::Pointer> {
        unsafe {
            if let Some(current) = self.current {
                if self.list.head == self.current {
                    self.list.head = self.list.adapter.link_ops().next(current);
                }
                if self.list.tail == self.current {
                    self.list.tail = self.list.adapter.link_ops().prev(current);
                }
                let next = self.list.adapter.link_ops().next(current);
                let result = current;
                remove(self.list.adapter.link_ops_mut(), current);
                self.current = next;
                Some(
                    self.list
                        .adapter
                        .pointer_ops()
                        .from_raw(self.list.adapter.get_value(result)),
                )
            } else {
                None
            }
        }
    }

    /// Inserts a new element into the `LinkedList` before the current one.
    ///
    /// If the cursor is pointing at the null object then the new element is
    /// inserted at the end of the `LinkedList`.
    ///
    /// # Panics
    ///
    /// Panics if the new element is already linked to a different intrusive
    /// collection.
    #[inline]
    pub fn insert_before(&mut self, val: <A::PointerOps as PointerOps>::Pointer) {
        unsafe {
            let new = self.list.node_from_value(val);

            let link_ops = self.list.adapter.link_ops_mut();

            if let Some(current) = self.current {
                link_before(link_ops, new, current);
            } else {
                link_between(link_ops, new, self.list.tail, None);
                self.list.tail = Some(new);
            }
            if self.list.head == self.current {
                self.list.head = Some(new);
            }
        }
    }
}

// =============================================================================
// LinkedList
// =============================================================================

/// An intrusive doubly-linked list.
///
/// When this collection is dropped, all elements linked into it will be
/// converted back to owned pointers and dropped.
pub struct LinkedList<A: Adapter>
where
    A::LinkOps: LinkedListOps,
{
    head: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    tail: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    adapter: A,
}

impl<A: Adapter> LinkedList<A>
where
    A::LinkOps: LinkedListOps,
{
    #[inline]
    fn node_from_value(
        &mut self,
        val: <A::PointerOps as PointerOps>::Pointer,
    ) -> <A::LinkOps as link_ops::LinkOps>::LinkPtr {
        use link_ops::LinkOps;

        unsafe {
            let raw = self.adapter.pointer_ops().into_raw(val);
            let link = self.adapter.get_link(raw);

            if !self.adapter.link_ops_mut().acquire_link(link) {
                // convert the node back into a pointer
                self.adapter.pointer_ops().from_raw(raw);

                panic!("attempted to insert an object that is already linked");
            }

            link
        }
    }

    /// Creates an empty `LinkedList`.
    #[inline]
    pub fn new(adapter: A) -> LinkedList<A> {
        LinkedList {
            head: None,
            tail: None,
            adapter,
        }
    }

    /// Returns a null `Cursor` for this list.
    #[inline]
    pub fn cursor(&self) -> Cursor<'_, A> {
        Cursor {
            current: None,
            list: self,
        }
    }

    /// Returns a null `CursorMut` for this list.
    #[inline]
    pub fn cursor_mut(&mut self) -> CursorMut<'_, A> {
        CursorMut {
            current: None,
            list: self,
        }
    }

    /// Creates a `CursorMut` from a pointer to an element.
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to an object that is part of this list.
    #[inline]
    pub unsafe fn cursor_mut_from_ptr(
        &mut self,
        ptr: *const <A::PointerOps as PointerOps>::Value,
    ) -> CursorMut<'_, A> {
        CursorMut {
            current: Some(self.adapter.get_link(ptr)),
            list: self,
        }
    }

    /// Returns a `CursorMut` pointing to the first element of the list. If the
    /// the list is empty then a null cursor is returned.
    #[inline]
    pub fn front_mut(&mut self) -> CursorMut<'_, A> {
        let mut cursor = self.cursor_mut();
        cursor.move_next();
        cursor
    }

    /// Gets an iterator over the objects in the `LinkedList`.
    #[inline]
    pub fn iter(&self) -> Iter<'_, A> {
        Iter {
            head: self.head,
            tail: self.tail,
            list: self,
        }
    }

    /// Removes all elements from the `LinkedList`.
    ///
    /// This will unlink all object currently in the list, which requires
    /// iterating through all elements in the `LinkedList`. Each element is
    /// converted back to an owned pointer and then dropped.
    #[inline]
    fn clear(&mut self) {
        use link_ops::LinkOps;

        let mut current = self.head;
        self.head = None;
        self.tail = None;
        while let Some(x) = current {
            unsafe {
                let next = self.adapter.link_ops().next(x);
                self.adapter.link_ops_mut().release_link(x);
                self.adapter
                    .pointer_ops()
                    .from_raw(self.adapter.get_value(x));
                current = next;
            }
        }
    }

    /// Inserts a new element at the end of the `LinkedList`.
    #[inline]
    pub fn push_back(&mut self, val: <A::PointerOps as PointerOps>::Pointer) {
        self.cursor_mut().insert_before(val);
    }

    /// Removes the first element of the `LinkedList`.
    ///
    /// This returns `None` if the `LinkedList` is empty.
    #[inline]
    pub fn pop_front(&mut self) -> Option<<A::PointerOps as PointerOps>::Pointer> {
        self.front_mut().remove()
    }
}

// Drop all owned pointers if the collection is dropped
impl<A: Adapter> Drop for LinkedList<A>
where
    A::LinkOps: LinkedListOps,
{
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

// =============================================================================
// Iter
// =============================================================================

/// An iterator over references to the items of a `LinkedList`.
pub struct Iter<'a, A: Adapter>
where
    A::LinkOps: LinkedListOps,
{
    head: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    tail: Option<<A::LinkOps as link_ops::LinkOps>::LinkPtr>,
    list: &'a LinkedList<A>,
}
impl<'a, A: Adapter + 'a> Iterator for Iter<'a, A>
where
    A::LinkOps: LinkedListOps,
{
    type Item = &'a <A::PointerOps as PointerOps>::Value;

    #[inline]
    fn next(&mut self) -> Option<&'a <A::PointerOps as PointerOps>::Value> {
        let head = self.head?;

        if Some(head) == self.tail {
            self.head = None;
            self.tail = None;
        } else {
            self.head = unsafe { self.list.adapter.link_ops().next(head) };
        }
        Some(unsafe { &*self.list.adapter.get_value(head) })
    }
}
