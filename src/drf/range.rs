use super::Range;
use combine::error::{ParseError, StreamError};
use combine::parser::{char, repeat};
use combine::stream::{Stream, StreamErrorFor};
use combine::{attempt, choice, optional, value, Parser};

// Consumes a block of digits and converts them to an integer type, if
// possible.

fn parse_int<Input, Output>() -> impl Parser<Input, Output = Output>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Output: std::str::FromStr,
    <Output as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    repeat::many1(char::digit())
        .and_then(|v: String| v.parse::<Output>().map_err(StreamErrorFor::<Input>::other))
}

fn parse_array_range<Input>() -> impl Parser<Input, Output = Range>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let one_element = parse_int()
        .skip(char::char(']'))
        .map(|v: u16| Range::Array {
            start_index: v,
            end_index: Some(v),
        });

    let multi_element = (
        optional(parse_int()).skip(char::char(':')),
        optional(parse_int()).skip(char::char(']')),
    )
        .and_then(|v| match v {
            (None, None) | (Some(0), None) => Ok(Range::Full),
            (Some(s), None) => Ok(Range::Array {
                start_index: s,
                end_index: None,
            }),
            (None, Some(e)) => Ok(Range::Array {
                start_index: 0,
                end_index: Some(e),
            }),
            (Some(s), Some(e)) => {
                if s <= e {
                    Ok(Range::Array {
                        start_index: s,
                        end_index: Some(e),
                    })
                } else {
                    Err(StreamErrorFor::<Input>::message("bad range"))
                }
            }
        });

    char::char('[').with(choice((
        char::char(']').with(value(Range::Full)),
        attempt(one_element),
        attempt(multi_element),
    )))
}

fn parse_byte_range<Input>() -> impl Parser<Input, Output = Range>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let one_element = parse_int().skip(char::char('}')).map(|v: u32| Range::Raw {
        offset: v,
        length: Some(1),
    });

    let multi_element = (
        optional(parse_int()).skip(char::char(':')),
        optional(parse_int()).skip(char::char('}')),
    )
        .and_then(|v| match v {
            (None, None) | (Some(0), None) => Ok(Range::Full),
            (Some(o), None) => Ok(Range::Raw {
                offset: o,
                length: None,
            }),
            (None, Some(l)) => {
                if l > 0 {
                    Ok(Range::Raw {
                        offset: 0,
                        length: Some(l),
                    })
                } else {
                    Err(StreamErrorFor::<Input>::message("bad length"))
                }
            }
            (Some(o), Some(l)) => {
                if l > 0 {
                    if o <= u32::MAX - l {
                        Ok(Range::Raw {
                            offset: o,
                            length: Some(l),
                        })
                    } else {
                        Err(StreamErrorFor::<Input>::message("bad range"))
                    }
                } else {
                    Err(StreamErrorFor::<Input>::message("bad length"))
                }
            }
        });

    char::char('{').with(choice((
        char::char('}').with(value(Range::Full)),
        attempt(one_element),
        attempt(multi_element),
    )))
}

pub fn parser<Input>() -> impl Parser<Input, Output = Range>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        parse_array_range(),
        parse_byte_range(),
        value(Range::Array {
            start_index: 0,
            end_index: Some(0),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_range_parsing() {
        assert_eq!(parser().parse("[]"), Ok((Range::Full, "")));
        assert_eq!(parser().parse("[:]"), Ok((Range::Full, "")));
        assert_eq!(parser().parse("[0:]"), Ok((Range::Full, "")));
        assert_eq!(parser().parse("{}"), Ok((Range::Full, "")));
        assert_eq!(parser().parse("{:}"), Ok((Range::Full, "")));
        assert_eq!(parser().parse("{0:}"), Ok((Range::Full, "")));
    }

    #[test]
    fn test_array_range_parsing() {
        let range_data = &[
            ("[0]", 0, Some(0), ""),
            ("[1]", 1, Some(1), ""),
            ("[65535]", 65535, Some(65535), ""),
            ("[:1]", 0, Some(1), ""),
            ("[1:2]", 1, Some(2), ""),
        ];

        for &(text, start_index, end_index, extra) in range_data {
            assert_eq!(
                parser().parse(text),
                Ok((
                    Range::Array {
                        start_index,
                        end_index
                    },
                    extra
                ))
            );
        }

        assert!(parser().parse("[A]").is_err());
        assert!(parser().parse("[65536]").is_err());
        assert!(parser().parse("[2:1]").is_err());
    }

    #[test]
    fn test_byte_range_parsing() {
        let range_data = &[
            ("{0}", 0, Some(1), ""),
            ("{1}", 1, Some(1), ""),
            ("{:1}", 0, Some(1), ""),
            ("{1:2}", 1, Some(2), ""),
            ("{4294967295}", 4294967295, Some(1), ""),
            ("{:4294967295}", 0, Some(4294967295), ""),
            ("{1:4294967294}", 1, Some(4294967294), ""),
        ];

        for &(text, offset, length, extra) in range_data {
            assert_eq!(
                parser().parse(text),
                Ok((Range::Raw { offset, length }, extra))
            );
        }

        assert!(parser().parse("{A}").is_err());
        assert!(parser().parse("{:0}").is_err());
        assert!(parser().parse("{:-1}").is_err());
        assert!(parser().parse("{4294967296}").is_err());
        assert!(parser().parse("{4000000000:294967296}").is_err());
    }
}
