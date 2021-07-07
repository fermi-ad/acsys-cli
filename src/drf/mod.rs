use combine::{error::StringStreamError, Parser};

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
        start_index: u16,
        end_index: Option<u16>,
    },
    Raw {
        offset: u32,
        length: Option<u32>,
    },
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

mod event;

pub fn parse_event(ev_str: &str) -> Result<Event, StringStreamError> {
    Ok(event::parser().parse(ev_str)?.0)
}
