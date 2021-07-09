use super::{ClockType, Event, StateOp};
use combine::error::{ParseError, StreamError};
use combine::parser::{char, repeat};
use combine::stream::{Stream, StreamErrorFor};
use combine::{attempt, choice, one_of, optional, value, Parser};

// Takes numeric text and an optional suffix character and computes
// the microsecond, periodic rate that represents it. If the user
// provided bad input, the result is clipped to the extreme that was
// exceeded.

fn scale_rate(v: u32, suf: Option<char>) -> u32 {
    match suf {
        Some('s') | Some('S') => {
            if v > u32::MAX / 1000000 {
                u32::MAX
            } else {
                v * 1000000
            }
        }
        Some('m') | Some('M') | None => {
            if v > u32::MAX / 1000 {
                u32::MAX
            } else {
                v * 1000
            }
        }
        Some('u') | Some('U') => v,
        Some('k') | Some('K') => {
            if v == 0 {
                u32::MAX
            } else if v > 1000 {
                1
            } else {
                1000 / v
            }
        }
        Some('h') | Some('H') => {
            if v == 0 {
                u32::MAX
            } else if v > 1000000 {
                1
            } else {
                1000000 / v
            }
        }
        Some(_) => unreachable!(),
    }
}

// Consumes a block of digits and converts them to a `u32` type, if
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

// Consumes a block of hexadecimal digits and converts them to a `u16`
// type, if possible.

fn parse_clock_event<Input>() -> impl Parser<Input, Output = u16>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    repeat::many1(char::hex_digit())
        .and_then(|v: String| u16::from_str_radix(&v, 16).map_err(StreamErrorFor::<Input>::other))
}

// Returns a time-freq value (u32) of the form ",TIME-FREQ". This
// field is assumed to be optional, so the function may return None.

fn parse_time_freq<Input>() -> impl Parser<Input, Output = u32>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (parse_int(), optional(one_of("sSmMuUkKhH".chars())))
        .map(|(val, suf): (u32, Option<char>)| scale_rate(val, suf))
}

// Returns a parser that looks for the trailing ",TRUE/FALSE" portion
// of a periodic event string.

fn parse_periodic_imm<Input>() -> impl Parser<Input, Output = bool>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(
        char::char(',').with(repeat::many1(char::letter()).and_then(|v: String| {
            match v.to_uppercase().as_str() {
                "TRUE" | "T" => Ok(true),
                "FALSE" | "F" => Ok(false),
                _ => Err(StreamErrorFor::<Input>::message("unknown keyword")),
            }
        })),
    )
    .map(|v| if let Some(v) = v { v } else { true })
}

// Returns a parser that looks for the trailing state-op portion
// of a state event string.

fn parse_ops<Input>() -> impl Parser<Input, Output = StateOp>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        attempt(char::string("<=")).with(value(StateOp::LEq)),
        attempt(char::string(">=")).with(value(StateOp::GEq)),
        attempt(char::string("!=")).with(value(StateOp::NEq)),
        char::char('>').with(value(StateOp::GT)),
        char::char('<').with(value(StateOp::LT)),
        char::char('=').with(value(StateOp::Eq)),
        char::char('*').with(value(StateOp::All)),
    ))
}

// Returns a parser that looks for the ",rate[,imm]" portion of a
// periodic event string.

fn parse_periodic_rate<Input>() -> impl Parser<Input, Output = (u32, bool)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(char::char(',').with(parse_time_freq()))
        .and(parse_periodic_imm())
        .map(|(r, i)| (if let Some(r) = r { r } else { 1000000u32 }, i))
}

// Returns a parser that understands the clock type field in a clock
// event string.

fn parse_clock_type<Input>() -> impl Parser<Input, Output = ClockType>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char::char(',').with(
        one_of("eE".chars())
            .with(value(ClockType::Either))
            .or(one_of("hH".chars()).with(value(ClockType::Hardware)))
            .or(one_of("sS".chars()).with(value(ClockType::Software))),
    )
}

// This is the entry point for this module. It uses the other parsers
// in this module to decode the event string from the incoming text.

