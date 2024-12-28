use crate::flatgfa;

pub fn position(
    gfa: &flatgfa::FlatGFA,
    path: &flatgfa::Path,
    offset: usize,
) -> Option<(flatgfa::Handle, usize)> {
    // Traverse the path until we reach the position.
    let mut cur_pos = 0;
    for step in &gfa.steps[path.steps] {
        let seg = gfa.get_handle_seg(*step);
        let end_pos = cur_pos + seg.len();
        if offset < end_pos {
            // Found it!
            return Some((*step, offset - cur_pos));
        }
        cur_pos = end_pos;
    }

    None
}
