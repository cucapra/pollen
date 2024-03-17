type ParseResult<'a> = Result<Line<'a>, &'static str>;

pub enum Line<'a> {
    Header { data: &'a [u8] },
}

pub fn parse_line(line: &[u8]) -> ParseResult {
    if line.len() < 2 || line[1] != b'\t' {
        return Err("expected marker and tab");
    }
    match line[0] {
        b'H' => parse_header(&line[2..]),
        _ => Err("unhandled line kind"),
    }
}

fn parse_header(line: &[u8]) -> ParseResult {
    Ok(Line::Header { data: line })
}
