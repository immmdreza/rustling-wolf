use std::time::Duration;

use super::periods::{Daytime, Period, RawPeriod};

#[derive(Debug)]
pub enum AddPersonResult {
    Added {
        person_id: String,
        current_count: u64,
    },
    Failed(String),
}

#[derive(Debug)]
pub enum VillageOutlet {
    RawString(String),
    AddPerson(AddPersonResult),
    PeriodReady(RawPeriod),
    PeriodCrossed(RawPeriod, Period),
    PopulatingTimedOut,
    DaytimeCycled(Daytime, Duration),

    WolvesTurn,
    DoctorTurn,
    SeerTurn,
}
