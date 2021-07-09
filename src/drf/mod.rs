use combine::{error::StringStreamError, optional, Parser};

#[derive(Clone, Debug, PartialEq)]
pub struct Device(String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadingField {
    Raw,
    Primary,
    Scaled,
}

impl ReadingField {
    pub const fn default() -> Self { ReadingField::Scaled }

    pub fn canonical(&self) -> &'static str {
        match *self {
            ReadingField::Raw => ".RAW",
            ReadingField::Primary => ".PRIMARY",
            ReadingField::Scaled => ".SCALED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingField {
    Raw,
    Primary,
    Scaled,
}

impl SettingField {
    pub const fn default() -> Self { SettingField::Scaled }

    pub fn canonical(&self) -> &'static str {
        match *self {
            SettingField::Raw => ".RAW",
            SettingField::Primary => ".PRIMARY",
            SettingField::Scaled => ".SCALED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl StatusField {
    pub const fn default() -> Self { StatusField::All }

    pub fn canonical(&self) -> &'static str {
        match *self {
            StatusField::Raw => ".RAW",
            StatusField::All => ".ALL",
            StatusField::Text => ".TEXT",
            StatusField::ExtText => ".EXTENDED_TEXT",
            StatusField::On => ".ON",
            StatusField::Ready => ".READY",
            StatusField::Remote => ".REMOTE",
            StatusField::Positive => ".POSITIVE",
            StatusField::Ramp => ".RAMP",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl AnalogField {
    pub const fn default() -> Self { AnalogField::All }

    pub fn canonical(&self) -> &'static str {
        match *self {
            AnalogField::Raw => ".RAW",
            AnalogField::All => ".ALL",
            AnalogField::Text => ".TEXT",
            AnalogField::Min => ".MIN",
            AnalogField::Max => ".MAX",
            AnalogField::Nom => ".NOM",
            AnalogField::Tol => ".TOL",
            AnalogField::RawMin => ".RAW_MIN",
            AnalogField::RawMax => ".RAW_MAX",
            AnalogField::RawNom => ".RAW_NOM",
            AnalogField::RawTol => ".RAW_TOL",
            AnalogField::Enable => ".ALARM_ENABLE",
            AnalogField::Status => ".ALARM_STATUS",
            AnalogField::TriesNeeded => ".TRIES_NEEDED",
            AnalogField::TriesNow => ".TRIES_NOW",
            AnalogField::FTD => ".ALARM_FTD",
            AnalogField::Abort => ".ABORT",
            AnalogField::AbortInhibit => ".ABORT_INHIBIT",
            AnalogField::Flags => ".FLAGS",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl DigitalField {
    pub const fn default() -> Self { DigitalField::All }

    pub fn canonical(&self) -> &'static str {
        match *self {
            DigitalField::Raw => ".RAW",
            DigitalField::All => ".ALL",
            DigitalField::Text => ".TEXT",
            DigitalField::Nom => ".NOM",
            DigitalField::Mask => ".MASK",
            DigitalField::Enable => ".ALARM_ENABLE",
            DigitalField::Status => ".ALARM_STATUS",
            DigitalField::TriesNeeded => ".TRIES_NEEDED",
            DigitalField::TriesNow => ".TRIES_NOW",
            DigitalField::FTD => ".ALARM_FTD",
            DigitalField::Abort => ".ABORT",
            DigitalField::AbortInhibit => ".ABORT_INHIBIT",
            DigitalField::Flags => ".FLAGS",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl Property {
    pub fn canonical(&self) -> (&'static str, &'static str) {
        match *self {
            Property::Reading(fld) => (".READING", fld.canonical()),
            Property::Setting(fld) => (".SETTING", fld.canonical()),
            Property::Status(fld) => (".STATUS", fld.canonical()),
            Property::Control => (".CONTROL", ""),
            Property::Analog(fld) => (".ANALOG", fld.canonical()),
            Property::Digital(fld) => (".DIGITAL", fld.canonical()),
            Property::Description => (".DESCRIPTION", ""),
            Property::Index => (".INDEX", ""),
            Property::LongName => (".LONG_NAME", ""),
            Property::AlarmList => (".ALARM_LIST_NAME", ""),
        }
    }
}

// Type which specifies a range of data.

#[derive(Clone, Debug, PartialEq)]
pub enum Range {
    Full,
    Array {
        start_index: u16,
        end_index: Option<u16>,
    },
    Raw {
        offset: u32,
        length: Option<u32>,
    },
}

impl Range {
    // Returns the canonical form of the range.

    pub fn canonical(&self) -> String {
        match *self {
            Range::Full => String::from("[]"),

            Range::Array {
                start_index,
                end_index,
            } => match (start_index, end_index) {
                (0, Some(0)) => String::from(""),
                (s, Some(e)) => {
                    if s == e {
                        format!("[{}]", s)
                    } else {
                        format!("[{}:{}]", s, e)
                    }
                }
                (s, None) => format!("[{}:]", s),
            },

            Range::Raw { offset, length } => match (offset, length) {
                (o, Some(1)) => format!("{{{}}}", o),
                (o, Some(l)) => format!("{{{}:{}}}", o, l),
                (o, None) => format!("{{{}:}}", o),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StateOp {
    Eq,
    NEq,
    GT,
    LT,
    LEq,
    GEq,
    All,
}

impl StateOp {
    pub fn canonical(&self) -> &'static str {
        match *self {
            StateOp::Eq => "=",
            StateOp::NEq => "!=",
            StateOp::GT => ">",
            StateOp::LT => "<",
            StateOp::LEq => "<=",
            StateOp::GEq => ">=",
            StateOp::All => "*",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClockType {
    Hardware,
    Software,
    Either,
}

impl ClockType {
    pub const fn default() -> Self { ClockType::Either }

    pub fn canonical(&self) -> &'static str {
        match *self {
            ClockType::Hardware => "H",
            ClockType::Software => "S",
            ClockType::Either => "E",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
        event: u16,
        clk_type: ClockType,
        delay: u32,
    },
    State {
        device: u32,
        value: u16,
        delay: u32,
        expr: StateOp,
    },
}

impl Event {
    pub fn canonical(&self) -> String {
        match *self {
            Event::Default => String::from(""),
            Event::Never => String::from("@N"),
            Event::Immediate => String::from("@I"),
            Event::Periodic {
                period,
                immediate,
                skip_dups,
            } => format!(
                "@{},{}U,{}",
                if skip_dups { 'Q' } else { 'P' },
                period,
                if immediate { "TRUE" } else { "FALSE" }
            ),
            Event::Clock {
                event,
                clk_type,
                delay,
            } => format!("@E,{:X},{},{}U", event, clk_type.canonical(), delay),
            Event::State {
                device,
                value,
                delay,
                expr,
            } => format!("@S,{},{},{}U,{}", device, value, delay, expr.canonical()),
        }
    }
}

pub struct Request {
    pub device: Device,
    pub property: Property,
    pub range: Range,
    pub event: Event,
}

impl Request {
    pub fn canonical(&self) -> String {
        let (prop, field) = self.property.canonical();

        format!(
            "{}{}{}{}{}",
            self.device.0,
            prop,
            self.range.canonical(),
            field,
            self.event.canonical()
        )
    }
}

mod device;
mod event;
mod prop_field;
mod range;

pub fn parse(drf: &str) -> Result<Request, StringStreamError> {
    let mut p = device::parser().then(move |(device, qual_property)| {
        (
            optional(prop_field::parse_property(qual_property))
                .map(move |v| v.unwrap_or(qual_property)),
            range::parser(),
            event::parser(),
        )
            .map(move |(property, range, event)|
                 Request {
                     device: device.clone(),
                     property,
                     range,
                     event,
                 }
            )
    });

    Ok(p.parse(drf)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_canonical_forms() {
        let data = &[
            ("@N", "@N"),
            ("@n", "@N"),
            ("@I", "@I"),
            ("@i", "@I"),
            ("@P,1s", "@P,1000000U,TRUE"),
            ("@P,1s,f", "@P,1000000U,FALSE"),
            ("@P,2h,false", "@P,500000U,FALSE"),
            ("@P,10u,false", "@P,10U,FALSE"),
            ("@P,20m", "@P,20000U,TRUE"),
            ("@P,30", "@P,30000U,TRUE"),
            ("@P,10k", "@P,100U,TRUE"),
            ("@E,008f,h,10h", "@E,8F,H,100000U"),
            ("@E,0", "@E,0,E,0U"),
            ("@S,1234,0,1s,=", "@S,1234,0,1000000U,="),
        ];

        for &(event, result) in data {
            assert_eq!(event::parser().parse(event).unwrap().0.canonical(), result)
        }
    }

    #[test]
    fn test_range_canonical_forms() {
        let data = &[
            ("[]", "[]"),
            ("{}", "[]"),
            ("[:]", "[]"),
            ("{:}", "[]"),
            ("[0:]", "[]"),
            ("{0:}", "[]"),
            ("[1:2]", "[1:2]"),
            ("[0:0]", ""),
            ("[1:1]", "[1]"),
            ("{1:1}", "{1}"),
            ("{1:2}", "{1:2}"),
        ];

        for &(range, result) in data {
            assert_eq!(range::parser().parse(range).unwrap().0.canonical(), result)
        }
    }

    #[test]
    fn test_request_canonical_forms() {
        let data = &[
            ("M:OUTTMP", "M:OUTTMP.READING.SCALED"),
            ("M:OUTTMP[0:3]", "M:OUTTMP.READING[0:3].SCALED"),
            ("M|OUTTMP[]", "M:OUTTMP.STATUS[].ALL"),
            ("M|OUTTMP[]@e,02", "M:OUTTMP.STATUS[].ALL@E,2,E,0U"),
        ];

        for &(drf, result) in data {
            assert_eq!(parse(drf).unwrap().canonical(), result)
        }
    }
}
