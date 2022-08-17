pub mod inlet_data;
pub mod outlet_data;
pub mod transports;
mod village_main;

use std::{
    sync::mpsc::{channel, Receiver, SendError, Sender},
    thread,
};

use self::{
    inlet_data::VillageInlet, outlet_data::VillageOutlet, transports::Transporter,
    village_main::village_main,
};

pub(super) struct VillageInfo {
    village_id: String,
    village_name: String,
    receiver: Receiver<VillageInlet>,
    sender: Sender<VillageOutlet>,
}

pub struct Village {
    village_id: String,
    village_name: String,
    sender: Sender<VillageInlet>,
    // receiver: Receiver<VillageOutlet>,
}

impl Village {
    pub fn new(village_name: &str, callback: Option<fn(VillageOutlet, &String) -> ()>) -> Self {
        let village_id: String = uuid::Uuid::new_v4().to_string();
        let (inlet_tx, inlet_rx) = channel::<VillageInlet>();
        let (outlet_tx, outlet_rx) = channel::<VillageOutlet>();

        let info = VillageInfo {
            village_id: village_id.clone(),
            village_name: village_name.to_string(),
            receiver: inlet_rx,
            sender: outlet_tx,
        };
        thread::spawn(move || {
            village_main(info);
        });

        match callback {
            Some(callback) => Transporter::spawn(outlet_rx, callback, village_id.to_string()),
            None => (),
        };

        Village {
            village_id,
            village_name: village_name.to_string(),
            sender: inlet_tx,
        }
    }

    pub fn transmit(&self, inlet_data: VillageInlet) -> Result<(), SendError<VillageInlet>> {
        self.sender.send(inlet_data)
    }

    /// Returns a reference to the get village id of this [`Village`].
    pub fn get_village_id(&self) -> &String {
        &self.village_id
    }

    /// Returns a reference to the get village name of this [`Village`].
    pub fn get_village_name(&self) -> &String {
        &self.village_name
    }

    /// Returns the add player's id of this [`Village`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn add_player(&self) -> Result<(), SendError<VillageInlet>> {
        self.transmit(VillageInlet::AddPlayer)
    }
}
