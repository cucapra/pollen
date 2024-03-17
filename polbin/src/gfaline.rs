use crate::flatgfa::{AlignOp, Orientation};
use gfa::cigar;

type ParseResult<'a> = Result<Line<'a>, &'static str>;

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

pub enum Line<'a> {
    Header(&'a [u8]),
    Segment(Segment<'a>),
    Link(Link),
    Path(Path<'a>),
}

pub fn parse_line(line: &[u8]) -> ParseResult {
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

fn parse_header(line: &[u8]) -> ParseResult {
    Ok(Line::Header(line))
}

fn parse_seg(line: &[u8]) -> ParseResult {
    let (name, rest) = parse_num(line)?;
    let rest = parse_byte(rest, b'\t')?;
    let (seq, data) = parse_field(rest)?;
    Ok(Line::Segment(Segment { name, seq, data }))
}

fn parse_link(line: &[u8]) -> ParseResult {
    let (from_seg, rest) = parse_num(line)?;
    let rest = parse_byte(rest, b'\t')?;
    let (from_orient, rest) = parse_orient(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (to_seg, rest) = parse_num(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (to_orient, rest) = parse_orient(rest)?;
    let rest = parse_byte(rest, b'\t')?;
    let (overlap, rest) = parse_field(rest)?;
    if !rest.is_empty() {
        return Err("expected end of line");
    }
    Ok(Line::Link(Link {
        from_seg,
        from_orient,
        to_seg,
        to_orient,
        overlap: parse_cigar(overlap),
    }))
}

fn parse_path(line: &[u8]) -> ParseResult {
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
pub fn parse_maybe_overlap_list(s: &[u8]) -> Result<(Vec<Vec<AlignOp>>, &[u8]), &'static str> {
    if s == b"*" {
        Ok((vec![], &s[1..]))
    } else {
        parse_overlap_list(s)
    }
}

/// Parse a comma-separated list of CIGAR strings.
///
/// TODO: This could be optimized to avoid accumulating into a vector.
fn parse_overlap_list(s: &[u8]) -> Result<(Vec<Vec<AlignOp>>, &[u8]), &'static str> {
    let mut rest = s;
    let mut overlaps = vec![];
    while !rest.is_empty() {
        let (overlap, new_rest) = parse_until(rest, b',')?;
        overlaps.push(parse_cigar(overlap));
        rest = new_rest;
    }
    Ok((overlaps, rest))
}

fn parse_until(line: &[u8], marker: u8) -> Result<(&[u8], &[u8]), &'static str> {
    let end = line.iter().position(|&b| b == marker).unwrap_or(line.len());
    let rest = if end == line.len() {
        &[]
    } else {
        &line[end + 1..]
    };
    Ok((&line[..end], rest))
}

/// Consume a string from the line, until a tab (or the end of the line).
pub fn parse_field(line: &[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    parse_until(line, b'\t')
}

fn parse_byte(s: &[u8], byte: u8) -> Result<&[u8], &'static str> {
    if s.is_empty() || s[0] != byte {
        return Err("expected byte");
    }
    Ok(&s[1..])
}

fn parse_num(line: &[u8]) -> Result<(usize, &[u8]), &'static str> {
    // Scan for digits.
    let mut index = 0;
    while index < line.len() && line[index].is_ascii_digit() {
        index += 1;
    }
    if index == 0 {
        return Err("expected number");
    }

    // Convert the digits to a number.
    // TODO could use `unsafe` here to avoid the cost of `from_utf8`...
    let s = &line[0..index];
    let num = std::str::from_utf8(s)
        .unwrap()
        .parse()
        .map_err(|_| "number too large")?;

    Ok((num, &line[index..]))
}

fn parse_orient(line: &[u8]) -> Result<(Orientation, &[u8]), &'static str> {
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

/// Parse a CIGAR string.
///
/// TODO: This both relies on the `gfa-rs` crate and collects results into a
/// `Vec` instead of streaming. Both could be fixed.
fn parse_cigar(s: &[u8]) -> Vec<AlignOp> {
    let cigar = cigar::CIGAR::from_bytestring(s).unwrap();
    convert_cigar(&cigar)
}

pub fn convert_cigar(c: &cigar::CIGAR) -> Vec<AlignOp> {
    c.0.iter().map(convert_align_op).collect()
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
///
/// The underlying gfa-rs library does not yet parse the actual segments
/// involved in the path. So we do it ourselves: splitting on commas and
/// matching the direction.
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
