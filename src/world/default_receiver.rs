use std::time::Duration;

use crate::{
    mongo_fns::world::person::count_village_persons,
    prelude::{SimplifiedVillage, VillageOutlet},
    world::village::periods::RawPeriod,
};

use super::village::outlet_data::AddPersonResult;

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
            AddPersonResult::Failed(_) => println!(
                "[🌴 {}]: Failed creating person.",
                simp_village.get_village_name(),
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
        VillageOutlet::PeriodCrossed(crossed) => {
            println!(
                "[🌴 {}]: Next period started: {:?}",
                simp_village.get_village_name(),
                crossed
            );
            match crossed {
                RawPeriod::Populating => {
                    simp_village.add_player().await.unwrap();

                    simp_village
                        .extend_population_dur(Duration::from_secs(10))
                        .await
                        .unwrap();
                }
                _ => (),
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
