type ParseResult<'a> = Result<Line<'a>, &'static str>;

pub enum Line<'a> {
    Header {
        data: &'a [u8],
    },
    Segment {
        name: usize,
        seq: &'a [u8],
        data: &'a [u8],
    },
}

pub fn parse_line(line: &[u8]) -> ParseResult {
    if line.len() < 2 || line[1] != b'\t' {
        return Err("expected marker and tab");
    }
    let rest = &line[2..];
    match line[0] {
        b'H' => parse_header(rest),
        b'S' => parse_seg(rest),
        _ => Err("unhandled line kind"),
    }
}

fn parse_header(line: &[u8]) -> ParseResult {
    Ok(Line::Header { data: line })
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
    Ok(Line::Segment { name, seq, data })
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
