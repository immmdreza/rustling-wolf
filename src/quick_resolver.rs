use crate::{
    console_answer,
    world::world_inlet::{FromHeaven},
};

pub trait QuickResolver<T, F> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> FromHeaven
    where
        F: FnOnce(T) -> FromHeaven;
}

impl<T, F> QuickResolver<T, F> for Option<T> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> FromHeaven
    where
        F: FnOnce(T) -> FromHeaven,
    {
        match self {
            Some(target) => on_some(target),
            None => {
                console_answer!("{}", msg);
                FromHeaven::Nothing
            }
        }
    }
}

impl<T, F, U> QuickResolver<T, F> for Result<T, U> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> FromHeaven
    where
        F: FnOnce(T) -> FromHeaven,
    {
        match self {
            Ok(target) => on_some(target),
            Err(_) => {
                console_answer!("{}", msg);
                FromHeaven::Nothing
            }
        }
    }
}
