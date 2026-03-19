use flatgfa::{
    self,
    flatgfa::{Orientation, Segment},
    memfile,
    pool::Id,
    FlatGFA, HeapGFAStore,
};
use std::ffi::CStr;

/// A datastore for a variation graph with a flat representation.
pub struct FlatGFARef(Box<HeapGFAStore>);

impl FlatGFARef {
    /// Parse a text GFA file.
    fn parse_file(filename: &str) -> Self {
        let file = memfile::map_file(filename);
        Self::parse_gfa(file.as_ref())
    }

    /// Parse a GFA graph from a byte buffer.
    fn parse_gfa(data: &[u8]) -> Self {
        let store = flatgfa::parse::Parser::for_heap().parse_mem(data);
        Self(Box::new(store))
    }

    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA<'_> {
        (*self.0).as_ref()
    }

    /// Get a pointer we can hand off to C.
    fn pointer(self) -> *mut FlatGFARef {
        Box::into_raw(Box::new(self))
    }
}

/// Parse a GFA text file and create a new FlatGFA, returning a handle. The
/// caller must free this with `flatgfa_free`.
#[no_mangle]
pub extern "C" fn flatgfa_parse(filename: *const std::os::raw::c_char) -> *mut FlatGFARef {
    let filename = unsafe { CStr::from_ptr(filename) }.to_str().unwrap();
    let store = FlatGFARef::parse_file(filename);
    store.pointer()
}

/// Free a FlatGFA handle.
#[no_mangle]
pub extern "C" fn flatgfa_free(gfa: *mut FlatGFARef) {
    if !gfa.is_null() {
        unsafe { drop(Box::from_raw(gfa)) };
    }
}

/// Get the number of segments in the graph.
#[no_mangle]
pub extern "C" fn flatgfa_get_segment_count(gfa: *const FlatGFARef) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    view.segs.all().len()
}

/// Get the DNA sequence for a segment. Returns a pointer to the raw bytes (not
/// null-terminated) and sets `*len`. The pointer is valid as long as the
/// FlatGFAHandle is alive. Returns null if segment_id is out of bounds.
/// Note: always returns the forward-strand sequence regardless of orientation.
#[no_mangle]
pub extern "C" fn flatgfa_get_segment_seq(
    gfa: *const FlatGFARef,
    segment_id: u32,
    len: *mut usize,
) -> *const u8 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    let segs = view.segs.all();
    if segment_id as usize >= segs.len() {
        return std::ptr::null();
    }
    let id: Id<Segment> = segment_id.into();
    let seq = view.get_seq(&view.segs[id]);
    unsafe { *len = seq.len() };
    seq.as_ptr()
}

/// Get number of paths in the graph.
#[no_mangle]
pub extern "C" fn flatgfa_path_count(gfa: *const FlatGFARef) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    view.paths.all().len()
}

/// Get the name of a path by its index.
///
/// This is a pointer/length string, i.e., it is not null-terminated. We return
/// a pointer to the name data and set the length via a pointer. The pointer is
/// valid as long as the FlatGFAHandle is alive. Returns null if index is out of
/// bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_name(
    gfa: *const FlatGFARef,
    path_index: usize,
    len: *mut usize,
) -> *const u8 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.all().get(path_index) {
        None => std::ptr::null(),
        Some(path) => {
            let name = view.get_path_name(path);
            unsafe { *len = name.len() };
            name.as_ptr()
        }
    }
}

/// Get the number of steps in a path by index. Returns usize::MAX if index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_step_count(gfa: *const FlatGFARef, path_index: usize) -> usize {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.all().get(path_index) {
        None => usize::MAX,
        Some(path) => path.step_count(),
    }
}

/// An oriented reference to a segment within a FlatGFA.
#[repr(C)]
pub struct FlatGFAHandle {
    pub segment_id: u32,
    pub is_forward: bool,
}

/// Get a single step from a path by path index and step index. Returns true on
/// success and writes into `*out`. Returns false if either index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_step(
    gfa: *const FlatGFARef,
    path_index: usize,
    step_index: usize,
    out: *mut FlatGFAHandle,
) -> bool {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();

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
