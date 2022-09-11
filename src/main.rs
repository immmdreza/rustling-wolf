use rustling_wolf::{
    console::prelude::{
        get_console_input_receiver, receive_neither_console_or_other, ConsoleMotor, Received,
    },
    world::{
        person,
        world_inlet::{self, WorldInlet},
        world_outlet::{self, WorldOutlet},
        World, WorldAntenna,
    },
};
use tokio::sync::mpsc::Sender;

async fn handle_world_input(_tx: &Sender<WorldInlet>, antenna: &WorldAntenna, input: WorldOutlet) {
    use rustling_wolf::world::world_outlet::WithVillage::*;

    match input {
        WorldOutlet::RawStringResult(result) => match result {
            Ok(ok) => println!("[🧁✅]: {ok}"),
            Err(err) => println!("[🧁❌]: {err}"),
        },
        WorldOutlet::VillageList(_) => todo!(),
        WorldOutlet::WithVillage { village_id, data } => match data {
            RawString(raw) => println!("[🧀 {village_id}]: {raw}"),
            VillageDisposed => println!("[🧀 {village_id}]: Disposed!"),
            PopulationDone(count) => println!("[🧀 {village_id}]: Populated with {count} persons."),
            PeriodReady(period) => {
                println!(
                    "[🧀 {village_id}]: Ready to merge to new period {:?}",
                    period
                )
            }
            NewPeriod(period) => println!("[🧀 {village_id}]: New period {:?}", period),
            PopulationTimedOut => println!("[🧀 {village_id}] Failed to populate, disposing ..."),
            DaytimeCycled(daytime, dur) => {
                println!(
                    "[🧀 {village_id}]: New daytime {} for {:#?} long",
                    daytime, dur
                )
            }
            AddPersonResult(result) => match result {
                world_inlet::AddPersonResult::Added {
                    person_id,
                    current_count,
                } => {
                    println!(
                        "[🧀 {village_id}]: Created person with id {} ({} persons in village).",
                        person_id, current_count
                    );
                }
                world_inlet::AddPersonResult::Failed(err) => {
                    println!("[🧀 {village_id}]: Failed creating person: {}.", err);
                }
            },
            NightActionResultReport(report) => match report {
                world_inlet::NightActionResult::NoneEaten => {
                    println!("[🧀 {village_id}]: No one eaten last night.");
                }
                world_inlet::NightActionResult::PersonEaten(person_id) => {
                    println!(
                        "[🧀 {village_id}]: A person is eaten last night ({}).",
                        person_id
                    );
                }
                world_inlet::NightActionResult::PersonSaved(person_id) => {
                    println!(
                        "[🧀 {village_id}]: A person is saved last night ({}).",
                        person_id
                    );
                }
                world_inlet::NightActionResult::SeerReport(person_id, is_wolf) => {
                    let is_wolf_text = match is_wolf {
                        true => "",
                        false => " not",
                    };
                    println!(
                        "[🧀 {village_id}]: Seer report: person {} is{} wolf.",
                        person_id, is_wolf_text
                    );
                }
            },
            NightTurn {
                turn,
                available_persons,
            } => {
                let village_name = antenna.ask_village_name(&village_id).await.unwrap();
                match turn {
                    world_outlet::NightTurn::Wolf => {
                        println!(
                            "[🧀 {}]: Hungry wolves in {} village, who to eat?",
                            village_id, village_name,
                        );
                        println!("[! 🍴] Possible eatable persons:");
                        for eatable in available_persons {
                            println!("{}", eatable.get_id());
                        }
                    }
                    world_outlet::NightTurn::Doctor => {
                        println!(
                            "[🧀 {}]: Doctor in {} village, who to save tonight?",
                            village_id, village_name,
                        );
                        println!("[! ❤️‍🩹] Possible saveable persons:");
                        for person in available_persons {
                            match person.get_role() {
                                person::roles::Role::Doctor => continue,
                                _ => println!("{}", person.get_id()),
                            }
                        }
                    }
                    world_outlet::NightTurn::Seer => {
                        println!(
                            "[🧀 {}]: Wise seer in {} village, who to ...?",
                            village_id, village_name,
                        );
                        println!("[! 🔍] Possible ... persons:");
                        for person in available_persons {
                            match person.get_role() {
                                person::roles::Role::Seer => continue,
                                _ => println!("{}", person.get_id()),
                            }
                        }
                    }
                }
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let (world, mut world_receiver) = World::new().await;

    let mut console_receiver = get_console_input_receiver();

    let tx = world.sender().clone();
    let antenna = world.antenna().clone();

    let console_motor = ConsoleMotor::new(tx.clone(), antenna.clone());

    world.live();

    loop {
        match receive_neither_console_or_other(&mut console_receiver, &mut world_receiver).await {
            Received::FromConsole(from_console) => {
                console_motor.dispatch(from_console).await;
            }
            Received::FromOther(from_world) => handle_world_input(&tx, &antenna, from_world).await,
        }
    }
}
