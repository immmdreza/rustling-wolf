use std::time::{Duration, Instant};

use mongodb::Client;
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};

use crate::{
    mongo_fns::world::{
        person::{add_person_to_village, cleanup_persons, count_village_persons},
        village::{cleanup_village_period, get_village_period, set_or_update_village_period},
    },
    world::village::{
        periods::{Period, RawPeriod},
        VillageInternal,
    },
};

use super::{
    inlet_data::VillageInlet,
    outlet_data::{AddPersonResult, VillageOutlet},
    transports::Transporter,
    VillageInfo,
};

async fn received(
    received: VillageInlet,
    (client, village_id, sender, internal_sender, max_persons): (
        Client,
        String,
        mpsc::Sender<VillageOutlet>,
        mpsc::Sender<VillageInternal>,
        u8,
    ),
) {
    match received {
        VillageInlet::RawString(s) => {
            println!("[🌏]: {}", s);
            sender.send(VillageOutlet::RawString(s)).await.unwrap();
        }
        VillageInlet::AddPerson => match get_village_period(&client, &village_id).await {
            Some(period) => match period {
                RawPeriod::Populating => {
                    let current_person_count =
                        count_village_persons(&client, village_id.clone()).await;
                    if current_person_count < max_persons.into() {
                        add_person_to_village(&client, village_id.clone()).await;
                        sender
                            .send(VillageOutlet::AddPerson(AddPersonResult::Added {
                                person_id: uuid::Uuid::new_v4().to_string(),
                                current_count: current_person_count + 1,
                            }))
                            .await
                            .unwrap();
                    } else {
                        internal_sender
                            .send(VillageInternal::PersonsFilled)
                            .await
                            .unwrap();
                    }
                }
                _ => {
                    sender
                        .send(VillageOutlet::AddPerson(AddPersonResult::Failed(
                            "Not populating!".to_string(),
                        )))
                        .await
                        .unwrap();
                }
            },
            None => todo!(),
        },
        VillageInlet::ExtendPopulationTime(time) => {
            internal_sender
                .send(VillageInternal::ExtendPopulationTime(time))
                .await
                .unwrap();
        }
    };
}

pub async fn cleanup_steps(client: &Client, village_id: &String, tp: JoinHandle<()>) {
    cleanup_persons(client, village_id).await.unwrap();
    cleanup_village_period(client, village_id).await.unwrap();

    tp.abort();
}

pub(super) async fn village_main(info: VillageInfo) -> () {
    println!(
        "Village {} ( {} ) thread started.",
        info.village_name, info.village_id
    );

    let (w_tx, mut w_tr) = mpsc::channel(1024);

    let village_id = info.village_id.clone();
    let max_people = match (info.period_maker)(&RawPeriod::Populating) {
        Period::Populating {
            min_persons: _,
            max_persons,
            max_dur: _,
        } => max_persons,
        _ => 0,
    };
    let tp = Transporter::spawn(
        "Village/World to InnerVillage!".to_string(),
        info.receiver,
        received,
        (
            info.client.clone(),
            village_id,
            info.sender.clone(),
            w_tx,
            max_people,
        ),
    );
    println!("Transporter spawned!");

    let mut current_raw_period = RawPeriod::None;
    let mut _current_period = (info.period_maker)(&current_raw_period);

    loop {
        current_raw_period = match current_raw_period.cross() {
            Ok(cur) => cur,
            Err(_) => break,
        };
        _current_period = (info.period_maker)(&current_raw_period);
        set_or_update_village_period(&info.client, &info.village_id, &current_raw_period).await;
        info.sender
            .send(VillageOutlet::PeriodCrossed(current_raw_period))
            .await
            .unwrap();

        match _current_period {
            Period::None => break,
            Period::Populating {
                min_persons,
                max_persons: _,
                max_dur,
            } => {
                // Callback should only call if players are filled!
                let mut base_dur = Duration::ZERO;
                let mut first_loop = true;
                loop {
                    let active_dur = if first_loop {
                        base_dur + max_dur
                    } else {
                        base_dur
                    };

                    info.sender
                        .send(VillageOutlet::RawString(format!(
                            "Population cycle starts with {:#?}.",
                            active_dur
                        )))
                        .await
                        .unwrap();

                    let loop_start = Instant::now();
                    match timeout(active_dur, w_tr.recv()).await {
                        Ok(recv_result) => match recv_result {
                            Some(from_callback) => match from_callback {
                                VillageInternal::PersonsFilled => {
                                    info.sender
                                        .send(VillageOutlet::PeriodReady(
                                            current_raw_period.cross().unwrap(),
                                        ))
                                        .await
                                        .unwrap();
                                    break;
                                }
                                VillageInternal::ExtendPopulationTime(time) => {
                                    let remained = active_dur - loop_start.elapsed();
                                    base_dur = remained + time;
                                    first_loop = false;
                                    continue;
                                }
                            },
                            None => break,
                        },
                        // Timed out! ask the callback for gathered players
                        Err(_) => {
                            let current_joined =
                                count_village_persons(&info.client, info.village_id.clone()).await;

                            if current_joined < min_persons.into() {
                                info.sender
                                    .send(VillageOutlet::PopulatingTimedOut)
                                    .await
                                    .unwrap();

                                // ❌ Village may gone ...
                                cleanup_steps(&info.client, &info.village_id, tp).await;
                                return;
                            } else {
                                info.sender
                                    .send(VillageOutlet::PeriodReady(
                                        current_raw_period.cross().unwrap(),
                                    ))
                                    .await
                                    .unwrap();
                                break;
                            }
                        }
                    }
                }
            }
            Period::Assignments(_) => {
                tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
            }
            Period::DaytimeCycle(_) => todo!(),
            Period::Ending => todo!(),
        }
    }
}
