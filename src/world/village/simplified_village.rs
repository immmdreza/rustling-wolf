use std::time::Duration;

use tokio::sync::mpsc::error::SendError;

use crate::world::WorldInlet;

use super::{inlet_data::VillageInlet, Village};

#[derive(Clone)]
pub struct SimplifiedVillage {
    pub(crate) village: Village,
}

impl SimplifiedVillage {
    pub fn new(village: Village) -> Self {
        SimplifiedVillage { village }
    }

    /// Returns a reference to the get village id of this [`Village`].
    pub fn get_village_id(&self) -> &String {
        &self.village.village_id
    }

    /// Returns a reference to the get village name of this [`Village`].
    pub fn get_village_name(&self) -> &String {
        &self.village.village_name
    }

    /// Returns the add player's id of this [`Village`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn add_player(&self, name: &str) -> Result<(), SendError<VillageInlet>> {
        self.village
            .transmit(VillageInlet::AddPerson(name.to_string()))
            .await
    }

    pub async fn extend_population_dur(
        &self,
        dur: Duration,
    ) -> Result<(), SendError<VillageInlet>> {
        self.village
            .transmit(VillageInlet::ExtendPopulationTime(dur))
            .await
    }

    pub async fn send_to_world(&self, inlet: WorldInlet) -> Result<(), SendError<WorldInlet>> {
        self.village.to_world_sender.send(inlet).await
    }

    pub async fn notify_my_die(&self) -> Result<(), SendError<WorldInlet>> {
        self.send_to_world(WorldInlet::VillageDisposed(
            self.get_village_id().to_string(),
        ))
        .await
    }

    pub async fn die(&self) -> Result<(), SendError<VillageInlet>> {
        self.village.transmit(VillageInlet::Die).await
    }
}
