use super::Event;
use combine::error::ParseError;
use combine::parser::{char, repeat};
use combine::stream::Stream;
use combine::{attempt, choice, one_of, optional, value, Parser};

// Takes numeric text and an optional suffix character and computes
// the microsecond, periodic rate that represents it. If the user
// provided bad input, the result is clipped to the extreme that was
// exceeded.

fn scale_rate(suf: Option<char>, text: String) -> u32 {
    // The only way `text.parse()` can fail is if the digits in the
    // string exceed the value that a u32 can hold. So clip it to the
    // max value.

    let v = if let Ok(v) = text.parse::<u32>() { v } else { u32::MAX };

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

// Returns a time-freq value (u32) of the form ",TIME-FREQ". This
// field is assumed to be optional, so the function may return None.

fn parse_time_freq<Input>() -> impl Parser<Input, Output = Option<u32>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(char::char(',').with((
        repeat::many1(char::digit()),
        optional(one_of("sSmMuUkKhH".chars())),
    )))
    .map(|v: Option<(String, Option<char>)>| {
        if let Some((text, suf)) = v {
            Some(scale_rate(suf, text))
        } else {
            None
        }
    })
}

// Returns a parser that looks for the trailing ",TRUE/FALSE" portion
// of a periodic event string.

fn parse_periodic_imm<Input>() -> impl Parser<Input, Output = bool>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(
        char::char(',').with(
            attempt(char::string("TRUE").with(value(true)))
                .or(attempt(char::string("FALSE").with(value(false))))
                .or(char::char('T').with(value(true)))
                .or(char::char('F').with(value(false)))
        ),
    )
    .map(|v| if let Some(v) = v { v } else { false })
}

// Returns a parser that looks for the ",rate[,imm]" portion of a
// periodic event string.

fn parse_periodic_rate<Input>() -> impl Parser<Input, Output = (u32, bool)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    parse_time_freq()
        .and(parse_periodic_imm())
        .map(|(r, i)| (if let Some(r) = r { r } else { 1000000u32 }, i))
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

    choice((
        parse_never,
        parse_immediate,
        parse_periodic,
        parse_periodic_filt,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_freq_parsing() {
        let data: &[(&str, Option<u32>, &str)] = &[
            // These make sure it starts with a comma.
            ("", None, ""),
            ("X", None, "X"),
            // Tests for "milliseconds" input (which is the default).
            (",100", Some(100000u32), ""),
            (",200-", Some(200000u32), "-"),
            (",1000m", Some(1000000u32), ""),
            (",2000M", Some(2000000u32), ""),
            // Tests for "seconds" input.
            (",2s", Some(2000000u32), ""),
            (",5S", Some(5000000u32), ""),
            // Tests for "microseconds" input.
            (",10uB", Some(10u32), "B"),
            (",1000U", Some(1000u32), ""),
            // Tests for Hertz input.
            (",10h", Some(100000u32), ""),
            (",200H", Some(5000u32), ""),
            (",3h", Some(333333u32), ""),
            (",1000000h", Some(1u32), ""),
            // Tests for KHertz input.
            (",10k", Some(100u32), ""),
            (",200K", Some(5u32), ""),
            (",1000K", Some(1u32), ""),
            // Check the clipping algorithms.
            (",4294S", Some(4294000000u32), ""),
            (",4295S", Some(u32::MAX), ""),
            (",4294967m", Some(4294967000u32), ""),
            (",4294968m", Some(u32::MAX), ""),
            (",4294968", Some(u32::MAX), ""),
            (",0h", Some(u32::MAX), ""),
            (",1000001H", Some(1u32), ""),
            (",0k", Some(u32::MAX), ""),
            (",1001K", Some(1u32), ""),
        ];

        for &(p, r, x) in data {
            assert_eq!(parse_time_freq().parse(p), Ok((r, x)));
        }
    }

    #[test]
    fn test_event_parsing() {
        assert_eq!(parser().parse("N"), Ok((Event::Never, "")));
        assert_eq!(parser().parse("n"), Ok((Event::Never, "")));

        assert_eq!(parser().parse("I"), Ok((Event::Immediate, "")));
        assert_eq!(parser().parse("i"), Ok((Event::Immediate, "")));

        let periodic_data = &[
            ("P", 1000000u32, false, false, ""),
            ("pD", 1000000u32, false, false, "D"),
            ("P,1000", 1000000u32, false, false, ""),
            ("p,1S,T", 1000000u32, true, false, ""),
            ("P,10S,TRUE", 10000000u32, true, false, ""),
            ("P,1U", 1u32, false, false, ""),
            ("P,1K,F", 1000u32, false, false, ""),
            ("P,2K,FALSE", 500u32, false, false, ""),
            ("P,1H", 1000000u32, false, false, ""),
            ("P,10H", 100000u32, false, false, ""),
            ("Q", 1000000u32, false, true, ""),
            ("QD", 1000000u32, false, true, "D"),
            ("Q,1000", 1000000u32, false, true, ""),
            ("Q,2000z", 2000000u32, false, true, "z"),
            ("Q,1S,T", 1000000u32, true, true, ""),
            ("Q,10S", 10000000u32, false, true, ""),
            ("Q,1U,TRUE", 1u32, true, true, ""),
            ("Q,1K", 1000u32, false, true, ""),
            ("Q,2K", 500u32, false, true, ""),
            ("Q,1H", 1000000u32, false, true, ""),
            ("Q,10hz", 100000u32, false, true, "z"),
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

        assert!(parser().parse("P,").is_err());
        assert!(parser().parse("P,junk").is_err());
        assert!(parser().parse("P,1000,").is_err());

        // These should fail because, if we don't have the time-freq
        // field, then we can't proceed to parse the immediate flag
        // field.

        assert!(parser().parse("P,TRUE").is_err());
        assert!(parser().parse("p,T").is_err());
        assert!(parser().parse("P,FALSE").is_err());
        assert!(parser().parse("P,F").is_err());
    }
}
