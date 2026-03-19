#![allow(non_camel_case_types)]
#![allow(private_interfaces)]

use bstr::BStr;
use flatgfa::{
    self,
    flatgfa::{Orientation, Segment},
    memfile,
    pool::Id,
    FlatGFA, HeapGFAStore,
};
use std::ffi::{c_int, CStr};

/// An opaque wrapper for a store, for exporting to C.
struct CStore(HeapGFAStore);

impl CStore {
    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA<'_> {
        self.0.as_ref()
    }

    /// Get a pointer we can hand off to C.
    fn pointer(self) -> *mut CStore {
        Box::into_raw(Box::new(self))
    }
}

/// A datastore for a variation graph with a flat representation.
pub type flatgfa_t = *mut CStore;

/// A byte string represented using a pointer/length pair.
///
/// The string is not null-terminated; the `len` field is the number of bytes.
#[repr(C)]
pub struct flatgfa_string_t {
    pub data: *const u8,
    pub len: c_int,
}

impl Default for flatgfa_string_t {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0,
        }
    }
}

impl From<&BStr> for flatgfa_string_t {
    fn from(string: &BStr) -> Self {
        Self {
            data: string.as_ptr(),
            len: string.len().try_into().unwrap(),
        }
    }
}

/// Parse a GFA text file and create a new FlatGFA, returning a handle. The
/// caller must free this with `flatgfa_free`.
#[no_mangle]
pub extern "C" fn flatgfa_parse(filename: *const std::os::raw::c_char) -> flatgfa_t {
    let filename = unsafe { CStr::from_ptr(filename) }.to_str().unwrap();
    let file = memfile::map_file(filename);
    let store = flatgfa::parse::Parser::for_heap().parse_mem(&file);
    CStore(store).pointer()
}

/// Free a FlatGFA handle.
#[no_mangle]
pub extern "C" fn flatgfa_free(gfa: flatgfa_t) {
    if !gfa.is_null() {
        unsafe { drop(Box::from_raw(gfa)) };
    }
}

/// Get the number of segments in the graph.
#[no_mangle]
pub extern "C" fn flatgfa_get_segment_count(gfa: flatgfa_t) -> u32 {
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
pub extern "C" fn flatgfa_get_seq(gfa: flatgfa_t, segment_id: u32) -> flatgfa_string_t {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    let segs = view.segs.all();
    if segment_id as usize >= segs.len() {
        return flatgfa_string_t::default();
    }
    let id: Id<Segment> = segment_id.into();
    view.get_seq(&view.segs[id]).into()
}

/// Get number of paths in the graph.
#[no_mangle]
pub extern "C" fn flatgfa_path_count(gfa: flatgfa_t) -> u32 {
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
pub extern "C" fn flatgfa_get_path_name(gfa: flatgfa_t, path_index: u32) -> flatgfa_string_t {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.get(path_index) {
        None => flatgfa_string_t::default(),
        Some(path) => view.get_path_name(path).into(),
    }
}

/// Get the number of steps in a path by index. Returns UINT32_MAX if index is
/// out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_path_step_count(gfa: flatgfa_t, path_index: u32) -> u32 {
    let gfa = unsafe { &*gfa };
    let view = gfa.view();
    match view.paths.get(path_index) {
        None => u32::MAX,
        Some(path) => path.step_count().try_into().unwrap(),
    }
}

/// An oriented reference to a segment within a FlatGFA.
#[repr(C)]
pub struct flatgfa_handle_t {
    pub segment_id: u32,
    pub is_forward: bool,
}

/// Get a single step from a path by path index and step index. Returns true on
/// success and writes into `*out`. Returns false if either index is out of bounds.
#[no_mangle]
pub extern "C" fn flatgfa_get_step(
    gfa: flatgfa_t,
    path_index: usize,
    step_index: usize,
    out: *mut flatgfa_handle_t,
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
