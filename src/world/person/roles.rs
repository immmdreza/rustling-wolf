use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    NoRole,
    Villager,
    Wolf,
    MasterWolf,
    Seer,
    Doctor,
}

impl Role {
    pub fn is_eatable(&self) -> bool {
        match self {
            Role::Wolf => false,
            Role::MasterWolf => false,
            _ => true,
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::NoRole => write!(f, "NoRole"),
            Role::Villager => write!(f, "Villager 👩"),
            Role::Wolf => write!(f, "Wolf 🐺"),
            Role::MasterWolf => write!(f, "MasterWolf ⚡"),
            Role::Seer => write!(f, "Seer 🔍"),
            Role::Doctor => write!(f, "Doctor 🩺"),
        }
    }
}

impl Into<u8> for Role {
    fn into(self) -> u8 {
        match self {
            Role::NoRole => 0,
            Role::Villager => 1,
            Role::Wolf => 2,
            Role::MasterWolf => 3,
            Role::Seer => 4,
            Role::Doctor => 5,
        }
    }
}

impl From<u8> for Role {
    fn from(i: u8) -> Self {
        match i {
            0 => Role::NoRole,
            1 => Role::Villager,
            2 => Role::Wolf,
            3 => Role::MasterWolf,
            4 => Role::Seer,
            5 => Role::Doctor,
            _ => Role::NoRole,
        }
    }
}
