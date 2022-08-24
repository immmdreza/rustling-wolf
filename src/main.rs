use std::collections::VecDeque;

use rustling_wolf::{
    console::setup_console_receiver,
    console_answer,
    quick_resolver::QuickResolver,
    world::{
        world_inlet::{FromHeaven, WorldInlet},
        World,
    },
};

#[tokio::main]
async fn main() {
    use FromHeaven::*;

    let mut world = World::new().await;

    let rx = setup_console_receiver(|s| {
        let mut parts = VecDeque::from_iter(s.split(' ').into_iter());
        let command = parts.pop_front().unwrap();

        WorldInlet::FromHeaven(match command {
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
        })
    });

    console_answer!("The world has been initialized, you can command now ...");
    world.idle(rx).await
}
