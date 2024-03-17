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

pub enum Line<'a> {
    Header(&'a [u8]),
    Segment(Segment<'a>),
    Link(Link),
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
        _ => Err("unhandled line kind"),
    }
}

fn parse_header(line: &[u8]) -> ParseResult {
    Ok(Line::Header(line))
}

fn parse_seg(line: &[u8]) -> ParseResult {
    let (name, rest) = parse_num(line)?;
    let rest = parse_byte(rest, b'\t')?;
    let (seq, rest) = parse_field(rest)?;
    let data = if rest.is_empty() {
        &[]
    } else {
        parse_byte(rest, b'\t')?
    };
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

fn parse_field(line: &[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    let end = line.iter().position(|&b| b == b'\t').unwrap_or(line.len());
    Ok((&line[..end], &line[end..]))
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
