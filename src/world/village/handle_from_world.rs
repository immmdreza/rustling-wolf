use std::time::Duration;

use tokio::sync::mpsc;

use crate::{
    mongo_fns::world::{
        person::{add_person_to_village, count_village_persons, person_name_exists},
        village::get_village_period,
    },
    world::{village::periods::RawPeriod, world_inlet::FromVillage, AddPersonResult, WorldInlet},
};

use super::{inlet_data::VillageInlet, village_info::VillageInfo};

#[derive(Debug)]
pub(super) enum VillageInternal {
    PersonsFilled,
    ExtendPopulationTime(Duration),
    Die,

    WolvesVictimSelected(String),
    DoctorTargetSelected(String),
    SeerTargetSelected(String),
}

impl From<VillageInternal> for SafeVillageInternal {
    fn from(vi: VillageInternal) -> Self {
        match vi {
            VillageInternal::PersonsFilled => SafeVillageInternal::PersonsFilled,
            VillageInternal::ExtendPopulationTime(e) => {
                SafeVillageInternal::ExtendPopulationTime(e)
            }
            VillageInternal::Die => panic!("SafeVillageInternal is suppose to filter this."),
            VillageInternal::WolvesVictimSelected(s) => {
                SafeVillageInternal::WolvesVictimSelected(s)
            }
            VillageInternal::DoctorTargetSelected(s) => {
                SafeVillageInternal::DoctorTargetSelected(s)
            }
            VillageInternal::SeerTargetSelected(s) => SafeVillageInternal::SeerTargetSelected(s),
        }
    }
}

pub(super) enum SafeVillageInternal {
    PersonsFilled,
    ExtendPopulationTime(Duration),

    WolvesVictimSelected(String),
    DoctorTargetSelected(String),
    SeerTargetSelected(String),
}

pub(super) async fn received_from_world(
    received: VillageInlet,
    (
        VillageInfo {
            client,
            village_id,
            sender,
            village_name: _,
            period_maker: _,
        },
        internal_sender,
        max_persons,
    ): (VillageInfo, mpsc::Sender<VillageInternal>, u8),
) {
    use FromVillage::*;

    let send_to_world = |data: FromVillage| async {
        sender
            .send(WorldInlet::from_village(&village_id, data))
            .await
    };

    match received {
        VillageInlet::AddPerson(name) => match get_village_period(&client, &village_id).await {
            Some(period) => match period {
                RawPeriod::Populating => {
                    let current_person_count = count_village_persons(&client, &village_id).await;
                    if current_person_count < max_persons.into() {
                        if person_name_exists(&client, &village_id, &name).await {
                            send_to_world(AddPerson(AddPersonResult::Failed(
                                "The person name is duplicated".to_string(),
                            )))
                            .await
                            .unwrap_or(())
                        } else {
                            if let Some(pr) =
                                add_person_to_village(&client, &village_id, name.as_str()).await
                            {
                                send_to_world(AddPerson(AddPersonResult::Added {
                                    person_id: pr.get_id(),
                                    current_count: current_person_count + 1,
                                }))
                                .await
                                .unwrap_or(());

                                if current_person_count + 1 >= max_persons.into() {
                                    internal_sender
                                        .send(VillageInternal::PersonsFilled)
                                        .await
                                        .unwrap_or(());
                                }
                            } else {
                                send_to_world(AddPerson(AddPersonResult::Failed(
                                    "Error while inserting person.".to_string(),
                                )))
                                .await
                                .unwrap_or(());
                            }
                        }
                    } else {
                        internal_sender
                            .send(VillageInternal::PersonsFilled)
                            .await
                            .unwrap_or(());
                    }
                }
                _ => send_to_world(AddPerson(AddPersonResult::Failed(
                    "Not populating!".to_string(),
                )))
                .await
                .unwrap_or(()),
            },
            None => (),
        },
        VillageInlet::ExtendPopulationTime(time) => internal_sender
            .send(VillageInternal::ExtendPopulationTime(time))
            .await
            .unwrap_or(()),
        VillageInlet::Die => internal_sender.send(VillageInternal::Die).await.unwrap(),
        VillageInlet::RawString(s) => send_to_world(RawString(s)).await.unwrap_or(()),
    };
}
