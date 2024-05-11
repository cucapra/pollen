use std::ops::Deref;
use tinyvec::SliceVec;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// An index into a pool.
///
/// TODO: Consider using newtypes for each distinct type.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Id(u32);

impl Id {
    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn new(index: usize) -> Self {
        Self(index.try_into().expect("id too large"))
    }
}

impl From<u32> for Id {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<Id> for u32 {
    fn from(v: Id) -> Self {
        v.0
    }
}

/// A range of indices into a pool.
///
/// TODO: Consider smaller indices for this, and possibly base/offset instead
/// of start/end.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Span {
    pub start: Id,
    pub end: Id,
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> std::ops::Range<usize> {
        (span.start.index())..(span.end.index())
    }
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }

    pub fn len(&self) -> usize {
        (self.end.0 - self.start.0) as usize
    }
}

/// A simple arena for objects of a single type.
///
/// This trait provides convenient accessors for treating Vec and Vec-like objects
/// as allocation arenas. This trait supports adding to the pool (i.e., growing the
/// arena). Pools also `Deref` to slices, which are `&Pool`s and support convenient
/// access to the current set of objects (but not addition of new objects).
pub trait Store<T: Clone>: Deref<Target = [T]> {
    /// Add an item to the pool and get the new id.
    fn add(&mut self, item: T) -> Id;

    /// Add an entire sequence of items to a "pool" vector and return the
    /// range of new indices (IDs).
    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span;

    /// Like `add_iter`, but for slices.
    fn add_slice(&mut self, slice: &[T]) -> Span;
}

impl<T: Clone> Store<T> for Vec<T> {
    fn add(&mut self, item: T) -> Id {
        let id = self.next_id();
        self.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span {
        let start = self.next_id();
        self.extend(iter);
        Span {
            start,
            end: self.next_id(),
        }
    }

    fn add_slice(&mut self, slice: &[T]) -> Span {
        let start = self.next_id();
        self.extend_from_slice(slice);
        Span {
            start,
            end: self.next_id(),
        }
    }
}

impl<'a, T: Clone> Store<T> for SliceVec<'a, T> {
    fn add(&mut self, item: T) -> Id {
        let id = self.next_id();
        self.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span {
        let start = self.next_id();
        self.extend(iter);
        Span {
            start,
            end: self.next_id(),
        }
    }

    fn add_slice(&mut self, slice: &[T]) -> Span {
        let start = self.next_id();
        self.extend_from_slice(slice);
        Span {
            start,
            end: self.next_id(),
        }
    }
}

/// A fixed-sized arena.
///
/// This trait allows id-based access to a fixed-size chunk of objects reflecting
/// a `Store`. Unlike `Store`, it does not support adding new objects.
pub trait Pool<T> {
    /// Get a single element from the pool by its ID.
    fn get_id(&self, id: Id) -> &T;

    /// Get a range of elements from the pool using their IDs.
    fn get_span(&self, span: Span) -> &[T];

    /// Get the number of items in the pool.
    fn count(&self) -> usize;

    /// Get the next available ID.
    fn next_id(&self) -> Id;
}

impl<T> Pool<T> for [T] {
    fn get_id(&self, id: Id) -> &T {
        &self[id.index()]
    }

    fn get_span(&self, span: Span) -> &[T] {
        &self[std::ops::Range::from(span)]
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn next_id(&self) -> Id {
        Id::new(self.count())
    }
}
