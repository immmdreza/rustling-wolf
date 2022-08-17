use std::time::Duration;

use rustling_wolf::prelude::*;

fn main() {
    let village = Village::new(
        "Mujik Village",
        Some(|data, id| match data {
            VillageOutlet::RawString(s) => {
                println!("[🌴- {}]: {}", id, s);
            }
            VillageOutlet::PlayerAdded { player_id } => {
                println!("[🌴- {}]: Created player with id {}.", id, player_id);
            }
        }),
    );

    println!("Hello, world from {}!", village.get_village_name());

    village.add_player().unwrap();

    idle_for(Duration::from_secs(5))
}
