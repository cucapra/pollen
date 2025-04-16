use zerocopy::{AsBytes, FromBytes, FromZeroes};
use crate::memfile::MemchrSplit;
use std::io::BufRead;
use atoi::FromRadix10;
use bstr::BStr;
use crate::pool::{Pool, Span, Store, HeapStore, FixedStore, Id};

#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct BEDEntry {
    pub name: Span<u8>,
    pub start: u64,
    pub end: u64,
}

pub struct FlatBED<'a> {
    pub name_data: Pool<'a, u8>,
    pub entries: Pool<'a, BEDEntry>,
}

impl<'a> FlatBED<'a> {
    /// Get the base-pair sequence for a segment.
    pub fn get_num_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn get_name_of_entry(&self, entry: &BEDEntry) -> &BStr {
        self.name_data[entry.name].as_ref()
    }

    pub fn get_intersects(&self, bed: &FlatBED, entry: &BEDEntry) -> Vec<BEDEntry> {
        self.entries.items()
            .map(|x| BEDEntry {
                name: x.1.name,
                start: if x.1.start < entry.start { entry.start } else { x.1.start },
                end: if entry.end < x.1.end { entry.end } else { x.1.end },
            })
            .filter(|x| bed.get_name_of_entry(entry).eq(self.get_name_of_entry(x)) && x.end > x.start)
            .collect()
    }
}











/// The data storage pools for a `FlatBED`.
#[derive(Default)]
pub struct BEDStore<'a, P: StoreFamily<'a>> {
    pub name_data: P::Store<u8>,
    pub entries: P::Store<BEDEntry>,
}

impl<'a, P: StoreFamily<'a>> BEDStore<'a, P> {
    pub fn add_entry(&mut self, name: &[u8], start: u64, end: u64) -> Id<BEDEntry> {
        let name = self.name_data.add_slice(name);
        self.entries.add(BEDEntry {
            name,
            start,
            end,
        })
    }

    pub fn as_ref(&self) -> FlatBED {
        FlatBED {
            name_data: self.name_data.as_ref(),
            entries: self.entries.as_ref(),
        }
    }
}

pub trait StoreFamily<'a> {
    type Store<T: Clone + 'a>: Store<T>;
}

#[derive(Default)]
pub struct HeapFamily;
impl<'a> StoreFamily<'a> for HeapFamily {
    type Store<T: Clone + 'a> = HeapStore<T>;
}

pub struct FixedFamily;
impl<'a> StoreFamily<'a> for FixedFamily {
    type Store<T: Clone + 'a> = FixedStore<'a, T>;
}

/// A store for `FlatBED` data backed by fixed-size slices.
///
/// This store contains `SliceVec`s, which act like `Vec`s but are allocated within
/// a fixed region. This means they have a maximum size, but they can directly map
/// onto the contents of a file.
pub type FixedBEDStore<'a> = BEDStore<'a, FixedFamily>;

/// A mutable, in-memory data store for `FlatBED`.
///
/// This store contains a bunch of `Vec`s: one per array required to implement a
/// `FlatBED`. It exposes an API for building up a BED data structure, so it is
/// useful for creating new ones from scratch.
pub type HeapBEDStore = BEDStore<'static, HeapFamily>;


















type ParseResult<T> = Result<T, &'static str>;
type PartialParseResult<'a, T> = ParseResult<(T, &'a [u8])>;
fn parse_num<T: FromRadix10>(s: &[u8]) -> PartialParseResult<T> {
    match T::from_radix_10(s) {
        (_, 0) => Err("expected number"),
        (num, used) => Ok((num, &s[used..])),
    }
}

pub struct BEDParser<'a, P: StoreFamily<'a>> {
    /// The flat representation we're building.
    flat: BEDStore<'a, P>,
}

impl<'a, P: StoreFamily<'a>> BEDParser<'a, P> {
    pub fn new(builder: BEDStore<'a, P>) -> Self {
        Self {
            flat: builder,
        }
    }

    /// Parse a GFA text file from an I/O stream.
    pub fn parse_stream<R: BufRead>(mut self, stream: R) -> BEDStore<'a, P> {
        for line in stream.split(b'\n') {
            let line = line.unwrap();

            let first_tab_index = line.iter().position(|&x| x == b'\t').unwrap();
            let name_slice = &line[0..first_tab_index];

            let rest_of_vec = &line[first_tab_index+1..];
            let (start_num, rest) = parse_num(rest_of_vec).unwrap();
            let (end_num, _) = parse_num(&rest[1..]).unwrap();

            self.flat.add_entry(name_slice, start_num, end_num);
        }

        self.flat
    }

    /// Parse a GFA text file from an in-memory buffer.
    pub fn parse_mem(mut self, buf: &[u8]) -> BEDStore<'a, P> {
        for line in MemchrSplit::new(b'\n', buf) {
            let first_tab_index = line.iter().position(|&x| x == b'\t').unwrap();
            let name_slice = &line[0..first_tab_index];

            let rest_of_vec = &line[first_tab_index+1..];
            let (start_num, rest) = parse_num(rest_of_vec).unwrap();
            let (end_num, _) = parse_num(&rest[1..]).unwrap();

            self.flat.add_entry(name_slice, start_num, end_num);
        }

        self.flat
    }

}

impl BEDParser<'static, HeapFamily> {
    pub fn for_heap() -> Self {
        Self::new(HeapBEDStore::default())
    }
}

impl<'a> BEDParser<'a, FixedFamily> {
    pub fn for_slice(store: FixedBEDStore<'a>) -> Self {
        Self::new(store)
    }
}