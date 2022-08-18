pub struct Person {
    person_id: String,
    village_id: String,
    is_alive: bool,
}

impl Person {
    pub fn new(person_id: String, village_id: &String) -> Self {
        Person {
            person_id,
            village_id: village_id.to_string(),
            is_alive: true,
        }
    }
}
