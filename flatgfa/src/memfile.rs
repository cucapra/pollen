use memmap::{Mmap, MmapMut};
use rayon::iter::{
    plumbing::{bridge_unindexed, UnindexedConsumer, UnindexedProducer},
    ParallelIterator,
};

pub fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

pub fn map_new_file(name: &str, size: u64) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(name)
        .unwrap();
    file.set_len(size).unwrap();
    unsafe { MmapMut::map_mut(&file) }.unwrap()
}

pub fn map_file_mut(name: &str) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name)
        .unwrap();
    unsafe { MmapMut::map_mut(&file) }.unwrap()
}

pub struct MemchrSplit<'a> {
    needle: u8,
    haystack: &'a [u8],
    memchr: memchr::Memchr<'a>,
    pub pos: usize,
}

impl MemchrSplit<'_> {
    pub fn new(needle: u8, haystack: &[u8]) -> MemchrSplit<'_> {
        MemchrSplit {
            needle,
            haystack,
            memchr: memchr::memchr_iter(needle, haystack),
            pos: 0,
        }
    }
}

impl<'a> Iterator for MemchrSplit<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.haystack.len() {
            return None;
        }
        let start = self.pos;
        let end = self.memchr.next()?;
        self.pos = end + 1;
        Some(&self.haystack[start..end])
    }
}

impl<'a> UnindexedProducer for MemchrSplit<'a> {
    type Item = &'a [u8];

    fn split(self) -> (Self, Option<Self>) {
        // Roughly chop the buffer in half. Maybe this should give up if the current
        // size is already below a threshold.
        let mid = self.pos + (self.haystack.len() - self.pos) / 2;
        if mid >= self.haystack.len() || mid == 0 {
            return (self, None);
        };

        // Advance the midpoint to a needle boundary.
        let mid_nl = memchr::memchr(self.needle, &self.haystack[mid..]);
        let right_start = match mid_nl {
            Some(mid_nl) => mid + mid_nl + 1,
            None => return (self, None),
        };

        // Create two sub-iterators.
        let left = Self {
            needle: self.needle,
            haystack: &self.haystack[..right_start],
            memchr: self.memchr,
            pos: self.pos,
        };
        let right_buf = &self.haystack[right_start..];
        let right = Self {
            needle: self.needle,
            haystack: right_buf,
            memchr: memchr::memchr_iter(self.needle, right_buf),
            pos: 0,
        };
        (left, Some(right))
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: rayon::iter::plumbing::Folder<Self::Item>,
    {
        folder.consume_iter(self)
    }
}

impl<'a> ParallelIterator for MemchrSplit<'a> {
    type Item = &'a [u8];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}
