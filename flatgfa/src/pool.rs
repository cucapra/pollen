use std::ops::Deref;
use std::{hash::Hash, marker::PhantomData};
use tinyvec::SliceVec;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// An index into a pool.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(transparent)]
pub struct Id<T>(u32, PhantomData<T>);

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Id<T> {
    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn new(index: usize) -> Self {
        Self(index.try_into().expect("id too large"), PhantomData)
    }
}

impl<T> From<u32> for Id<T> {
    fn from(v: u32) -> Self {
        Self(v, PhantomData)
    }
}

impl<T> From<Id<T>> for u32 {
    fn from(v: Id<T>) -> Self {
        v.0
    }
}

/// A range of indices into a pool.
///
/// TODO: Consider smaller indices for this, and possibly base/offset instead
/// of start/end.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Span<T> {
    pub start: Id<T>,
    pub end: Id<T>,
    _marker: PhantomData<T>,
}

impl<T> From<Span<T>> for std::ops::Range<usize> {
    fn from(span: Span<T>) -> std::ops::Range<usize> {
        (span.start.index())..(span.end.index())
    }
}

impl<T> Span<T> {
    pub fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }

    pub fn len(&self) -> usize {
        (self.end.0 - self.start.0) as usize
    }

    pub fn new(start: Id<T>, end: Id<T>) -> Self {
        Self {
            start,
            end,
            _marker: PhantomData,
        }
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
    fn add(&mut self, item: T) -> Id<T>;

    /// Add an entire sequence of items to a "pool" vector and return the
    /// range of new indices (IDs).
    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T>;

    /// Like `add_iter`, but for slices.
    fn add_slice(&mut self, slice: &[T]) -> Span<T>;
}

impl<T: Clone> Store<T> for Vec<T> {
    fn add(&mut self, item: T) -> Id<T> {
        let id = self.next_id();
        self.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T> {
        let start = self.next_id();
        self.extend(iter);
        Span::new(start, self.next_id())
    }

    fn add_slice(&mut self, slice: &[T]) -> Span<T> {
        let start = self.next_id();
        self.extend_from_slice(slice);
        Span::new(start, self.next_id())
    }
}

impl<'a, T: Clone> Store<T> for SliceVec<'a, T> {
    fn add(&mut self, item: T) -> Id<T> {
        let id = self.next_id();
        self.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T> {
        let start = self.next_id();
        self.extend(iter);
        Span::new(start, self.next_id())
    }

    fn add_slice(&mut self, slice: &[T]) -> Span<T> {
        let start = self.next_id();
        self.extend_from_slice(slice);
        Span::new(start, self.next_id())
    }
}

/// A fixed-sized arena.
///
/// This trait allows id-based access to a fixed-size chunk of objects reflecting
/// a `Store`. Unlike `Store`, it does not support adding new objects.
pub trait Pool<T> {
    /// Get a single element from the pool by its ID.
    fn get_id(&self, id: Id<T>) -> &T;

    /// Get a range of elements from the pool using their IDs.
    fn get_span(&self, span: Span<T>) -> &[T];

    /// Get the number of items in the pool.
    fn count(&self) -> usize;

    /// Get the next available ID.
    fn next_id(&self) -> Id<T>;
}

impl<T> Pool<T> for [T] {
    fn get_id(&self, id: Id<T>) -> &T {
        &self[id.index()]
    }

    fn get_span(&self, span: Span<T>) -> &[T] {
        &self[std::ops::Range::from(span)]
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn next_id(&self) -> Id<T> {
        Id::new(self.count())
    }
}
