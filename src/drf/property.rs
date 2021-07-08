use super::{AnalogField, DigitalField, Property, ReadingField, SettingField, StatusField};
use combine::error::ParseError;
use combine::parser::{char, choice, repeat};
use combine::stream::Stream;
use combine::{optional, Parser};

pub fn parser<Input>(current: Property) -> impl Parser<Input, Output = Property>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(char::char('.').with(
        repeat::many1(choice::or(char::letter(), char::char('_'))).map(move |v: String| {
            match v.to_uppercase().as_str() {
                "READING" | "READ" | "PRREAD" => Property::Reading(ReadingField::default()),
                "SETTING" | "SET" | "PRSET" => Property::Setting(SettingField::default()),
                "STATUS" | "BASIC_STATUS" | "STS" | "PRBSTS" => {
                    Property::Status(StatusField::default())
                }
                "CONTROL" | "BASIC_CONTROL" | "CTRL" | "PRBCTL" => Property::Control,
                "ANALOG" | "ANALOG_ALARM" | "AA" | "PRANAB" => {
                    Property::Analog(AnalogField::default())
                }
                "DIGITAL" | "DIGITAL_ALARM" | "DA" | "PRDABL" => {
                    Property::Digital(DigitalField::default())
                }
                "DESCRIPTION" | "DESC" | "PRDESC" => Property::Description,
                "INDEX" => Property::Index,
                "LONG_NAME" | "LNGNAM" | "PRLNAM" => Property::LongName,
                "ALARM_LIST_NAME" | "LSTNAM" | "PRALNM" => Property::AlarmList,
                _ => current,
            }
        }),
    ))
    .map(move |v| v.unwrap_or(current))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_parsing() {
        let rdg_prop = Property::Reading(ReadingField::default());
        let set_prop = Property::Setting(SettingField::default());
        let sts_prop = Property::Status(StatusField::default());
        let ana_prop = Property::Analog(AnalogField::default());
        let dig_prop = Property::Digital(DigitalField::default());

        let device_data = &[
            (".READING", rdg_prop, ""),
            (".READ", rdg_prop, ""),
            (".PRREAD", rdg_prop, ""),
            (".SETTING", set_prop, ""),
            (".SET", set_prop, ""),
            (".PRSET", set_prop, ""),
            (".STATUS", sts_prop, ""),
            (".BASIC_STATUS", sts_prop, ""),
            (".STS", sts_prop, ""),
            (".PRBSTS", sts_prop, ""),
            (".CONTROL", Property::Control, ""),
            (".BASIC_CONTROL", Property::Control, ""),
            (".CTRL", Property::Control, ""),
            (".PRBCTL", Property::Control, ""),
            (".ANALOG", ana_prop, ""),
            (".ANALOG_ALARM", ana_prop, ""),
            (".AA", ana_prop, ""),
            (".PRANAB", ana_prop, ""),
            (".DIGITAL", dig_prop, ""),
            (".DIGITAL_ALARM", dig_prop, ""),
            (".DA", dig_prop, ""),
            (".PRDABL", dig_prop, ""),
            (".DESCRIPTION", Property::Description, ""),
            (".DESC", Property::Description, ""),
            (".PRDESC", Property::Description, ""),
            (".INDEX", Property::Index, ""),
            (".LONG_NAME", Property::LongName, ""),
            (".LNGNAM", Property::LongName, ""),
            (".PRLNAM", Property::LongName, ""),
            (".ALARM_LIST_NAME", Property::AlarmList, ""),
            (".LSTNAM", Property::AlarmList, ""),
            (".PRALNM", Property::AlarmList, ""),
        ];

        for &(d, o, x) in device_data {
            assert_eq!(parser(o).parse(d), Ok((o, x)));
        }
    }
}
