pub(super) mod enums;
pub(super) mod parser;
pub mod prelude;

use dptree::prelude::*;

use self::enums::{ConsoleCommand, EndpointParser, RoutingName};

pub type ConsoleHandler = Endpoint<'static, DependencyMap, ()>;

pub(super) fn parse_route_name_and_args<T>(t: T) -> Option<(String, Vec<String>)>
where
    T: IntoIterator<Item = String>,
{
    let mut iterator = t.into_iter();

    let first = iterator.next()?;
    let args = iterator.collect();

    Some((first, args))
}

pub(super) fn root() -> ConsoleHandler {
    dptree::filter_map(|req: String| {
        let args = req.split(' ').map(|s| s.to_owned()).collect();
        Some(ConsoleCommand::Routing {
            name: RoutingName::Root,
            args,
        })
    })
}

pub(super) fn routing(trigger: &'static str, to: RoutingName) -> ConsoleHandler {
    dptree::filter_map(move |cmd: ConsoleCommand| match cmd {
        ConsoleCommand::Routing { name: _, args } => {
            let (first, args) = parse_route_name_and_args(args)?;
            if first.to_lowercase() == trigger {
                Some(ConsoleCommand::Routing {
                    name: to.to_owned(),
                    args,
                })
            } else {
                None
            }
        }
    })
}

pub(super) fn ending<M, T>(trigger: &'static str, mapper: M) -> ConsoleHandler
where
    M: Fn(&[String]) -> Option<T> + Sync + Send + 'static,
    T: Send + Sync + 'static,
{
    dptree::filter_map(move |cmd: ConsoleCommand| match cmd {
        ConsoleCommand::Routing { name: _, args } => {
            let (cmd, args) = parse_route_name_and_args(args)?;
            if cmd.to_lowercase() == trigger {
                mapper(&args)
            } else {
                None
            }
        }
    })
}

pub(super) fn unmapped_ending(trigger: &'static str) -> ConsoleHandler {
    ending(trigger, |_| Some(()))
}

#[allow(dead_code)]
pub(super) fn classified_ending<F, T>(f: F) -> ConsoleHandler
where
    F: EndpointParser<T> + Sync + Send + 'static,
    T: Send + Sync + 'static,
{
    dptree::filter_map(move |cmd: ConsoleCommand| match cmd {
        ConsoleCommand::Routing { name: _, args } => {
            let (cmd, args) = parse_route_name_and_args(args)?;
            if cmd.to_lowercase() == f.get_trigger() {
                f.map(args)
            } else {
                None
            }
        }
    })
}
