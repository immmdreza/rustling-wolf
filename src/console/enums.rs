#[derive(Clone, PartialEq, Eq)]
pub enum RoutingName {
    Root,
    Persons,
    Village,
}

#[derive(Clone)]
pub enum ConsoleCommand {
    Routing {
        name: RoutingName,
        args: Vec<String>,
    },
}

pub trait EndpointParser<T>
where
    T: Send + Sync,
{
    fn get_trigger(&self) -> &'static str;

    fn map(&self, args: Vec<String>) -> Option<T>;
}

pub struct EndpointModel<M, T>
where
    M: Fn(&Vec<String>) -> Option<T>,
    T: Send + Sync,
{
    trigger: &'static str,
    mapper: M,
}

impl<M, T> EndpointModel<M, T>
where
    M: Fn(&Vec<String>) -> Option<T>,
    T: Send + Sync,
{
    #[allow(dead_code)]
    pub fn new(trigger: &'static str, mapper: M) -> Self {
        Self { trigger, mapper }
    }
}

impl<M, T> EndpointParser<T> for EndpointModel<M, T>
where
    M: Fn(&Vec<String>) -> Option<T>,
    T: Send + Sync,
{
    fn get_trigger(&self) -> &'static str {
        self.trigger
    }

    fn map(&self, args: Vec<String>) -> Option<T> {
        (self.mapper)(&args)
    }
}
