use std::{fmt::Display, time::Duration};

#[derive(Debug, Clone, Copy)]
pub enum AssignmentMode {
    Normal,
}

#[derive(Debug, Clone, Copy)]
pub enum Daytime {
    MidNight,
    SunRaise,
    LynchTime,
}

impl Display for Daytime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Daytime::MidNight => write!(f, "🌃 Mid Night"),
            Daytime::SunRaise => write!(f, "🌇 Sun Raise"),
            Daytime::LynchTime => write!(f, "⚔️ Lynch Time"),
        }
    }
}

impl Daytime {
    pub fn cross(&self) -> Self {
        use Daytime::*;
        match *self {
            MidNight => SunRaise,
            SunRaise => LynchTime,
            LynchTime => MidNight,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RawPeriod {
    None,
    Populating,
    Assignments,
    FirstNight,
    DaytimeCycle,
    Ending,
}

impl RawPeriod {
    pub fn cross(&self) -> Result<Self, ()> {
        use RawPeriod::*;
        match *self {
            None => Ok(Populating),
            Populating => Ok(Assignments),
            Assignments => Ok(FirstNight),
            FirstNight => Ok(DaytimeCycle),
            DaytimeCycle => Ok(Ending),
            Ending => Err(()),
        }
    }
}

impl Into<i32> for RawPeriod {
    fn into(self) -> i32 {
        use RawPeriod::*;
        match self {
            None => 0,
            Populating => 1,
            Assignments => 2,
            FirstNight => 3,
            DaytimeCycle => 4,
            Ending => 5,
        }
    }
}

impl From<i32> for RawPeriod {
    fn from(i: i32) -> Self {
        use RawPeriod::*;
        match i {
            0 => None,
            1 => Populating,
            2 => Assignments,
            3 => FirstNight,
            4 => DaytimeCycle,
            _ => Ending,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Period {
    None,
    Populating {
        min_persons: u8,
        max_persons: u8,
        max_dur: Duration,
    },
    Assignments(AssignmentMode),
    FirstNight(Duration),
    DaytimeCycle(fn(Daytime) -> Duration),
    Ending,
}

impl Into<RawPeriod> for Period {
    fn into(self) -> RawPeriod {
        use Period::*;
        match self {
            None => RawPeriod::None,
            Populating {
                min_persons: _,
                max_persons: _,
                max_dur: _,
            } => RawPeriod::Populating,
            Assignments(_) => RawPeriod::Assignments,
            FirstNight(_) => RawPeriod::FirstNight,
            DaytimeCycle(_) => RawPeriod::DaytimeCycle,
            Ending => RawPeriod::Ending,
        }
    }
}