pub fn parser<Input>() -> impl Parser<Input, Output = Event>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let parse_never = one_of("nN".chars()).with(value(Event::Never));

    let parse_immediate = one_of("iI".chars()).with(value(Event::Immediate));

    let parse_periodic = one_of("pP".chars())
        .with(parse_periodic_rate())
        .map(|(r, i)| Event::Periodic {
            period: r,
            immediate: i,
            skip_dups: false,
        });

    let parse_periodic_filt = one_of("qQ".chars())
        .with(parse_periodic_rate())
        .map(|(r, i)| Event::Periodic {
            period: r,
            immediate: i,
            skip_dups: true,
        });

    let parse_clock = one_of("eE".chars())
        .with((
            char::char(',').with(parse_clock_event()),
            optional(parse_clock_type()),
            optional(char::char(',').with(parse_time_freq())),
        ))
        .map(
            |(event, ct, r): (u16, Option<ClockType>, Option<u32>)| Event::Clock {
                event,
                clk_type: ct.unwrap_or(ClockType::default()),
                delay: r.unwrap_or(0),
            },
        );

    let parse_state = one_of("sS".chars())
        .with((
            char::char(',').with(parse_int()),
            char::char(',').with(parse_int()),
            char::char(',').with(parse_time_freq()),
            char::char(',').with(parse_ops()),
        ))
        .map(|(device, value, delay, expr)| Event::State {
            device,
            value,
            delay,
            expr,
        });

    // Create an all-encompassing parser which tries each of the
    // parsers above. None of these need to be wtapped with `attempt`
    // because they all start with a parser that looks for a single
    // matching character so no input will be consumed if it doesn't
    // match. Once the character matches, however, that parser is
    // committed to finishing (because `choice` won't try an
    // alternative if input is consumed -- event partially -- by a
    // sub-parser.)

    char::char('@')
        .with(choice((
            parse_never,
            parse_immediate,
            parse_periodic,
            parse_periodic_filt,
            parse_clock,
            parse_state,
        )))
        .or(value(Event::Default))
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::EasyParser;

    #[test]
    fn test_time_freq_parsing() {
        let data: &[(&str, Option<u32>, &str)] = &[
            // These make sure it starts with a comma.
            ("", None, ""),
            ("X", None, "X"),
            // Tests for "milliseconds" input (which is the default).
            ("100", Some(100000u32), ""),
            ("200-", Some(200000u32), "-"),
            ("1000m", Some(1000000u32), ""),
            ("2000M", Some(2000000u32), ""),
            // Tests for "seconds" input.
            ("2s", Some(2000000u32), ""),
            ("5S", Some(5000000u32), ""),
            // Tests for "microseconds" input.
            ("10uB", Some(10u32), "B"),
            ("1000U", Some(1000u32), ""),
            // Tests for Hertz input.
            ("10h", Some(100000u32), ""),
            ("200H", Some(5000u32), ""),
            ("3h", Some(333333u32), ""),
            ("1000000h", Some(1u32), ""),
            // Tests for KHertz input.
            ("10k", Some(100u32), ""),
            ("200K", Some(5u32), ""),
            ("1000K", Some(1u32), ""),
            // Check the clipping algorithms.
            ("4294S", Some(4294000000u32), ""),
            ("4295S", Some(u32::MAX), ""),
            ("4294967m", Some(4294967000u32), ""),
            ("4294968m", Some(u32::MAX), ""),
            ("4294968", Some(u32::MAX), ""),
            ("0h", Some(u32::MAX), ""),
            ("1000001H", Some(1u32), ""),
            ("0k", Some(u32::MAX), ""),
            ("1001K", Some(1u32), ""),
        ];

        for &(p, r, x) in data {
            assert_eq!(optional(parse_time_freq()).parse(p), Ok((r, x)));
        }
    }

    #[test]
    fn test_event_parsing() {
        assert_eq!(parser().parse("@N"), Ok((Event::Never, "")));
        assert_eq!(parser().parse("@n"), Ok((Event::Never, "")));

        assert_eq!(parser().parse("@I"), Ok((Event::Immediate, "")));
        assert_eq!(parser().parse("@i"), Ok((Event::Immediate, "")));

        let periodic_data = &[
            ("@P", 1000000u32, true, false, ""),
            ("@pD", 1000000u32, true, false, "D"),
            ("@P,1000", 1000000u32, true, false, ""),
            ("@p,1S,t", 1000000u32, true, false, ""),
            ("@P,10S,TRUE", 10000000u32, true, false, ""),
            ("@P,1U", 1u32, true, false, ""),
            ("@P,1K,f", 1000u32, false, false, ""),
            ("@P,2K,FALSE", 500u32, false, false, ""),
            ("@P,1H", 1000000u32, true, false, ""),
            ("@P,10H", 100000u32, true, false, ""),
            ("@Q", 1000000u32, true, true, ""),
            ("@QD", 1000000u32, true, true, "D"),
            ("@Q,1000", 1000000u32, true, true, ""),
            ("@Q,2000z", 2000000u32, true, true, "z"),
            ("@Q,1S,T", 1000000u32, true, true, ""),
            ("@Q,10S", 10000000u32, true, true, ""),
            ("@Q,1U,true", 1u32, true, true, ""),
            ("@Q,1K", 1000u32, true, true, ""),
            ("@Q,2K", 500u32, true, true, ""),
            ("@Q,1H", 1000000u32, true, true, ""),
            ("@Q,10hz", 100000u32, true, true, "z"),
        ];

        for &(p, r, i, s, x) in periodic_data {
            assert_eq!(
                parser().parse(p),
                Ok((
                    Event::Periodic {
                        period: r,
                        immediate: i,
                        skip_dups: s
                    },
                    x
                ))
            );
        }

        // This should fail because we consumed the comma, but didn't
        // find a digit.

        assert!(parser().parse("@P,").is_err());
        assert!(parser().parse("@P,junk").is_err());
        assert!(parser().parse("@P,1000,").is_err());

        // These should fail because, if we don't have the time-freq
        // field, then we can't proceed to parse the immediate flag
        // field.

        assert!(parser().parse("@P,1s,TASK").is_err());
        assert!(parser().parse("@Q,1s,FLOAT").is_err());
        assert!(parser().parse("@P,TRUE").is_err());
        assert!(parser().parse("@p,T").is_err());
        assert!(parser().parse("@P,FALSE").is_err());
        assert!(parser().parse("@P,F").is_err());

        let clock_data = &[
            ("@E,0", 0, ClockType::Either, 0, ""),
            ("@E,0,e", 0, ClockType::Either, 0, ""),
            ("@E,0,s", 0, ClockType::Software, 0, ""),
            ("@E,0,h", 0, ClockType::Hardware, 0, ""),
            ("@E,0,h,100", 0, ClockType::Hardware, 100000, ""),
            ("@E,2", 0x2u16, ClockType::Either, 0, ""),
            ("@E,8f", 0x8fu16, ClockType::Either, 0, ""),
            ("@E,89ab", 0x89abu16, ClockType::Either, 0, ""),
            ("@E,000089ab", 0x89abu16, ClockType::Either, 0, ""),
        ];

        for &(txt, ev, ct, dly, extra) in clock_data {
            assert_eq!(
                parser().easy_parse(txt),
                Ok((
                    Event::Clock {
                        event: ev,
                        clk_type: ct,
                        delay: dly
                    },
                    extra
                ))
            );
        }

        assert!(parser().parse("@E,12345").is_err());
        assert!(parser().parse("@E,12345,e").is_err());
        assert!(parser().parse("@E,1234,a").is_err());

        let state_data = &[
            ("@S,100,10,0,*", 100, 10, 0, StateOp::All, ""),
            ("@S,200,15,0,=", 200, 15, 0, StateOp::Eq, ""),
            ("@S,100,10,30,!=", 100, 10, 30000, StateOp::NEq, ""),
            ("@S,100,10,70,>", 100, 10, 70000, StateOp::GT, ""),
            ("@S,1000,10,0,>=", 1000, 10, 0, StateOp::GEq, ""),
            ("@S,100,1000,0,<", 100, 1000, 0, StateOp::LT, ""),
            ("@S,100,100,100,<=", 100, 100, 100000, StateOp::LEq, ""),
        ];

        for &(txt, device, value, delay, expr, extra) in state_data {
            assert_eq!(
                parser().easy_parse(txt),
                Ok((
                    Event::State {
                        device,
                        value,
                        delay,
                        expr
                    },
                    extra
                ))
            );
        }
    }
}
