pub(super) struct NightEventsStorage {
    wolves_choice_person_id: Option<String>,
    doctor_choice_person_id: Option<String>,
    seer_choice_person_id: Option<String>,
}

impl NightEventsStorage {
    pub(super) fn new() -> Self {
        Self {
            wolves_choice_person_id: None,
            doctor_choice_person_id: None,
            seer_choice_person_id: None,
        }
    }

    pub(super) fn set_wolves_choice(&mut self, person_id: &str) {
        self.wolves_choice_person_id = Some(person_id.to_string());
    }

    pub(super) fn set_doctor_choice(&mut self, person_id: &str) {
        self.doctor_choice_person_id = Some(person_id.to_string());
    }

    pub(super) fn set_seer_choice(&mut self, person_id: &str) {
        self.seer_choice_person_id = Some(person_id.to_string());
    }

    pub(super) fn wolves_doctor_seer_choices(
        &self,
    ) -> (Option<String>, Option<String>, Option<String>) {
        (
            self.wolves_choice_person_id.clone(),
            self.doctor_choice_person_id.clone(),
            self.seer_choice_person_id.clone(),
        )
    }
}
