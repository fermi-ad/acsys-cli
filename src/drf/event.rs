use combine::error::ParseError;
use combine::parser::{char, repeat};
use combine::stream::Stream;
use combine::{attempt, choice, one_of, optional, value, Parser};
use super::Event;

fn scale_rate(suf: Option<char>, text: String) -> u32 {
    match text.parse::<u32>() {
        Ok(v) => match suf {
            Some('s') | Some('S') => v * 1000000,
            Some('m') | Some('M') | None => v * 1000,
            Some('u') | Some('U') => v,
            Some('k') | Some('K') => 1000 / v,
            Some('h') | Some('H') => 1000000 / v,
            Some(_) => unreachable!(),
        },
        Err(_) => panic!("bad value"),
    }
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
            attempt(char::string("TRUE"))
                .or(attempt(char::string("FALSE")))
                .or(char::string("T"))
                .or(char::string("F")),
        ),
    )
    .map(|v| match v {
        Some("TRUE") | Some("T") => true,
        Some("FALSE") | Some("F") | None => false,
        Some(_) => unreachable!(),
    })
}

// Returns a parser that looks for the ",rate[,imm]" portion of a
// periodic event string.

fn parse_periodic_rate<Input>() -> impl Parser<Input, Output = (u32, bool)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(char::char(',').with((
        repeat::many1(char::digit()),
        optional(one_of("sSmMuUkKhH".chars())),
        parse_periodic_imm(),
    )))
    .map(|v: Option<(String, Option<char>, bool)>| {
        if let Some((text, suf, imm)) = v {
            (scale_rate(suf, text), imm)
        } else {
            (1000000u32, false)
        }
    })
}

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
    fn test_event_parsing() {
        assert_eq!(parser().parse("N"), Ok((Event::Never, "")));
        assert_eq!(parser().parse("n"), Ok((Event::Never, "")));

        assert_eq!(parser().parse("I"), Ok((Event::Immediate, "")));
        assert_eq!(parser().parse("i"), Ok((Event::Immediate, "")));

        let periodic_data = &[
            ("P", 1000000u32, false, false, ""),
            ("PD", 1000000u32, false, false, "D"),
            ("P,1000", 1000000u32, false, false, ""),
            ("P,1S", 1000000u32, false, false, ""),
            ("P,10S", 10000000u32, false, false, ""),
            ("P,1U", 1u32, false, false, ""),
            ("P,1K", 1000u32, false, false, ""),
            ("P,2K", 500u32, false, false, ""),
            ("P,1H", 1000000u32, false, false, ""),
            ("P,10H", 100000u32, false, false, ""),
            ("Q", 1000000u32, false, true, ""),
            ("QD", 1000000u32, false, true, "D"),
            ("Q,1000", 1000000u32, false, true, ""),
            ("Q,1S", 1000000u32, false, true, ""),
            ("Q,10S", 10000000u32, false, true, ""),
            ("Q,1U", 1u32, false, true, ""),
            ("Q,1K", 1000u32, false, true, ""),
            ("Q,2K", 500u32, false, true, ""),
            ("Q,1H", 1000000u32, false, true, ""),
            ("Q,10H", 100000u32, false, true, ""),
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
    }
}
