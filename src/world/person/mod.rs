pub mod assigner;
pub(crate) mod roles;

#[allow(dead_code)]
pub struct Person {
    person_id: String,
    village_id: String,
    is_alive: bool,
    role_code: u8,
}

impl Person {
    pub fn new(person_id: String, village_id: &str, role_code: u8) -> Self {
        Person {
            person_id,
            village_id: village_id.to_string(),
            is_alive: true,
            role_code,
        }
    }
}
