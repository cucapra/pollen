use std::ffi::CStr;
use flatgfa::{self, file, flatgfa::{Orientation, Segment}, memfile, pool::Id, FlatGFA, HeapGFAStore};

pub struct FlatGFAHandle {
    store: Store,
}

/// From flatgfa-py
/// Storage for a FlatGFA.
///
/// This may be either an in-memory data structure or a memory-mapped file. It exposes a
/// uniform interface to the FlatGFA data via `view`.
///
enum Store {
    Heap(Box<HeapGFAStore>),
    File(memmap::Mmap),
}

impl Store {
    /// Parse a text GFA file.
    fn parse_file(filename: &str) -> Self {
        let file = memfile::map_file(filename);
        Self::parse_gfa(file.as_ref())
    }

    /// Parse a GFA graph from a byte buffer.
    fn parse_gfa(data: &[u8]) -> Self {
        let store = flatgfa::parse::Parser::for_heap().parse_mem(data);
        Self::Heap(Box::new(store))
    }

    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA<'_> {
        // TK It seems wasteful to check the type of store every time... and to construct
        // the view every time. It's probably possible to fix this with a self-reference,
        // e.g., with the `owning_ref` crate.
        match self {
            Store::Heap(ref store) => (**store).as_ref(),
            Store::File(ref mmap) => file::view(mmap),
        }
    }
}



/// Parse a GFA file and return an opaque handle.
/// Caller must free with flatgfa_free().
#[no_mangle]
pub extern "C" fn flatgfa_parse(filename: *const std::os::raw::c_char) -> *mut FlatGFAHandle {
    let filename = unsafe { CStr::from_ptr(filename) }.to_str().unwrap();
    let store = Store::parse_file(filename);
    Box::into_raw(Box::new(FlatGFAHandle { store }))
}

/// Free a FlatGFA handle.
#[no_mangle]
pub extern "C" fn flatgfa_free(gfa: *mut FlatGFAHandle) {
    if !gfa.is_null() {
        unsafe { drop(Box::from_raw(gfa)) };
    }
}

#[no_mangle]
pub extern "C" fn flatgfa_path_count(gfa: *const FlatGFAHandle) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    view.paths.all().len()
}

/// Get the name of a path by index. Returns a pointer to the name bytes (not
/// null-terminated) and sets `*len` to the byte length. The pointer is valid
/// as long as the FlatGFAHandle is alive. Returns null if index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_name(
    gfa: *const FlatGFAHandle,
    path_index: usize,
    len: *mut usize,
) -> *const u8 {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    match view.paths.all().get(path_index) {
        None => std::ptr::null(),
        Some(path) => {
            let name = view.get_path_name(path);
            unsafe { *len = name.len() };
            name.as_ptr()
        }
    }
}

/// Get the number of steps in a path. Returns usize::MAX if index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_step_count(
    gfa: *const FlatGFAHandle,
    path_index: usize,
) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    match view.paths.all().get(path_index) {
        None => usize::MAX,
        Some(path) => path.step_count(),
    }
}

/// A single step in a path: a segment ID and an orientation.
#[repr(C)]
pub struct CStep {
    pub segment_id: u32,
    pub is_forward: bool,
}

/// Get a single step from a path by path index and step index. Returns true on
/// success and writes into `*out`. Returns false if either index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_step(
    gfa: *const FlatGFAHandle,
    path_index: usize,
    step_index: usize,
    out: *mut CStep,
) -> bool {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    let path = match view.paths.all().get(path_index) {
        None => return false,
        Some(p) => p,
    };
    let handle = match view.get_path_steps(path).nth(step_index) {
        None => return false,
        Some(h) => *h,
    };
    unsafe {
        (*out).segment_id = handle.segment().into();
        (*out).is_forward = handle.orient() == Orientation::Forward;
    }
    true
}

/// Get number of DNA sequences in GFA file
#[no_mangle]
pub extern "C" fn flatgfa_get_segment_count(gfa: *const FlatGFAHandle) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    view.segs.all().len()
}

/// Get the DNA sequence for a segment. Returns a pointer to the raw bytes (not
/// null-terminated) and sets `*len`. The pointer is valid as long as the
/// FlatGFAHandle is alive. Returns null if segment_id is out of bounds.
/// Note: always returns the forward-strand sequence regardless of orientation.
#[no_mangle]
pub extern "C" fn flatgfa_get_segment_seq(
    gfa: *const FlatGFAHandle,
    segment_id: u32,
    len: *mut usize,
) -> *const u8 {
    let gfa = unsafe { &*gfa };
    let view = gfa.store.view();
    let segs = view.segs.all();
    if segment_id as usize >= segs.len() {
        return std::ptr::null();
    }
    let id: Id<Segment> = segment_id.into();
    let seq = view.get_seq(&view.segs[id]);
    unsafe { *len = seq.len() };
    seq.as_ptr()
}

