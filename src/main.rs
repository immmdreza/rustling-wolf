use std::collections::VecDeque;

use rustling_wolf::{
    console::{get_console_input_receiver, receive_neither_console_or_other},
    console_answer,
    quick_resolver::QuickResolver,
    tower::Antenna,
    world::{
        by_tower::{AskWorld, WorldAnswered},
        world_inlet::{FromHeaven, WorldInlet},
        world_outlet::WorldOutlet,
        World,
    },
};
use tokio::sync::mpsc::Sender;

async fn handle_console_input(
    tx: &Sender<WorldInlet>,
    antenna: &Antenna<AskWorld, WorldAnswered>,
    input: String,
) {
    use FromHeaven::*;

    let mut parts = VecDeque::from_iter(input.split(' ').into_iter());
    let command = parts.pop_front().unwrap();

    match command {
        "echo" => match parts.pop_front() {
            Some(txt) => {
                let res = antenna
                    .ask(AskWorld::RawString(txt.to_string()))
                    .await
                    .unwrap();
                match res {
                    WorldAnswered::RawString(replied) => {
                        println!("World replied with: {}", replied)
                    }
                }

                return ();
            }
            None => (),
        },
        _ => (),
    };

    tx.send(WorldInlet::FromHeaven(match command {
        "vg" => parts.pop_front().resolve_world_inlet(
            |villages_command| match villages_command {
                "new" => NewVillage,
                "list" => ListVillages,
                "kill" => parts.pop_front().resolve_world_inlet(
                    |res| KillVillage {
                        village_id: res.to_string(),
                    },
                    "Missing village id.",
                ),
                "pr" => parts.pop_front().resolve_world_inlet(
                    |persons_command| match persons_command {
                        "add" => parts.pop_front().resolve_world_inlet(
                            |village_id| {
                                parts.pop_front().resolve_world_inlet(
                                    |person_name| RequestPerson {
                                        village_id: village_id.to_string(),
                                        person_name: person_name.to_string(),
                                    },
                                    "Missing person name.",
                                )
                            },
                            "Missing village id",
                        ),
                        "fill" => parts.pop_front().resolve_world_inlet(
                            |village_id| {
                                parts.pop_front().resolve_world_inlet(
                                    |count| {
                                        count.parse::<u8>().resolve_world_inlet(
                                            |count| FillPersons {
                                                village_id: village_id.to_string(),
                                                count,
                                            },
                                            "Invalid count given.",
                                        )
                                    },
                                    "Missing count.",
                                )
                            },
                            "Missing village id.",
                        ),
                        _ => Nothing,
                    },
                    "Missing persons command! (add, fill)",
                ),
                _ => {
                    console_answer!("Unknown village command! (new, list, kill, persons).");
                    Nothing
                }
            },
            "Unknown command! (villages).",
        ),
        _ => {
            console_answer!("Unknown village command! (new, list, kill, persons).");
            Nothing
        }
    }))
    .await
    .unwrap();
}

async fn handle_world_input(tx: &Sender<WorldInlet>, input: WorldOutlet) {
    use rustling_wolf::world::world_outlet::WithVillage::*;

    match input {
        WorldOutlet::VillageList(_) => todo!(),
        WorldOutlet::WithVillage { village_id, data } => match data {
            RawString(_) => todo!(),
            VillageDisposed => todo!(),
            PopulationDone(_) => todo!(),
            PeriodReady(_) => todo!(),
            NewPeriod(_) => todo!(),
            PopulationTimedOut => todo!(),
            DaytimeCycled(_, _) => todo!(),
            AddPersonResult(_) => todo!(),
            NightActionResultReport(_) => todo!(),
            NightTurn {
                turn,
                available_persons,
            } => todo!(),
        },
    }
}

#[tokio::main]
async fn main() {
    let (world, mut world_receiver) = World::new().await;

    console_answer!("The world has been initialized, you can command now ...");
    let mut console_receiver = get_console_input_receiver();

    let tx = world.sender().clone();
    let antenna = world.antenna().clone();

    world.live();

    loop {
        match receive_neither_console_or_other(&mut console_receiver, &mut world_receiver).await {
            rustling_wolf::console::Received::FromConsole(from_console) => {
                handle_console_input(&tx, &antenna, from_console).await
            }
            rustling_wolf::console::Received::FromOther(from_world) => {
                handle_world_input(&tx, from_world).await
            }
        }
    }
}
