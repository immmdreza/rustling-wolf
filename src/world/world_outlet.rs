use std::time::Duration;

use tokio::sync::mpsc::{error::SendError, Sender};

use super::{
    person::Person,
    village::periods::{Daytime, Period, RawPeriod},
    world_inlet::{AddPersonResult, NightActionResult},
};

#[derive(Debug)]
pub struct VillageLiteInfo {
    village_id: String,
    village_name: String,
}

impl VillageLiteInfo {
    pub fn new(village_id: &str, village_name: &str) -> Self {
        Self {
            village_id: village_id.to_string(),
            village_name: village_name.to_string(),
        }
    }

    pub fn village_id(&self) -> &str {
        self.village_id.as_ref()
    }

    pub fn village_name(&self) -> &str {
        self.village_name.as_ref()
    }
}

#[derive(Debug)]
pub enum NightTurn {
    Wolf,
    Doctor,
    Seer,
}

#[derive(Debug)]
pub enum WithVillage {
    RawString(String),
    VillageDisposed,
    PopulationDone(u64),
    PeriodReady(RawPeriod),
    NewPeriod(Period),
    PopulationTimedOut,
    DaytimeCycled(Daytime, Duration),
    AddPersonResult(AddPersonResult),
    NightActionResultReport(NightActionResult),
    NightTurn {
        turn: NightTurn,
        available_persons: Vec<Person>,
    },
}

#[derive(Debug)]
pub enum WorldOutlet {
    VillageList(Vec<VillageLiteInfo>),
    WithVillage {
        village_id: String,
        data: WithVillage,
    },
}

impl<'tx> WorldOutlet {
    pub fn send_ctx(tx: &'tx Sender<WorldOutlet>) -> SendWorldOutletContext<'tx> {
        SendWorldOutletContext { tx }
    }
}

pub struct ReadyToSend<'tx> {
    outlet: WorldOutlet,
    tx: &'tx Sender<WorldOutlet>,
}

impl<'tx> ReadyToSend<'tx> {
    pub async fn send(self) -> Result<(), tokio::sync::mpsc::error::SendError<WorldOutlet>> {
        self.tx.send(self.outlet).await
    }
}

pub struct SendWorldOutletContext<'tx> {
    tx: &'tx Sender<WorldOutlet>,
}

impl<'tx> SendWorldOutletContext<'tx> {
    pub fn new(tx: &'tx Sender<WorldOutlet>) -> SendWorldOutletContext<'tx> {
        SendWorldOutletContext { tx }
    }

    pub fn with_village(self, village_id: &str) -> SendWorldOutletWithVillageContext<'tx> {
        SendWorldOutletWithVillageContext {
            village_id: village_id.to_string(),
            tx: self.tx,
        }
    }

    pub async fn send(self, data: WorldOutlet) -> Result<(), SendError<WorldOutlet>> {
        ReadyToSend {
            outlet: data,
            tx: self.tx,
        }
        .send()
        .await
    }

    pub async fn village_list(
        self,
        villages: Vec<VillageLiteInfo>,
    ) -> Result<(), SendError<WorldOutlet>> {
        self.send(WorldOutlet::VillageList(villages)).await
    }
}

pub struct SendWorldOutletWithVillageContext<'tx> {
    village_id: String,
    tx: &'tx Sender<WorldOutlet>,
}

impl<'tx> SendWorldOutletWithVillageContext<'tx> {
    pub async fn send(self, data: WithVillage) -> Result<(), SendError<WorldOutlet>> {
        ReadyToSend {
            outlet: WorldOutlet::WithVillage {
                village_id: self.village_id,
                data,
            },
            tx: self.tx,
        }
        .send()
        .await
    }

    pub async fn raw_string(self, raw: String) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::RawString(raw)).await
    }

    pub async fn disposed(self) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::VillageDisposed).await
    }

    pub async fn populated(self, count: u64) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::PopulationDone(count)).await
    }

    pub async fn period_ready(self, period: RawPeriod) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::PeriodReady(period)).await
    }

    pub async fn new_period(self, period: Period) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::NewPeriod(period)).await
    }

    pub async fn population_timed_out(self) -> Result<(), SendError<WorldOutlet>> {
        self.send(WithVillage::PopulationTimedOut).await
    }
}
