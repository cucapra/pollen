use memmap::{Mmap, MmapMut};

pub fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

pub fn map_new_file(name: &str, size: u64) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
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
    haystack: &'a [u8],
    memchr: memchr::Memchr<'a>,
    pos: usize,
}

impl<'a> Iterator for MemchrSplit<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        let end = self.memchr.next()?;
        self.pos = end + 1;
        Some(&self.haystack[start..end])
    }
}

impl MemchrSplit<'_> {
    pub fn new(needle: u8, haystack: &[u8]) -> MemchrSplit {
        MemchrSplit {
            haystack,
            memchr: memchr::memchr_iter(needle, haystack),
            pos: 0,
        }
    }
}
