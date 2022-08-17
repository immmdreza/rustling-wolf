use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::world::village::{outlet_data::VillageOutlet, transports::Transporter};

use super::{inlet_data::VillageInlet, VillageInfo};

fn received(received: VillageInlet, village_data: &(String, Sender<VillageOutlet>)) -> () {
    let (_village_id, sender) = village_data;
    match received {
        VillageInlet::RawString(s) => {
            println!("[🌏]: {}", s);
            sender
                .send(VillageOutlet::RawString("Give answer".to_string()))
                .unwrap();
        }
        VillageInlet::AddPlayer => sender
            .send(VillageOutlet::PlayerAdded {
                player_id: uuid::Uuid::new_v4().to_string(),
            })
            .unwrap(),
    }
}

pub(super) fn village_main(info: VillageInfo) {
    println!(
        "Village {} ( {} ) thread started.",
        info.village_name, info.village_id
    );

    let village_id = info.village_id.clone();

    Transporter::spawn(info.receiver, received, (village_id, info.sender.clone()));
    println!("Transporter spawned!");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
