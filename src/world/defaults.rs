use std::time::Duration;

use crate::world::village::periods::RawPeriod;

use super::village::periods::{AssignmentMode, Daytime, Period};

pub(crate) fn default_period_maker(raw: &RawPeriod) -> Period {
    match raw {
        RawPeriod::Populating => Period::Populating {
            min_persons: 5,
            max_persons: 7,
            max_dur: Duration::from_secs(30),
        },
        RawPeriod::Assignments => Period::Assignments(AssignmentMode::Normal),
        RawPeriod::DaytimeCycle => Period::DaytimeCycle(|dt| match dt {
            Daytime::MidNight => Duration::from_secs(30),
            Daytime::SunRaise => Duration::from_secs(30),
            Daytime::LynchTime => Duration::from_secs(30),
        }),
        RawPeriod::Ending => Period::Ending,
        RawPeriod::None => Period::None,
        RawPeriod::FirstNight => Period::FirstNight(Duration::from_secs(20)),
    }
}
