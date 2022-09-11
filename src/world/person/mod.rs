use self::roles::Role;

pub mod assigner;
pub mod roles;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Person {
    person_id: String,
    village_id: String,
    is_alive: bool,
    role_code: u8,
    eatable: bool,
}

impl Person {
    pub fn new(person_id: String, village_id: &str, role_code: u8, eatable: bool) -> Self {
        Person {
            person_id,
            village_id: village_id.to_string(),
            is_alive: true,
            role_code,
            eatable,
        }
    }

    pub fn get_id(&self) -> String {
        self.person_id.to_string()
    }

    pub fn get_role(&self) -> Role {
        Role::from(self.role_code)
    }
}
