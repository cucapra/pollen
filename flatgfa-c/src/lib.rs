use bstr::BStr;
use flatgfa::{
    self,
    flatgfa::{Orientation, Segment},
    memfile,
    pool::Id,
    FlatGFA, HeapGFAStore,
};
use std::ffi::CStr;

/// A datastore for a variation graph with a flat representation.
pub struct FlatGFARef(HeapGFAStore);

impl FlatGFARef {
    /// Parse a text GFA file.
    fn parse_file(filename: &str) -> Self {
        let file = memfile::map_file(filename);
        Self::parse_gfa(file.as_ref())
    }

    /// Parse a GFA graph from a byte buffer.
    fn parse_gfa(data: &[u8]) -> Self {
        let store = flatgfa::parse::Parser::for_heap().parse_mem(data);
        Self(store)
    }

    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA<'_> {
        self.0.as_ref()
    }

    /// Get a pointer we can hand off to C.
    fn pointer(self) -> *mut FlatGFARef {
        Box::into_raw(Box::new(self))
    }
}

/// A byte string represented using a pointer/length pair.
///
/// The string is not null-terminated; the `len` field is the number of bytes.
#[repr(C)]
pub struct FlatGFAString {
    pub data: *const u8,
    pub len: usize,
}

impl Default for FlatGFAString {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0,
        }
    }
}

impl From<&BStr> for FlatGFAString {
    fn from(string: &BStr) -> Self {
        Self {
            data: string.as_ptr(),
            len: string.len(),
        }
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
pub extern "C" fn flatgfa_get_segment_count(gfa: *const FlatGFARef) -> u32 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    view.segs.len().try_into().unwrap()
}

/// Get the DNA sequence for a segment.
///
/// The string is valid as long as the FlatGFAHandle is alive. Returns a null
/// string if segment_id is out of bounds. Always returns the forward-strand
/// sequence regardless of orientation.
#[no_mangle]
pub extern "C" fn flatgfa_get_seq(gfa: *const FlatGFARef, segment_id: u32) -> FlatGFAString {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    let segs = view.segs.all();
    if segment_id as usize >= segs.len() {
        return FlatGFAString::default();
    }
    let id: Id<Segment> = segment_id.into();
    view.get_seq(&view.segs[id]).into()
}

/// Get number of paths in the graph.
#[no_mangle]
pub extern "C" fn flatgfa_path_count(gfa: *const FlatGFARef) -> u32 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    view.paths.len().try_into().unwrap()
}

/// Get the name of a path by its index.
///
/// This is a pointer/length string, i.e., it is not null-terminated. We return
/// a pointer to the name data and set the length via a pointer. The pointer is
/// valid as long as the FlatGFAHandle is alive. Returns null if index is out of
/// bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_name(gfa: *const FlatGFARef, path_index: u32) -> FlatGFAString {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.get(path_index) {
        None => FlatGFAString::default(),
        Some(path) => view.get_path_name(path).into(),
    }
}

/// Get the number of steps in a path by index. Returns UINT32_MAX if index is
/// out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_step_count(gfa: *const FlatGFARef, path_index: u32) -> u32 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.get(path_index) {
        None => u32::MAX,
        Some(path) => path.step_count().try_into().unwrap(),
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
