use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::pool::{Pool, Span};

#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct BEDEntry {
    pub name: Span<u8>,
    pub start: u64,
    pub end: u64,
}

#[derive(FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct FlatBED<'a> {
    pub name_data: Pool<'a, u8>,
    pub entries: Pool<'a, BEDEntry>,
}
