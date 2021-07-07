use super::{Device, Property};
use combine::error::ParseError;
use combine::parser::char;
use combine::stream::Stream;
use combine::{choice, many1, one_of, value, Parser};

/*
prop-qualifier	  = ":"       ; Reading and default
                  | "?"       ; Reading
                  | "_"       ; Setting
                  | "|"       ; Basic Status
                  | "&"       ; Basic Control
                  | "@"       ; Analog Alarm
                  | "$"       ; Digital Alarm
                  | "~"       ; Description
*/
fn parse_prop_symbol<Input>() -> impl Parser<Input, Output = Property>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        char::char(':').with(value(Property::Reading(None))),
        char::char('?').with(value(Property::Reading(None))),
        char::char('_').with(value(Property::Setting(None))),
        char::char('|').with(value(Property::Status(None))),
        char::char('&').with(value(Property::Control)),
        char::char('@').with(value(Property::Analog(None))),
        char::char('$').with(value(Property::Digital(None))),
        char::char('~').with(value(Property::Description)),
    ))
}

pub fn parser<Input>() -> impl Parser<Input, Output = (Device, Property)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let valid_characters = choice((char::alpha_num(), one_of("_-:<>;".chars())));

    let parse_di = (char::char('0'), parse_prop_symbol(), many1(char::digit())).map(
        |(character, prop, device): (char, Property, String)| {
            (Device(format!("{}:{}", character, device)), prop)
        },
    );

    let parse_string = (char::letter(), parse_prop_symbol(), many1(valid_characters)).map(
        |(character, prop, device): (char, Property, String)| {
            (Device(format!("{}:{}", character, device)), prop)
        },
    );

    choice((parse_di, parse_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_parsing() {
        let device_data = &[
            ("M:OUTTMP", "M:OUTTMP", Property::Reading(None), ""),
            ("M?OUTTMP", "M:OUTTMP", Property::Reading(None), ""),
            ("M_OUTTMP", "M:OUTTMP", Property::Setting(None), ""),
            ("M|OUTTMP", "M:OUTTMP", Property::Status(None), ""),
            ("M&OUTTMP", "M:OUTTMP", Property::Control, ""),
            ("M@OUTTMP", "M:OUTTMP", Property::Analog(None), ""),
            ("M$OUTTMP", "M:OUTTMP", Property::Digital(None), ""),
            ("M~OUTTMP", "M:OUTTMP", Property::Description, ""),
            ("0:123456", "0:123456", Property::Reading(None), ""),
            ("0?123456", "0:123456", Property::Reading(None), ""),
            ("0_123456", "0:123456", Property::Setting(None), ""),
            ("0|123456", "0:123456", Property::Status(None), ""),
            ("0&123456", "0:123456", Property::Control, ""),
            ("0@123456", "0:123456", Property::Analog(None), ""),
            ("0$123456", "0:123456", Property::Digital(None), ""),
            ("0~123456", "0:123456", Property::Description, ""),
            (
                "M:OUTTMP:outdoor:temp.VAL",
                "M:OUTTMP:outdoor:temp",
                Property::Reading(None),
                ".VAL",
            ),
            (
                "M:OUTTMP.outdoor.temp.VAL",
                "M:OUTTMP",
                Property::Reading(None),
                ".outdoor.temp.VAL",
            ),
            (
                "M:OUTTMP.SETTING",
                "M:OUTTMP",
                Property::Reading(None),
                ".SETTING",
            ),
            (
                "M:OUTTMP[0:3]",
                "M:OUTTMP",
                Property::Reading(None),
                "[0:3]",
            ),
            (
                "M:OUTTMP{0:3}",
                "M:OUTTMP",
                Property::Reading(None),
                "{0:3}",
            ),
            ("M:OUTTMP{:}", "M:OUTTMP", Property::Reading(None), "{:}"),
            ("M:OUTTMP{0:}", "M:OUTTMP", Property::Reading(None), "{0:}"),
            ("M:OUTTMP{:3}", "M:OUTTMP", Property::Reading(None), "{:3}"),
            ("M:OUT~TMP", "M:OUT", Property::Reading(None), "~TMP"),
            ("M:OUT`TMP", "M:OUT", Property::Reading(None), "`TMP"),
            ("M:OUT!TMP", "M:OUT", Property::Reading(None), "!TMP"),
            ("M:OUT#TMP", "M:OUT", Property::Reading(None), "#TMP"),
            ("M:OUT%TMP", "M:OUT", Property::Reading(None), "%TMP"),
            ("M:OUT^TMP", "M:OUT", Property::Reading(None), "^TMP"),
            ("M:OUT*TMP", "M:OUT", Property::Reading(None), "*TMP"),
            ("M:OUT(TMP", "M:OUT", Property::Reading(None), "(TMP"),
            ("M:OUT)TMP", "M:OUT", Property::Reading(None), ")TMP"),
            ("M:OUT-TMP", "M:OUT-TMP", Property::Reading(None), ""),
            ("M:OUT+TMP", "M:OUT", Property::Reading(None), "+TMP"),
            ("M:OUT=TMP", "M:OUT", Property::Reading(None), "=TMP"),
            ("M:OUT{TMP", "M:OUT", Property::Reading(None), "{TMP"),
            ("M:OUT}TMP", "M:OUT", Property::Reading(None), "}TMP"),
            ("M:OUT[TMP", "M:OUT", Property::Reading(None), "[TMP"),
            ("M:OUT]TMP", "M:OUT", Property::Reading(None), "]TMP"),
            ("M:OUT\\TMP", "M:OUT", Property::Reading(None), "\\TMP"),
            ("M:OUT;TMP", "M:OUT;TMP", Property::Reading(None), ""),
            ("M:OUT'TMP", "M:OUT", Property::Reading(None), "'TMP"),
            ("M:OUT\"TMP", "M:OUT", Property::Reading(None), "\"TMP"),
            ("M:OUT<TMP", "M:OUT<TMP", Property::Reading(None), ""),
            ("M:OUT>TMP", "M:OUT>TMP", Property::Reading(None), ""),
            ("M:OUT,TMP", "M:OUT", Property::Reading(None), ",TMP"),
            ("M:OUT/TMP", "M:OUT", Property::Reading(None), "/TMP"),
        ];

        for &(d, o, p, x) in device_data {
            assert_eq!(parser().parse(d), Ok(((Device(o.to_string()), p), x)));
        }

        assert!(parser().parse("M`OUTTMP").is_err());
        assert!(parser().parse("M!OUTTMP").is_err());
        assert!(parser().parse("M#OUTTMP").is_err());
        assert!(parser().parse("M%OUTTMP").is_err());
        assert!(parser().parse("M^OUTTMP").is_err());
        assert!(parser().parse("M*OUTTMP").is_err());
        assert!(parser().parse("M(OUTTMP").is_err());
        assert!(parser().parse("M)OUTTMP").is_err());
        assert!(parser().parse("M-OUTTMP").is_err());
        assert!(parser().parse("M+OUTTMP").is_err());
        assert!(parser().parse("M=OUTTMP").is_err());
        assert!(parser().parse("M{OUTTMP").is_err());
        assert!(parser().parse("M}OUTTMP").is_err());
        assert!(parser().parse("M[OUTTMP").is_err());
        assert!(parser().parse("M]OUTTMP").is_err());
        assert!(parser().parse("M\\OUTTMP").is_err());
        assert!(parser().parse("M;OUTTMP").is_err());
        assert!(parser().parse("M'OUTTMP").is_err());
        assert!(parser().parse("M\"OUTTMP").is_err());
        assert!(parser().parse("M<OUTTMP").is_err());
        assert!(parser().parse("M>OUTTMP").is_err());
        assert!(parser().parse("M,OUTTMP").is_err());
        assert!(parser().parse("M.OUTTMP").is_err());
        assert!(parser().parse("M/OUTTMP").is_err());
        assert!(parser().parse("1:123456").is_err());
    }
}
