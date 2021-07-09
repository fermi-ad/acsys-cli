use super::{AnalogField, DigitalField, Property, ReadingField, SettingField, StatusField};
use combine::error::{ParseError, StreamError};
use combine::parser::{char, choice, repeat};
use combine::stream::{Stream, StreamErrorFor};
use combine::{Parser, EasyParser};

fn lookup<PropField: Clone>(
    v: &str,
    key_values: &'static [(&'static str, PropField)],
) -> Option<PropField>
{
    for (d, o) in key_values {
        if v.eq(*d) {
            return Some(o.clone());
        }
    }
    None
}

pub fn parse_field<Input>(use_prop: Property) -> impl Parser<Input, Output = Property>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char::char('.').with(
        repeat::many1(choice::or(char::letter(), char::char('_')))
            .and_then(move |v: String| {
                let v = v.to_uppercase();

                match use_prop {
                    Property::Reading(_) => {
                        const FIELDS: [(&'static str, ReadingField); 5] = [
                            ("COMMON", ReadingField::Scaled),
                            ("PRIMARY", ReadingField::Primary),
                            ("RAW", ReadingField::Raw),
                            ("SCALED", ReadingField::Scaled),
                            ("VOLTS", ReadingField::Primary),
                        ];

                        if let Some(v) = lookup(&v, &FIELDS) {
                            return Ok(Property::Reading(v))
                        }
                    }
                    Property::Setting(_) => {
                        const FIELDS: [(&'static str, SettingField); 5] = [
                            ("COMMON", SettingField::Scaled),
                            ("PRIMARY", SettingField::Primary),
                            ("RAW", SettingField::Raw),
                            ("SCALED", SettingField::Scaled),
                            ("VOLTS", SettingField::Primary),
                        ];

                        if let Some(v) = lookup(&v, &FIELDS) {
                            return Ok(Property::Setting(v))
                        }
                    }
                    Property::Status(_) => {
                        const FIELDS: [(&'static str, StatusField); 9] = [
                            ("ALL", StatusField::All),
                            ("EXTENDED_TEXT", StatusField::ExtText),
                            ("ON", StatusField::On),
                            ("POSITIVE", StatusField::Positive),
                            ("RAMP", StatusField::Ramp),
                            ("RAW", StatusField::Raw),
                            ("READY", StatusField::Ready),
                            ("REMOTE", StatusField::Remote),
                            ("TEXT", StatusField::Text),
                        ];

                        if let Some(v) = lookup(&v, &FIELDS) {
                            return Ok(Property::Status(v))
                        }
                    }
                    Property::Analog(_) => {
                        const FIELDS: [(&'static str, AnalogField); 30] = [
                            ("ABORT", AnalogField::Abort),
                            ("ABORT_INHIBIT", AnalogField::AbortInhibit),
                            ("ALARM_ENABLE", AnalogField::Enable),
                            ("ALARM_FTD", AnalogField::FTD),
                            ("ALARM_STATUS", AnalogField::Status),
                            ("ALL", AnalogField::All),
                            ("ENABLE", AnalogField::Enable),
                            ("FLAGS", AnalogField::Flags),
                            ("FTD", AnalogField::FTD),
                            ("MAX", AnalogField::Max),
                            ("MAXIMUM", AnalogField::Max),
                            ("MIN" , AnalogField::Min),
                            ("MINIMUM", AnalogField::Min),
                            ("NOM", AnalogField::Nom),
                            ("NOMINAL", AnalogField::Nom),
                            ("RAW", AnalogField::Raw),
                            ("RAW_MAX", AnalogField::RawMax),
                            ("RAWMAX", AnalogField::RawMax),
                            ("RAW_MIN", AnalogField::RawMin),
                            ("RAWMIN", AnalogField::RawMin),
                            ("RAW_NOM", AnalogField::RawNom),
                            ("RAWNOM", AnalogField::RawNom),
                            ("RAW_TOL", AnalogField::RawTol),
                            ("RAWTOL", AnalogField::RawTol),
                            ("STATUS", AnalogField::Status),
                            ("TEXT", AnalogField::Text),
                            ("TOL", AnalogField::Tol),
                            ("TOLERANCE", AnalogField::Tol),
                            ("TRIES_NEEDED", AnalogField::TriesNeeded),
                            ("TRIES_NOW", AnalogField::TriesNow),
                        ];

                        if let Some(v) = lookup(&v, &FIELDS) {
                            return Ok(Property::Analog(v))
                        }
                    }
                    Property::Digital(_) => {
                        const FIELDS: [(&'static str, DigitalField); 17] = [
                            ("ABORT", DigitalField::Abort),
                            ("ABORT_INHIBIT", DigitalField::AbortInhibit),
                            ("ALARM_ENABLE", DigitalField::Enable),
                            ("ALARM_FTD", DigitalField::FTD),
                            ("ALARM_STATUS", DigitalField::Status),
                            ("ALL", DigitalField::All),
                            ("ENABLE", DigitalField::Enable),
                            ("FLAGS", DigitalField::Flags),
                            ("FTD", DigitalField::FTD),
                            ("NOM", DigitalField::Nom),
                            ("NOMINAL", DigitalField::Nom),
                            ("MASK", DigitalField::Mask),
                            ("RAW", DigitalField::Raw),
                            ("STATUS", DigitalField::Status),
                            ("TEXT", DigitalField::Text),
                            ("TRIES_NEEDED", DigitalField::TriesNeeded),
                            ("TRIES_NOW", DigitalField::TriesNow),
                        ];

                        if let Some(v) = lookup(&v, &FIELDS) {
                            return Ok(Property::Digital(v))
                        }
                    }
                    Property::Control | Property::Description |
                    Property::Index | Property::LongName |
                    Property::AlarmList =>
                        return Err(StreamErrorFor::<Input>::message("property has no fields"))
                }
                Err(StreamErrorFor::<Input>::message("invalid field"))
            }))
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

    char::char('.').with(
        repeat::many1(choice::or(char::letter(), char::char('_')))
            .and_then(move |v: String| {
                let v = v.to_uppercase();

                if let Some(property) = lookup(&v, &PROPERTIES) {
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
                        (Property::AlarmList, Property::AlarmList) =>
                            Ok(property),
                        _ =>
                            Err(StreamErrorFor::<Input>::message("mismatched properties"))
                    }
                } else {
                    Err(StreamErrorFor::<Input>::message("unknown property"))
                }
            })
    )
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
            assert_eq!(parse_property(o).easy_parse(d), Ok((o, x)),
                       "\n input: \"{}\"", d);
        }
    }
}
