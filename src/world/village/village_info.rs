use mongodb::Client;
use tokio::sync::mpsc::Sender;

use crate::world::WorldInlet;

use super::periods::{Period, RawPeriod};

#[derive(Clone)]
pub(super) struct VillageInfo {
    pub(super) client: Client,
    pub(super) village_id: String,
    pub(super) village_name: String,
    pub(super) sender: Sender<WorldInlet>,
    pub(super) period_maker: fn(&RawPeriod) -> Period,
}

impl VillageInfo {
    pub(crate) fn resolve_period(&self, raw: &RawPeriod) -> Period {
        (self.period_maker)(raw)
    }
}
