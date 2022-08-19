use std::time::Duration;

use crate::{
    mongo_fns::world::person::count_village_persons,
    world::{village::periods::RawPeriod, WorldInlet},
};

use super::village::{
    outlet_data::{AddPersonResult, VillageOutlet},
    periods::{AssignmentMode, Daytime, Period},
    simplified_village::SimplifiedVillage,
};

pub(crate) async fn received_from_village(data: VillageOutlet, simp_village: SimplifiedVillage) {
    match data {
        VillageOutlet::RawString(s) => {
            println!("[🌴 {}]: {}", simp_village.get_village_name(), s);
        }
        VillageOutlet::AddPerson(r) => match r {
            AddPersonResult::Added {
                person_id,
                current_count,
            } => println!(
                "[🌴 {}]: Created person with id {} ({} persons in village).",
                simp_village.get_village_name(),
                person_id,
                current_count
            ),
            AddPersonResult::Failed(err) => println!(
                "[🌴 {}]: Failed creating person: {}.",
                simp_village.get_village_name(),
                err
            ),
        },
        VillageOutlet::PeriodReady(new_period) => {
            println!(
                "[🌴 {}]: Village is ready to merge to the next period: {:?}",
                simp_village.get_village_name(),
                new_period
            );
            match new_period {
                RawPeriod::Assignments => {
                    let joined_persons = count_village_persons(
                        &simp_village.village.client,
                        simp_village.get_village_id().to_string(),
                    )
                    .await;
                    println!(
                        "[🌴 {}]: Village populated with {} persons",
                        simp_village.get_village_name(),
                        joined_persons
                    );
                }
                _ => (),
            }
        }
        VillageOutlet::PeriodCrossed(crossed, detailed) => {
            println!(
                "[🌴 {}]: Next period started: {:?}",
                simp_village.get_village_name(),
                crossed
            );
            match crossed {
                _ => {
                    simp_village
                        .send_to_world(WorldInlet::NewPeriod {
                            village_id: simp_village.get_village_id().clone(),
                            period: detailed,
                        })
                        .await
                        .unwrap();
                }
            }
        }
        VillageOutlet::PopulatingTimedOut => {
            simp_village.notify_my_die().await.unwrap();
            println!(
                "[🌴 {}]: Populating timed out! village is terminated,",
                simp_village.get_village_name(),
            );
        }
    }
}

pub(crate) fn default_period_maker(raw: &RawPeriod) -> Period {
    match raw {
        RawPeriod::Populating => Period::Populating {
            min_persons: 5,
            max_persons: 10,
            max_dur: Duration::from_secs(300),
        },
        RawPeriod::Assignments => Period::Assignments(AssignmentMode::Normal),
        RawPeriod::DaytimeCycle => Period::DaytimeCycle(|dt| match dt {
            Daytime::MidNight => Duration::from_secs(30),
            Daytime::SunRaise => Duration::from_secs(30),
            Daytime::LynchTime => Duration::from_secs(30),
        }),
        RawPeriod::Ending => Period::Ending,
        RawPeriod::None => Period::None,
    }
}
