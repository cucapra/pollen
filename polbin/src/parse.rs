use crate::flatgfa::{AlignOp, FlatGFAStore, Handle, LineKind, Orientation};
use crate::gfaline;
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
    /// The flat representation we're building.
    flat: FlatGFAStore,

    /// All segment IDs, indexed by their names, which we need to refer to segments in paths.
    seg_ids: NameMap,
}

/// Holds data structures that we haven't added to the flat representation yet.
struct Deferred {
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
        let mut deferred = Deferred {
            links: Vec::new(),
            paths: Vec::new(),
        };
        for line in stream.lines() {
            let line = line.unwrap();

            // Try using our hand-rolled parser for some lines.
            match gfaline::parse_line(line.as_ref()) {
                Ok(l) => {
                    parser.add_line(l);
                    continue;
                }
                Err(_) => {}
            }

            // Use the gfa-rs parser to parse most lines.
            let gfa_line = gfa_parser.parse_gfa_line(line.as_ref()).unwrap();
            parser.parse_line(gfa_line, &mut deferred);
        }
        parser.finish(deferred)
    }

    /// Parse a single GFA line.
    ///
    /// We add *segments* to the flat representation immediately. We buffer *links* and *paths*
    /// in our internal vectors, because we must see all the segments first before we can
    /// resolve their segment name references.
    fn parse_line(&mut self, line: Line<usize, OptFields>, deferred: &mut Deferred) {
        match line {
            Line::Header(_) | Line::Segment(_) => {
                panic!("handled by hand-rolled parser");
            }
            Line::Link(l) => {
                self.flat.record_line(LineKind::Link);
                deferred.links.push(l);
            }
            Line::Path(p) => {
                self.flat.record_line(LineKind::Path);
                deferred.paths.push(p);
            }
            Line::Containment(_) => unimplemented!(),
        }
    }

    /// Handle lines from our hand-rolled parser.
    fn add_line(&mut self, line: gfaline::Line) {
        match line {
            gfaline::Line::Header { data } => {
                self.flat.record_line(LineKind::Header);
                self.flat.add_header(data);
            }
            gfaline::Line::Segment { name, seq, data } => {
                self.flat.record_line(LineKind::Segment);
                let seg_id = self.flat.add_seg(name, seq, data);
                self.seg_ids.insert(name, seg_id);
            }
        }
    }

    fn add_link(&mut self, link: gfa::gfa::Link<usize, OptFields>) {
        let cigar = cigar::CIGAR::from_bytestring(&link.overlap).unwrap();
        let from = Handle::new(
            self.seg_ids.get(link.from_segment),
            convert_orient(link.from_orient),
        );
        let to = Handle::new(
            self.seg_ids.get(link.to_segment),
            convert_orient(link.to_orient),
        );
        self.flat.add_link(from, to, convert_cigar(&cigar));
    }

    fn add_path(&mut self, path: gfa::gfa::Path<usize, OptFields>) {
        let steps = StepsParser::new(&path.segment_names)
            .into_iter()
            .map(|(name, dir)| {
                Handle::new(
                    self.seg_ids.get(name),
                    if dir {
                        Orientation::Forward
                    } else {
                        Orientation::Backward
                    },
                )
            });

        // When the overlaps section is just `*`, the rs-gfa library produces a
        // vector like `[None]`. I'm not sure if we really need to handle `None`
        // otherwise: all the real data I've seen either has *real* overlaps or
        // is just `*`.
        let overlaps = if path.overlaps.len() == 1 && path.overlaps[0].is_none() {
            vec![]
        } else {
            path.overlaps
        };
        let overlaps = overlaps.iter().map(|o| match o {
            Some(c) => convert_cigar(c),
            None => unimplemented!(),
        });

        self.flat
            .add_path(path.path_name, steps.into_iter(), overlaps);
    }

    /// Finish parsing and return the flat representation.
    ///
    /// We "unwind" the buffers of links and paths, now that we have all
    /// the segments.
    fn finish(mut self, deferred: Deferred) -> FlatGFAStore {
        for link in deferred.links {
            self.add_link(link);
        }
        for path in deferred.paths {
            self.add_path(path);
        }
        self.flat
    }
}

