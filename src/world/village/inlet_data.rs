use std::time::Duration;

#[derive(Debug)]
pub enum VillageInlet {
    RawString(String),
    AddPerson,
    ExtendPopulationTime(Duration),
}
