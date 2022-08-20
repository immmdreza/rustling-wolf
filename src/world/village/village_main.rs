use std::{future::Future, time::Duration};

use mongodb::Client;
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
    time::{timeout, Instant},
};

use crate::{
    mongo_fns::world::{
        person::{
            add_person_to_village, assign_roles, cleanup_persons, count_village_persons,
            person_name_exists,
        },
        village::{cleanup_village_period, get_village_period, set_or_update_village_period},
    },
    world::village::periods::{Daytime, Period, RawPeriod},
};

use super::{
    inlet_data::VillageInlet,
    outlet_data::{AddPersonResult, VillageOutlet},
    transports::Transporter,
    VillageInfo,
};

#[derive(Debug)]
pub(super) enum VillageInternal {
    PersonsFilled,
    ExtendPopulationTime(Duration),
    Die,
}

impl From<VillageInternal> for SafeVillageInternal {
    fn from(vi: VillageInternal) -> Self {
        match vi {
            VillageInternal::PersonsFilled => SafeVillageInternal::PersonsFilled,
            VillageInternal::ExtendPopulationTime(e) => {
                SafeVillageInternal::ExtendPopulationTime(e)
            }
            VillageInternal::Die => panic!("SafeVillageInternal is suppose to filter this."),
        }
    }
}

enum SafeVillageInternal {
    PersonsFilled,
    ExtendPopulationTime(Duration),
}

enum InternalResult {
    Break,
    Return,
    Continue,
}

