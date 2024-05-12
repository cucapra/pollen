use std::ops::Index;
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

    pub fn contains(&self, id: Id<T>) -> bool {
        self.start.0 <= id.0 && id.0 < self.end.0
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
pub trait Store<T: Clone> {
    /// Get a fixed-size view of the arena.
    fn as_ref(&self) -> Pool<T>;

    /// Add an item to the pool and get the new id.
    fn add(&mut self, item: T) -> Id<T>;

    /// Add an entire sequence of items to a "pool" vector and return the
    /// range of new indices (IDs).
    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T>;

    /// Like `add_iter`, but for slices.
    fn add_slice(&mut self, slice: &[T]) -> Span<T>;

    /// Get the number of items in the pool.
    fn len(&self) -> usize;

    /// Check whether the pool is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the next available ID.
    fn next_id(&self) -> Id<T> {
        Id::new(self.len())
    }
}

/// A store that uses a `Vec` to allocate objects on the heap.
///
/// This is a "normal" arena that can freely grow to fill available memory.
#[repr(transparent)]
pub struct HeapStore<T>(Vec<T>);

impl<T: Clone> Store<T> for HeapStore<T> {
    fn as_ref(&self) -> Pool<T> {
        Pool(&self.0)
    }

    fn add(&mut self, item: T) -> Id<T> {
        let id = self.as_ref().next_id();
        self.0.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T> {
        let start = self.as_ref().next_id();
        self.0.extend(iter);
        Span::new(start, self.as_ref().next_id())
    }

    fn add_slice(&mut self, slice: &[T]) -> Span<T> {
        let start = self.as_ref().next_id();
        self.0.extend_from_slice(slice);
        Span::new(start, self.as_ref().next_id())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> Default for HeapStore<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// A store that keeps its data in fixed locations in memory.
///
/// This is a funkier kind of arena that uses memory that has already been pre-allocated
/// somewhere else, such as in a memory-mapped file. A consequence is that there is a
/// fixed maximum size for the arena; it's possible to add objects only until it fills up.
#[repr(transparent)]
pub struct FixedStore<'a, T>(SliceVec<'a, T>);

impl<'a, T: Clone> Store<T> for FixedStore<'a, T> {
    fn as_ref(&self) -> Pool<T> {
        Pool(&self.0)
    }

    fn add(&mut self, item: T) -> Id<T> {
        let id = self.next_id();
        self.0.push(item);
        id
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span<T> {
        let start = self.next_id();
        self.0.extend(iter);
        Span::new(start, self.next_id())
    }

    fn add_slice(&mut self, slice: &[T]) -> Span<T> {
        let start = self.next_id();
        self.0.extend_from_slice(slice);
        Span::new(start, self.next_id())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T> FixedStore<'a, T> {
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<'a, T> From<SliceVec<'a, T>> for FixedStore<'a, T> {
    fn from(slice: SliceVec<'a, T>) -> Self {
        Self(slice)
    }
}

/// A fixed-sized arena.
///
/// This trait allows id-based access to a fixed-size chunk of objects reflecting
/// a `Store`. Unlike `Store`, it does not support adding new objects.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pool<'a, T>(&'a [T]);

impl<'a, T> Pool<'a, T> {
    /// Get the number of items in the pool.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the next available ID.
    pub fn next_id(&self) -> Id<T> {
        Id::new(self.len())
    }

    /// Get the entire pool as a slice.
    pub fn all(&self) -> &'a [T] {
        self.0
    }

    /// Find the first item in the pool that satisfies a predicate.
    pub fn search(&self, pred: impl Fn(&T) -> bool) -> Option<Id<T>> {
        self.0.iter().position(pred).map(|i| Id::new(i))
    }

    /// Iterate over id/item pairs in the pool.
    pub fn items(&self) -> impl Iterator<Item = (Id<T>, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, item)| (Id::new(i), item))
    }
}

impl<T> Index<Id<T>> for Pool<'_, T> {
    type Output = T;

    fn index(&self, id: Id<T>) -> &T {
        &self.0[id.index()]
    }
}

impl<T> Index<Span<T>> for Pool<'_, T> {
    type Output = [T];

    fn index(&self, span: Span<T>) -> &[T] {
        &self.0[std::ops::Range::from(span)]
    }
}

impl<'a, T> From<&'a [T]> for Pool<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self(slice)
    }
}
