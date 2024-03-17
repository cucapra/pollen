use crate::flatgfa::{FlatGFAStore, Handle, Orientation, Span};
use combine::error::UnexpectedParse;
use combine::parser::byte::{byte, digit};
use combine::parser::range::recognize;
use combine::parser::repeat::{sep_by, skip_many1};
use combine::{parser, Parser};
use std::str;

fn number<'a>() -> impl Parser<&'a [u8], Output = u32> {
    recognize(skip_many1(digit())).and_then(|bs: &[u8]| {
        // Following the Combine docs' example, this is safe because the string is
        // guaranteed to be ASCII.
        let s = unsafe { str::from_utf8_unchecked(bs) };
        s.parse::<u32>().map_err(|_| UnexpectedParse::Unexpected)
    })
}

fn orient<'a>() -> impl Parser<&'a [u8], Output = Orientation> {
    byte(b'+')
        .map(|_| Orientation::Forward)
        .or(byte(b'-').map(|_| Orientation::Backward))
}

fn handle<'a>() -> impl Parser<&'a [u8], Output = Handle> {
    number().and(orient()).map(|(n, o)| Handle::new(n, o))
}

fn steps<'a>() -> impl Parser<&'a [u8], Output = Vec<Handle>> {
    sep_by(handle(), byte(b','))
}

fn steps_insert<'a, 'b: 'a>(
    store: &'a mut FlatGFAStore,
) -> impl Parser<&'b [u8], Output = Span> + 'a {
    parser(|input: &mut &[u8]| {
        // A bit of a hack to get `sep_by` behavior with iteration: first parse one step.
        let first;
        (first, *input) = handle().parse(input).unwrap();
        store.steps.push(first);

        // Then iterate over all the rest of the steps, requiring a comma before each.
        let mut iter = byte(b',').with(handle()).iter(input);
        let span = store.add_steps(&mut iter);

        iter.into_result(Span {
            start: span.start - 1, // Account for the `push` above.
            end: span.end,
        })
    })
}

pub fn parse_steps(store: &mut FlatGFAStore, str: &[u8]) -> Span {
    let mut parser = steps_insert(store);
    let (parsed, _) = parser.parse(str).unwrap();
    parsed
}
