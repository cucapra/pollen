use crate::flatgfa::{Handle, Orientation, Span};
use crate::parse;
use combine::error::UnexpectedParse;
use combine::parser::byte::{byte, digit};
use combine::parser::range::recognize;
use combine::parser::repeat::skip_many1;
use combine::{parser, Parser};
use std::str;

fn number<'a>() -> impl Parser<&'a [u8], Output = usize> {
    recognize(skip_many1(digit())).and_then(|bs: &[u8]| {
        // Following the Combine docs' example, this is safe because the string is
        // guaranteed to be ASCII.
        let s = unsafe { str::from_utf8_unchecked(bs) };
        s.parse::<usize>().map_err(|_| UnexpectedParse::Unexpected)
    })
}

fn orient<'a>() -> impl Parser<&'a [u8], Output = Orientation> {
    byte(b'+')
        .map(|_| Orientation::Forward)
        .or(byte(b'-').map(|_| Orientation::Backward))
}

fn handle<'a, 'b: 'a>(seg_ids: &'a parse::NameMap) -> impl Parser<&'b [u8], Output = Handle> + 'a {
    number()
        .and(orient())
        .map(|(n, o)| Handle::new(seg_ids.get(n), o))
}

impl parse::Parser {
    fn steps_insert<'a, 'b: 'a>(&'a mut self) -> impl Parser<&'b [u8], Output = Span> + 'a {
        parser(|input: &mut &[u8]| {
            // A bit of a hack to get `sep_by` behavior with iteration: first parse one step.
            let first;
            (first, *input) = handle(&self.seg_ids).parse(input).unwrap();
            self.flat.steps.push(first);

            // Then iterate over all the rest of the steps, requiring a comma before each.
            let mut iter = byte(b',').with(handle(&self.seg_ids)).iter(input);
            let span = self.flat.add_steps(&mut iter);

            iter.into_result(Span {
                start: span.start - 1, // Account for the `push` above.
                end: span.end,
            })
        })
    }

    pub fn parse_steps(&mut self, str: &[u8]) -> Span {
        let mut parser = self.steps_insert();
        let (parsed, _) = parser.parse(str).unwrap();
        parsed
    }
}
