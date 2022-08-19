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

    pub fn to_num(&self) -> i32 {
        use RawPeriod::*;
        match *self {
            None => 0,
            Populating => 1,
            Assignments => 2,
            DaytimeCycle => 3,
            Ending => 4,
        }
    }

    pub fn from_num(num: i32) -> Self {
        use RawPeriod::*;
        match num {
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
