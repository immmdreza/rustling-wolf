use std::time::Duration;

use rustling_wolf::{
    prelude::*,
    world::village::periods::{AssignmentMode, Daytime, Period, RawPeriod},
};

fn default_period_maker(raw: &RawPeriod) -> Period {
    match raw {
        RawPeriod::Populating => Period::Populating {
            min_persons: 5,
            max_persons: 10,
            max_dur: Duration::from_secs(5),
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

#[tokio::main]
async fn main() {
    let mut world = World::new().await;

    let _ = world.create_village_default_receiver(None, default_period_maker);

    world.idle().await;
}
