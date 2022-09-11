use crate::tower::Antenna;

#[derive(Debug, Clone)]
pub enum AskWorld {
    RawString(String),
    AskVillageName(String),
}

#[derive(Debug, Clone)]
pub enum WorldAnswered {
    RawString(String),
    VillageName(Option<String>),
}

#[derive(Debug, Clone)]
pub struct WorldAntenna {
    antenna: Antenna<AskWorld, WorldAnswered>,
}

impl WorldAntenna {
    pub async fn ask(&self, asked: AskWorld) -> Option<WorldAnswered> {
        self.antenna.ask(asked).await
    }

    pub async fn ask_village_name(&self, village_id: &str) -> Option<String> {
        match self
            .ask(AskWorld::AskVillageName(village_id.to_string()))
            .await?
        {
            WorldAnswered::VillageName(name) => name,
            _ => None,
        }
    }

    pub async fn ask_raw_string(&self, raw: &str) -> Option<String> {
        match self.ask(AskWorld::RawString(raw.to_string())).await? {
            WorldAnswered::RawString(name) => Some(name),
            _ => None,
        }
    }
}

pub trait ToWorldAntenna {
    fn to_world_antenna(self) -> WorldAntenna;
}

impl ToWorldAntenna for Antenna<AskWorld, WorldAnswered> {
    fn to_world_antenna(self) -> WorldAntenna {
        WorldAntenna { antenna: self }
    }
}
