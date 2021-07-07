use combine::{error::StringStreamError, Parser};

#[derive(Debug, PartialEq)]
pub struct Device(String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadingField {
    Raw,
    Primary,
    Scaled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingField {
    Raw,
    Primary,
    Scaled,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Property {
    Reading(Option<ReadingField>),
    Setting(Option<SettingField>),
    Status(Option<StatusField>),
    Control,
    Analog(Option<AnalogField>),
    Digital(Option<DigitalField>),
    Description,
    Index,
    LongName,
    AlarmList,
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

            Range::Array { start_index, end_index } =>
                match (start_index, end_index) {
                    (0, Some(0)) => String::from(""),
                    (s, Some(e)) =>
                        if s == e {
                            format!("[{}]", s)
                        } else {
                            format!("[{}:{}]", s, e)
                        },
                    (s, None) => format!("[{}:]", s)
                },

            Range::Raw { offset, length } =>
                match (offset, length) {
                    (o, Some(1)) => format!("{{{}}}", o),
                    (o, Some(l)) => format!("{{{}:{}}}", o, l),
                    (o, None) => format!("{{{}:}}", o)
                }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClockType {
    Hardware,
    Software,
    Either,
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

mod device;
mod range;
mod event;

pub fn parse_device(dev_str: &str) -> Result<(Device, Property), StringStreamError> {
    Ok(device::parser().parse(dev_str)?.0)
}

pub fn parse_event(ev_str: &str) -> Result<Event, StringStreamError> {
    Ok(event::parser().parse(ev_str)?.0)
}