#[derive(Default)]
struct NameMap {
    /// Names at most this are assigned *sequential* IDs, i.e., the ID is just the name
    /// minus one.
    sequential_max: usize,

    /// Non-sequential names go here.
    others: HashMap<usize, u32>,
}

impl NameMap {
    fn insert(&mut self, name: usize, id: u32) {
        // Is this the next sequential name? If so, no need to record it in our hash table;
        // just bump the number of sequential names we've seen.
        if (name - 1) == self.sequential_max && (name - 1) == (id as usize) {
            self.sequential_max += 1;
        } else {
            self.others.insert(name, id);
        }
    }

    fn get(&self, name: usize) -> u32 {
        if name <= self.sequential_max {
            (name - 1) as u32
        } else {
            self.others[&name]
        }
    }
}

fn convert_orient(o: gfa::gfa::Orientation) -> Orientation {
    match o {
        gfa::gfa::Orientation::Forward => Orientation::Forward,
        gfa::gfa::Orientation::Backward => Orientation::Backward,
    }
}

fn convert_align_op(c: &cigar::CIGARPair) -> AlignOp {
    AlignOp::new(
        match c.op() {
            cigar::CIGAROp::M => crate::flatgfa::AlignOpcode::Match,
            cigar::CIGAROp::N => crate::flatgfa::AlignOpcode::Gap,
            cigar::CIGAROp::D => crate::flatgfa::AlignOpcode::Deletion,
            cigar::CIGAROp::I => crate::flatgfa::AlignOpcode::Insertion,
            _ => unimplemented!(),
        },
        c.len(),
    )
}

fn convert_cigar(c: &cigar::CIGAR) -> Vec<AlignOp> {
    c.0.iter().map(convert_align_op).collect()
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
///
/// The underlying gfa-rs library does not yet parse the actual segments
/// involved in the path. So we do it ourselves: splitting on commas and
/// matching the direction.
struct StepsParser<'a> {
    str: &'a [u8],
    index: usize,
    state: StepsParseState,
    seg: usize,
}

/// The parser state: we're either looking for a segment name (or a +/- terminator),
/// or we're expecting a comma (or end of string).
enum StepsParseState {
    Seg,
    Comma,
}

impl<'a> StepsParser<'a> {
    pub fn new(str: &'a [u8]) -> Self {
        StepsParser {
            str,
            index: 0,
            state: StepsParseState::Seg,
            seg: 0,
        }
    }
}

impl<'a> Iterator for StepsParser<'a> {
    type Item = (usize, bool);
    fn next(&mut self) -> Option<(usize, bool)> {
        while self.index < self.str.len() {
            // Consume one byte.
            let byte = self.str[self.index];
            self.index += 1;

            match self.state {
                StepsParseState::Seg => {
                    if byte == b'+' || byte == b'-' {
                        self.state = StepsParseState::Comma;
                        return Some((self.seg, byte == b'+'));
                    } else if byte.is_ascii_digit() {
                        self.seg *= 10;
                        self.seg += (byte - b'0') as usize;
                    } else {
                        panic!("unexpected character in path: {}", byte as char);
                    }
                }
                StepsParseState::Comma => {
                    if byte == b',' {
                        self.state = StepsParseState::Seg;
                        self.seg = 0;
                    } else {
                        panic!("unexpected character in path: {}", byte as char);
                    }
                }
            }
        }

        None
    }
}

#[test]
fn test_parse_path() {
    let str = b"1+,23-,4+";
    let path: Vec<_> = StepsParser::new(str).collect();
    assert_eq!(path, vec![(1, true), (23, false), (4, true)]);
}
