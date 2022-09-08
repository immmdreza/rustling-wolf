use std::time::Duration;

use tokio::sync::mpsc::error::SendError;

use crate::world::world_outlet::VillageLiteInfo;

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
    pub fn get_village_id(&self) -> &str {
        self.village.village_id.as_ref()
    }

    /// Returns a reference to the get village name of this [`Village`].
    pub fn get_village_name(&self) -> &str {
        self.village.village_name.as_ref()
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

    pub async fn die(&self) -> Result<(), SendError<VillageInlet>> {
        self.village.transmit(VillageInlet::Die).await
    }

    pub fn get_info(&self) -> VillageLiteInfo {
        VillageLiteInfo::new(self.get_village_id(), self.get_village_name())
    }
}
