use super::{AnalogField, Device, DigitalField, Property, ReadingField, SettingField, StatusField};
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
        char::char(':').with(value(Property::Reading(ReadingField::default()))),
        char::char('?').with(value(Property::Reading(ReadingField::default()))),
        char::char('_').with(value(Property::Setting(SettingField::default()))),
        char::char('|').with(value(Property::Status(StatusField::default()))),
        char::char('&').with(value(Property::Control)),
        char::char('@').with(value(Property::Analog(AnalogField::default()))),
        char::char('$').with(value(Property::Digital(DigitalField::default()))),
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
        let rdg_prop = Property::Reading(ReadingField::default());
        let set_prop = Property::Setting(SettingField::default());
        let sts_prop = Property::Status(StatusField::default());
        let ana_prop = Property::Analog(AnalogField::default());
        let dig_prop = Property::Digital(DigitalField::default());

        let device_data = &[
            ("M:OUTTMP", "M:OUTTMP", rdg_prop, ""),
            ("M?OUTTMP", "M:OUTTMP", rdg_prop, ""),
            ("M_OUTTMP", "M:OUTTMP", set_prop, ""),
            ("M|OUTTMP", "M:OUTTMP", sts_prop, ""),
            ("M&OUTTMP", "M:OUTTMP", Property::Control, ""),
            ("M@OUTTMP", "M:OUTTMP", ana_prop, ""),
            ("M$OUTTMP", "M:OUTTMP", dig_prop, ""),
            ("M~OUTTMP", "M:OUTTMP", Property::Description, ""),
            ("0:123456", "0:123456", rdg_prop, ""),
            ("0?123456", "0:123456", rdg_prop, ""),
            ("0_123456", "0:123456", set_prop, ""),
            ("0|123456", "0:123456", sts_prop, ""),
            ("0&123456", "0:123456", Property::Control, ""),
            ("0@123456", "0:123456", ana_prop, ""),
            ("0$123456", "0:123456", dig_prop, ""),
            ("0~123456", "0:123456", Property::Description, ""),
            (
                "M:OUTTMP:outdoor:temp.VAL",
                "M:OUTTMP:outdoor:temp",
                rdg_prop,
                ".VAL",
            ),
            (
                "M:OUTTMP.outdoor.temp.VAL",
                "M:OUTTMP",
                rdg_prop,
                ".outdoor.temp.VAL",
            ),
            ("M:OUTTMP.SETTING", "M:OUTTMP", rdg_prop, ".SETTING"),
            ("M:OUTTMP[0:3]", "M:OUTTMP", rdg_prop, "[0:3]"),
            ("M:OUTTMP{0:3}", "M:OUTTMP", rdg_prop, "{0:3}"),
            ("M:OUTTMP{:}", "M:OUTTMP", rdg_prop, "{:}"),
            ("M:OUTTMP{0:}", "M:OUTTMP", rdg_prop, "{0:}"),
            ("M:OUTTMP{:3}", "M:OUTTMP", rdg_prop, "{:3}"),
            ("M:OUT~TMP", "M:OUT", rdg_prop, "~TMP"),
            ("M:OUT`TMP", "M:OUT", rdg_prop, "`TMP"),
            ("M:OUT!TMP", "M:OUT", rdg_prop, "!TMP"),
            ("M:OUT#TMP", "M:OUT", rdg_prop, "#TMP"),
            ("M:OUT%TMP", "M:OUT", rdg_prop, "%TMP"),
            ("M:OUT^TMP", "M:OUT", rdg_prop, "^TMP"),
            ("M:OUT*TMP", "M:OUT", rdg_prop, "*TMP"),
            ("M:OUT(TMP", "M:OUT", rdg_prop, "(TMP"),
            ("M:OUT)TMP", "M:OUT", rdg_prop, ")TMP"),
            ("M:OUT-TMP", "M:OUT-TMP", rdg_prop, ""),
            ("M:OUT+TMP", "M:OUT", rdg_prop, "+TMP"),
            ("M:OUT=TMP", "M:OUT", rdg_prop, "=TMP"),
            ("M:OUT{TMP", "M:OUT", rdg_prop, "{TMP"),
            ("M:OUT}TMP", "M:OUT", rdg_prop, "}TMP"),
            ("M:OUT[TMP", "M:OUT", rdg_prop, "[TMP"),
            ("M:OUT]TMP", "M:OUT", rdg_prop, "]TMP"),
            ("M:OUT\\TMP", "M:OUT", rdg_prop, "\\TMP"),
            ("M:OUT;TMP", "M:OUT;TMP", rdg_prop, ""),
            ("M:OUT'TMP", "M:OUT", rdg_prop, "'TMP"),
            ("M:OUT\"TMP", "M:OUT", rdg_prop, "\"TMP"),
            ("M:OUT<TMP", "M:OUT<TMP", rdg_prop, ""),
            ("M:OUT>TMP", "M:OUT>TMP", rdg_prop, ""),
            ("M:OUT,TMP", "M:OUT", rdg_prop, ",TMP"),
            ("M:OUT/TMP", "M:OUT", rdg_prop, "/TMP"),
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
