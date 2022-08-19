use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum AssignmentMode {
    Normal,
}

pub enum Daytime {
    MidNight,
    SunRaise,
    LynchTime,
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
    DaytimeCycle,
    Ending,
}

impl RawPeriod {
    pub fn cross(&self) -> Result<Self, ()> {
        use RawPeriod::*;
        match *self {
            None => Ok(Populating),
            Populating => Ok(Assignments),
            Assignments => Ok(DaytimeCycle),
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
            DaytimeCycle => 3,
            Ending => 4,
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
            3 => DaytimeCycle,
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
    DaytimeCycle(fn(Daytime) -> Duration),
    Ending,
}