async fn received(
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
    match received {
        VillageInlet::RawString(s) => {
            println!("[🌏]: {}", s);
            sender.send(VillageOutlet::RawString(s)).await.unwrap_or(());
        }
        VillageInlet::AddPerson(name) => match get_village_period(&client, &village_id).await {
            Some(period) => match period {
                RawPeriod::Populating => {
                    let current_person_count = count_village_persons(&client, &village_id).await;
                    if current_person_count < max_persons.into() {
                        if person_name_exists(&client, &village_id, &name).await {
                            sender
                                .send(VillageOutlet::AddPerson(AddPersonResult::Failed(
                                    "The person name is duplicated".to_string(),
                                )))
                                .await
                                .unwrap_or(());
                        } else {
                            if let Some(pr) =
                                add_person_to_village(&client, &village_id, name.as_str()).await
                            {
                                sender
                                    .send(VillageOutlet::AddPerson(AddPersonResult::Added {
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
                                sender
                                    .send(VillageOutlet::AddPerson(AddPersonResult::Failed(
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
                _ => {
                    sender
                        .send(VillageOutlet::AddPerson(AddPersonResult::Failed(
                            "Not populating!".to_string(),
                        )))
                        .await
                        .unwrap_or(());
                }
            },
            None => todo!(),
        },
        VillageInlet::ExtendPopulationTime(time) => internal_sender
            .send(VillageInternal::ExtendPopulationTime(time))
            .await
            .unwrap_or(()),
        VillageInlet::Die => internal_sender.send(VillageInternal::Die).await.unwrap(),
    };
}

pub async fn cleanup_steps(client: &Client, village_id: &String, tp: &JoinHandle<()>) {
    cleanup_persons(client, village_id).await.unwrap();
    cleanup_village_period(client, village_id).await.unwrap();

    tp.abort();
}

pub(super) async fn village_main(info: VillageInfo, receiver: Receiver<VillageInlet>) -> () {
    println!(
        "Village {} ( {} ) thread started.",
        info.village_name, info.village_id
    );

    let (w_tx, mut w_tr) = mpsc::channel(1024);
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
        receiver,
        received,
        (info.clone(), w_tx, max_people),
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
            .send(VillageOutlet::PeriodCrossed(
                current_raw_period,
                _current_period,
            ))
            .await
            .unwrap();

        let send_out = |outlet: VillageOutlet| async { info.sender.send(outlet).await.unwrap() };

        match _current_period {
            Period::None => break,
            Period::Populating {
                min_persons,
                max_persons: _,
                max_dur,
            } => {
                // Callback should only call if players are filled!
                match continuously_listen_to_safe_internals_within_timeout(
                    &info,
                    &mut w_tr,
                    &tp,
                    max_dur,
                    |data| async {
                        match data {
                            Some(from_callback) => match from_callback {
                                SafeVillageInternal::PersonsFilled => {
                                    send_out(VillageOutlet::PeriodReady(
                                        current_raw_period.cross().unwrap(),
                                    ))
                                    .await;
                                    (Some(InternalResult::Break), Duration::ZERO)
                                }
                                SafeVillageInternal::ExtendPopulationTime(time) => (None, time),
                            },
                            None => (Some(InternalResult::Return), Duration::ZERO),
                        }
                    },
                    |_, dur| async move {
                        send_out(VillageOutlet::RawString(format!(
                            "Population cycle starts with {:#?}.",
                            dur
                        )))
                        .await;
                    },
                )
                .await
                {
                    Some(res) => match res {
                        InternalResult::Return => return,
                        _ => (),
                    },
                    // Timed out!
                    None => {
                        let current_joined =
                            count_village_persons(&info.client, &info.village_id).await;

                        if current_joined < min_persons.into() {
                            send_out(VillageOutlet::PopulatingTimedOut).await;

                            // ❌ Village may gone ...
                            cleanup_steps(&info.client, &info.village_id, &tp).await;
                            return;
                        } else {
                            send_out(VillageOutlet::PeriodReady(
                                current_raw_period.cross().unwrap(),
                            ))
                            .await;
                            break;
                        }
                    }
                }
            }
            Period::Assignments(_) => loop {
                let roles = assign_roles(&info.client, &info.village_id).await.unwrap();
                send_out(VillageOutlet::RawString(format!(
                    "Roles assigned: {:#?}.",
                    roles
                )))
                .await;
                break;
            },
            Period::FirstNight(dur) => {
                send_out(VillageOutlet::RawString(
                    "Wolves may know each other now ...".to_string(),
                ))
                .await;
                loop {
                    match timeout(dur, listen_to_default_internals(&info, &mut w_tr, &tp)).await {
                        Ok(res) => match res {
                            InternalResult::Return => return,
                            InternalResult::Continue => continue,
                            InternalResult::Break => break,
                        },
                        Err(_) => break,
                    }
                }
            }
            Period::DaytimeCycle(get_len) => {
                let mut current_daytime = Daytime::MidNight;
                loop {
                    current_daytime = current_daytime.cross();
                    let daytime_len = get_len(current_daytime);
                    send_out(VillageOutlet::DaytimeCycled(current_daytime, daytime_len)).await;

                    let daytime_start = Instant::now();
                    loop {
                        match current_daytime {
                            Daytime::SunRaise => match w_tr.recv().await {
                                Some(internal) => todo!(),
                                None => todo!(),
                            },
                            Daytime::MidNight => todo!(),
                            Daytime::LynchTime => todo!(),
                        }
                    }
                    // listen_to_default_internals(&info, &mut w_tr, &tp).await;
                }
            }
            Period::Ending => todo!(),
        }
    }
}

/// If this returns None, the village_main should return.
async fn listen_to_safe_internals(
    info: &VillageInfo,
    w_tr: &mut mpsc::Receiver<VillageInternal>,
    tp: &JoinHandle<()>,
) -> Option<SafeVillageInternal> {
    match w_tr.recv().await {
        Some(data) => match data {
            // Let finish it right now ...
            VillageInternal::Die => {
                // ❌ No more sender? no more village ...
                cleanup_steps(&info.client, &info.village_id, &tp).await;
                None
            }
            _ => Some(data.into()),
        },
        None => {
            // ❌ No more sender? no more village ...
            cleanup_steps(&info.client, &info.village_id, &tp).await;
            None
        }
    }
}

/// This will send a default answer to internals.
async fn listen_to_default_internals(
    info: &VillageInfo,
    w_tr: &mut mpsc::Receiver<VillageInternal>,
    tp: &JoinHandle<()>,
) -> InternalResult {
    match listen_to_safe_internals(info, w_tr, tp).await {
        Some(data) => match data {
            _ => InternalResult::Continue,
        },
        None => InternalResult::Return,
    }
}

async fn continuously_listen_to_safe_internals_within_timeout<F, Fut, F2, F2ut>(
    info: &VillageInfo,
    w_tr: &mut mpsc::Receiver<VillageInternal>,
    tp: &JoinHandle<()>,
    duration: Duration,
    handler: F,
    pre_loop: F2,
) -> Option<InternalResult>
where
    F: FnOnce(Option<SafeVillageInternal>) -> Fut + Copy,
    F2: FnOnce(VillageInfo, Duration) -> F2ut + Copy,
    Fut: Future<Output = (Option<InternalResult>, Duration)>,
    F2ut: Future<Output = ()>,
{
    let mut base_dur = duration;
    loop {
        pre_loop(info.clone(), base_dur.clone()).await;
        let loop_start = Instant::now();
        match timeout(base_dur, listen_to_safe_internals(info, w_tr, tp)).await {
            Ok(data) => {
                let (should_exit, requested_dur_addon) = handler(data).await;
                match should_exit {
                    Some(exit) => return Some(exit),
                    None => (),
                };

                let remaining = base_dur - loop_start.elapsed();
                base_dur = remaining + requested_dur_addon;
                continue;
            }
            Err(_) => return None,
        };
    }
}
