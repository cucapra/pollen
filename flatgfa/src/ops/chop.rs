use crate::flatgfa::{self, Handle, Link, Orientation, Path, Segment};
use crate::pool::{Id, Span, Store};
use crate::{GFAStore, HeapFamily, SeqSpan};

pub fn chop(gfa: &flatgfa::FlatGFA, max_size: usize, incl_links: bool) -> flatgfa::HeapGFAStore {
    let mut flat = flatgfa::HeapGFAStore::default();

    // when segment S is chopped into segments S1 through S2 (exclusive),
    // seg_map[S.name] = Span(Id(S1.name), Id(S2.name)). If S is not chopped: S=S1, S2.name = S1.name+1
    let mut seg_map: Vec<Span<Segment>> = Vec::new();
    // The smallest id (>0) which does not already belong to a segment in `flat`
    let mut max_node_id = 1;

    fn link_forward(flat: &mut GFAStore<'static, HeapFamily>, span: &Span<Segment>) {
        // Link segments spanned by `span` from head to tail
        let overlap = Span::new_empty();
        flat.add_links((span.start.index()..span.end.index() - 1).map(|idx| Link {
            from: Handle::new(Id::new(idx), Orientation::Forward),
            to: Handle::new(Id::new(idx + 1), Orientation::Forward),
            overlap,
        }));
    }

    // Add new, chopped segments
    for seg in gfa.segs.all().iter() {
        let len = seg.len();
        if len <= max_size {
            // Leave the segment as is
            let id = flat.segs.add(Segment {
                name: max_node_id,
                seq: seg.seq,
                optional: Span::new_empty(), // TODO: Optional data may stay valid when seg not chopped?
            });
            max_node_id += 1;
            seg_map.push(Span::new(id, flat.segs.next_id()));
        } else {
            let seq_end = seg.seq.end;
            let mut offset = seg.seq.start;
            let segs_start = flat.segs.next_id();
            // Could also generate end_id by setting it equal to the start_id and
            // updating it for each segment that is added - only benefits us if we
            // don't unroll the last iteration of this loop
            while offset < seq_end - max_size {
                // Generate a new segment of length c
                flat.segs.add(Segment {
                    name: max_node_id,
                    seq: SeqSpan::from_range(std::ops::Range {
                        // Note for reviwer: Change made here
                        start: offset,
                        end: offset + max_size,
                    }),
                    optional: Span::new_empty(),
                });
                offset += max_size;
                max_node_id += 1;
            }
            // Generate the last segment
            flat.segs.add(Segment {
                name: max_node_id,
                seq: SeqSpan::from_range(std::ops::Range {
                    // Note for reviwer: Change made here
                    start: offset,
                    end: seq_end,
                }),
                optional: Span::new_empty(),
            });
            max_node_id += 1;
            let new_seg_span = Span::new(segs_start, flat.segs.next_id());
            seg_map.push(new_seg_span);
            if incl_links {
                link_forward(&mut flat, &new_seg_span);
            }
        }
    }

    // For each path, add updated handles. Then add the updated path
    for path in gfa.paths.all().iter() {
        let path_start = flat.steps.next_id();
        let mut path_end = flat.steps.next_id();
        // Generate the new handles
        // Tentative to-do: see if it is faster to read Id from segs than to re-generate it?
        for step in gfa.get_path_steps(path) {
            let range = {
                let span = seg_map[step.segment().index()];
                std::ops::Range::from(span)
            };
            match step.orient() {
                Orientation::Forward => {
                    // In this builder, Id.index() == seg.name - 1 for all seg
                    path_end = flat
                        .add_steps(range.map(|idx| Handle::new(Id::new(idx), Orientation::Forward)))
                        .end;
                }
                Orientation::Backward => {
                    path_end = flat
                        .add_steps(
                            range
                                .rev()
                                .map(|idx| Handle::new(Id::new(idx), Orientation::Backward)),
                        )
                        .end;
                }
            }
        }

        // Add the updated path
        flat.paths.add(Path {
            name: path.name,
            steps: Span::new(path_start, path_end),
            overlaps: Span::new_empty(),
        });
    }

    // If the 'l' flag is specified, compute the links in the new graph
    if incl_links {
        // For each link in the old graph, from handle A -> B:
        //      Add a link from
        //          (A.forward ? (A.end, forward) : (A.begin, backwards))
        //          -> (B.forward ? (B.begin, forward) : (B.end ? backwards))

        for link in gfa.links.all().iter() {
            let new_from = {
                let old_from = link.from;
                let chopped_segs = seg_map[old_from.segment().index()];
                let seg_id = match old_from.orient() {
                    Orientation::Forward => chopped_segs.end - 1,
                    Orientation::Backward => chopped_segs.start,
                };
                seg_id.handle(old_from.orient())
            };
            let new_to = {
                let old_to = link.to;
                let chopped_segs = seg_map[old_to.segment().index()];
                let seg_id = match old_to.orient() {
                    Orientation::Forward => chopped_segs.start,
                    Orientation::Backward => chopped_segs.end - 1,
                };
                seg_id.handle(old_to.orient())
            };
            flat.add_link(new_from, new_to, vec![]);
        }
    }

    flat
}
