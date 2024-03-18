use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// An index into a pool.
///
/// TODO: Consider using newtypes for each distinct type.
pub type Index = u32;

/// A range of indices into a pool.
///
/// TODO: Consider smaller indices for this, and possibly base/offset instead
/// of start/end.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Span {
    pub start: Index,
    pub end: Index,
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> std::ops::Range<usize> {
        (span.start as usize)..(span.end as usize)
    }
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        (*self).into()
    }
}

pub trait Pool<T: Clone> {
    /// Add an item to the pool and get the new index (ID).
    fn add(&mut self, item: T) -> Index;

    /// Add an entire sequence of items to a "pool" vector and return the
    /// range of new indices (IDs).
    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span;

    /// Like `add_iter`, but for slices.
    fn add_slice(&mut self, slice: &[T]) -> Span;

    /// Get a single element from the pool by its ID.
    fn get(&self, index: Index) -> &T;

    /// Get a range of elements from the pool using their IDs.
    fn get_span(&self, span: Span) -> &[T];

    /// Get the number of items in the pool.
    fn count(&self) -> usize;

    /// Get all items in the pool.
    fn all(&self) -> &[T];
}

impl<T: Clone> Pool<T> for Vec<T> {
    fn add(&mut self, item: T) -> Index {
        let len: u32 = self.len().try_into().expect("size too large");
        self.push(item);
        len
    }

    fn add_iter(&mut self, iter: impl IntoIterator<Item = T>) -> Span {
        let old_len: u32 = self.len().try_into().expect("old size too large");
        self.extend(iter);
        Span {
            start: old_len,
            end: self.len().try_into().expect("new size too large"),
        }
    }

    fn add_slice(&mut self, slice: &[T]) -> Span {
        let old_len: u32 = self.len().try_into().expect("old size too large");
        self.extend_from_slice(slice);
        Span {
            start: old_len,
            end: self.len().try_into().expect("new size too large"),
        }
    }

    fn get(&self, index: Index) -> &T {
        &self[index as usize]
    }

    fn get_span(&self, span: Span) -> &[T] {
        &self[span.range()]
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn all(&self) -> &[T] {
        self
    }
}
