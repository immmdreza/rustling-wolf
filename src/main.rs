use rustling_wolf::{
    console::setup_console_receiver,
    world::{World, WorldInlet},
};

#[tokio::main]
async fn main() {
    let mut world = World::new().await;

    let rx = setup_console_receiver(|s| {
        let parts = s.split(' ').collect::<Vec<_>>();
        let command = *parts.get(0).unwrap();

        match command {
            "list-villages" => WorldInlet::ListVillages,
            "new-village" => WorldInlet::NewVillage,
            "add-player" => {
                let village_id = parts[1].to_string();
                let person_name = parts[2].to_string();
                WorldInlet::RequestPerson {
                    village_id,
                    person_name,
                }
            }
            "kill" => {
                let village_id = parts[1].to_string();
                WorldInlet::KillVillage { village_id }
            }
            _ => WorldInlet::None,
        }
    });

    world.idle(rx).await
}
