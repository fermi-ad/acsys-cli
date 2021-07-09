use super::{AnalogField, DigitalField, Property, ReadingField, SettingField, StatusField};
use combine::error::{ParseError, StreamError};
use combine::parser::{char, choice, repeat};
use combine::stream::{Stream, StreamErrorFor};
use combine::Parser;

const READING_FIELDS: [(&'static str, ReadingField); 5] = [
    ("COMMON", ReadingField::Scaled),
    ("PRIMARY", ReadingField::Primary),
    ("RAW", ReadingField::Raw),
    ("SCALED", ReadingField::Scaled),
    ("VOLTS", ReadingField::Primary),
];

const SETTING_FIELDS: [(&'static str, SettingField); 5] = [
    ("COMMON", SettingField::Scaled),
    ("PRIMARY", SettingField::Primary),
    ("RAW", SettingField::Raw),
    ("SCALED", SettingField::Scaled),
    ("VOLTS", SettingField::Primary),
];

fn get_parser<Input, PropField: Copy>(
    key_values: &'static [(&'static str, PropField)],
) -> impl Parser<Input, Output = PropField>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char::char('.').with(
        repeat::many1(choice::or(char::letter(), char::char('_'))).and_then(move |v: String| {
            for (d, o) in key_values {
                if &v == d {
                    return Ok(o.clone());
                }
            }

            return Err(StreamErrorFor::<Input>::message("Unknown property"));
        }),
    )
}

pub fn parse_property<Input>(qual_prop: Property) -> impl Parser<Input, Output = Property>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    const PROPERTIES: [(&str, Property); 32] = [
        ("AA", Property::Analog(AnalogField::default())),
        ("ALARM_LIST_NAME", Property::AlarmList),
        ("ANALOG_ALARM", Property::Analog(AnalogField::default())),
        ("ANALOG", Property::Analog(AnalogField::default())),
        ("BASIC_CONTROL", Property::Control),
        ("BASIC_STATUS", Property::Status(StatusField::default())),
        ("CONTROL", Property::Control),
        ("CTRL", Property::Control),
        ("DA", Property::Digital(DigitalField::default())),
        ("DESC", Property::Description),
        ("DESCRIPTION", Property::Description),
        ("DIGITAL_ALARM", Property::Digital(DigitalField::default())),
        ("DIGITAL", Property::Digital(DigitalField::default())),
        ("INDEX", Property::Index),
        ("LNGNAM", Property::LongName),
        ("LONG_NAME", Property::LongName),
        ("LSTNAM", Property::AlarmList),
        ("PRALNM", Property::AlarmList),
        ("PRANAB", Property::Analog(AnalogField::default())),
        ("PRBCTL", Property::Control),
        ("PRBSTS", Property::Status(StatusField::default())),
        ("PRDABL", Property::Digital(DigitalField::default())),
        ("PRDESC", Property::Description),
        ("PRLNAM", Property::LongName),
        ("PRREAD", Property::Reading(ReadingField::default())),
        ("PRSET", Property::Setting(SettingField::default())),
        ("READ", Property::Reading(ReadingField::default())),
        ("READING", Property::Reading(ReadingField::default())),
        ("SET", Property::Setting(SettingField::default())),
        ("SETTING", Property::Setting(SettingField::default())),
        ("STATUS", Property::Status(StatusField::default())),
        ("STS", Property::Status(StatusField::default())),
    ];

    get_parser(&PROPERTIES)
        .and_then(move |property| {
            match (qual_prop, property) {
                (Property::Reading(_), _) |
                (Property::Setting(_), Property::Setting(_)) |
                (Property::Status(_), Property::Status(_)) |
                (Property::Analog(_), Property::Analog(_)) |
                (Property::Digital(_), Property::Digital(_)) |
                (Property::Control, Property::Control) |
                (Property::Description, Property::Description) |
                (Property::Index, Property::Index) |
                (Property::LongName, Property::LongName) |
                (Property::AlarmList, Property::AlarmList) => Ok(property),
                _ =>
                    Err(StreamErrorFor::<Input>::message("mismatched properties"))
            }
        })
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
            assert_eq!(parse_property(o).parse(d), Ok((o, x)));
        }
    }
}
