use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    gpt,
    world::{
        world_inlet::{FromHeaven, WorldInlet},
        WorldAntenna,
    },
};

use super::{ending, root, routing, unmapped_ending, ConsoleHandler, RoutingName};

pub fn get_console_input_receiver() -> Receiver<String> {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    tokio::spawn(async move {
        // read from console and send watchers
        let mut input = String::new();
        loop {
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();
            tx.send(input.trim().to_string()).await.unwrap();
        }
    });

    rx
}

pub enum Received<T> {
    FromConsole(String),
    FromOther(T),
}

pub async fn receive_neither_console_or_other<T>(
    console_receiver: &mut Receiver<String>,
    other_receiver: &mut Receiver<T>,
) -> Received<T> {
    tokio::select! {
        v = console_receiver.recv() => Received::FromConsole(v.unwrap()),
        v = other_receiver.recv() => Received::FromOther(v.unwrap()),
    }
}

pub struct ConsoleMotor {
    dispatcher: ConsoleHandler,
    tx: Sender<WorldInlet>,
    antenna: WorldAntenna,
}

impl ConsoleMotor {
    pub fn new(rx: Sender<WorldInlet>, antenna: WorldAntenna) -> Self {
        Self {
            dispatcher: get_dispatcher(),
            tx: rx,
            antenna,
        }
    }

    pub async fn dispatch(
        &self,
        input: String,
    ) -> std::ops::ControlFlow<(), dptree::prelude::DependencyMap> {
        self.dispatcher
            .dispatch(dptree::deps![input, self.tx.clone(), self.antenna.clone()])
            .await
    }
}

fn get_dispatcher() -> ConsoleHandler {
    dptree::entry().branch(
        root().branch(
            routing("vg", RoutingName::Village)
                .branch(ending("kill", parse_kill_village).endpoint(kill_village))
                .branch(unmapped_ending("list").endpoint(list_villages))
                .branch(unmapped_ending("new").endpoint(new_village))
                .branch(
                    routing("pr", RoutingName::Persons)
                        .branch(ending("add", parse_add_person).endpoint(add_person))
                        .branch(ending("fill", parse_fill_person).endpoint(fill_person))
                        .endpoint(|| async { println!("Unknown persons command.") }),
                )
                .endpoint(|| async { println!("Unknown village command") }),
        ),
    )
}

fn parse_kill_village(args: &[String]) -> Option<String> {
    let (village_id,) = gpt!(; args => String)?;
    Some(village_id)
}

async fn kill_village(village_id: String, rx: Sender<WorldInlet>) {
    rx.send(WorldInlet::FromHeaven(FromHeaven::KillVillage {
        village_id,
    }))
    .await
    .unwrap_or_default()
}

async fn list_villages(rx: Sender<WorldInlet>) {
    rx.send(WorldInlet::FromHeaven(FromHeaven::ListVillages))
        .await
        .unwrap_or_default()
}

async fn new_village(rx: Sender<WorldInlet>) {
    rx.send(WorldInlet::FromHeaven(FromHeaven::NewVillage))
        .await
        .unwrap_or_default()
}

fn parse_add_person(args: &[String]) -> Option<(String, String)> {
    Some(gpt!(; args => String, String)?)
}

async fn add_person((village_id, person_name): (String, String), rx: Sender<WorldInlet>) {
    rx.send(WorldInlet::FromHeaven(FromHeaven::RequestPerson {
        village_id,
        person_name,
    }))
    .await
    .unwrap_or_default()
}

fn parse_fill_person(args: &[String]) -> Option<(String, u8)> {
    Some(gpt!(; args => String, u8)?)
}

async fn fill_person((village_id, count): (String, u8), rx: Sender<WorldInlet>) {
    println!("Filling persons ...");
    rx.send(WorldInlet::FromHeaven(FromHeaven::FillPersons {
        village_id,
        count,
    }))
    .await
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_name() {
        let dp = get_dispatcher();
        let _ = dp.dispatch(dptree::deps!["vg pr dick 12345678 10"]).await;
        let _ = dp.dispatch(dptree::deps!["vg pare"]).await;
    }
}
