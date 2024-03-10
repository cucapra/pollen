use crate::flatgfa::{AlignOp, FlatGFAStore, Handle, LineKind, Orientation};
use gfa::{self, cigar, gfa::Line, parser::GFAParserBuilder};
use std::collections::HashMap;

/// A newtype to preserve optional fields without parsing them.
///
/// The underlying gfa-rs library lets you specify a type to hold optional
/// fields. We just store a plain (byte) string.
#[derive(Clone, Default, Debug)]
struct OptFields(Vec<u8>);

impl gfa::optfields::OptFields for OptFields {
    fn get_field(&self, _: &[u8]) -> Option<&gfa::optfields::OptField> {
        None
    }

    fn fields(&self) -> &[gfa::optfields::OptField] {
        &[]
    }

    fn parse<T>(input: T) -> Self
    where
        T: IntoIterator,
        T::Item: AsRef<[u8]>,
    {
        let mut out: Vec<u8> = vec![];
        let mut first = true;
        for i in input {
            if first {
                first = false;
            } else {
                out.push(b'\t');
            }
            out.extend(i.as_ref());
        }
        Self(out)
    }
}

#[derive(Default)]
pub struct Parser {
    // The flat representation we're building.
    flat: FlatGFAStore,

    // Track the segment IDs by their name, which we need to refer to segments in paths.
    segs_by_name: HashMap<usize, usize>,

    links: Vec<gfa::gfa::Link<usize, OptFields>>,
    paths: Vec<gfa::gfa::Path<usize, OptFields>>,
}

impl Parser {
    /// Parse a GFA text file.
    pub fn parse<R: std::io::BufRead>(stream: R) -> FlatGFAStore {
        let gfa_parser = GFAParserBuilder::none()
            .segments(true)
            .paths(true)
            .build_usize_id::<OptFields>();
        let mut parser = Self::default();
        for line in stream.lines() {
            let gfa_line = gfa_parser.parse_gfa_line(line.unwrap().as_ref()).unwrap();
            parser.parse_line(gfa_line);
        }
        parser.finish()
    }

    /// Parse a single GFA line.
    ///
    /// We add *segments* to the flat representation immediately. We buffer *links* and *paths*
    /// in our internal vectors, because we must see all the segments first before we can
    /// resolve their segment name references.
    fn parse_line(&mut self, line: Line<usize, OptFields>) {
        match line {
            Line::Header(h) => {
                self.flat.record_line(LineKind::Header);
                self.flat.add_header(h.version.unwrap());
            }
            Line::Segment(s) => {
                self.flat.record_line(LineKind::Segment);
                let seg_id = self.flat.add_seg(s.name, s.sequence, s.optional.0);
                self.segs_by_name.insert(s.name, seg_id);
            }
            Line::Link(l) => {
                self.flat.record_line(LineKind::Link);
                self.links.push(l);
            }
            Line::Path(p) => {
                self.flat.record_line(LineKind::Path);
                self.paths.push(p);
            }
            Line::Containment(_) => unimplemented!(),
        }
    }

    /// Finish parsing and return the flat representation.
    ///
    /// We "unwind" the buffers of links and paths, now that we have all
    /// the segments.
    fn finish(mut self) -> FlatGFAStore {
        // Add all the bufferred links.
        for link in self.links {
            let cigar = cigar::CIGAR::from_bytestring(&link.overlap).unwrap();
            let from = Handle {
                segment: self.segs_by_name[&link.from_segment],
                orient: convert_orient(link.from_orient),
            };
            let to = Handle {
                segment: self.segs_by_name[&link.to_segment],
                orient: convert_orient(link.to_orient),
            };
            self.flat.add_link(from, to, convert_cigar(&cigar));
        }

        // Add all the bufferred paths.
        for path in self.paths {
            let steps = parse_path_steps(path.segment_names)
                .into_iter()
                .map(|(name, dir)| Handle {
                    segment: self.segs_by_name[&name],
                    orient: if dir {
                        Orientation::Forward
                    } else {
                        Orientation::Backward
                    },
                })
                .collect();

            // When the overlaps section is just `*`, the rs-gfa library produces a
            // vector like `[None]`. I'm not sure if we really need to handle `None`
            // otherwise: all the real data I've seen either has *real* overlaps or
            // is just `*`.
            let overlaps: Vec<Vec<_>> = if path.overlaps.len() == 1 && path.overlaps[0].is_none() {
                vec![]
            } else {
                path.overlaps
                    .iter()
                    .map(|o| match o {
                        Some(c) => convert_cigar(c),
                        None => unimplemented!(),
                    })
                    .collect()
            };

            self.flat.add_path(path.path_name, steps, overlaps);
        }

        self.flat
    }
}

fn convert_orient(o: gfa::gfa::Orientation) -> Orientation {
    match o {
        gfa::gfa::Orientation::Forward => Orientation::Forward,
        gfa::gfa::Orientation::Backward => Orientation::Backward,
    }
}

fn convert_align_op(c: &cigar::CIGARPair) -> AlignOp {
    AlignOp {
        op: match c.op() {
            cigar::CIGAROp::M => crate::flatgfa::AlignOpcode::Match,
            cigar::CIGAROp::N => crate::flatgfa::AlignOpcode::Gap,
            cigar::CIGAROp::D => crate::flatgfa::AlignOpcode::Deletion,
            cigar::CIGAROp::I => crate::flatgfa::AlignOpcode::Insertion,
            _ => unimplemented!(),
        },
        len: c.len(),
    }
}

fn convert_cigar(c: &cigar::CIGAR) -> Vec<AlignOp> {
    c.0.iter().map(convert_align_op).collect()
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
///
/// The underlying gfa-rs library does not yet parse the actual segments
/// involved in the path. So we do it ourselves: splitting on commas and
/// matching the direction.
fn parse_path_steps(data: Vec<u8>) -> Vec<(usize, bool)> {
    // The parser state: we're either looking for a segment name (or a +/- terminator),
    // or we're expecting a comma (or end of string).
    enum PathParseState {
        Seg,
        Comma,
    }

    let mut state = PathParseState::Seg;
    let mut seg: usize = 0;
    let mut steps = vec![];
    for byte in data {
        match state {
            PathParseState::Seg => {
                if byte == b'+' || byte == b'-' {
                    steps.push((seg, byte == b'+'));
                    state = PathParseState::Comma;
                } else if byte.is_ascii_digit() {
                    seg *= 10;
                    seg += (byte - b'0') as usize;
                } else {
                    panic!("unexpected character in path: {}", byte as char);
                }
            }
            PathParseState::Comma => {
                if byte == b',' {
                    state = PathParseState::Seg;
                    seg = 0;
                } else {
                    panic!("unexpected character in path: {}", byte as char);
                }
            }
        }
    }
    steps
}

#[test]
fn test_parse_path() {
    let path = parse_path_steps(b"1+,23-,4+".to_vec());
    assert_eq!(path, vec![(1, true), (23, false), (4, true)]);
}
