mod handle_from_world;
pub mod inlet_data;
pub mod periods;
pub mod simplified_village;
pub mod transports;
mod village_info;
mod village_main;

use mongodb::Client;
use tokio::sync::mpsc::{channel, error::SendError, Sender};

use self::{
    inlet_data::VillageInlet,
    periods::{Period, RawPeriod},
    simplified_village::SimplifiedVillage,
    village_info::VillageInfo,
    village_main::VillageMain,
};

use super::WorldInlet;

#[derive(Clone)]
pub struct Village {
    village_id: String,
    village_name: String,
    current_period: Period,
    pub(crate) sender: Sender<VillageInlet>,
}

impl Village {
    pub fn new(
        client: Client,
        village_name: &str,
        period_maker: fn(&RawPeriod) -> Period,
        to_world_sender: Sender<WorldInlet>,
    ) -> Self {
        let village_id: String = uuid::Uuid::new_v4().to_string();
        let (inlet_tx, inlet_rx) = channel::<VillageInlet>(1024);

        let village = Village {
            village_id: village_id.clone(),
            village_name: village_name.to_string(),
            current_period: Period::None,
            sender: inlet_tx,
        };

        let info = VillageInfo {
            client,
            village_id,
            village_name: village_name.to_string(),
            sender: to_world_sender,
            period_maker,
        };

        tokio::spawn(async move {
            let mut vm = VillageMain::new(info, inlet_rx);
            vm.run().await
        });

        village
    }

    pub async fn transmit(&self, inlet_data: VillageInlet) -> Result<(), SendError<VillageInlet>> {
        self.sender.send(inlet_data).await
    }

    pub fn simplify(self) -> SimplifiedVillage {
        SimplifiedVillage::new(self)
    }

    pub fn get_current_period(&self) -> Period {
        self.current_period
    }

    pub(crate) fn set_current_period(&mut self, period: Period) {
        self.current_period = period;
    }
}
