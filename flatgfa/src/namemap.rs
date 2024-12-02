use crate::flatgfa::{FlatGFA, Segment};
use crate::pool::Id;
use std::collections::HashMap;

/// A fast way to look up segment IDs by their (integer) names.
#[derive(Default)]
pub struct NameMap {
    /// Names at most this are assigned *sequential* IDs, i.e., the ID is just the name
    /// minus one.
    sequential_max: usize,

    /// Non-sequential names go here.
    others: HashMap<usize, u32>,
}

impl NameMap {
    pub fn insert(&mut self, name: usize, id: Id<Segment>) {
        // Is this the next sequential name? If so, no need to record it in our hash table;
        // just bump the number of sequential names we've seen.
        if (name - 1) == self.sequential_max && (name - 1) == id.index() {
            self.sequential_max += 1;
        } else {
            self.others.insert(name, id.into());
        }
    }

    pub fn get(&self, name: usize) -> Id<Segment> {
        if name <= self.sequential_max {
            ((name - 1) as u32).into()
        } else {
            self.others[&name].into()
        }
    }

    /// Construct a name map for all the segments in a GFA.
    pub fn build(gfa: &FlatGFA) -> Self {
        let mut name_map = NameMap::default();
        for (id, seg) in gfa.segs.items() {
            name_map.insert(seg.name, id);
        }
        name_map
    }
}
