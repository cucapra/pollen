use crate::flatgfa::{AlignOp, Orientation};
use atoi::FromRadix10;

type ParseResult<T> = Result<T, &'static str>;
type LineResult<'a> = ParseResult<Line<'a>>;
type PartialParseResult<'a, T> = ParseResult<(T, &'a [u8])>;

/// A parsed GFA file line.
pub enum Line<'a> {
    Header(&'a [u8]),
    Segment(Segment<'a>),
    Link(Link),
    Path(Path<'a>),
}

pub struct Segment<'a> {
    pub name: usize,
    pub seq: &'a [u8],
    pub data: &'a [u8],
}

pub struct Link {
    pub from_seg: usize,
    pub from_orient: Orientation,
    pub to_seg: usize,
    pub to_orient: Orientation,
    pub overlap: Vec<AlignOp>,
}

pub struct Path<'a> {
    pub name: &'a [u8],
    pub steps: &'a [u8],
    pub overlaps: Vec<Vec<AlignOp>>,
}

/// Parse a single line of a GFA file.
pub fn parse_line(line: &[u8]) -> LineResult {
    if line.len() < 2 || line[1] != b'\t' {
        return Err("expected marker and tab");
    }
    let rest = &line[2..];
    match line[0] {
        b'H' => parse_header(rest),
        b'S' => parse_seg(rest),
        b'L' => parse_link(rest),
        b'P' => parse_path(rest),
        _ => Err("unhandled line kind"),
    }
}

/// Parse a header line, which looks like `H <data>`.
fn parse_header(line: &[u8]) -> LineResult {
    Ok(Line::Header(line))
}

/// Parse a segment line, which looks like `S <name> <seq> <data>`.
fn parse_seg(line: &[u8]) -> LineResult {
    let (name, rest) = parse_num(line)?;
    let rest = parse_byte(rest, b'\t')?;
    let (seq, data) = parse_field(rest)?;
    Ok(Line::Segment(Segment { name, seq, data }))
}

/// Parse a link line, which looks like `L <from> <+-> <to> <+-> <CIGAR>`.
fn parse_link(line: &[u8]) -> LineResult {
    let (from_seg, rest) = parse_num(line)?;
    let rest = parse_byte(rest, b'\t')?;
    let (from_orient, rest) = parse_orient(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (to_seg, rest) = parse_num(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (to_orient, rest) = parse_orient(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (overlap, rest) = parse_align(rest)?;
    if !rest.is_empty() {
        return Err("expected end of line");
    }
    Ok(Line::Link(Link {
        from_seg,
        from_orient,
        to_seg,
        to_orient,
        overlap,
    }))
}

/// Parse a path line, which looks like `P <name> <steps> <*|CIGARs>`.
fn parse_path(line: &[u8]) -> LineResult {
    let (name, rest) = parse_field(line)?;
    let (steps, rest) = parse_field(rest)?;
    let (overlaps, rest) = parse_maybe_overlap_list(rest)?;
    if !rest.is_empty() {
        return Err("expected end of line");
    }
    Ok(Line::Path(Path {
        name,
        steps,
        overlaps,
    }))
}

/// Parse a *possible* overlap list, which may be `*` (empty).
pub fn parse_maybe_overlap_list(s: &[u8]) -> PartialParseResult<Vec<Vec<AlignOp>>> {
    if s == b"*" {
        Ok((vec![], &s[1..]))
    } else {
        parse_overlap_list(s)
    }
}

/// Parse a comma-separated list of CIGAR strings.
///
/// TODO: This could be optimized to avoid accumulating into a vector.
fn parse_overlap_list(s: &[u8]) -> PartialParseResult<Vec<Vec<AlignOp>>> {
    let mut rest = s;
    let mut overlaps = vec![];
    while !rest.is_empty() {
        let overlap;
        (overlap, rest) = parse_align(rest)?;
        overlaps.push(overlap);
        if !rest.is_empty() {
            rest = parse_byte(rest, b',')?;
        }
    }
    Ok((overlaps, rest))
}

/// Consume a chunk of a string up to a given marker byte.
fn parse_until(line: &[u8], marker: u8) -> PartialParseResult<&[u8]> {
    let end = memchr::memchr(marker, line).unwrap_or(line.len());
    let rest = if end == line.len() {
        &[]
    } else {
        &line[end + 1..]
    };
    Ok((&line[..end], rest))
}

/// Consume a string from the line, until a tab (or the end of the line).
pub fn parse_field(line: &[u8]) -> PartialParseResult<&[u8]> {
    parse_until(line, b'\t')
}

/// Consume a specific byte.
fn parse_byte(s: &[u8], byte: u8) -> ParseResult<&[u8]> {
    if s.is_empty() || s[0] != byte {
        return Err("expected byte");
    }
    Ok(&s[1..])
}

/// Parse a single integer.
fn parse_num<T: FromRadix10>(s: &[u8]) -> PartialParseResult<T> {
    match T::from_radix_10(s) {
        (_, 0) => Err("expected number"),
        (num, used) => Ok((num, &s[used..])),
    }
}

/// Parse a segment orientation (+ or -).
fn parse_orient(line: &[u8]) -> PartialParseResult<Orientation> {
    if line.is_empty() {
        return Err("expected orientation");
    }
    let orient = match line[0] {
        b'+' => Orientation::Forward,
        b'-' => Orientation::Backward,
        _ => return Err("expected orient"),
    };
    Ok((orient, &line[1..]))
}

/// Parse a single CIGAR alignment operation (like `4D`).
fn parse_align_op(s: &[u8]) -> PartialParseResult<AlignOp> {
    let (len, rest) = parse_num::<u32>(s)?;
    let op = match rest[0] {
        b'M' => crate::flatgfa::AlignOpcode::Match,
        b'N' => crate::flatgfa::AlignOpcode::Gap,
        b'D' => crate::flatgfa::AlignOpcode::Deletion,
        b'I' => crate::flatgfa::AlignOpcode::Insertion,
        _ => return Err("expected align op"),
    };
    Ok((AlignOp::new(op, len), &rest[1..]))
}

/// Parse a complete CIGAR alignment string (like `3M2I`).
///
/// TODO This could be optimized to avoid collecting into a vector.
fn parse_align(s: &[u8]) -> PartialParseResult<Vec<AlignOp>> {
    let mut rest = s;
    let mut align = vec![];
    while !rest.is_empty() && rest[0].is_ascii_digit() {
        let op;
        (op, rest) = parse_align_op(rest)?;
        align.push(op);
    }
    Ok((align, rest))
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
pub struct StepsParser<'a> {
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

    pub fn rest(&self) -> &[u8] {
        &self.str[self.index..]
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
                        return None;
                    }
                }
                StepsParseState::Comma => {
                    if byte == b',' {
                        self.state = StepsParseState::Seg;
                        self.seg = 0;
                    } else {
                        return None;
                    }
                }
            }
        }

        None
    }
}

#[test]
fn test_parse_steps() {
    let s = b"1+,23-,4+ suffix";
    let mut parser = StepsParser::new(s);
    let path: Vec<_> = (&mut parser).collect();
    assert_eq!(path, vec![(1, true), (23, false), (4, true)]);
    assert_eq!(parser.rest(), b"suffix");
}
