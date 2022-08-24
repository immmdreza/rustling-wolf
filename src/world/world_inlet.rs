use std::time::Duration;

use super::village::periods::{Daytime, Period, RawPeriod};

#[derive(Debug)]
pub enum AddPersonResult {
    Added {
        person_id: String,
        current_count: u64,
    },
    Failed(String),
}

#[derive(Debug)]
pub enum FromVillage {
    RawString(String),
    VillageDisposed,
    PeriodReady(RawPeriod),
    NewPeriod(Period),

    PopulatingTimedOut,
    DaytimeCycled(Daytime, Duration),
    AddPerson(AddPersonResult),

    WolvesTurn,
    DoctorTurn,
    SeerTurn,
}

#[derive(Debug)]
pub enum FromHeaven {
    Nothing,
    RawString(String),
    RequestPerson {
        village_id: String,
        person_name: String,
    },
    FillPersons {
        village_id: String,
        count: u8,
    },
    KillVillage {
        village_id: String,
    },
    ListVillages,
    NewVillage,
}

#[derive(Debug)]
pub enum WorldInlet {
    FromVillage {
        village_id: String,
        data: FromVillage,
    },
    FromHeaven(FromHeaven),
}

impl WorldInlet {
    pub fn from_village(village_id: &str, data: FromVillage) -> WorldInlet {
        WorldInlet::FromVillage {
            village_id: village_id.to_string(),
            data,
        }
    }
}