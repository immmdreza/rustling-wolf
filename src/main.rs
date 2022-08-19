use std::collections::VecDeque;

use rustling_wolf::{
    console::setup_console_receiver,
    world::{World, WorldInlet},
};

macro_rules! console_answer {
    ($($arg:tt)*) => {
        print!("[🤖] ");
        println!($($arg)*);
    };
}

use console_answer;

fn simplify<T>(target: Option<T>, default: T, msg: &str) {}

#[tokio::main]
async fn main() {
    let mut world = World::new().await;

    let rx = setup_console_receiver(|s| {
        let mut parts = VecDeque::from_iter(s.split(' ').into_iter());
        let command = parts.pop_front().unwrap();

        match command {
            "villages" => match parts.pop_front() {
                Some(villages_command) => match villages_command {
                    "new" => WorldInlet::NewVillage,
                    "list" => WorldInlet::ListVillages,
                    "kill" => {
                        if let Some(village_id) = parts.pop_front() {
                            WorldInlet::KillVillage {
                                village_id: village_id.to_string(),
                            }
                        } else {
                            console_answer!("Missing village id.");
                            WorldInlet::None
                        }
                    }
                    "persons" => {
                        if let Some(persons_command) = parts.pop_front() {
                            match persons_command {
                                "add" => {
                                    if let Some(village_id) = parts.pop_front() {
                                        if let Some(person_name) = parts.pop_front() {
                                            WorldInlet::RequestPerson {
                                                village_id: village_id.to_string(),
                                                person_name: person_name.to_string(),
                                            }
                                        } else {
                                            console_answer!("Missing person name.");
                                            WorldInlet::None
                                        }
                                    } else {
                                        console_answer!("Missing village id.");
                                        WorldInlet::None
                                    }
                                }
                                "fill" => {
                                    if let Some(village_id) = parts.pop_front() {
                                        if let Some(count) = parts.pop_front() {
                                            if let Ok(count) = count.parse::<u8>() {
                                                WorldInlet::FillPersons {
                                                    village_id: village_id.to_string(),
                                                    count,
                                                }
                                            } else {
                                                console_answer!("Invalid count.");
                                                WorldInlet::None
                                            }
                                        } else {
                                            console_answer!("Missing count.");
                                            WorldInlet::None
                                        }
                                    } else {
                                        console_answer!("Missing village id.");
                                        WorldInlet::None
                                    }
                                }
                                _ => WorldInlet::None,
                            }
                        } else {
                            console_answer!("Missing persons command! (add, fill)");
                            WorldInlet::None
                        }
                    }

                    _ => {
                        console_answer!("Unknown village command! (new, list, kill, persons).");
                        WorldInlet::None
                    }
                },
                None => {
                    console_answer!("Provide a village command! (new, list, kill, persons).");
                    WorldInlet::None
                }
            },
            _ => {
                console_answer!("Unknown command! (villages).");
                WorldInlet::None
            }
        }
    });

    world.idle(rx).await
}
