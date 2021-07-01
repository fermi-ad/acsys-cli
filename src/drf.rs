use combine::{choice, one_of, value, Parser};
use combine::error::ParseError;
use combine::parser::char;
use combine::stream::Stream;

pub struct Device(String);

pub enum ReadingField {
    Raw,
    Primary,
    Scaled,
}

pub enum SettingField {
    Raw,
    Primary,
    Scaled,
}

pub enum StatusField {
    Raw,
    All,
    Text,
    ExtText,
    On,
    Ready,
    Remote,
    Positive,
    Ramp,
}

pub enum AnalogField {
    Raw,
    All,
    Text,
    Min,
    Max,
    Nom,
    Tol,
    RawMin,
    RawMax,
    RawNom,
    RawTol,
    Enable,
    Status,
    TriesNeeded,
    TriesNow,
    FTD,
    Abort,
    AbortInhibit,
    Flags,
}

pub enum DigitalField {
    Raw,
    All,
    Text,
    Nom,
    Mask,
    Enable,
    Status,
    TriesNeeded,
    TriesNow,
    FTD,
    Abort,
    AbortInhibit,
    Flags,
}

pub enum Property {
    Reading(ReadingField),
    Setting(SettingField),
    Status(StatusField),
    Control,
    Analog(AnalogField),
    Digital(DigitalField),
    Description,
    Index,
    LongName,
    AlarmList,
}

pub enum Range {
    Full,
    Array {
        start_index: Option<u16>,
        end_index: Option<u16>,
    },
    Raw {
        offset: Option<u32>,
        length: Option<u32>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateOp {
    Eq,
    NEq,
    GT,
    LT,
    LEq,
    GEq,
    All,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Never,
    Immediate,
    Default,
    Periodic {
        period: u32,
        immediate: bool,
        skip_dups: bool,
    },
    Clock {
        event: u8,
        delay: u32,
    },
    State {
        device: u32,
        value: u16,
        delay: u32,
        expr: StateOp,
    },
}

pub fn event_parser<Input>() -> impl Parser<Input, Output = Event>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let parse_never = one_of("nN".chars()).with(value(Event::Never));

    let parse_immediate = one_of("iI".chars()).with(value(Event::Immediate));

    let parse_periodic = one_of("pP".chars())
        .and(char::char(','))
        .with(value(Event::Default));

    choice((parse_never, parse_immediate, parse_periodic))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_parsing() {
        assert_eq!(event_parser().parse("N"), Ok((Event::Never, "")));
        assert_eq!(event_parser().parse("n"), Ok((Event::Never, "")));
        assert_eq!(event_parser().parse("I"), Ok((Event::Immediate, "")));
        assert_eq!(event_parser().parse("i"), Ok((Event::Immediate, "")));
    }
}
